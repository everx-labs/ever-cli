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

use clap::ArgMatches;
use serde_json::{Map, Value};
use ton_block::{Account, Deserializable, Message, Serializable};
use ton_client::abi::{FunctionHeader};
use ton_client::tvm::{ExecutionOptions, ParamsOfRunGet, ParamsOfRunTvm, run_get, run_tvm};
use crate::{abi_from_matches_or_config, AccountSource, Config, create_client_local, create_client_verbose, DebugLogger, load_abi, load_account, load_params, unpack_alternative_params};
use crate::call::{print_json_result};
use crate::debug::execute_debug;
use crate::helpers::{create_client, now, now_ms, SDK_EXECUTION_ERROR_CODE, TonClient, TRACE_PATH};
use crate::message::prepare_message;

pub async fn run_command(matches: &ArgMatches<'_>, config: &Config, is_alternative: bool) -> Result<(), String> {
    let address = if is_alternative {
        matches.value_of("ADDRESS")
            .map(|s| s.to_string())
            .or(config.addr.clone())
            .ok_or("ADDRESS is not defined. Supply it in the config file or command line.".to_string())?
    } else {
        matches.value_of("ADDRESS").unwrap().to_string()
    };
    let account_source = if matches.is_present("TVC") {
        AccountSource::TVC
    } else if matches.is_present("BOC") {
        AccountSource::BOC
    } else {
        AccountSource::NETWORK
    };

    let ton_client = if account_source == AccountSource::NETWORK {
        if config.debug_fail != "None".to_string() {
            log::set_max_level(log::LevelFilter::Trace);
            log::set_boxed_logger(
                Box::new(DebugLogger::new(TRACE_PATH.to_string()))
            ).map_err(|e| format!("Failed to set logger: {}", e))?;
            create_client(&config)?
        } else {
            create_client_verbose(&config)?
        }
    } else {
        create_client_local()?
    };

    let (account, account_boc) = load_account(
        &account_source,
        &address,
        Some(ton_client.clone()),
        &config
    ).await?;
    let address = match account_source {
        AccountSource::NETWORK => address,
        AccountSource::BOC => account.get_addr().unwrap().to_string(),
        AccountSource::TVC => std::iter::repeat("0").take(64).collect::<String>()
    };
    run(matches, config, Some(ton_client), &address, account_boc, is_alternative).await
}

pub async fn run(
    matches: &ArgMatches<'_>,
    config: &Config,
    ton_client: Option<TonClient>,
    address: &str,
    account_boc: String,
    is_alternative: bool,
) -> Result<(), String> {
    let method = if is_alternative {
        matches.value_of("METHOD").or(config.method.as_deref())
        .ok_or("Method is not defined. Supply it in the config file or command line.")?
    } else {
        matches.value_of("METHOD").unwrap()
    };
    let abi_path = abi_from_matches_or_config(matches, &config)?;
    let bc_config = matches.value_of("BCCONFIG");

    if !config.is_json {
        println!("Running get-method...");
    }
    let ton_client = match ton_client {
        Some(ton_client) => { ton_client },
        None => {
            create_client_local()?
        }
    };
    let abi = std::fs::read_to_string(abi_path.clone())
        .map_err(|e| format!("failed to read ABI file: {}", e.to_string()))?;

    let params = if is_alternative {
        unpack_alternative_params(matches, &abi, method, config)?
    } else {
        matches.value_of("PARAMS").map(|s| s.to_owned())
    };

    let params = Some(load_params(params.unwrap().as_ref())?);

    let abi = load_abi(&abi)?;
    let now = now()?;
    let expire_at = config.lifetime + now;
    let header = FunctionHeader {
        expire: Some(expire_at),
        ..Default::default()
    };

    let msg = prepare_message(
        ton_client.clone(),
        &address,
        abi.clone(),
        method,
        &params.unwrap(),
        Some(header),
        None,
        config.is_json,
    ).await?;

    let execution_options = prepare_execution_options(bc_config)?;
    let result = run_tvm(
        ton_client.clone(),
        ParamsOfRunTvm {
            message: msg.message.clone(),
            account: account_boc.clone(),
            abi: Some(abi.clone()),
            return_updated_account: Some(true),
            execution_options,
            ..Default::default()
        },
    ).await;

    if config.debug_fail != "None".to_string() && result.is_err()
        && result.clone().err().unwrap().code == SDK_EXECUTION_ERROR_CODE {
        // TODO: add code to use bc_config from file

        if config.is_json {
            let e = format!("{:#}", result.clone().err().unwrap());
            let err: serde_json::Value = serde_json::from_str(&e)
                .unwrap_or(serde_json::Value::String(e));
            let res = json!({"Error": err});
            println!("{}", serde_json::to_string_pretty(&res)
                .unwrap_or("{{ \"JSON serialization error\" }}".to_string()));
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
        match execute_debug(
            Some(matches),
            Some(ton_client),
            &mut account,
            Some(&message),
            (now / 1000) as u32,
            now,
            now,
            true,
            config
        ).await {
            Err(e) => {
                if !e.contains("Contract did not accept message") {
                    return Err(e);
                }
            },
            Ok(_) => {}
        }

        if !config.is_json {
            println!("Debug finished.");
            println!("Log saved to {}", TRACE_PATH);
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
        let config_boc = base64::encode(&bytes);
        let ex_opt = ExecutionOptions{
            blockchain_config: Some(config_boc),
            ..Default::default()
        };
        return Ok(Some(ex_opt));
    }
    Ok(None)
}

pub async fn run_get_method(config: &Config, addr: &str, method: &str, params: Option<String>, source_type: AccountSource, bc_config: Option<&str>) -> Result<(), String> {
    let ton = if source_type == AccountSource::NETWORK {
        create_client_verbose(&config)?
    } else {
        create_client_local()?
    };

    let (_, acc_boc) = load_account(&source_type, addr, Some(ton.clone()), config).await?;

    let params = params.map(|p| serde_json::from_str(&p))
        .transpose()
        .map_err(|e| format!("arguments are not in json format: {}", e))?;

    if !config.is_json {
        println!("Running get-method...");
    }
    let execution_options = prepare_execution_options(bc_config)?;
    let result = run_get(
        ton,
        ParamsOfRunGet {
            account: acc_boc,
            function_name: method.to_owned(),
            input: params,
            execution_options,
            ..Default::default()
        },
    ).await
        .map_err(|e| format!("run failed: {}", e.to_string()))?
        .output;

    if !config.is_json {
        println!("Succeeded.");
        println!("Result: {}", result);
    } else {
        let mut res = Map::new();
        match result {
            Value::Array(array) => {
                let mut i = 0;
                for val in array.iter() {
                    res.insert(format!("value{}", i), val.to_owned());
                    i = 1 + i;
                }
            },
            _ => {
                res.insert("value0".to_owned(), result);
            }
        }
        let res = Value::Object(res);
        println!("{}", serde_json::to_string_pretty(&res).unwrap_or("Undefined".to_string()));
    }
    Ok(())
}
