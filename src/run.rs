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

use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use clap::ArgMatches;
use ton_block::{Account, Deserializable, Message, Serializable};
use ton_client::abi::{Abi, FunctionHeader};
use ton_client::tvm::{ParamsOfRunTvm, run_tvm};
use ton_executor::{ExecuteParams, TransactionExecutor};
use ton_types::{HashmapE, UInt256};
use crate::{abi_from_matches_or_config, AccountSource, Config, create_client_local, create_client_verbose, DebugLogger, load_abi, load_account, unpack_alternative_params};
use crate::call::{prepare_execution_options, print_json_result};
use crate::debug_executor::{DebugTransactionExecutor, TraceLevel};
use crate::helpers::{now, now_ms, query_account_field, SDK_EXECUTION_ERROR_CODE, TonClient, TRACE_PATH};
use crate::message::prepare_message;
use crate::replay::{CONFIG_ADDR, construct_blockchain_config};

pub async fn run_command(matches: &ArgMatches<'_>, config: Config, is_alternative: bool) -> Result<(), String> {
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

    let ton_client = create_client_verbose(
        &config,
        !config.debug_fail,
    )?;
    if config.debug_fail {
        log::set_max_level(log::LevelFilter::Trace);
        log::set_boxed_logger(
            Box::new(DebugLogger::new(TRACE_PATH.to_string()))
        ).map_err(|e| format!("Failed to set logger: {}", e))?;
    }

    let account = load_account(
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
    let account = account.write_to_bytes()
        .map_err(|e| format!("failed to load data from the account: {}", e))?;
    let account = base64::encode(&account);
    run(matches, config, Some(ton_client), &address, account, is_alternative).await
}

pub async fn run(
    matches: &ArgMatches<'_>,
    config: Config,
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
        config.clone(),
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
    config: Config,
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
        let mut account = Account::construct_from_base64(&acc_boc)
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

        let executor = Box::new(
            DebugTransactionExecutor::new(
                bc_config,
                None,
                TraceLevel::Minimal,
                true
            )
        );
        let message = Message::construct_from_base64(&msg)
            .map_err(|e| format!("Faield to construct message: {}", e))?;

        let now = now_ms();
        let params = ExecuteParams {
            state_libs: HashmapE::default(),
            block_unixtime: (now / 1000) as u32,
            block_lt: now,
            last_tr_lt: Arc::new(AtomicU64::new(now)),
            seed_block: UInt256::default(),
            debug: true,
            ..ExecuteParams::default()
        };

        let trans = executor.execute_with_libs_and_params(
            Some(&message),
            &mut account,
            params
        );
        let msg_string = match trans {
            Ok(_trans) => {
                // decode_messages(trans.out_msgs,load_decode_abi(matches, config.clone())).await?;
                "Debug finished.".to_string()
            },
            Err(e) => {
                format!("Debug failed: {}", e)
            }
        };

        if !config.is_json {
            println!("{}", msg_string);
            println!("Log saved to {}", TRACE_PATH);
        }
    }
    let result = result.map_err(|e| format!("{:#}", e))?;
    let res = result.decoded.and_then(|d| d.output)
        .ok_or("Failed to decode the result. Check that abi matches the contract.")?;
    Ok(res)
}