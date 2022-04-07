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
use ton_block::{Account, Deserializable, Serializable};
use ton_client::abi::{Abi, FunctionHeader};
use ton_client::tvm::{ExecutionOptions, ParamsOfRunGet, ParamsOfRunTvm, run_get, run_tvm};
use crate::{abi_from_matches_or_config, AccountSource, Config, create_client_local, create_client_verbose, DebugLogger, load_abi, load_account, unpack_alternative_params};
use crate::call::{print_json_result};
use crate::debug::execute_debug;
use crate::helpers::{create_client, now, now_ms, query_account_field, SDK_EXECUTION_ERROR_CODE, TonClient, TRACE_PATH};
use crate::message::prepare_message;
use crate::replay::{CONFIG_ADDR, construct_blockchain_config};

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

    let ton_client = if config.debug_fail {
        log::set_max_level(log::LevelFilter::Trace);
        log::set_boxed_logger(
            Box::new(DebugLogger::new(TRACE_PATH.to_string()))
        ).map_err(|e| format!("Failed to set logger: {}", e))?;
        create_client(&config)?
    } else {
        create_client_verbose(&config)?
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
    let method = matches.value_of("METHOD").unwrap();
    let abi = abi_from_matches_or_config(matches, &config)?;
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
    let abi = std::fs::read_to_string(abi)
        .map_err(|e| format!("failed to read ABI file: {}", e.to_string()))?;

    let params = if is_alternative {
        unpack_alternative_params(matches, &abi, method)?
    } else {
        matches.value_of("PARAMS").map(|s| s.to_string())
    };

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

    let res = run_local(
        ton_client,
        abi,
        msg.message,
        account_boc,
        bc_config,
        config,
    ).await?;

    if !config.is_json {
        println!("Succeeded.");
    }

    print_json_result(res, config)?;
    Ok(())
}

pub async fn run_local(
    ton: TonClient,
    abi: Abi,
    msg: String,
    acc_boc: String,
    bc_config: Option<&str>,
    config: &Config,
) -> Result<serde_json::Value, String> {
    let execution_options = prepare_execution_options(bc_config)?;
    let result = run_tvm(
        ton.clone(),
        ParamsOfRunTvm {
            message: msg.clone(),
            account: acc_boc.clone(),
            abi: Some(abi.clone()),
            return_updated_account: Some(true),
            execution_options,
            ..Default::default()
        },
    ).await;
    if config.debug_fail && result.is_err()
        && result.clone().err().unwrap().code == SDK_EXECUTION_ERROR_CODE {
        if !config.is_json {
            println!("Execution failed. Starting debug...");
        }
        let account = Account::construct_from_base64(&acc_boc)
            .map_err(|e| format!("Failed to construct account: {}", e))?
            .serialize()
            .map_err(|e| format!("Failed to serialize account: {}", e))?;

        let config_acc = query_account_field(
            ton.clone(),
            CONFIG_ADDR,
            "boc",
        ).await?;

        let config_acc = Account::construct_from_base64(&config_acc)
            .map_err(|e| format!("Failed to construct config account: {}", e))?;
        let bc_config = construct_blockchain_config(&config_acc)?;
        let now = now_ms();
        let _ = execute_debug(bc_config, account, msg, now, now, true)?;

        if !config.is_json {
            println!("Debug finished.");
            println!("Log saved to {}", TRACE_PATH);
        }
    }
    let result = result.map_err(|e| format!("{:#}", e))?;
    let res = result.decoded.and_then(|d| d.output)
        .ok_or("Failed to decode the result. Check that abi matches the contract.")?;
    Ok(res)
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
    let ton = if source_type != AccountSource::NETWORK {
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
