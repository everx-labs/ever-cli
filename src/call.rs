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
use crate::config::Config;
use crate::convert;
use crate::helpers::{
    create_client, create_client_verbose, get_blockchain_config, load_abi, load_ton_abi, now_ms,
    query_account_field, TonClient, SDK_EXECUTION_ERROR_CODE,
};

use crate::debug::{execute_debug, DebugLogger};
use crate::message::{
    prepare_message_params, print_encoded_message, unpack_message, EncodedMessage,
};
use serde_json::{json, Value};
use std::str::FromStr;
use std::time::Duration;
use tokio::task::JoinSet;
use tokio::time::sleep;
use ton_abi::ParamType;
use ton_block::{Account, Deserializable, Message, Serializable};
use ton_client::abi::{
    decode_message, encode_message, Abi, ParamsOfDecodeMessage, ParamsOfEncodeMessage,
};
use ton_client::error::ClientError;
use ton_client::processing::{
    send_message, wait_for_transaction, ParamsOfProcessMessage, ParamsOfSendMessage,
    ParamsOfWaitForTransaction, ProcessingEvent,
};
use ton_client::tvm::{run_executor, AccountForExecutor, ParamsOfRunExecutor};

async fn decode_call_parameters(
    ton: TonClient,
    msg: &EncodedMessage,
    abi: Abi,
) -> Result<(String, String), String> {
    let result = decode_message(
        ton,
        ParamsOfDecodeMessage {
            abi,
            message: msg.message.clone(),
            ..Default::default()
        },
    )
    .await
    .map_err(|e| format!("couldn't decode message: {}", e))?;

    Ok((
        result.name,
        serde_json::to_string_pretty(&result.value.unwrap_or(json!({})))
            .map_err(|e| format!("failed to serialize result: {}", e))?,
    ))
}

fn parse_integer_param(value: &str) -> Result<String, String> {
    let value = value.trim_matches('\"');

    if value.ends_with('T') {
        convert::convert_token(value.trim_end_matches('T'))
    } else {
        Ok(value.to_owned())
    }
}

async fn build_json_from_params(
    params_vec: Vec<&str>,
    abi_path: &str,
    method: &str,
    config: &Config,
) -> Result<String, String> {
    let abi_obj = load_ton_abi(abi_path, config).await?;
    let functions = abi_obj.functions();

    let func_obj = functions
        .get(method)
        .ok_or("failed to load function from abi")?;
    let inputs = func_obj.input_params();

    let mut params_json = json!({});
    for input in inputs {
        let mut iter = params_vec.iter();
        let _param = iter
            .find(|x| x.starts_with('-') && (x.trim_start_matches('-') == input.name))
            .ok_or(format!(
                r#"argument "{}" of type "{}" not found"#,
                input.name, input.kind
            ))?;

        let value = iter
            .next()
            .ok_or(format!(
                r#"argument "{}" of type "{}" has no value"#,
                input.name, input.kind
            ))?
            .to_string();

        let value = match input.kind {
            ParamType::Uint(_) | ParamType::Int(_) => {
                json!(parse_integer_param(&value)?)
            }
            ParamType::Array(ref _x) => {
                let mut result_vec: Vec<String> = vec![];
                for i in value.split(|c| c == ',' || c == '[' || c == ']') {
                    if !i.is_empty() {
                        result_vec.push(parse_integer_param(i)?)
                    }
                }
                json!(result_vec)
            }
            _ => {
                json!(value)
            }
        };
        params_json[input.name.clone()] = value;
    }

    serde_json::to_string(&params_json).map_err(|e| format!("{}", e))
}

pub async fn emulate_locally(
    ton: TonClient,
    addr: &str,
    msg: String,
    is_fee: bool,
) -> Result<(), String> {
    let state: String;
    let state_boc = query_account_field(ton.clone(), addr, "boc").await;
    if state_boc.is_err() {
        if is_fee {
            let addr = ton_block::MsgAddressInt::from_str(addr)
                .map_err(|e| format!("couldn't decode address: {}", e))?;
            state = base64::encode(
                &ton_types::cells_serialization::serialize_toc(
                    &Account::with_address(addr).serialize().map_err(|e| {
                        format!("couldn't create dummy account for deploy emulation: {}", e)
                    })?,
                )
                .map_err(|e| format!("failed to serialize account cell: {}", e))?,
            );
        } else {
            return Err(state_boc.err().unwrap());
        }
    } else {
        state = state_boc.unwrap();
    }
    let res = run_executor(
        ton.clone(),
        ParamsOfRunExecutor {
            message: msg.clone(),
            account: AccountForExecutor::Account {
                boc: state,
                unlimited_balance: if is_fee { Some(true) } else { None },
            },
            ..Default::default()
        },
    )
    .await;

    if res.is_err() {
        return Err(format!("{:#}", res.err().unwrap()));
    }
    if is_fee {
        let fees = res.unwrap().fees;
        println!("{{");
        println!("  \"in_msg_fwd_fee\": \"{}\",", fees.in_msg_fwd_fee);
        println!("  \"storage_fee\": \"{}\",", fees.storage_fee);
        println!("  \"gas_fee\": \"{}\",", fees.gas_fee);
        println!("  \"out_msgs_fwd_fee\": \"{}\",", fees.out_msgs_fwd_fee);
        println!("  \"total_account_fees\": \"{}\",", fees.total_account_fees);
        println!("  \"total_output\": \"{}\"", fees.total_output);
        println!("}}");
    } else {
        println!("Local run succeeded. Executing onchain."); // TODO: check is_json
    }
    Ok(())
}

pub async fn send_message_and_wait(
    ton: TonClient,
    abi: Option<Abi>,
    msg: String,
    config: &Config,
) -> Result<Value, String> {
    if !config.is_json {
        println!("Processing... ");
    }
    let callback = |_| async move {};
    let result = send_message(
        ton.clone(),
        ParamsOfSendMessage {
            message: msg.clone(),
            abi: abi.clone(),
            send_events: false,
        },
        callback,
    )
    .await
    .map_err(|e| format!("{:#}", e))?;

    if !config.async_call {
        let result = wait_for_transaction(
            ton.clone(),
            ParamsOfWaitForTransaction {
                abi,
                message: msg.clone(),
                shard_block_id: result.shard_block_id,
                send_events: true,
                ..Default::default()
            },
            callback,
        )
        .await
        .map_err(|e| format!("{:#}", e))?;
        Ok(result.decoded.and_then(|d| d.output).unwrap_or(json!({})))
    } else {
        Ok(json!({}))
    }
}

fn processing_event_to_string(pe: ProcessingEvent) -> String {
    match pe {
        ProcessingEvent::WillSend {
            shard_block_id,
            message_id,
            message: _,
        } => format!(
            "\nWillSend: {{\n\t\
shard_block_id: \"{shard_block_id}\",\n\t\
message_id: \"{message_id}\"\n}}"
        ),
        ProcessingEvent::DidSend {
            shard_block_id,
            message_id,
            message: _,
        } => format!(
            "\nDidSend: {{\n\t\
shard_block_id: \"{shard_block_id}\",\n\t\
message_id: \"{message_id}\"\n}}"
        ),
        ProcessingEvent::SendFailed {
            shard_block_id,
            message_id,
            message: _,
            error,
        } => format!(
            "\nSendFailed: {{\n\t\
shard_block_id: \"{shard_block_id}\",\n\t\
message_id: \"{message_id}\"\n\t\
error: \"{error}\"\n}}"
        ),
        ProcessingEvent::WillFetchNextBlock {
            shard_block_id,
            message_id,
            message: _,
        } => format!(
            "\nWillFetchNextBlock: {{\n\t\
shard_block_id: \"{shard_block_id}\",\n\t\
message_id: \"{message_id}\"\n}}"
        ),
        ProcessingEvent::FetchNextBlockFailed {
            shard_block_id,
            message_id,
            message: _,
            error,
        } => format!(
            "\nFetchNextBlockFailed: {{\n\t\
shard_block_id: \"{shard_block_id}\",\n\t\
message_id: \"{message_id}\"\n\t\
error: \"{error}\"\n}}"
        ),
        ProcessingEvent::MessageExpired {
            message_id,
            message: _,
            error,
        } => format!(
            "\nMessageExpired: {{\n\t\
error: \"{error}\",\n\t\
message_id: \"{message_id}\"\n}}"
        ),

        _ => format!("{:#?}", pe),
    }
}

pub async fn process_message(
    ton: TonClient,
    msg: ParamsOfEncodeMessage,
    config: &Config,
) -> Result<Value, ClientError> {
    let callback = |event| async move {
        println!(
            "Process message event: {}",
            processing_event_to_string(event)
        );
    };

    let mut process_with_timeout: JoinSet<Result<Value, ClientError>> = JoinSet::new();
    let send_events = !config.is_json;
    process_with_timeout.spawn(async move {
        let res = ton_client::processing::process_message(
            ton.clone(),
            ParamsOfProcessMessage {
                message_encode_params: msg.clone(),
                send_events,
            },
            callback,
        )
        .await?;

        Ok(res.decoded.and_then(|d| d.output).unwrap_or(json!({})))
    });
    let timeout = config.global_timeout as u64;
    process_with_timeout.spawn(async move {
        sleep(Duration::from_secs(timeout)).await;
        Err(ClientError::with_code_message(
            99999,
            "Message processing with gosh-cli has been timed out.".to_string(),
        ))
    });
    while let Some(finished_task) = process_with_timeout.join_next().await {
        process_with_timeout.shutdown().await;
        match finished_task {
            Err(_) => {
                panic!("Failed to run process message with global timeout")
            }
            Ok(Err(e)) => {
                return Err(e);
            }
            Ok(Ok(res)) => {
                return Ok(res);
            }
        }
    }
    panic!("Failed to run process message with global timeout")
}

pub async fn call_contract_with_result(
    config: &Config,
    addr: &str,
    abi_path: &str,
    method: &str,
    params: &str,
    keys: Option<String>,
    is_fee: bool,
) -> Result<Value, String> {
    let ton = if config.debug_fail != *"None" {
        let log_path = format!("call_{}_{}.log", addr, method);
        log::set_max_level(log::LevelFilter::Trace);
        log::set_boxed_logger(Box::new(DebugLogger::new(log_path)))
            .map_err(|e| format!("Failed to set logger: {}", e))?;
        create_client(config)?
    } else {
        create_client_verbose(config)?
    };
    call_contract_with_client(ton, config, addr, abi_path, method, params, keys, is_fee).await
}

pub async fn call_contract_with_client(
    ton: TonClient,
    config: &Config,
    addr: &str,
    abi_path: &str,
    method: &str,
    params: &str,
    keys: Option<String>,
    is_fee: bool,
) -> Result<Value, String> {
    let abi = load_abi(abi_path, config).await?;

    let msg_params = prepare_message_params(addr, abi.clone(), method, params, None, keys.clone())?;

    let needs_encoded_msg =
        is_fee || config.async_call || config.local_run || config.debug_fail != *"None";

    let message = if needs_encoded_msg {
        let msg = encode_message(ton.clone(), msg_params.clone())
            .await
            .map_err(|e| format!("failed to create inbound message: {}", e))?;

        if config.local_run || is_fee {
            emulate_locally(ton.clone(), addr, msg.message.clone(), is_fee).await?;
            if is_fee {
                return Ok(Value::Null);
            }
        }
        if config.async_call {
            return send_message_and_wait(ton, Some(abi), msg.message.clone(), config).await;
        }
        Some(msg.message)
    } else {
        None
    };

    let dump = if config.debug_fail != *"None" {
        let acc_boc = query_account_field(ton.clone(), addr, "boc").await?;
        let account = Account::construct_from_base64(&acc_boc)
            .map_err(|e| format!("Failed to construct account: {}", e))?
            .serialize()
            .map_err(|e| format!("Failed to serialize account: {}", e))?;

        let now = now_ms();
        Some((
            account,
            message.unwrap(),
            now,
            get_blockchain_config(config, None).await?,
        ))
    } else {
        None
    };

    let res = process_message(ton.clone(), msg_params, config).await;

    if config.debug_fail != *"None"
        && res.is_err()
        && res.clone().err().unwrap().code == SDK_EXECUTION_ERROR_CODE
    {
        if config.is_json {
            let e = format!("{:#}", res.clone().err().unwrap());
            let err: Value = serde_json::from_str(&e).unwrap_or(Value::String(e));
            let res = json!({ "Error": err });
            println!(
                "{}",
                serde_json::to_string_pretty(&res)
                    .unwrap_or("{{ \"JSON serialization error\" }}".to_string())
            );
        } else {
            println!("Error: {:#}", res.clone().err().unwrap());
            println!("Execution failed. Starting debug...");
        }
        let (mut account, message, now, bc_config) = dump.unwrap();
        let message = Message::construct_from_base64(&message)
            .map_err(|e| format!("failed to construct message: {}", e))?;
        let _ = execute_debug(
            bc_config,
            &mut account,
            Some(&message),
            (now / 1000) as u32,
            now,
            now,
            false,
            config,
        )
        .await?;

        if !config.is_json {
            let log_path = format!("call_{}_{}.log", addr, method);
            println!("Debug finished.");
            println!("Log saved to {}", log_path);
        }
        return Err("".to_string());
    }
    res.map_err(|e| format!("{:#}", e))
}

pub fn print_json_result(result: Value, config: &Config) -> Result<(), String> {
    if !result.is_null() {
        let result = serde_json::to_string_pretty(&result)
            .map_err(|e| format!("Failed to serialize the result: {}", e))?;
        if !config.is_json {
            println!("Result: {}", result);
        } else {
            println!("{}", result);
        }
    }
    Ok(())
}

pub async fn call_contract(
    config: &Config,
    addr: &str,
    abi_path: &str,
    method: &str,
    params: &str,
    keys: Option<String>,
    is_fee: bool,
) -> Result<(), String> {
    let result =
        call_contract_with_result(config, addr, abi_path, method, params, keys, is_fee).await?;
    if !config.is_json {
        println!("Succeeded.");
    }
    print_json_result(result, config)?;
    Ok(())
}

pub async fn call_contract_with_msg(
    config: &Config,
    str_msg: String,
    abi_path: &str,
) -> Result<(), String> {
    let ton = create_client_verbose(config)?;
    let abi = load_abi(abi_path, config).await?;

    let (msg, _) = unpack_message(&str_msg)?;
    if config.is_json {
        println!("{{");
    }
    print_encoded_message(&msg, config.is_json);

    let params = decode_call_parameters(ton.clone(), &msg, abi.clone()).await?;

    if !config.is_json {
        println!("Calling method {} with parameters:", params.0);
        println!("{}", params.1);
        println!("Processing... ");
    } else {
        println!("  \"Method\": \"{}\",", params.0);
        println!("  \"Parameters\": {},", params.1);
        println!("}}");
    }
    let result = send_message_and_wait(ton, Some(abi), msg.message, config).await?;

    if !config.is_json {
        println!("Succeeded.");
        if !result.is_null() {
            println!(
                "Result: {}",
                serde_json::to_string_pretty(&result)
                    .map_err(|e| format!("failed to serialize result: {}", e))?
            );
        }
    }
    Ok(())
}

pub async fn parse_params(
    params_vec: Vec<&str>,
    abi_path: &str,
    method: &str,
    config: &Config,
) -> Result<String, String> {
    if params_vec.len() == 1 {
        // if there is only 1 parameter it must be a json string with arguments
        Ok(params_vec[0].to_owned())
    } else {
        build_json_from_params(params_vec, abi_path, method, config).await
    }
}
