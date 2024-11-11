/*
 * Copyright 2018-2023 EverX.
 *
 * Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
 * this file except in compliance with the License.
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific EVERX DEV software governing permissions and
 * limitations under the License.
 */

use crate::call::print_json_result;
use crate::config::{Config, FullConfig};
use crate::debug::{debug_error, init_debug_logger, DebugParams};
use crate::helpers::{
    abi_from_matches_or_config, contract_data_from_matches_or_config_alias, create_client,
    create_client_local, create_client_verbose, get_blockchain_config, load_abi, load_account,
    load_params, now, now_ms, unpack_alternative_params, AccountSource, TonClient,
};
use crate::message::prepare_message;
use crate::replay::construct_blockchain_config;
use clap::ArgMatches;
use ever_abi::token::Tokenizer;
use ever_abi::Function;
use ever_block::{Account, Deserializable, Serializable};
use ever_client::abi::FunctionHeader;
use ever_client::abi::{Abi, StackItemToJson, TokenValueToStackItem};
use ever_client::tvm::{
    run_get, run_solidity_getter, run_tvm, ExecutionOptions, ParamsOfRunGet, ParamsOfRunTvm,
};
use ever_vm::stack::integer::IntegerData;
use ever_vm::stack::StackItem;
use serde_json::{Map, Value};

fn get_address_and_abi_path(
    matches: &ArgMatches<'_>,
    full_config: &FullConfig,
    is_alternative: bool,
) -> Result<(String, String), String> {
    Ok(if is_alternative {
        let contract_data = contract_data_from_matches_or_config_alias(matches, full_config)?;
        (contract_data.address.unwrap(), contract_data.abi.unwrap())
    } else {
        (
            matches.value_of("ADDRESS").unwrap().to_string(),
            abi_from_matches_or_config(matches, &full_config.config)?,
        )
    })
}

fn get_method(
    matches: &ArgMatches<'_>,
    conf: &Config,
    is_alternative: bool,
) -> Result<String, String> {
    Ok(if is_alternative {
        matches
            .value_of("METHOD")
            .or(conf.method.as_deref())
            .ok_or("Method is not defined. Supply it in the config file or command line.")?
            .to_string()
    } else {
        matches.value_of("METHOD").unwrap().to_string()
    })
}

pub fn get_account_source(matches: &ArgMatches<'_>) -> AccountSource {
    if matches.is_present("TVC") {
        AccountSource::Tvc
    } else if matches.is_present("BOC") {
        AccountSource::Boc
    } else {
        AccountSource::Network
    }
}

pub async fn run_command(
    matches: &ArgMatches<'_>,
    full_config: &FullConfig,
    is_alternative: bool,
) -> Result<(), String> {
    let config = &full_config.config;
    let (address, abi_path) = get_address_and_abi_path(matches, full_config, is_alternative)?;
    let account_source = get_account_source(matches);

    let method = get_method(matches, &full_config.config, is_alternative)?;
    let trace_path;
    let ton_client = if account_source == AccountSource::Network {
        trace_path = format!("run_{}_{}.log", address, method);
        create_client(config)?
    } else {
        trace_path = "trace.log".to_string();
        create_client_local()?
    };

    let (account, account_boc) = load_account(
        &account_source,
        &address,
        Some(ton_client.clone()),
        config,
        matches.value_of("ADDR"),
    )
    .await?;
    let address = match account_source {
        AccountSource::Network => address,
        AccountSource::Boc => account.get_addr().unwrap().to_string(),
        AccountSource::Tvc => {
            if let Some(addr) = account.get_addr() {
                addr.to_string()
            } else {
                "0".repeat(64)
            }
        }
    };
    run(
        matches,
        config,
        ton_client,
        &address,
        account_boc,
        abi_path,
        is_alternative,
        trace_path,
    )
    .await
}

async fn run(
    matches: &ArgMatches<'_>,
    config: &Config,
    ton_client: TonClient,
    address: &str,
    account_boc: String,
    abi_path: String,
    is_alternative: bool,
    trace_path: String,
) -> Result<(), String> {
    let abi = load_abi(&abi_path, config).await?;
    let contract = abi.abi().map_err(|e| format!("{}", e))?;

    let method = get_method(matches, config, is_alternative)?;
    let params = if is_alternative {
        unpack_alternative_params(matches, &abi_path, &method, config).await?
    } else {
        matches.value_of("PARAMS").unwrap().to_string()
    };
    let params = load_params(&params)?;

    if let Ok(f) = contract.getter(method.as_str()) {
        return run_sol_getter(
            matches,
            config,
            ton_client,
            account_boc,
            method.as_str(),
            f,
            params,
            &abi,
        )
        .await;
    }

    let bc_config = matches.value_of("BCCONFIG");

    if !config.is_json {
        println!("Running get-method...");
    }

    let expire_at = config.lifetime + now();
    let header = FunctionHeader {
        expire: Some(expire_at),
        ..Default::default()
    };

    let msg = prepare_message(
        ton_client.clone(),
        address,
        abi.clone(),
        &method,
        &params,
        header,
        None,
        config.is_json,
        None,
    )
    .await?;

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
    )
    .await;

    let result = match result {
        Ok(result) => result,
        Err(e) => {
            let bc_config = get_blockchain_config(config, bc_config).await?;
            let now = now_ms();
            let debug_params = DebugParams {
                account: &account_boc,
                message: Some(&msg.message),
                time_in_ms: now,
                block_lt: now,
                last_tr_lt: now,
                is_getter: true,
                ..DebugParams::new(config, bc_config)
            };
            init_debug_logger(&trace_path)?;
            debug_error(&e, debug_params).await?;
            return Err(format!("{:#}", e));
        }
    };
    if !config.is_json {
        println!("Succeeded.");
    }
    if !result.out_messages.is_empty() {
        let res = result.decoded.and_then(|d| d.output);
        match res {
            Some(data) => {
                print_json_result(data, config)?;
            }
            None => {
                println!("Failed to decode output messages. Check that abi matches the contract.");
                println!("Messages in base64:\n{:?}", result.out_messages);
            }
        }
    }
    Ok(())
}

async fn run_sol_getter(
    matches: &ArgMatches<'_>,
    config: &Config,
    ton_client: TonClient,
    account_boc: String,
    method: &str,
    function: &Function,
    params: String,
    abi: &Abi,
) -> Result<(), String> {
    if !config.is_json {
        println!("Running get-method...");
    }

    let bc_config = matches.value_of("BCCONFIG");
    let js_params: Value = serde_json::from_str(&params).map_err(|e| format!("{}", e))?;
    let input_tokens = Tokenizer::tokenize_all_params(function.input_params(), &js_params)
        .map_err(|e| format!("{}", e))?;
    let mut stack_items: Vec<StackItem> = vec![];
    let abi_version = *abi.abi().unwrap().version();
    for token in input_tokens {
        let item = TokenValueToStackItem::convert_token_to_vm_type(token.value, &abi_version)
            .map_err(|e| e.to_string())?;
        stack_items.push(item);
    }
    let crc = ever_client::crypto::ton_crc16_from_raw_data(method.as_bytes().to_vec());
    let function_id = ((crc as u32) & 0xffff) | 0x10000;
    stack_items.push(ever_vm::int!(function_id));

    let execution_options = prepare_execution_options(bc_config)?;
    let result = run_solidity_getter(
        ton_client.clone(),
        ParamsOfRunTvm {
            account: account_boc.clone(),
            execution_options,
            ..Default::default()
        },
        stack_items,
    )
    .await;

    let out_stack_items = match result {
        Ok(items) => items,
        Err(e) => {
            return Err(format!("{:#}", e));
        }
    };
    let js_result = StackItemToJson::convert_vm_items_to_json(
        &out_stack_items,
        &function.outputs,
        &abi_version,
    )
    .map_err(|e| e.to_string())?;
    if !config.is_json {
        println!("Succeeded.");
    }
    println!("{}", js_result);
    Ok(())
}

fn prepare_execution_options(bc_config: Option<&str>) -> Result<Option<ExecutionOptions>, String> {
    if let Some(config) = bc_config {
        let mut bytes = std::fs::read(config)
            .map_err(|e| format!("Failed to read data from file {config}: {e}"))?;
        let cell = ever_block::read_single_root_boc(&bytes)
            .map_err(|e| format!("Failed to deserialize {config}: {e}"))?;
        if let Ok(acc) = Account::construct_from_cell(cell.clone()) {
            let config = construct_blockchain_config(&acc)?;
            bytes = config
                .raw_config()
                .write_to_bytes()
                .map_err(|e| format!("Failed to serialize config params: {e}"))?;
        }
        let blockchain_config = Some(base64::encode(bytes));
        let ex_opt = ExecutionOptions {
            blockchain_config,
            ..Default::default()
        };
        return Ok(Some(ex_opt));
    }
    Ok(None)
}

pub async fn run_get_method(
    config: &Config,
    addr: &str,
    method: &str,
    params: Option<String>,
    source_type: AccountSource,
    bc_config: Option<&str>,
) -> Result<(), String> {
    let ton = if source_type == AccountSource::Network {
        create_client_verbose(config)?
    } else {
        create_client_local()?
    };

    let (_, acc_boc) = load_account(&source_type, addr, Some(ton.clone()), config, None).await?;

    let params = params
        .map(|p| serde_json::from_str(&p))
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
    )
    .await
    .map_err(|e| format!("run failed: {}", e))?
    .output;

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
            }
            _ => {
                res.insert("value0".to_owned(), result);
            }
        }
        let res = Value::Object(res);
        println!("{:#}", res);
    }
    Ok(())
}
