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
use crate::helpers::{load_ton_address, create_client, load_abi, now_ms, construct_account_from_tvc};
use crate::replay::{fetch, CONFIG_ADDR, replay, DUMP_NONE, DUMP_CONFIG, DUMP_ACCOUNT, construct_blockchain_config};
use std::io::Write;
use crate::call::{query_account_boc};
use ton_block::{Message, Account, Serializable, Deserializable};
use ton_types::{UInt256, HashmapE};
use ton_client::abi::{CallSet, Signer, FunctionHeader, encode_message, ParamsOfEncodeMessage};
use ton_executor::{ExecuteParams, BlockchainConfig, TransactionExecutor};
use std::sync::{Arc, atomic::AtomicU64};
use crate::crypto::load_keypair;
use crate::debug_executor::{DebugTransactionExecutor, TraceLevel};

const DEFAULT_TRACE_PATH: &'static str = "./trace.log";
const DEFAULT_CONFIG_PATH: &'static str = "config.txns";
const DEFAULT_CONTRACT_PATH: &'static str = "contract.txns";


struct DebugLogger {
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
        .help("Contract address.");

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

    let min_trace_arg = Arg::with_name("MIN_TRACE")
        .long("--min_trace")
        .help("Flag that changes trace to minimal version.");

    SubCommand::with_name("debug")
        .about("Debug commands.")
        .subcommand(SubCommand::with_name("transaction")
            .about("Replay transaction with specified ID.")
            .arg(Arg::with_name("EMPTY_CONFIG")
                .help("Replay transaction without full dump of the config contract.")
                .long("--empty_config")
                .short("-e"))
            .arg(Arg::with_name("CONFIG_PATH")
                .help("Path to the file with saved config contract transactions. If not set transactions will be fetched to file \"config.txns\".")
                .long("--config")
                .short("-c")
                .takes_value(true))
            .arg(Arg::with_name("CONTRACT_PATH")
                .help("Path to the file with saved target contract transactions. If not set transactions will be fetched to file \"contract.txns\".")
                .long("--contract")
                .short("-t")
                .takes_value(true))
            .arg(output_arg.clone())
            .arg(dbg_info_arg.clone())
            .arg(address_arg.clone())
            .arg(min_trace_arg.clone())
            .arg(Arg::with_name("TX_ID")
                .required(true)
                .takes_value(true)
                .help("ID of the transaction that should be replayed."))
            .arg(Arg::with_name("DUMP_CONFIG")
                .help("Dump the replayed config contract account state.")
                .long("--dump_config"))
            .arg(Arg::with_name("DUMP_CONTRACT")
                .help("Dump the replayed target contract account state.")
                .long("--dump_contract")))
        .subcommand(SubCommand::with_name("call")
            .about("Play call locally with trace")
            .arg(output_arg.clone())
            .arg(dbg_info_arg.clone())
            .arg(address_arg.clone()
                .help("Contract address or path the file with saved contract state if corresponding flag is used."))
            .arg(method_arg.clone())
            .arg(params_arg.clone())
            .arg(abi_arg.clone())
            .arg(sign_arg.clone())
            .arg(min_trace_arg.clone())
            .arg(Arg::with_name("BOC")
                .long("--boc")
                .conflicts_with("TVC")
                .help("Flag that changes behavior of the command to work with the saved account state (account BOC)."))
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
            .arg(Arg::with_name("CONFIG_PATH")
                .help("Path to the file with saved config contract state.")
                .long("--config")
                .short("-c")
                .takes_value(true)))
}

pub async fn debug_command(matches: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    if let Some(matches) = matches.subcommand_matches("transaction") {
        return debug_transaction_command(matches, config).await;
    }
    if let Some(matches) = matches.subcommand_matches("call") {
        return debug_call_command(matches, config).await;
    }
    Err("unknown command".to_owned())
}

async fn debug_transaction_command(matches: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let address = matches.value_of("ADDRESS");
    let tx_id = matches.value_of("TX_ID");
    let trace_path = Some(matches.value_of("LOG_PATH").unwrap_or(DEFAULT_TRACE_PATH));
    let debug_info = Some(matches.value_of("DBG_INFO").unwrap_or(""));
    let config_path = matches.value_of("CONFIG_PATH");
    let contract_path = matches.value_of("CONTRACT_PATH");
    if !config.is_json {
        print_args!(address, tx_id, trace_path, config_path, contract_path);
    }

    let is_empty_config = matches.is_present("EMPTY_CONFIG");

    let address = load_ton_address(address.unwrap(), &config)?;

    let config_path = match config_path {
        Some(config_path) => {
            config_path
        },
        _ => {
            println!("Fetching config contract transactions...");
            fetch(&config.url,CONFIG_ADDR, DEFAULT_CONFIG_PATH, is_empty_config).await?;
            DEFAULT_CONFIG_PATH
        }
    };
    let contract_path = match contract_path {
        Some(contract_path) => {
            contract_path
        },
        _ => {
            println!("Fetching contract transactions...");
            fetch(&config.url, &address, DEFAULT_CONTRACT_PATH, false).await?;
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

    let trace_level = if matches.is_present("MIN_TRACE") {
        TraceLevel::Minimal
    } else {
        TraceLevel::Full
    };

    let mut dump_mask = DUMP_NONE;
    if matches.is_present("DUMP_CONFIG") {
        dump_mask |= DUMP_CONFIG;
    }
    if matches.is_present("DUMP_CONTRACT") {
        dump_mask |= DUMP_ACCOUNT;
    }
    println!("Replaying the last transactions...");
    replay(contract_path, config_path, &tx_id.unwrap(),false, false, false, trace_level, init_logger, debug_info.unwrap(), dump_mask).await?;
    println!("Log saved to {}.", trace_path);
    Ok(())
}

async fn debug_call_command(matches: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let input = matches.value_of("ADDRESS");
    let output = Some(matches.value_of("LOG_PATH").unwrap_or(DEFAULT_TRACE_PATH));
    let debug_info = Some(matches.value_of("DBG_INFO").unwrap_or(""));
    let method = matches.value_of("METHOD");
    let params = matches.value_of("PARAMS");
    let sign = matches.value_of("SIGN")
        .map(|s| s.to_string())
        .or(config.keys_path.clone());
    let abi = Some(abi_from_matches_or_config(matches, &config)?);
    let is_boc = matches.is_present("BOC");
    let is_tvc = matches.is_present("TVC");
    let params = Some(load_params(params.unwrap())?);

    if !config.is_json {
        print_args!(input, method, params, sign, abi, output, debug_info);
    }

    let is_min_trace = matches.is_present("MIN_TRACE");
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
        let account = query_account_boc(ton_client.clone(), &address).await?;
        Account::construct_from_base64(&account)
            .map_err(|e| format!("Failed to construct account: {}", e))?
    };

    let abi = std::fs::read_to_string(&abi.unwrap())
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

    let header = FunctionHeader { // TODO: add options or now + config.lifetime
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
        address: Some(format!("0:{}", std::iter::repeat("0").take(64).collect::<String>())),  // TODO: add option or get from input
        call_set: Some(call_set),
        signer: if keys.is_some() {
            Signer::Keys { keys: keys.unwrap() }
        } else {
            Signer::None
        },
        ..Default::default()
    };

    let message = encode_message(
        ton_client,
        msg_params
    ).await
        .map_err(|e| format!("Failed to encode message: {}", e))?;

    let message = Message::construct_from_base64(&message.message)
        .map_err(|e| format!("Failed to construct message: {}", e))?;

    let params = ExecuteParams {
        state_libs: HashmapE::default(),
        block_unixtime: (now / 1000) as u32,
        block_lt: now,
        last_tr_lt: Arc::new(AtomicU64::new(now)),
        seed_block: UInt256::default(),
        debug: true,
    };

    let config = match matches.value_of("CONFIG_PATH") {
        Some(config) => {
            let account = Account::construct_from_file(config)
                .map_err(|e| format!("Failed to construct config account: {}", e))?;
            construct_blockchain_config(&account)?
        }
        _ => { BlockchainConfig::default() }
    };

    let executor =
        Box::new(
            DebugTransactionExecutor::new(
                config,
                debug_info.unwrap(),
                if is_min_trace {
                    TraceLevel::Minimal
                } else {
                    TraceLevel::Full
                }
            )
        );

    let mut acc_root = account.serialize()
        .map_err(|e| format!("Failed to serialize account: {}", e))?;

    let trace_path = output.unwrap().to_string();

    log::set_max_level(log::LevelFilter::Trace);
    log::set_boxed_logger(
        Box::new(DebugLogger::new(trace_path.clone()))
    ).map_err(|e| format!("Failed to set logger: {}", e))?;

    let _ = executor.execute_with_libs_and_params(
        Some(&message),
        &mut acc_root,
        params
    );

    println!("Execution finished.");
    println!("Log saved to {}", trace_path);
    Ok(())
}
