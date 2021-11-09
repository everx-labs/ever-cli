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
use crate::{print_args, VERBOSE_MODE};
use clap::{ArgMatches, SubCommand, Arg, App};
use crate::config::Config;
use crate::helpers::{
    load_ton_address,
};
use crate::replay::{fetch, CONFIG_ADDR, replay};
use std::io::Write;

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
    SubCommand::with_name("debug")
        .about("Debug commands.")
        .subcommand(SubCommand::with_name("transaction")
            .about("Replay transaction with specified ID.")
            .arg(Arg::with_name("EMPTY_CONFIG")
                .help("Replay transaction without full dump of the config contract.")
                .long("--empty-config"))
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
            // .arg(Arg::with_name("LOCAL")
            //     .long("--local")
            //     .help("Flag that changes behavior of the command to work with the saved account state (account BOC)."))
            .arg(Arg::with_name("LOG_PATH")
                .help("Path where to store the trace. Default path is \"./trace.log\". Note: old file will be removed.")
                .takes_value(true)
                .long("--output")
                .short("-o"))
            .arg(Arg::with_name("ADDRESS")
                .required(true)
                .takes_value(true)
                .help("Contract address or path to the saved account state if --local flag is specified."))
            .arg(Arg::with_name("TX_ID")
                .required(true)
                .takes_value(true)
                .help("ID of the transaction that should be replayed.")))
}

pub async fn debug_command(matches: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    if let Some(matches) = matches.subcommand_matches("transaction") {
        return debug_transaction_command(matches, config).await;
    }
    Err("unknown command".to_owned())
}

async fn debug_transaction_command(matches: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let address = matches.value_of("ADDRESS");
    let tx_id = matches.value_of("TX_ID");
    let trace_path = Some(matches.value_of("LOG_PATH").unwrap_or(DEFAULT_TRACE_PATH));
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

    println!("Replaying the last transactions...");
    replay(contract_path, config_path, &tx_id.unwrap(),false, false, false, true, init_logger).await?;
    println!("Log saved to {}.", trace_path);
    Ok(())
}

