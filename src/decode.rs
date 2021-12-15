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
use crate::{load_ton_address, print_args, VERBOSE_MODE};
use crate::config::Config;
use crate::helpers::{decode_msg_body, print_account, create_client_local, create_client_verbose, query, TonClient};
use clap::{ArgMatches, SubCommand, Arg, App, AppSettings};
use ton_types::cells_serialization::serialize_tree_of_cells;
use ton_types::Cell;
use std::fmt::Write;
use ton_block::{Account, Deserializable, Serializable, AccountStatus, StateInit};
use ton_client::abi::{decode_account_data, ParamsOfDecodeAccountData, Abi};
use crate::decode::msg_printer::tree_of_cells_into_base64;

fn match_abi_path(matches: &ArgMatches, config: &Config) -> Option<String> {
    matches.value_of("ABI")
        .map(|s| s.to_string())
        .or(config.abi_path.clone())
}

pub fn create_decode_command<'a, 'b>() -> App<'a, 'b> {
    let tvc_cmd = SubCommand::with_name("stateinit")
        .setting(AppSettings::AllowLeadingHyphen)
        .about("Decodes tvc data (including compiler version) from different sources.")
        .arg(Arg::with_name("TVC")
            .long("--tvc")
            .conflicts_with("BOC")
            .help("Contract is passed via path to the TVC file."))
        .arg(Arg::with_name("BOC")
            .long("--boc")
            .conflicts_with("TVC")
            .help("Contract is passed via path to the account BOC file."))
        .arg(Arg::with_name("INPUT")
            .required(true)
            .help("Contract address or path to the file with contract data."));
    SubCommand::with_name("decode")
        .about("Decode commands.")
        .setting(AppSettings::AllowLeadingHyphen)
        .setting(AppSettings::TrailingVarArg)
        .setting(AppSettings::DontCollapseArgsInUsage)
        .subcommand(SubCommand::with_name("body")
            .about("Decodes body base64 string.")
            .arg(Arg::with_name("BODY")
                .required(true)
                .help("Message body encoded as base64."))
            .arg(Arg::with_name("ABI")
                .long("--abi")
                .takes_value(true)
                .help("Path to the contract ABI file.")))
        .subcommand(SubCommand::with_name("msg")
            .about("Decodes message file.")
            .arg(Arg::with_name("MSG")
                    .required(true)
                    .help("Path to the message boc file."))
            .arg(Arg::with_name("ABI")
                    .long("--abi")
                    .takes_value(true)
                    .help("Path to the contract ABI file.")))
        .subcommand(tvc_cmd)
        .subcommand(SubCommand::with_name("account")
            .about("Top level command of account decode commands.")
            .subcommand(SubCommand::with_name("data")
                .about("Decodes data fields from the contract state.")
                .arg(Arg::with_name("TVC")
                    .long("--tvc")
                    .short("-t")
                    .takes_value(true)
                    .help("Path to the tvc file with contract state.")
                    .conflicts_with("ADDRESS"))
                .arg(Arg::with_name("ADDRESS")
                    .long("--addr")
                    .short("-a")
                    .takes_value(true)
                    .help("Contract address.")
                    .conflicts_with("TVC"))
                .arg(Arg::with_name("ABI")
                    .long("--abi")
                    .takes_value(true)
                    .help("Path to the contract ABI file.")))
            .subcommand(SubCommand::with_name("boc")
                .about("Decodes data from the file with boc of the account and saves contract tvc file if needed.")
                .arg(Arg::with_name("BOCFILE")
                    .required(true)
                    .help("Path to the account boc file."))
                .arg(Arg::with_name("DUMPTVC")
                    .long("--dumptvc")
                    .short("-d")
                    .takes_value(true)
                    .help("Path to the TVC file where to save the dump."))))
}

pub async fn decode_command(m: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    if let Some(m) = m.subcommand_matches("body") {
        return decode_body_command(m, config).await;
    }
    if let Some(m) = m.subcommand_matches("msg") {
        return decode_message_command(m, config).await;
    }
    if let Some(m) = m.subcommand_matches("stateinit") {
        return decode_tvc_command(m, config).await;
    }
    if let Some(m) = m.subcommand_matches("account") {
        if let Some(m) = m.subcommand_matches("boc") {
            return decode_account_from_boc(m, config).await;
        }
        if let Some(m) = m.subcommand_matches("data") {
            return decode_data_command(m, config).await;
        }
    }
    Err("unknown command".to_owned())
}

async fn decode_data_command(m: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    if m.is_present("TVC") {
        return decode_tvc_fields(m, config).await;
    }
    if m.is_present("ADDRESS") {
        return decode_account_fields(m, config).await;
    }
    Err("unknown command".to_owned())
}

async fn decode_body_command(m: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let body = m.value_of("BODY");
    let abi = Some(
        match_abi_path(m, &config)
            .ok_or("ABI file is not defined. Supply it in the config file or command line.".to_string())?
    );
    if !config.is_json {
        print_args!(body, abi);
    }
    println!("{}", decode_body(body.unwrap(), &abi.unwrap(), config.is_json).await?);
    Ok(())
}

async fn decode_account_from_boc(m: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let boc = m.value_of("BOCFILE");
    let tvc_path = m.value_of("DUMPTVC");

    if !config.is_json {
        print_args!(boc, tvc_path);
    }

    let account = Account::construct_from_file(boc.unwrap())
        .map_err(|e| format!(" failed to load account from the boc file: {}", e))?;

    print_account_data(&account, tvc_path, config).await
}

pub async fn print_account_data(account: &Account, tvc_path: Option<&str>, config: Config) -> Result<(), String> {
    if account.is_none() {
        println!("\nAccount is None");
        return Ok(());
    }
    let state_init = account.state_init();

    let address = match account.get_addr() {
        Some(address) => format!("{}", address),
        _ => "Undefined".to_owned(),
    };

    let state = match account.status() {
        AccountStatus::AccStateUninit => "Uninit".to_owned(),
        AccountStatus::AccStateFrozen => "Frozen".to_owned(),
        AccountStatus::AccStateActive => "Active".to_owned(),
        AccountStatus::AccStateNonexist => "NonExist".to_owned(),
    };

    let balance = match account.balance() {
        Some(balance) => format!("{}", balance.grams.clone()),
        _ => "Undefined".to_owned(),
    };

    let trans_lt = account.last_tr_time()
        .map_or("Undefined".to_owned(), |v| format!("{}", v));
    let paid = format!("{}", account.last_paid());

    let (si, code_hash) = match state_init {
        Some(state_init) => {
            let code = state_init.code.clone()
                .ok_or("failed to obtain code from the StateInit")?;
            let ton = create_client_local()?;
            (
                serde_json::to_string_pretty(
                    &msg_printer::serialize_state_init(state_init, ton)
                        .await?)
                 .map_err(|e| format!("Failed to serialize stateInit: {}", e))?,
                Some(code.repr_hash().to_hex_string())
            )
        },
        _ => ("Undefined".to_owned(), None)
    };

    print_account(
        &config,
        Some(state),
        Some(address),
        Some(balance),
        Some(paid),
        Some(trans_lt),
        None,
        code_hash,
        Some(si),
    );

    if tvc_path.is_some() && state_init.is_some() {
        state_init.unwrap().write_to_file(tvc_path.unwrap())
            .map_err(|e| format!("{}", e))?;
    }

    Ok(())
}

async fn decode_message_command(m: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let msg = m.value_of("MSG");
    let abi = Some(
        match_abi_path(m, &config)
            .ok_or("ABI file is not defined. Supply it in the config file or command line.".to_string())?
    );
    if !config.is_json {
        print_args!(msg, abi);
    }
    let msg = msg.map(|f| std::fs::read(f))
        .transpose()
        .map_err(|e| format!(" failed to read msg boc file: {}", e))?
        .unwrap();
    println!("{}", decode_message(msg, abi).await?);
    Ok(())
}

async fn decode_tvc_fields(m: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let tvc = m.value_of("TVC");
    let abi = Some(
        match_abi_path(m, &config)
            .ok_or("ABI file is not defined. Supply it in the config file or command line.".to_string())?
    );
    if !config.is_json {
        print_args!(tvc, abi);
    }
    let abi = std::fs::read_to_string(abi.unwrap())
        .map_err(|e| format!("failed to read ABI file: {}", e))?;
    let state = StateInit::construct_from_file(tvc.unwrap())
        .map_err(|e| format!("failed to load StateInit from the tvc file: {}", e))?;
    let b64 = tree_of_cells_into_base64(state.data.as_ref())?;
    let ton = create_client_local()?;
    let res = decode_account_data(
        ton,
        ParamsOfDecodeAccountData {
                abi: Abi::Json(abi),
                data: b64,
            }
        )
        .await
        .map_err(|e| format!("failed to decode data: {}", e))?;
    println!("TVC fields:\n{}", serde_json::to_string_pretty(&res.data)
        .map_err(|e| format!("failed to serialize the result: {}", e))?);
    Ok(())
}

async fn decode_account_fields(m: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let address = m.value_of("ADDRESS");
    let abi = Some(
        match_abi_path(m, &config)
            .ok_or("ABI file is not defined. Supply it in the config file or command line.".to_string())?
    );
    if !config.is_json {
        print_args!(address, abi);
    }
    let abi = std::fs::read_to_string(abi.unwrap())
        .map_err(|e| format!("failed to read ABI file: {}", e))?;

    let ton = create_client_verbose(&config)?;
    let address = load_ton_address(address.unwrap(), &config)?;
    let data = query_account(ton.clone(), &address, "data").await?;

    let res = decode_account_data(
        ton,
        ParamsOfDecodeAccountData {
                abi: Abi::Json(abi),
                data,
            }
        )
        .await
        .map_err(|e| format!("failed to decode data: {}", e))?;
    if !config.is_json {
        println!("Account fields:");
    }
    println!("{}", serde_json::to_string_pretty(&res.data)
        .map_err(|e| format!("failed to serialize the result: {}", e))?);
    Ok(())
}

async fn print_decoded_body(body_vec: Vec<u8>, abi: &str, is_json: bool) -> Result<String, String> {
    let ton = create_client_local()?;
    let mut empty_boc = vec![];
    serialize_tree_of_cells(&Cell::default(), &mut empty_boc)
        .map_err(|e| format!("failed to serialize tree of cells: {}", e))?;
    if body_vec.cmp(&empty_boc) == std::cmp::Ordering::Equal {
        return Err(format!("body is empty"));
    }

    let body_base64 = base64::encode(&body_vec);
    let mut res = {
        match decode_msg_body(ton.clone(), abi, &body_base64, false).await {
            Ok(res) => res,
            Err(_) => decode_msg_body(ton.clone(), abi, &body_base64, true).await?,
        }
    };
    let output = res.value.take().ok_or("failed to obtain the result")?;
    Ok(if is_json {
        format!(" \"BodyCall\": {{\n  \"{}\": {}\n }}", res.name, output)
    } else {
        format!("{}: {}", res.name, serde_json::to_string_pretty(&output)
            .map_err(|e| format!("failed to serialize the result: {}", e))?)
    })
}

async fn decode_body(body: &str, abi: &str, is_json: bool) -> Result<String, String> {
    let abi = std::fs::read_to_string(abi)
        .map_err(|e| format!("failed to read ABI file: {}", e))?;

    let body_vec  = base64::decode(body)
        .map_err(|e| format!("body is not a valid base64 string: {}", e))?;

    let mut result = String::new();
    let s = &mut result;
    if is_json { writeln!(s, "{{").map_err(|e| format!("failed to serialize the result: {}", e))?; }
    writeln!(s, "{}", print_decoded_body(body_vec, &abi, is_json).await?)
        .map_err(|e| format!("failed to serialize the result: {}", e))?;
    if is_json { writeln!(s, "}}").map_err(|e| format!("failed to serialize the result: {}", e))?; }
    Ok(result)
}

async fn decode_message(msg_boc: Vec<u8>, abi: Option<String>) -> Result<String, String> {
    let abi = abi.map(|f| std::fs::read_to_string(f))
        .transpose()
        .map_err(|e| format!("failed to read ABI file: {}", e))?;

    let tvm_msg = ton_sdk::Contract::deserialize_message(&msg_boc[..])
        .map_err(|e| format!("failed to deserialize message boc: {}", e))?;
    let result = msg_printer::serialize_msg(&tvm_msg, abi).await?;
    Ok(serde_json::to_string_pretty(&result)
        .map_err(|e| format!("Failed to serialize the result: {}", e))?)
}

fn load_state_init(m: &ArgMatches<'_>) -> Result<StateInit, String> {
    let input = m.value_of("INPUT").unwrap();
    let stat_init = if m.is_present("BOC") {
        let account = Account::construct_from_file(input)
            .map_err(|e| format!(" failed to load account from the boc file {}: {}", input, e))?;
        account.state_init().ok_or("Failed to load stateInit from the BOC.")?.to_owned()
    } else {
        StateInit::construct_from_file(input)
            .map_err(|e| format!("failed to load StateInit from the tvc file: {}", e))?
    };
    Ok(stat_init)
}

async fn query_account(ton: TonClient, address: &str, field: &str) -> Result<String, String> {
    let accounts = query(
        ton.clone(),
        "accounts",
        json!({ "id": { "eq": address } }),
        field,
        None,
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

async fn decode_tvc_command(m: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let input = m.value_of("INPUT");
    if !config.is_json {
        print_args!(input);
    }
    let is_local = m.is_present("BOC") || m.is_present("TVC");
    let ton = if is_local {
        create_client_local()?
    } else {
        create_client_verbose(&config)?
    };
    let input = input.unwrap().to_owned();

    let state = if is_local {
        load_state_init(m)?
    } else {
        let input = if input.contains(":") {
            input
        } else {
            format!("{}:{}", config.wc, input)
        };
        let boc = query_account(ton.clone(), &input, "boc").await?;
        let account = Account::construct_from_base64(&boc)
            .map_err(|e| format!("Failed to query account BOC: {}", e))?;
        account.state_init().ok_or("Failed to load stateInit from the BOC.")?.to_owned()
    };

    if !config.is_json {
        println!("Decoded data:");
    }
    let result = msg_printer::serialize_state_init(&state, ton.clone()).await?;
    println!("{}", serde_json::to_string_pretty(&result)
        .map_err(|e| format!("Failed to serialize json: {}", e))?);

    Ok(())
}

mod msg_printer {
    use serde_json::Value;
    use ton_block::{CurrencyCollection, StateInit, Message, CommonMsgInfo, Grams};
    use ton_types::cells_serialization::serialize_tree_of_cells;
    use ton_types::Cell;
    use crate::helpers::{TonClient, create_client_local, decode_msg_body};
    use ton_client::boc::{get_compiler_version, ParamsOfGetCompilerVersion};

    pub fn tree_of_cells_into_base64(root_cell: Option<&Cell>) -> Result<String, String> {
        match root_cell {
            Some(cell) => {
                let mut bytes = Vec::new();
                serialize_tree_of_cells(cell, &mut bytes)
                    .map_err(|e| format!("failed to serialize tree of cells: {}", e))?;
                Ok(base64::encode(&bytes))
            }
            None => Ok("".to_string())
        }
    }

    async fn get_code_version(ton: TonClient, code: String) -> String {
        let result = get_compiler_version(
            ton,
            ParamsOfGetCompilerVersion {
                code
            }
        ).await;

        if let Ok(result) = result {
            if let Some(version) = result.version {
                return version;
            }
        }
        "Undefined".to_owned()
    }

    pub async fn serialize_state_init (state: &StateInit, ton: TonClient) -> Result<Value, String> {
        let code = tree_of_cells_into_base64(state.code.as_ref())?;
        Ok(json!({
            "split_depth" : state.split_depth.as_ref().map(|x| format!("{:?}", (x.0 as u8))).unwrap_or("None".to_string()),
            "special" : state.special.as_ref().map(|x| format!("{:?}", x)).unwrap_or("None".to_string()),
            "data" : tree_of_cells_into_base64(state.data.as_ref())?,
            "code" : code.clone(),
            "code_hash" : state.code.as_ref().map(|code| code.repr_hash().to_hex_string()).unwrap_or("None".to_string()),
            "data_hash" : state.data.as_ref().map(|code| code.repr_hash().to_hex_string()).unwrap_or("None".to_string()),
            "code_depth" : state.code.as_ref().map(|code| code.repr_depth().to_string()).unwrap_or("None".to_string()),
            "data_depth" : state.data.as_ref().map(|code| code.repr_depth().to_string()).unwrap_or("None".to_string()),
            "version" : get_code_version(ton, code).await,
            "lib" : tree_of_cells_into_base64(state.library.root())?,
        }))
    }

    fn serialize_msg_type(header: &CommonMsgInfo) -> Value {
        json!(match header {
            CommonMsgInfo::IntMsgInfo(_) => "internal",
            CommonMsgInfo::ExtInMsgInfo(_) => "external inbound",
            CommonMsgInfo::ExtOutMsgInfo(_) => "external outbound",
        }.to_owned() + " message")
    }

    fn serialize_grams(grams: &Grams) -> Value {
        json!(grams.0.to_string())
    }

    fn serialize_currency_collection(cc: &CurrencyCollection) -> Value {
        let grams = serialize_grams(&cc.grams);
        if cc.other.is_empty() {
            return grams;
        }
        let mut other = json!({});
        cc.other.iterate_with_keys(|key: u32, value| {
            other[key.to_string()] = json!(value.0.to_string());
            Ok(true)
        }).ok();
        json!({
            "value" : grams,
            "other" : other,
        })
    }

    fn serialize_msg_header(header: &CommonMsgInfo) -> Value {
        match header {
            CommonMsgInfo::IntMsgInfo(header) => {
                json!({
                    "ihr_disabled": &header.ihr_disabled.to_string(),
                    "bounce" : &header.bounce.to_string(),
                    "bounced" : &header.bounced.to_string(),
                    "source" : &header.src.to_string(),
                    "destination" : &header.dst.to_string(),
                    "value" : &serialize_currency_collection(&header.value),
                    "ihr_fee" : &serialize_grams(&header.ihr_fee),
                    "fwd_fee" : &serialize_grams(&header.fwd_fee),
                    "created_lt" : &header.created_lt.to_string(),
                    "created_at" : &header.created_at.to_string(),
                })
            },
            CommonMsgInfo::ExtInMsgInfo(header) => {
                json!({
                    "source" : &header.src.to_string(),
                    "destination" : &header.dst.to_string(),
                    "import_fee" : &serialize_grams(&header.import_fee),
                })
            },
            CommonMsgInfo::ExtOutMsgInfo(header) => {
                json!({
                    "source" : &header.src.to_string(),
                    "destination" : &header.dst.to_string(),
                    "created_lt" : &header.created_lt.to_string(),
                    "created_at" : &header.created_at.to_string(),
                })
            }
        }
    }

    pub async fn serialize_body(body_vec: Vec<u8>, abi: &str, ton: TonClient) -> Result<Value, String> {
        let mut empty_boc = vec![];
        serialize_tree_of_cells(&Cell::default(), &mut empty_boc)
            .map_err(|e| format!("failed to serialize tree of cells: {}", e))?;
        if body_vec.cmp(&empty_boc) == std::cmp::Ordering::Equal {
            return Ok(json!("empty"));
        }
        let body_base64 = base64::encode(&body_vec);
        let mut res = {
            match decode_msg_body(ton.clone(), abi, &body_base64, false).await {
                Ok(res) => res,
                Err(_) => decode_msg_body(ton.clone(), abi, &body_base64, true).await?,
            }
        };
        let output = res.value.take().ok_or("failed to obtain the result")?;
        Ok(json!({res.name : output}))
    }

    pub async fn serialize_msg(msg: &Message, abi: Option<String>) -> Result<Value, String> {
        let mut res = json!({ });
        let ton = create_client_local()?;
        res["Type"] = serialize_msg_type(msg.header());
        res["Header"] = serialize_msg_header(msg.header());
        if msg.state_init().is_some() {
            res["Init"] = json!({"StateInit" : serialize_state_init(msg.state_init().unwrap(), ton.clone()).await?});
        }
        res["Body"] = json!(&tree_of_cells_into_base64(
            msg.body().map(|slice| slice.into_cell()).as_ref()
        )?);
        if abi.is_some() && msg.body().is_some() {
            let abi = abi.unwrap();
            let mut body_vec = Vec::new();
            serialize_tree_of_cells(&msg.body().unwrap().into_cell(), &mut body_vec)
                .map_err(|e| format!("failed to serialize body: {}", e))?;
            res["BodyCall"] = serialize_body(body_vec, &abi, ton).await?;
        }
        Ok(res)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_decode_msg_json() {
        let msg_boc = std::fs::read("tests/samples/wallet.boc").unwrap();
        let out = decode_message(msg_boc, Some("tests/samples/wallet.abi.json".to_owned())).await.unwrap();
        let _ : serde_json::Value = serde_json::from_str(&out).unwrap();
    }

    #[tokio::test]
    async fn test_decode_body_json() {
        let body = "te6ccgEBAQEARAAAgwAAALqUCTqWL8OX7JivfJrAAzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMQAAAAAAAAAAAAAAAEeGjADA==";
        let out = decode_body(body, "tests/samples/wallet.abi.json", true).await.unwrap();
        let _ : serde_json::Value = serde_json::from_str(&out).unwrap();
    }
}
