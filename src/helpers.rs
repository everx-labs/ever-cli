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
use std::env;
use std::path::PathBuf;
use crate::config::{Config, LOCALNET};

use std::sync::Arc;
use std::time::{Duration, SystemTime};
use ton_client::abi::{
    Abi, AbiConfig, AbiContract, DecodedMessageBody, DeploySet, ParamsOfDecodeMessageBody,
    ParamsOfEncodeMessage, Signer,
};
use ton_client::crypto::{CryptoConfig, KeyPair};
use ton_client::error::ClientError;
use ton_client::net::{query_collection, OrderBy, ParamsOfQueryCollection, NetworkConfig};
use ton_client::{ClientConfig, ClientContext};
use ton_block::{Account, MsgAddressInt, Deserializable, CurrencyCollection, StateInit, Serializable};
use std::str::FromStr;
use clap::ArgMatches;
use serde_json::Value;
use ton_client::abi::Abi::Contract;
use url::Url;
use crate::call::parse_params;
use crate::{FullConfig, resolve_net_name};

pub const TEST_MAX_LEVEL: log::LevelFilter = log::LevelFilter::Debug;
pub const MAX_LEVEL: log::LevelFilter = log::LevelFilter::Warn;

pub const HD_PATH: &str = "m/44'/396'/0'/0/0";
pub const WORD_COUNT: u8 = 12;

pub const SDK_EXECUTION_ERROR_CODE: u32 = 414;
const CONFIG_BASE_NAME: &str = "tonos-cli.conf.json";
const GLOBAL_CONFIG_PATH: &str = ".tonos-cli.global.conf.json";

pub fn default_config_name() -> String {
    env::current_dir()
        .map(|dir| {
            dir.join(PathBuf::from(CONFIG_BASE_NAME)).to_str().unwrap().to_string()
        })
        .unwrap_or(CONFIG_BASE_NAME.to_string())
}

pub fn global_config_path() -> String {
    env::current_exe()
        .map(|mut dir| {
            dir.set_file_name(GLOBAL_CONFIG_PATH);
            dir.to_str().unwrap().to_string()
        })
        .unwrap_or(GLOBAL_CONFIG_PATH.to_string())
}

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
        .map_err(|e| format!("failed to read the keypair file: {}", e))?;
    let keys: KeyPair = serde_json::from_str(&keys_str)
        .map_err(|e| format!("failed to load keypair: {}", e))?;
    Ok(keys)
}

pub fn load_ton_address(addr: &str, config: &Config) -> Result<String, String> {
    let addr = if addr.find(':').is_none() {
        format!("{}:{}", config.wc, addr)
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

pub fn get_server_endpoints(config: &Config) -> Vec<String> {
    let mut cur_endpoints = match config.endpoints.len() {
        0 => vec![config.url.clone()],
        _ => config.endpoints.clone(),
    };
    cur_endpoints.iter_mut().map(|end| {
            let mut end = end.trim_end_matches('/').to_owned();
        if config.project_id.is_some() {
            end.push_str("/");
            end.push_str(&config.project_id.clone().unwrap());
        }
        end.to_owned()
    }).collect::<Vec<String>>()
}

pub fn get_network_context(config: &Config) -> Result<Arc<ClientContext>, failure::Error> {
    let endpoints = get_server_endpoints(config);
    Ok(Arc::new(
        ClientContext::new(ClientConfig {
            network: NetworkConfig {
                sending_endpoint_count: endpoints.len() as u8,
                endpoints: Some(endpoints),
                access_key: config.access_key.clone(),
                ..Default::default()
            },
            ..Default::default()
        })?
    ))
}

pub fn create_client(config: &Config) -> Result<TonClient, String> {
    let modified_endpoints = get_server_endpoints(config);
    if !config.is_json {
        println!("Connecting to:\n\tUrl: {}", config.url);
        println!("\tEndpoints: {:?}\n", modified_endpoints);
    }
    let endpoints_cnt = if resolve_net_name(&config.url).unwrap_or(config.url.clone()).eq(LOCALNET) {
        1_u8
    } else {
        modified_endpoints.len() as u8
    };
    let cli_conf = ClientConfig {
        abi: AbiConfig {
            workchain: config.wc,
            message_expiration_timeout: config.lifetime * 1000,
            message_expiration_timeout_grow_factor: 1.3,
        },
        crypto: CryptoConfig {
            mnemonic_dictionary: 1,
            mnemonic_word_count: WORD_COUNT,
            hdkey_derivation_path: HD_PATH.to_string(),
        },
        network: NetworkConfig {
            server_address: Some(config.url.to_owned()),
            sending_endpoint_count: endpoints_cnt,
            endpoints: if modified_endpoints.is_empty() {
                    None
                } else {
                    Some(modified_endpoints)
                },
            message_retries_count: config.retries as i8,
            message_processing_timeout: 30000,
            wait_for_timeout: config.timeout,
            out_of_sync_threshold: config.out_of_sync_threshold * 1000,
            access_key: config.access_key.clone(),
            ..Default::default()
        },
        ..Default::default()
    };
    let cli =
        ClientContext::new(cli_conf).map_err(|e| format!("failed to create tonclient: {}", e))?;
    Ok(Arc::new(cli))
}

pub fn create_client_verbose(config: &Config) -> Result<TonClient, String> {
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
    create_client(config)
}


pub async fn query_raw(
    config: &Config,
    collection: &str,
    filter: Option<&str>,
    limit: Option<&str>,
    order: Option<&str>,
    result: &str
) -> Result<(), String>
{
    let context = create_client_verbose(config)?;

    let filter = filter.map(|s| serde_json::from_str(s)).transpose()
        .map_err(|e| format!("Failed to parse filter field: {}", e))?;
    let limit = limit.map(|s| s.parse::<u32>()).transpose()
        .map_err(|e| format!("Failed to parse limit field: {}", e))?;
    let order = order.map(|s| serde_json::from_str(s)).transpose()
        .map_err(|e| format!("Failed to parse order field: {}", e))?;

    let query = ton_client::net::query_collection(
        context.clone(),
        ParamsOfQueryCollection {
            collection: collection.to_owned(),
            filter,
            limit,
            order,
            result: result.to_owned(),
            ..Default::default()
        }
    ).await.map_err(|e| format!("Failed to execute query: {}", e))?;

    println!("{:#}", Value::Array(query.result));
    Ok(())
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
    if accounts.is_empty() {
        return Err(format!("account with address {} not found", address));
    }
    let data = accounts[0][field].as_str();
    if data.is_none() {
        return Err(format!("account doesn't contain {}", field));
    }
    Ok(data.unwrap().to_string())
}


pub async fn decode_msg_body(
    ton: TonClient,
    abi_path: &str,
    body: &str,
    is_internal: bool,
    config: &Config,
) -> Result<DecodedMessageBody, String> {

    let abi = load_abi(abi_path, config).await?;
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

pub async fn load_abi_str(abi_path: &str, config: &Config) -> Result<String, String> {
    let abi_from_json = serde_json::from_str::<AbiContract>(abi_path);
    if abi_from_json.is_ok() {
        return Ok(abi_path.to_string());
    }
    if Url::parse(abi_path).is_ok() {
        let abi_bytes = load_file_with_url(abi_path, config.timeout as u64).await?;
        return Ok(String::from_utf8(abi_bytes)
            .map_err(|e| format!("Downloaded string contains not valid UTF8 characters: {}", e))?);
    }
    Ok(std::fs::read_to_string(&abi_path)
        .map_err(|e| format!("failed to read ABI file: {}", e))?)
}

pub async fn load_abi(abi_path: &str, config: &Config) -> Result<Abi, String> {
    let abi_str = load_abi_str(abi_path, config).await?;
    Ok(Contract(serde_json::from_str::<AbiContract>(&abi_str)
            .map_err(|e| format!("ABI is not a valid json: {}", e))?,
    ))
}

pub async fn load_ton_abi(abi_path: &str, config: &Config) -> Result<ton_abi::Contract, String> {
    let abi_str = load_abi_str(abi_path, config).await?;
    Ok(ton_abi::Contract::load(abi_str.as_bytes())
        .map_err(|e| format!("Failed to load ABI: {}", e))?)
}

pub async fn load_file_with_url(url: &str, timeout: u64) -> Result<Vec<u8>, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(timeout))
        .build()
        .map_err(|e| format!("Failed to create client: {e}"))?;
    let res = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to send get request: {e}"))?
        .bytes()
        .await
        .map_err(|e| format!("Failed to get response bytes: {e}"))?;
    Ok(res.to_vec())

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
        .map(serde_json::from_str)
        .transpose()
        .map_err(|e| format!("initial data is not in json: {}", e))?;

    let dset = DeploySet {
        tvc: base64::encode(tvc),
        workchain_id: Some(wc),
        initial_data: init_data_json,
        initial_pubkey: pubkey.clone(),
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

pub async fn print_message(ton: TonClient, message: &Value, abi: &str, is_internal: bool) -> Result<(String, String), String> {
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
        let def_config = Config::default();
        let result = ton_client::abi::decode_message_body(
            ton.clone(),
            ParamsOfDecodeMessageBody {
                abi: load_abi(abi, &def_config).await?,
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
    Ok(("".to_owned(), "".to_owned()))
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
    Account::active_by_init_code_hash(
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
    ).map_err(|e| format!(" failed to create account with the stateInit: {}",e))
}

pub fn check_dir(path: &str) -> Result<(), String> {
    if !path.is_empty() && !std::path::Path::new(path).exists() {
        std::fs::create_dir(path)
            .map_err(|e| format!("Failed to create folder {}: {}", path, e))?;
    }
    Ok(())
}

#[derive(PartialEq)]
pub enum AccountSource {
    NETWORK,
    BOC,
    TVC,
}

pub async fn load_account(
    source_type: &AccountSource,
    source: &str,
    ton_client: Option<TonClient>,
    config: &Config
) -> Result<(Account, String), String> {
    match source_type {
        AccountSource::NETWORK => {
            let ton_client = match ton_client {
                Some(ton_client) => ton_client,
                None => {
                    create_client(&config)?
                }
            };
            let boc = query_account_field(ton_client.clone(),source, "boc").await?;
            Ok((Account::construct_from_base64(&boc)
                .map_err(|e| format!("Failed to construct account: {}", e))?,
                boc))
        },
        _ => {
            let account = if source_type == &AccountSource::BOC {
                Account::construct_from_file(source)
                    .map_err(|e| format!(" failed to load account from the file {}: {}", source, e))?
            } else {
                construct_account_from_tvc(source, None, None)?
            };
            let account_bytes = account.write_to_bytes()
                .map_err(|e| format!(" failed to load data from the account: {}", e))?;
            Ok((account, base64::encode(&account_bytes)))
        },
    }
}


pub fn load_debug_info(abi: &str) -> Option<String> {
    check_file_exists(abi, &[".json", ".abi"], ".dbg.json")
}

pub fn load_abi_from_tvc(tvc: &str) -> Option<String> {
    check_file_exists(tvc, &[".tvc"], ".abi.json")
}

pub fn check_file_exists(path: &str, trim: &[&str], ending: &str) -> Option<String> {
    let mut path = path;
    for end in trim {
        path = path.trim_end_matches(end);
    }
    let mut path = path.to_string();
    path.push_str(ending);
    if std::path::Path::new(&path).exists() {
        return Some(path);
    }
    None
}

pub fn abi_from_matches_or_config(matches: &ArgMatches<'_>, config: &Config) -> Result<String, String> {
    matches.value_of("ABI")
        .map(|s| s.to_string())
        .or(config.abi_path.clone())
        .ok_or("ABI file is not defined. Supply it in the config file or command line.".to_string())
}

pub fn parse_lifetime(lifetime: Option<&str>, config: &Config) -> Result<u32, String> {
    Ok(lifetime.map(|val| {
        u32::from_str_radix(val, 10)
            .map_err(|e| format!("failed to parse lifetime: {}", e))
    })
        .transpose()?
        .unwrap_or(config.lifetime))
}


#[macro_export]
macro_rules! print_args {
    ($( $arg:ident ),* ) => {
        println!("Input arguments:");
        $(
            println!(
                "{:>width$}: {}",
                stringify!($arg),
                if let Some(ref arg) = $arg { arg.as_ref() } else { "None" },
                width = 8
            );
        )*
    };
}

pub fn load_params(params: &str) -> Result<String, String> {
    Ok(if params.find('{').is_none() {
        std::fs::read_to_string(params)
            .map_err(|e| format!("failed to load params from file: {}", e))?
    } else {
        params.to_string()
    })
}

pub async fn unpack_alternative_params(matches: &ArgMatches<'_>, abi_path: &str, method: &str, config: &Config) -> Result<Option<String>, String> {
    if matches.is_present("PARAMS") {
        let params = matches.values_of("PARAMS").unwrap().collect::<Vec<_>>();
        Ok(Some(parse_params(params, abi_path, method, config).await?))
    } else {
        Ok(config.parameters.clone().or(Some("{}".to_string())))
    }
}

pub fn wc_from_matches_or_config(matches: &ArgMatches<'_>, config: &Config) -> Result<i32 ,String> {
    Ok(matches.value_of("WC")
        .map(|v| i32::from_str_radix(v, 10))
        .transpose()
        .map_err(|e| format!("failed to parse workchain id: {}", e))?
        .unwrap_or(config.wc))
}

pub fn contract_data_from_matches_or_config_alias(
    matches: &ArgMatches<'_>,
    full_config: &FullConfig
) -> Result<(Option<String>, Option<String>, Option<String>), String> {
    let address = matches.value_of("ADDRESS")
        .map(|s| s.to_string())
        .or(full_config.config.addr.clone())
        .ok_or("ADDRESS is not defined. Supply it in the config file or command line.".to_string())?;
    let (address, abi, keys) = if full_config.aliases.contains_key(&address) {
        let alias = full_config.aliases.get(&address).unwrap();
        (alias.address.clone(), alias.abi_path.clone(), alias.key_path.clone())
    } else {
        (Some(address), None, None)
    };
    let abi = matches.value_of("ABI")
        .map(|s| s.to_string())
        .or(full_config.config.abi_path.clone())
        .or(abi)
        .ok_or("ABI file is not defined. Supply it in the config file or command line.".to_string())?;
    let keys = matches.value_of("KEYS")
        .map(|s| s.to_string())
        .or(full_config.config.keys_path.clone())
        .or(keys);
    Ok((address, Some(abi), keys))
}
