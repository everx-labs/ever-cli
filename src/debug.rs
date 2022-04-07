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
use crate::{print_args, VERBOSE_MODE, abi_from_matches_or_config, load_params};
use clap::{ArgMatches, SubCommand, Arg, App};
use crate::config::Config;
use crate::helpers::{load_ton_address, create_client, load_abi, now_ms, construct_account_from_tvc, TonClient, query_account_field, query_with_limit};
use crate::replay::{
    fetch, CONFIG_ADDR, replay, DUMP_NONE, DUMP_CONFIG, DUMP_ACCOUNT, construct_blockchain_config
};
use std::io::{Write};
use ton_block::{Message, Account, Serializable, Deserializable, OutMessages, Transaction};
use ton_types::{UInt256, HashmapE, Cell};
use ton_client::abi::{CallSet, Signer, FunctionHeader, encode_message, ParamsOfEncodeMessage};
use ton_executor::{BlockchainConfig, ExecuteParams, TransactionExecutor};
use std::sync::{Arc, atomic::AtomicU64};
use ton_client::net::{OrderBy, ParamsOfQueryCollection, query_collection, SortDirection};
use crate::crypto::load_keypair;
use crate::debug_executor::{DebugTransactionExecutor, TraceLevel};
use std::fmt;

const DEFAULT_TRACE_PATH: &str = "./trace.log";
const DEFAULT_CONFIG_PATH: &str = "config.txns";
const DEFAULT_CONTRACT_PATH: &str = "contract.txns";
const TRANSACTION_QUANTITY: u32 = 10;


pub struct DebugLogger {
    tvm_trace: String,
}

impl DebugLogger {
    pub fn new(path: String) -> Self {
        if std::path::Path::new(&path).exists() {
            std::fs::remove_file(&path)
                .expect("Failed to remove old trace log.");
        }

        DebugLogger {
            tvm_trace: path,
        }
    }
}

impl log::Log for DebugLogger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        match record.target() {
            "tvm" | "executor" => {
                match std::fs::OpenOptions::new()
                    .write(true)
                    .append(true)
                    .create(true)
                    .open(&self.tvm_trace)
                    .as_mut()
                {
                    Ok(file) => {
                        let _ = file.write(format!("{}\n", record.args()).as_bytes())
                            .expect("Failed to write trace");
                    }
                    Err(_) => {
                        println!("{}", record.args());
                    }
                }
            }
            _ => {
                match record.level() {
                    log::Level::Error | log::Level::Warn => {
                        eprintln!("{}", record.args());
                    }
                    _ => {}
                }
            }
        }
    }

    fn flush(&self) {}
}

pub fn create_debug_command<'a, 'b>() -> App<'a, 'b> {
    let output_arg = Arg::with_name("LOG_PATH")
        .help("Path where to store the trace. Default path is \"./trace.log\". Note: old file will be removed.")
        .takes_value(true)
        .long("--output")
        .short("-o");

    let dbg_info_arg = Arg::with_name("DBG_INFO")
        .help("Path to the file with debug info.")
        .takes_value(true)
        .long("--dbg_info")
        .short("-d");

    let address_arg = Arg::with_name("ADDRESS")
        .required(true)
        .takes_value(true)
        .help("Contract address or path the file with saved contract state if corresponding flag is used.");

    let method_arg = Arg::with_name("METHOD")
        .required(true)
        .takes_value(true)
        .help("Name of the function being called.");

    let params_arg = Arg::with_name("PARAMS")
        .required(true)
        .takes_value(true)
        .help("Function arguments. Can be specified with a filename, which contains json data.");

    let sign_arg = Arg::with_name("SIGN")
        .long("--sign")
        .takes_value(true)
        .help("Seed phrase or path to the file with keypair used to sign the message. Can be specified in the config.");

    let abi_arg = Arg::with_name("ABI")
        .long("--abi")
        .takes_value(true)
        .help("Path to the contract ABI file. Can be specified in the config file.");

    let decode_abi_arg = Arg::with_name("DECODE_ABI")
        .long("--decode_abi")
        .takes_value(true)
        .help("Path to the ABI file used to decode output messages. Can be specified in the config file.");

    let full_trace_arg = Arg::with_name("FULL_TRACE")
        .long("--full_trace")
        .help("Flag that changes trace to full version.");

    let boc_arg = Arg::with_name("BOC")
        .long("--boc")
        .conflicts_with("TVC")
        .help("Flag that changes behavior of the command to work with the saved account state (account BOC).");

    let tx_id_arg = Arg::with_name("TX_ID")
        .required(true)
        .takes_value(true)
        .help("ID of the transaction that should be replayed.");

    let config_path_arg = Arg::with_name("CONFIG_PATH")
        .help("Path to the file with saved config contract state.")
        .long("--config")
        .short("-c")
        .takes_value(true);

    let empty_config_arg = Arg::with_name("EMPTY_CONFIG")
        .help("Replay transaction without full dump of the config contract.")
        .long("--empty_config")
        .short("-e");

    let config_save_path_arg = Arg::with_name("CONFIG_PATH")
        .help("Path to the file with saved config contract transactions. If not set transactions will be fetched to file \"config.txns\".")
        .long("--config")
        .short("-c")
        .takes_value(true);

    let contract_path_arg = Arg::with_name("CONTRACT_PATH")
        .help("Path to the file with saved target contract transactions. If not set transactions will be fetched to file \"contract.txns\".")
        .long("--contract")
        .short("-t")
        .takes_value(true);

    let dump_config_arg = Arg::with_name("DUMP_CONFIG")
        .help("Dump the replayed config contract account state.")
        .long("--dump_config");

    let dump_contract_arg = Arg::with_name("DUMP_CONTRACT")
        .help("Dump the replayed target contract account state.")
        .long("--dump_contract");

    let run_cmd = SubCommand::with_name("run")
        .about("Play getter locally with trace")
        .arg(output_arg.clone())
        .arg(dbg_info_arg.clone())
        .arg(address_arg.clone())
        .arg(method_arg.clone())
        .arg(params_arg.clone())
        .arg(abi_arg.clone())
        .arg(full_trace_arg.clone())
        .arg(decode_abi_arg.clone())
        .arg(boc_arg.clone())
        .arg(Arg::with_name("TVC")
            .long("--tvc")
            .conflicts_with("BOC")
            .help("Flag that changes behavior of the command to work with the saved contract state (stateInit TVC)."))
        .arg(Arg::with_name("ACCOUNT_ADDRESS")
            .takes_value(true)
            .long("--tvc_address")
            .help("Account address for account constructed from TVC.")
            .requires("TVC"))
        .arg(Arg::with_name("NOW")
            .takes_value(true)
            .long("--now")
            .help("Now timestamp (in milliseconds) for execution. If not set it is equal to the current timestamp."))
        .arg(config_path_arg.clone());

    let call_cmd = run_cmd.clone().name("call")
        .about("Play call locally with trace")
        .arg(sign_arg.clone());

    SubCommand::with_name("debug")
        .about("Debug commands.")
        .subcommand(SubCommand::with_name("transaction")
            .about("Replay transaction with specified ID.")
            .arg(empty_config_arg.clone())
            .arg(config_save_path_arg.clone())
            .arg(contract_path_arg.clone())
            .arg(output_arg.clone())
            .arg(dbg_info_arg.clone())
            .arg(full_trace_arg.clone())
            .arg(decode_abi_arg.clone())
            .arg(tx_id_arg.clone())
            .arg(dump_config_arg.clone())
            .arg(dump_contract_arg.clone()))
        .subcommand(SubCommand::with_name("account")
            .about("Loads list of the last transactions for the specified account. User should choose which one to debug.")
            .arg(empty_config_arg.clone())
            .arg(config_save_path_arg.clone())
            .arg(contract_path_arg.clone())
            .arg(output_arg.clone())
            .arg(dbg_info_arg.clone())
            .arg(full_trace_arg.clone())
            .arg(decode_abi_arg.clone())
            .arg(address_arg.clone())
            .arg(dump_config_arg.clone())
            .arg(dump_contract_arg.clone()))
        .subcommand(SubCommand::with_name("replay")
            .about("Replay transaction on the saved account state.")
            .arg(output_arg.clone())
            .arg(dbg_info_arg.clone())
            .arg(full_trace_arg.clone())
            .arg(tx_id_arg.clone())
            .arg(config_path_arg.clone())
            .arg(decode_abi_arg.clone())
            .arg(Arg::with_name("UPDATE_STATE")
                .help("Update state of the contract.")
                .long("--update_state"))
            .arg(Arg::with_name("INPUT")
                .help("Path to the saved account state.")
                .required(true)
                .takes_value(true)))
        .subcommand(call_cmd)
        .subcommand(run_cmd)
}

pub async fn debug_command(matches: &ArgMatches<'_>, config: &Config) -> Result<(), String> {
    if let Some(matches) = matches.subcommand_matches("transaction") {
        return debug_transaction_command(matches, config, false).await;
    }
    if let Some(matches) = matches.subcommand_matches("account") {
        return debug_transaction_command(matches, config, true).await;
    }
    if let Some(matches) = matches.subcommand_matches("call") {
        return debug_call_command(matches, config, false).await;
    }
    if let Some(matches) = matches.subcommand_matches("run") {
        return debug_call_command(matches, config, true).await;
    }
    if let Some(matches) = matches.subcommand_matches("replay") {
        return replay_transaction_command(matches, config).await;
    }
    Err("unknown command".to_owned())
}

async fn debug_transaction_command(matches: &ArgMatches<'_>, config: &Config, is_account: bool) -> Result<(), String> {
    let trace_path = Some(matches.value_of("LOG_PATH").unwrap_or(DEFAULT_TRACE_PATH));
    let debug_info = matches.value_of("DBG_INFO").map(|s| s.to_string());
    let config_path = matches.value_of("CONFIG_PATH");
    let contract_path = matches.value_of("CONTRACT_PATH");
    let is_empty_config = matches.is_present("EMPTY_CONFIG");

    let (tx_id, address) = if !is_account {
        let tx_id = matches.value_of("TX_ID");
        if !config.is_json {
            print_args!(tx_id, trace_path, config_path, contract_path);
        }
        let address = query_address(tx_id.unwrap(), &config).await?;
        (tx_id.unwrap().to_string(), address)
    } else {
        let address = matches.value_of("ADDRESS");
        if !config.is_json {
            print_args!(address, trace_path, config_path, contract_path);
        }
        let address = address.unwrap();
        let transactions = query_transactions(address, &config).await?;
        let tr_id = choose_transaction(transactions)?;
        (tr_id, address.to_string())
    };

    let config_path = match config_path {
        Some(config_path) => {
            config_path
        },
        _ => {
            if !config.is_json {
                println!("Fetching config contract transactions...");
            }
            fetch(&config.url, CONFIG_ADDR, DEFAULT_CONFIG_PATH, is_empty_config, None, true).await?;
            DEFAULT_CONFIG_PATH
        }
    };
    let contract_path = match contract_path {
        Some(contract_path) => {
            contract_path
        },
        _ => {
            if !config.is_json {
                println!("Fetching contract transactions...");
            }
            fetch(&config.url, &address, DEFAULT_CONTRACT_PATH, false, None, true).await?;
            DEFAULT_CONTRACT_PATH
        }
    };

    let trace_path = trace_path.unwrap().to_string();
    let init_logger = || {
        log::set_max_level(log::LevelFilter::Trace);
        log::set_boxed_logger(
            Box::new(DebugLogger::new(trace_path.clone()))
        ).map_err(|e| format!("Failed to set logger: {}", e))?;
        Ok(())
    };

    let trace_level = if matches.is_present("FULL_TRACE") {
        TraceLevel::Full
    } else {
        TraceLevel::Minimal
    };

    let mut dump_mask = DUMP_NONE;
    if matches.is_present("DUMP_CONFIG") {
        dump_mask |= DUMP_CONFIG;
    }
    if matches.is_present("DUMP_CONTRACT") {
        dump_mask |= DUMP_ACCOUNT;
    }
    if !config.is_json {
        println!("Replaying the last transactions...");
    }
    let tr = replay(
        contract_path,
        config_path,
        &tx_id,
        false,
        trace_level,
        init_logger,
        debug_info,
        dump_mask,
        if is_empty_config { Some(&config) } else { None },
    ).await?;

    decode_messages(tr.out_msgs, load_decode_abi(matches, config)).await?;
    if !config.is_json {
        println!("Log saved to {}.", trace_path);
    }
    Ok(())
}

fn construct_bc_config(matches: &ArgMatches<'_>) -> Result<Option<BlockchainConfig>, String> {
    Ok(match matches.value_of("CONFIG_PATH") {
        Some(bc_config) => {
            let config_account = Account::construct_from_file(bc_config)
                .map_err(|e| format!("Failed to construct config account: {}", e))?;
            Some(construct_blockchain_config(&config_account)?)
        },
        None => None
    })
}

async fn replay_transaction_command(matches: &ArgMatches<'_>, config: &Config) -> Result<(), String> {
    let tx_id = matches.value_of("TX_ID");
    let config_path = matches.value_of("CONFIG_PATH");
    let debug_info = matches.value_of("DBG_INFO").map(|s| s.to_string());
    let is_full_trace = matches.is_present("FULL_TRACE");
    let output = Some(matches.value_of("LOG_PATH").unwrap_or(DEFAULT_TRACE_PATH));
    let input = matches.value_of("INPUT");
    let do_update = matches.is_present("UPDATE_STATE");

    if !config.is_json {
        print_args!(input, tx_id, output, config_path, debug_info);
    }

    let ton_client = create_client(&config)?;
    let trans = query_collection(
        ton_client.clone(),
        ParamsOfQueryCollection {
            collection: "transactions".to_owned(),
            filter: Some(json!({
                "id": {
                    "eq": tx_id.unwrap()
                },
            })),
            result: "lt block { start_lt } boc".to_string(),
            limit: Some(1),
            order: None,
        },
    ).await
        .map_err(|e| format!("Failed to query transaction: {}", e))?;

    if trans.result.is_empty() {
        return Err("Transaction with specified id was not found".to_string());
    }

    let trans = trans.result[0].clone();
    let block_lt = trans["block"]["start_lt"].as_str()
        .ok_or("Failed to parse block_lt.".to_string())?;
    let block_lt = u64::from_str_radix(&block_lt[2..], 16)
        .map_err(|e| format!("Failed to convert block_lt: {}", e))?;
    let boc = trans["boc"].as_str()
        .ok_or("Failed to parse boc.".to_string())?;

    let trans = Transaction::construct_from_base64(boc)
        .map_err(|e| format!("Failed to parse transaction: {}", e))?;

    let mut account = Account::construct_from_file(input.unwrap())
        .map_err(|e| format!("Failed to construct account from the file: {}", e))?
        .serialize()
        .map_err(|e| format!("Failed to serialize account: {}", e))?;

    let msg = trans.in_msg_cell().map(|c| Message::construct_from_cell(c)
        .map_err(|e| format!("failed to construct message: {}", e))).transpose()?;

    log::set_max_level(log::LevelFilter::Trace);
    log::set_boxed_logger(
        Box::new(DebugLogger::new(output.unwrap().to_string()))
    ).map_err(|e| format!("Failed to set logger: {}", e))?;

    let result_trans = execute_debug(
        construct_bc_config(matches)?,
        Some(ton_client),
        &mut account,
        msg.as_ref(),
        trans.now(),
        block_lt,
        trans.logical_time(),
        debug_info,
        is_full_trace,
        false
    ).await;

    if do_update && result_trans.is_ok() {
        Account::construct_from_cell(account.clone())
            .map_err(|e| format!("Failed to construct account: {}", e))?
            .write_to_file(input.unwrap())
            .map_err(|e| format!("Failed to save account state: {}", e))?;
        if !config.is_json {
            println!("Contract state was updated.");
        }
    }

    match result_trans {
        Ok(result_trans) => {
            decode_messages(result_trans.out_msgs,load_decode_abi(matches, config)).await?;
            if !config.is_json {
                println!("Execution finished.");
            }
        }
        Err(e) => {
            if !config.is_json {
                println!("Execution failed: {}", e);
            }
        }
    }
    if !config.is_json {
        println!("Log saved to {}", output.unwrap());
    }
    Ok(())
}

fn load_decode_abi(matches: &ArgMatches<'_>, config: &Config) -> Option<String> {
    let abi = matches.value_of("DECODE_ABI")
        .map(|s| s.to_owned())
        .or(abi_from_matches_or_config(matches, &config).ok());
    match abi {
        Some(path) => match std::fs::read_to_string(path) {
            Ok(res) => Some(res),
            Err(e) => {
                if !config.is_json {
                    println!("Failed to read abi: {}", e);
                }
                None
            }
        }
        _ => None
    }
}

async fn debug_call_command(matches: &ArgMatches<'_>, config: &Config, is_getter: bool) -> Result<(), String> {
    let input = matches.value_of("ADDRESS");
    let output = Some(matches.value_of("LOG_PATH").unwrap_or(DEFAULT_TRACE_PATH));
    let debug_info = matches.value_of("DBG_INFO").map(|s| s.to_string());
    let method = matches.value_of("METHOD");
    let params = matches.value_of("PARAMS");
    let sign = matches.value_of("SIGN")
        .map(|s| s.to_string())
        .or(config.keys_path.clone());
    let opt_abi = Some(abi_from_matches_or_config(matches, &config)?);
    let is_boc = matches.is_present("BOC");
    let is_tvc = matches.is_present("TVC");
    let params = Some(load_params(params.unwrap())?);

    if !config.is_json {
        print_args!(input, method, params, sign, opt_abi, output, debug_info);
    }

    let is_full_trace = matches.is_present("FULL_TRACE");
    let ton_client = create_client(&config)?;
    let input = input.unwrap();
    let account = if is_tvc {
        construct_account_from_tvc(input,
                                   matches.value_of("ACCOUNT_ADDRESS"),
                                   Some(u64::MAX))?
    } else if is_boc {
        Account::construct_from_file(input)
            .map_err(|e| format!(" failed to load account from the file {}: {}", input, e))?
    } else {
        let address = load_ton_address(input, &config)?;
        let account = query_account_field(ton_client.clone(), &address, "boc").await?;
        Account::construct_from_base64(&account)
            .map_err(|e| format!("Failed to construct account: {}", e))?
    };

    let abi = std::fs::read_to_string(&opt_abi.clone().unwrap())
        .map_err(|e| format!("failed to read ABI file: {}", e))?;
    let abi = load_abi(&abi)?;
    let params = serde_json::from_str(&params.unwrap())
        .map_err(|e| format!("params are not in json format: {}", e))?;

    let keys = sign.map(|k| load_keypair(&k)).transpose()?;

    let now = match matches.value_of("NOW") {
        Some(now) => u64::from_str_radix(now, 10)
            .map_err(|e| format!("Failed to convert now to u64: {}", e))?,
        _ => now_ms()
    };

    let header = FunctionHeader {
        expire: Some((now / 1000) as u32 + config.lifetime),
        time: Some(now),
        ..Default::default()
    };
    let call_set = CallSet {
        function_name: method.unwrap().to_string(),
        input: Some(params),
        header: Some(header)
    };
    let msg_params = ParamsOfEncodeMessage {
        abi,
        address: Some(format!("0:{}", "0".repeat(64))),  // TODO: add option or get from input
        call_set: Some(call_set),
        signer: if keys.is_some() {
            Signer::Keys { keys: keys.unwrap() }
        } else {
            Signer::None
        },
        ..Default::default()
    };

    let message = encode_message(
        ton_client.clone(),
        msg_params
    ).await
        .map_err(|e| format!("Failed to encode message: {}", e))?;

    let message = Message::construct_from_base64(&message.message)
        .map_err(|e| format!("Failed to construct message: {}", e))?;

    let mut acc_root = account.serialize()
        .map_err(|e| format!("Failed to serialize account: {}", e))?;

    let trace_path = output.unwrap().to_string();

    log::set_max_level(log::LevelFilter::Trace);
    log::set_boxed_logger(
        Box::new(DebugLogger::new(trace_path.clone()))
    ).map_err(|e| format!("Failed to set logger: {}", e))?;

    let trans = execute_debug(
        construct_bc_config(matches)?,
        Some(ton_client),
        &mut acc_root,
        Some(&message),
        (now / 1000) as u32,
        now,
        now,
        debug_info,
        is_full_trace,
        is_getter,
    ).await;

    let msg_string = match trans {
        Ok(trans) => {
            decode_messages(trans.out_msgs,load_decode_abi(matches, config)).await?;
            "Execution finished.".to_string()
        }
        Err(e) => {
            format!("Execution failed: {}", e)
        }
    };
    if !config.is_json {
        println!("{}", msg_string);
        println!("Log saved to {}", trace_path);
    } else {
        println!("{{}}");
    }
    Ok(())
}

use crate::decode::msg_printer::serialize_msg;

async fn decode_messages(msgs: OutMessages, abi: Option<String>) -> Result<(), String> {
    if !msgs.is_empty() {
        log::debug!(target: "executor", "Output messages:\n----------------");
    }
    let msgs = msgs.export_vector()
        .map_err(|e| format!("Failed to parse out messages: {}", e))?;

    for msg in msgs {

        let ser_msg = serialize_msg(&msg.0, abi.clone()).await
            .map_err(|e| format!("Failed to serialize message: {}", e))?;
        log::debug!(target: "executor", "\n{}\n", serde_json::to_string_pretty(&ser_msg)
            .map_err(|e| format!("Failed to serialize json: {}", e))?);
    }
    Ok(())
}

async fn query_address(tr_id: &str, config: &Config) -> Result<String, String> {
    let ton_client = create_client(config)?;
    let query_result = query_with_limit(
        ton_client,
        "transactions",
        json!({
            "id": {
                "eq": tr_id
            }
        }),
        "account_addr",
        None,
        Some(1)
    ).await
        .map_err(|e| format!("Failed to query address: {}", e))?;
    match query_result.len() {
        0 => Err("Transaction was not found".to_string()),
        _ => Ok(query_result[0]["account_addr"]
            .to_string()
            .trim_start_matches(|c| c == '"')
            .trim_end_matches(|c| c == '"')
            .to_string())
    }
}

struct TrDetails {
    transaction_id: String,
    timestamp: String,
    source_address: String,
    message_type: String
}

impl fmt::Display for TrDetails {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "\ttransaction_id: {}", self.transaction_id)?;
        writeln!(f, "\ttimestamp     : {}", self.timestamp)?;
        writeln!(f, "\tmessage_type  : {}", self.message_type)?;
        writeln!(f, "\tsource_address: {}", self.source_address)
    }
}

async fn query_transactions(address: &str, config: &Config) -> Result<Vec<TrDetails>, String> {
    let ton_client = create_client(config)?;
    let order = vec![OrderBy{ path: "lt".to_string(), direction: SortDirection::DESC }];
    let query_result = query_with_limit(
        ton_client,
        "transactions",
        json!({
            "account_addr": {
                "eq": address
            }
        }),
        "id now_string in_message { src msg_type_name }",
        Some(order),
        Some(TRANSACTION_QUANTITY)
    ).await
        .map_err(|e| format!("Failed to query address: {}", e))?;
    match query_result.len() {
        0 => Err("Transaction list is empty.".to_string()),
        _ => {
            Ok(query_result.iter().map(|query| {
                TrDetails{
                    transaction_id: query["id"].to_string(),
                    timestamp: query["now_string"].to_string(),
                    source_address: query["in_message"]["src"].to_string(),
                    message_type: query["in_message"]["msg_type_name"].to_string()
                }
            }).collect::<Vec<TrDetails>>())
        }
    }
}

fn choose_transaction(transactions: Vec<TrDetails>) -> Result<String, String> {
    println!("\n\nChoose transaction you want to debug:");
    for index in 1..=transactions.len() {
        println!("{}){}", index, transactions[index - 1]);
    }
    println!("\n\nEnter number of the chosen transaction (from 1 to {}):", transactions.len());
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let chosen: usize = input.trim().parse()
        .map_err(|e| format!("Failed to parse user input as integer: {}", e))?;
    if !(1..=transactions.len()).contains(&chosen) {
        return Err("Wrong transaction number".to_string());
    }
    Ok(transactions[chosen-1].transaction_id.trim_start_matches(|c| c == '"').trim_end_matches(|c| c == '"').to_string())
}

pub async fn execute_debug(
    bc_config: Option<BlockchainConfig>,
    ton_client: Option<TonClient>,
    account: &mut Cell,
    message: Option<&Message>,
    block_unixtime: u32,
    block_lt: u64,
    last_tr_lt: u64,
    dbg_info: Option<String>,
    is_full_trace: bool,
    is_getter: bool,
) -> Result<Transaction, String> {
    let bc_config = match bc_config {
        Some(bc_config) => bc_config,
        None => {
            let config_boc = query_account_field(
                ton_client.unwrap(),
                CONFIG_ADDR,
                "boc",
            ).await?;
            let config_account = Account::construct_from_base64(&config_boc)
                .map_err(|e| format!("failed to construct config account: {}", e))?;
            construct_blockchain_config(&config_account)?
        }
    };

    let executor = Box::new(
        DebugTransactionExecutor::new(
            bc_config,
            dbg_info,
            if is_full_trace {
                TraceLevel::Full
            } else {
                TraceLevel::Minimal
            },
            is_getter
        )
    );
    let params = ExecuteParams {
        state_libs: HashmapE::default(),
        block_unixtime,
        block_lt,
        last_tr_lt: Arc::new(AtomicU64::new(last_tr_lt)),
        seed_block: UInt256::default(),
        debug: true,
        ..ExecuteParams::default()
    };

    executor.execute_with_libs_and_params(
        message,
         account,
        params
    ).map_err(|e| format!("Debug failed: {}", e))
}