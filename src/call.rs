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
use crate::config::Config;
use crate::convert;
use crate::helpers::{TonClient, now_ms, create_client_verbose, load_abi, query_account_field,
    create_client, load_ton_abi, get_blockchain_config};

use ton_client::abi::{encode_message, decode_message, ParamsOfDecodeMessage, ParamsOfEncodeMessage,
                      Abi};
use ton_client::processing::{
    ParamsOfSendMessage,
    ParamsOfWaitForTransaction,
    ParamsOfProcessMessage,
    ProcessingEvent,
    wait_for_transaction,
    send_message,
};
use ton_client::tvm::{
    run_executor,
    ParamsOfRunExecutor,
    AccountForExecutor
};
use ever_block::{Account, Serializable};
use std::str::FromStr;
use serde_json::{json, Value};
use ever_abi::ParamType;
use ton_client::error::ClientError;
use crate::debug::{init_debug_logger, debug_error, DebugParams};
use crate::message::{EncodedMessage, prepare_message_params, print_encoded_message, unpack_message};

async fn decode_call_parameters(ton: TonClient, msg: &EncodedMessage, abi: Abi) -> Result<(String, String), String> {
    let result = decode_message(
        ton,
        ParamsOfDecodeMessage {
            abi,
            message: msg.message.clone(),
            ..Default::default()
        },
    )
    .map_err(|e| format!("couldn't decode message: {}", e))?;

    Ok((
        result.name,
        format!("{:#}", result.value.unwrap_or(json!({})))
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

async fn build_json_from_params(params_vec: Vec<&str>, abi_path: &str, method: &str, config: &Config) -> Result<String, String> {
    let abi_obj = load_ton_abi(abi_path, config).await?;
    let functions = abi_obj.functions();

    let func_obj = functions.get(method).ok_or("failed to load function from abi")?;
    let inputs = func_obj.input_params();

    let mut params_json = json!({ });
    for input in inputs {
        let mut iter = params_vec.iter();
        let _param = iter.find(|x| x.starts_with('-') && (x.trim_start_matches('-') == input.name))
            .ok_or(format!(r#"argument "{}" of type "{}" not found"#, input.name, input.kind))?;

        let value = iter.next()
            .ok_or(format!(r#"argument "{}" of type "{}" has no value"#, input.name, input.kind))?
            .to_string();

        let value = match input.kind {
            ParamType::Uint(_) | ParamType::Int(_) => {
                json!(parse_integer_param(&value)?)
            },
            ParamType::Array(ref _x) => {
                let mut result_vec: Vec<String> = vec![];
                for i in value.split(|c| c == ',' || c == '[' || c == ']') {
                    if !i.is_empty() {
                        result_vec.push(parse_integer_param(i)?)
                    }
                }
                json!(result_vec)
            },
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
            let addr = ever_block::MsgAddressInt::from_str(addr)
                .map_err(|e| format!("couldn't decode address: {}", e))?;
            state = base64::encode(
                &ever_block::write_boc(
                    &Account::with_address(addr)
                        .serialize()
                        .map_err(|e| format!("couldn't create dummy account for deploy emulation: {}", e))?
                ).map_err(|e| format!("failed to serialize account cell: {}", e))?
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
                unlimited_balance: if is_fee {
                    Some(true)
                } else {
                    None
                },
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
    let callback = |_| {
        async move {}
    };
    let result = send_message(
        ton.clone(),
        ParamsOfSendMessage {
            message: msg.clone(),
            abi: abi.clone(),
            send_events: false,
            ..Default::default()
        },
        callback,
    ).await
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
        ).await
            .map_err(|e| format!("{:#}", e))?;
        Ok(result.decoded.and_then(|d| d.output).unwrap_or(json!({})))
    } else {
        Ok(json!({}))
    }
}

pub async fn process_message(
    ton: TonClient,
    msg: ParamsOfEncodeMessage,
    config: &Config,
) -> Result<Value, ClientError> {
    let callback = |event| { async move {
        if let ProcessingEvent::DidSend {
            shard_block_id: _,
            message_id,
            message_dst: _,
            message: _,
        } = event {
            println!("MessageId: {}", message_id)
        }
    }};
    let res = if !config.is_json {
        ton_client::processing::process_message(
            ton.clone(),
            ParamsOfProcessMessage {
                message_encode_params: msg.clone(),
                send_events: true,
                ..Default::default()
            },
            callback,
        ).await
    } else {
        ton_client::processing::process_message(
            ton.clone(),
            ParamsOfProcessMessage {
                message_encode_params: msg.clone(),
                send_events: true,
                ..Default::default()
            },
            |_| { async move {} },
        ).await
    }?;

    Ok(res.decoded.and_then(|d| d.output).unwrap_or(json!({})))
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
    let ton = if config.debug_fail != "None".to_string() {
        init_debug_logger(&format!("call_{}_{}.log", addr, method))?;
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

    let msg_params = prepare_message_params(
        addr,
        abi.clone(),
        method,
        params,
        None,
        keys.clone(),
        None,
    )?;

    let needs_encoded_msg = is_fee ||
        config.async_call ||
        config.local_run ||
        config.debug_fail != "None".to_string();

    let message = if needs_encoded_msg {
        let msg = encode_message(ton.clone(), msg_params.clone()).await
            .map_err(|e| format!("failed to create inbound message: {}", e))?;

        if config.local_run || is_fee {
            emulate_locally(ton.clone(), addr, msg.message.clone(), is_fee).await?;
            if is_fee {
                return Ok(Value::Null);
            }
        }
        if config.async_call {
            return send_message_and_wait(ton,
                                         Some(abi),
                                         msg.message.clone(),
                                         config).await;
        }
        Some(msg.message)
    } else {
        None
    };

    match process_message(ton.clone(), msg_params, config).await {
        Ok(result) => Ok(result),
        Err(e) => {
            let acc_boc = query_account_field(
                ton.clone(),
                addr,
                "boc",
            ).await?;
            let now = now_ms();
            let bc_config = get_blockchain_config(config, None).await?;
            let debug_params = DebugParams {
                account: &acc_boc,
                message: message.as_deref(),
                time_in_ms: now,
                block_lt: now,
                last_tr_lt: now,
                ..DebugParams::new(config, bc_config)
            };
            debug_error(&e, debug_params).await?;
            return Err(format!("{:#}", e));
        }
    }
}

pub fn print_json_result(result: Value, config: &Config) -> Result<(), String> {
    if !result.is_null() {
        if !config.is_json {
            println!("Result: {:#}", result);
        } else {
            println!("{:#}", result);
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
    let result = call_contract_with_result(config, addr, abi_path, method, params, keys, is_fee).await?;
    if !config.is_json {
        println!("Succeeded.");
    }
    print_json_result(result, config)?;
    Ok(())
}


pub async fn call_contract_with_msg(config: &Config, str_msg: String, abi_path: &str) -> Result<(), String> {
    let ton = create_client_verbose(&config)?;
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
    let result = send_message_and_wait(ton, Some(abi), msg.message,  config).await?;

    if !config.is_json {
        println!("Succeeded.");
        if !result.is_null() {
            println!("Result: {:#}", result);
        }
    }
    Ok(())
}

pub async fn parse_params(params_vec: Vec<&str>, abi_path: &str, method: &str, config: &Config) -> Result<String, String> {
    if params_vec.len() == 1 {
        // if there is only 1 parameter it must be a json string with arguments
        Ok(params_vec[0].to_owned())
    } else {
        build_json_from_params(params_vec, abi_path, method, config).await
    }
}
