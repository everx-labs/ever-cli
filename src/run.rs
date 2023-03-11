/*
 * Copyright 2018-2023 EverX
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

use clap::ArgMatches;
use serde_json::{Map, Value, json};
use ton_block::{Account, Deserializable, Message, Serializable};
use ton_client::abi::{FunctionHeader};
use ton_client::tvm::{ExecutionOptions, ParamsOfRunGet, ParamsOfRunTvm, run_get, run_tvm};
use crate::RUNTIME;
use crate::config::{Config, FullConfig};
use crate::call::print_json_result;
use crate::debug::{execute_debug, DebugLogger};
use crate::helpers::{create_client, now, now_ms, SDK_EXECUTION_ERROR_CODE, TonClient,
                     contract_data_from_matches_or_config_alias, abi_from_matches_or_config,
                     AccountSource, create_client_local, create_client_verbose, load_abi,
                     load_account, load_params, unpack_alternative_params, get_blockchain_config};
use crate::message::prepare_message;

pub fn run_command(matches: &ArgMatches<'_>, full_config: &FullConfig, is_alternative: bool) -> Result<(), String> {
    let config = &full_config.config;
    let (address, abi_path) = if is_alternative {
        let (address,abi, _) = contract_data_from_matches_or_config_alias(matches, full_config)?;
        (address.unwrap(), abi.unwrap())
    } else {
        (
            matches.value_of("ADDRESS").unwrap().to_string(),
            abi_from_matches_or_config(matches, config)?
        )
    };
    let account_source = if matches.is_present("TVC") {
        AccountSource::Tvc
    } else if matches.is_present("BOC") {
        AccountSource::Boc
    } else {
        AccountSource::Network
    };

    let ton_client = if account_source == AccountSource::Network {
        if &config.debug_fail != "None" {
            let method = if is_alternative {
                matches.value_of("METHOD").or(config.method.as_deref())
                    .ok_or("Method is not defined. Supply it in the config file or command line.")?
            } else {
                matches.value_of("METHOD").unwrap()
            };
            let log_path = format!("run_{}_{}.log", address, method);
            log::set_max_level(log::LevelFilter::Trace);
            log::set_boxed_logger(
                Box::new(DebugLogger::new(log_path))
            ).map_err(|e| format!("Failed to set logger: {}", e))?;
            create_client(config)?
        } else {
            create_client_verbose(config)?
        }
    } else {
        create_client_local()?
    };

    let (account, account_boc) = load_account(
        &account_source,
        &address,
        Some(ton_client.clone()),
        config
    )?;
    let address = match account_source {
        AccountSource::Network => address,
        AccountSource::Boc => account.get_addr().unwrap().to_string(),
        AccountSource::Tvc => "0".repeat(64)
    };
    run(matches, config, Some(ton_client), &address, &account_boc, &abi_path, is_alternative)
}

fn run(
    matches: &ArgMatches<'_>,
    config: &Config,
    ton_client: Option<TonClient>,
    address: &str,
    account_boc: &str,
    abi_path: &str,
    is_alternative: bool,
) -> Result<(), String> {
    let method = if is_alternative {
        matches.value_of("METHOD").or(config.method.as_deref())
        .ok_or("Method is not defined. Supply it in the config file or command line.")?
    } else {
        matches.value_of("METHOD").unwrap()
    };
println!("55");
    let bc_config = matches.value_of("BCCONFIG");
    let execution_options = prepare_execution_options(bc_config)?;
    let bc_config = get_blockchain_config(config, None)?;

    if !config.is_json {
        println!("Running get-method...");
    }
    let ton_client = match ton_client {
        Some(ton_client) => { ton_client },
        None => {
            create_client_local()?
        }
    };

    let abi = RUNTIME.block_on(async move {
        load_abi(abi_path, config).await
    })?;
    let params = if is_alternative {
        unpack_alternative_params(matches, abi_path, method, config)?
    } else {
        matches.value_of("PARAMS").map(|s| s.to_owned())
    };
    let params = Some(load_params(params.unwrap().as_ref())?);

    let now = now();
    let expire_at = config.lifetime + now;
    let header = FunctionHeader {
        expire: Some(expire_at),
        ..Default::default()
    };

    let msg = prepare_message(
        ton_client.clone(),
        address,
        abi.clone(),
        method,
        &params.unwrap(),
        Some(header),
        None,
        config.is_json,
    )?;

    let message = msg.message.clone();
    let result = RUNTIME.block_on(async move { run_tvm(
        ton_client.clone(),
        ParamsOfRunTvm {
            message,
            account: account_boc.to_string(),
            abi: Some(abi.clone()),
            return_updated_account: Some(true),
            execution_options,
            ..Default::default()
        }
    ).await });

    if &config.debug_fail != "None" && result.is_err()
        && result.clone().err().unwrap().code == SDK_EXECUTION_ERROR_CODE {
        // TODO: add code to use bc_config from file

        if config.is_json {
            let e = format!("{:#}", result.clone().err().unwrap());
            let err: Value = serde_json::from_str(&e)
                .unwrap_or(Value::String(e));
            let res = json!({"Error": err});
            println!("{}", serde_json::to_string_pretty(&res)
                .unwrap_or_else(|_| "{{ \"JSON serialization error\" }}".to_string()));
        } else {
            println!("Error: {:#}", result.clone().err().unwrap());
            println!("Execution failed. Starting debug...");
        }

        let mut account = Account::construct_from_base64(&account_boc)
            .map_err(|e| format!("Failed to construct account: {}", e))?
            .serialize()
            .map_err(|e| format!("Failed to serialize account: {}", e))?;

        let now = now_ms();
        let message = Message::construct_from_base64(&msg.message)
            .map_err(|e| format!("failed to construct message: {}", e))?;
        let result = execute_debug(
            bc_config,
            &mut account,
            Some(&message),
            (now / 1000) as u32,
            now,
            now,
            true,
            config
        );
        if let Err(e) = result {
            if !e.contains("Contract did not accept message") {
                return Err(e);
            }
        }

        if !config.is_json {
            let log_path = format!("run_{}_{}.log", address, method);
            println!("Debug finished.");
            println!("Log saved to {}", log_path);
        }
        return Err("".to_string());
    }

    let result = result.map_err(|e| format!("{:#}", e))?;
    if !config.is_json {
        println!("Succeeded.");
    }
    if !result.out_messages.is_empty() {
        let res = result.decoded.and_then(|d| d.output);
        match res {
            Some(data) => {
                print_json_result(data, config)?;
            },
            None => {
                println!("Failed to decode output messages. Check that abi matches the contract.");
                println!("Messages in base64:\n{:?}", result.out_messages);
            }
        }
    }
    Ok(())
}

fn prepare_execution_options(bc_config: Option<&str>) -> Result<Option<ExecutionOptions>, String> {
    if let Some(config) = bc_config {
        let bytes = std::fs::read(config)
            .map_err(|e| format!("Failed to read data from file {}: {}", config, e))?;
        let config_boc = base64::encode(bytes);
        let ex_opt = ExecutionOptions{
            blockchain_config: Some(config_boc),
            ..Default::default()
        };
        return Ok(Some(ex_opt));
    }
    Ok(None)
}

pub fn run_get_method(config: &Config, addr: &str, method: &str, params: Option<String>, source_type: AccountSource, bc_config: Option<&str>) -> Result<(), String> {
    let ton = if source_type == AccountSource::Network {
        create_client_verbose(config)?
    } else {
        create_client_local()?
    };

    let (_, acc_boc) = load_account(&source_type, addr, Some(ton.clone()), config)?;

    let params = params.map(|p| serde_json::from_str(&p))
        .transpose()
        .map_err(|e| format!("arguments are not in json format: {}", e))?;

    if !config.is_json {
        println!("Running get-method...");
    }
    let execution_options = prepare_execution_options(bc_config)?;
    let function_name = method.to_owned();
    let result = RUNTIME.block_on(async move { run_get(
        ton,
        ParamsOfRunGet {
            account: acc_boc,
            function_name,
            input: params,
            execution_options,
            ..Default::default()
        },
    ).await }).map_err(|e| format!("run failed: {}", e))?.output;

    if !config.is_json {
        println!("Succeeded.");
        println!("Result: {}", result);
    } else {
        let mut res = Map::new();
        match result {
            Value::Array(array) => {
                for (i, val) in array.iter().enumerate() {
                    res.insert(format!("value{}", i), val.to_owned());
                }
            },
            _ => {
                res.insert("value0".to_owned(), result);
            }
        }
        let res = Value::Object(res);
        println!("{}", serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Undefined".to_string()));
    }
    Ok(())
}
