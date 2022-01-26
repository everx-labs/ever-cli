/*
 * Copyright 2018-2021 TON DEV SOLUTIONS LTD.
 *
 * Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
 * this file except in compliance with the License.
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific TON DEV software governing permissions and
 * limitations under the License.
 */
use crate::config::Config;
use log;
use std::sync::Arc;
use std::time::SystemTime;
use ton_client::abi::{
    Abi, AbiConfig, AbiContract, DecodedMessageBody, DeploySet, ParamsOfDecodeMessageBody,
    ParamsOfEncodeMessage, Signer,
};
use ton_client::crypto::{CryptoConfig, KeyPair};
use ton_client::error::ClientError;
use ton_client::net::{query_collection, OrderBy, ParamsOfQueryCollection};
use ton_client::{ClientConfig, ClientContext};
use ton_block::{Account, MsgAddressInt, Deserializable, CurrencyCollection, StateInit};
use std::str::FromStr;
use serde_json::Value;

const TEST_MAX_LEVEL: log::LevelFilter = log::LevelFilter::Debug;
const MAX_LEVEL: log::LevelFilter = log::LevelFilter::Warn;

pub const HD_PATH: &str = "m/44'/396'/0'/0/0";
pub const WORD_COUNT: u8 = 12;

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() < MAX_LEVEL
    }

    fn log(&self, record: &log::Record) {
        match record.level() {
            log::Level::Error | log::Level::Warn => {
                eprintln!("{}", record.args());
            }
            _ => {
                println!("{}", record.args());
            }
        }
    }

    fn flush(&self) {}
}

pub fn read_keys(filename: &str) -> Result<KeyPair, String> {
    let keys_str = std::fs::read_to_string(filename)
        .map_err(|e| format!("failed to read the keypair file: {}", e.to_string()))?;
    let keys: KeyPair = serde_json::from_str(&keys_str)
        .map_err(|e| format!("failed to load keypair: {}", e))?;
    Ok(keys)
}

pub fn load_ton_address(addr: &str, conf: &Config) -> Result<String, String> {
    let addr = if addr.find(':').is_none() {
        format!("{}:{}", conf.wc, addr)
    } else {
        addr.to_owned()
    };
    let _ = MsgAddressInt::from_str(&addr)
        .map_err(|e| format!("Address is specified in the wrong format. Error description: {}", e))?;
    Ok(addr)
}

pub fn now() -> Result<u32, String> {
    Ok(SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| format!("failed to obtain system time: {}", e))?
        .as_secs() as u32
    )
}

pub fn now_ms() -> u64 {
    chrono::prelude::Utc::now().timestamp_millis() as u64
}

pub type TonClient = Arc<ClientContext>;

pub fn create_client_local() -> Result<TonClient, String> {
    let cli = ClientContext::new(ClientConfig::default())
        .map_err(|e| format!("failed to create tonclient: {}", e))?;
    Ok(Arc::new(cli))
}

pub fn create_client(conf: &Config) -> Result<TonClient, String> {
    let cli_conf = ClientConfig {
        abi: AbiConfig {
            workchain: conf.wc,
            message_expiration_timeout: conf.lifetime * 1000,
            message_expiration_timeout_grow_factor: 1.3,
        },
        crypto: CryptoConfig {
            mnemonic_dictionary: 1,
            mnemonic_word_count: WORD_COUNT,
            hdkey_derivation_path: HD_PATH.to_string(),
        },
        network: ton_client::net::NetworkConfig {
            server_address: Some(conf.url.to_owned()),
            endpoints: if conf.endpoints.is_empty() {
                    None
                } else {
                    Some(conf.endpoints.to_owned())
                },
            // network_retries_count: 3,
            message_retries_count: conf.retries as i8,
            message_processing_timeout: 30000,
            wait_for_timeout: conf.timeout,
            out_of_sync_threshold: conf.out_of_sync_threshold * 1000,
            // max_reconnect_timeout: 1000,
            ..Default::default()
        },
        ..Default::default()
    };
    let cli =
        ClientContext::new(cli_conf).map_err(|e| format!("failed to create tonclient: {}", e))?;
    Ok(Arc::new(cli))
}

pub fn create_client_verbose(conf: &Config) -> Result<TonClient, String> {
    if !conf.is_json {
        println!("Connecting to:\n\tUrl: {}", conf.url);
        println!("\tEndpoints: {:?}\n", conf.endpoints);
    }

    let level = if std::env::var("RUST_LOG")
        .unwrap_or_default()
        .eq_ignore_ascii_case("debug")
    {
        TEST_MAX_LEVEL
    } else {
        MAX_LEVEL
    };
    log::set_max_level(level);
    log::set_boxed_logger(Box::new(SimpleLogger))
        .map_err(|e| format!("failed to init logger: {}", e))?;

    create_client(conf)
}

pub async fn query_with_limit(
    ton: TonClient,
    collection: &str,
    filter: serde_json::Value,
    result: &str,
    order: Option<Vec<OrderBy>>,
    limit: Option<u32>,
) -> Result<Vec<serde_json::Value>, ClientError> {
    query_collection(
        ton,
        ParamsOfQueryCollection {
            collection: collection.to_owned(),
            filter: Some(filter),
            result: result.to_owned(),
            order,
            limit,
            ..Default::default()
        },
    )
        .await
        .map(|r| r.result)
}

pub async fn query_account_field(ton: TonClient, address: &str, field: &str) -> Result<String, String> {
    let accounts = query_with_limit(
        ton.clone(),
        "accounts",
        json!({ "id": { "eq": address } }),
        field,
        None,
        Some(1),
    ).await
        .map_err(|e| format!("failed to query account data: {}", e))?;

    if accounts.len() == 0 {
        return Err(format!("account not found"));
    }
    let data = accounts[0][field].as_str();
    if data.is_none() {
        return Err(format!("account doesn't contain {}", field));
    }
    Ok(data.unwrap().to_string())
}


pub async fn decode_msg_body(
    ton: TonClient,
    abi: &str,
    body: &str,
    is_internal: bool,
) -> Result<DecodedMessageBody, String> {
    let abi = load_abi(abi)?;
    ton_client::abi::decode_message_body(
        ton,
        ParamsOfDecodeMessageBody {
            abi,
            body: body.to_owned(),
            is_internal,
            ..Default::default()
        },
    )
    .await
    .map_err(|e| format!("failed to decode body: {}", e))
}

pub fn load_abi(abi: &str) -> Result<Abi, String> {
    Ok(Abi::Contract(
        serde_json::from_str::<AbiContract>(abi)
            .map_err(|e| format!("ABI is not a valid json: {}", e))?,
    ))
}

pub async fn calc_acc_address(
    tvc: &[u8],
    wc: i32,
    pubkey: Option<String>,
    init_data: Option<&str>,
    abi: Abi,
) -> Result<String, String> {
    let ton = create_client_local()?;

    let init_data_json = init_data
        .map(|d| serde_json::from_str(d))
        .transpose()
        .map_err(|e| format!("initial data is not in json: {}", e))?;

    let dset = DeploySet {
        tvc: base64::encode(tvc),
        workchain_id: Some(wc),
        initial_data: init_data_json,
        ..Default::default()
    };
    let result = ton_client::abi::encode_message(
        ton.clone(),
        ParamsOfEncodeMessage {
            abi,
            deploy_set: Some(dset),
            signer: if pubkey.is_some() {
                Signer::External {
                    public_key: pubkey.unwrap(),
                }
            } else {
                Signer::None
            },
            ..Default::default()
        },
    )
    .await
    .map_err(|e| format!("cannot generate address: {}", e))?;
    Ok(result.address)
}

pub fn answer_filter(depool: &str, wallet: &str, since: u32) -> serde_json::Value {
    json!({
        "src": { "eq": depool },
        "dst": { "eq": wallet },
        "created_at": {"ge": since }
    })
}

pub fn events_filter(addr: &str, since: u32) -> serde_json::Value {
    json!({
        "src": { "eq": addr },
        "msg_type": {"eq": 2 },
        "created_at": {"ge": since }
    })
}

pub async fn print_message(ton: TonClient, message: &serde_json::Value, abi: &str, is_internal: bool) -> Result<(String, String), String> {
    println!("Id: {}", message["id"].as_str().unwrap_or("Undefined"));
    let value = message["value"].as_str().unwrap_or("0x0");
    let value = u64::from_str_radix(value.trim_start_matches("0x"), 16)
        .map_err(|e| format!("failed to decode msg value: {}", e))?;
    let value: f64 = value as f64 / 1e9;
    println!("Value: {:.9}", value);
    println!("Created at: {} ({})",
        message["created_at"].as_u64().unwrap_or(0),
        message["created_at_string"].as_str().unwrap_or("Undefined")
    );
    
    let body = message["body"].as_str();
    if body.is_some() {
        let body = body.unwrap();
        let result = ton_client::abi::decode_message_body(
            ton.clone(),
            ParamsOfDecodeMessageBody {
                abi: load_abi(abi)?,
                body: body.to_owned(),
                is_internal,
                ..Default::default()
            },
        ).await;
        let (name, args) = if result.is_err() {
            ("unknown".to_owned(), "{}".to_owned())
        } else {
            let result = result.unwrap();
            (result.name, serde_json::to_string(&result.value)
                .map_err(|e| format!("failed to serialize the result: {}", e))?)
        };
        println!("Decoded body:\n{} {}\n", name, args);
        return Ok((name, args));
    }
    println!();
    return Ok(("".to_owned(), "".to_owned()));
}

pub fn json_account(
    acc_type: Option<String>,
    address: Option<String>,
    balance: Option<String>,
    last_paid: Option<String>,
    last_trans_lt: Option<String>,
    data: Option<String>,
    code_hash: Option<String>,
    state_init: Option<String>,
) -> Value {
    let mut res = json!({ });
    if acc_type.is_some() {
        res["acc_type"] = json!(acc_type.unwrap());
    }
    if address.is_some() {
        res["address"] = json!(address.unwrap());
    }
    if balance.is_some() {
        res["balance"] = json!(balance.unwrap());
    }
    if last_paid.is_some() {
        res["last_paid"] = json!(last_paid.unwrap());
    }
    if last_trans_lt.is_some() {
        res["last_trans_lt"] = json!(last_trans_lt.unwrap());
    }
    if data.is_some() {
        res["data(boc)"] = json!(data.unwrap());
    }
    if code_hash.is_some() {
        res["code_hash"] = json!(code_hash.unwrap());
    }
    if state_init.is_some() {
        res["state_init"] = json!(state_init.unwrap());
    }
    res
}


pub fn print_account(
    config: &Config,
    acc_type: Option<String>,
    address: Option<String>,
    balance: Option<String>,
    last_paid: Option<String>,
    last_trans_lt: Option<String>,
    data: Option<String>,
    code_hash: Option<String>,
    state_init: Option<String>,
) {
    if config.is_json {
        let acc = json_account(
            acc_type,
            address,
            balance,
            last_paid,
            last_trans_lt,
            data,
            code_hash,
            state_init,
        );
        println!("{}", serde_json::to_string_pretty(&acc).unwrap_or("Undefined".to_string()));
    } else {
        if acc_type.is_some() && acc_type.clone().unwrap() == "NonExist" {
            println!("Account does not exist.");
            return;
        }
        if address.is_some() {
            println!("address:       {}", address.unwrap());
        }
        if acc_type.is_some() {
            println!("acc_type:      {}", acc_type.unwrap());
        }
        if balance.is_some() {
            println!("balance:       {}", balance.unwrap());
        }
        if last_paid.is_some() {
            println!("last_paid:     {}", last_paid.unwrap());
        }
        if last_trans_lt.is_some() {
            println!("last_trans_lt: {}", last_trans_lt.unwrap());
        }
        if data.is_some() {
            println!("data(boc):     {}", data.unwrap());
        }
        if code_hash.is_some() {
            println!("code_hash:     {}", code_hash.unwrap());
        }
        if state_init.is_some() {
            println!("state_init: {}", state_init.unwrap());
        }
    }
}

pub fn construct_account_from_tvc(tvc_path: &str, address: Option<&str>, balance: Option<u64>) -> Result<Account, String> {
    Ok(Account::active_by_init_code_hash(
        match address {
            Some(address) => MsgAddressInt::from_str(address)
                .map_err(|e| format!("Failed to set address: {}", e))?,
            _ => MsgAddressInt::default()
        },
        match balance {
            Some(balance) => CurrencyCollection::with_grams(balance),
            _ => CurrencyCollection::default()
        },
        0,
        StateInit::construct_from_file(tvc_path)
            .map_err(|e| format!(" failed to load TVC from the file {}: {}", tvc_path, e))?,
        true
    ).map_err(|e| format!(" failed to create account with the stateInit: {}",e))?)
}

pub fn check_dir(path: &str) -> Result<(), String> {
    if !path.is_empty() && !std::path::Path::new(path).exists() {
        std::fs::create_dir(path)
            .map_err(|e| format!("Failed to create folder {}: {}", path, e))?;
    }
    Ok(())
}
