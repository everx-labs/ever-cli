/*
 * Copyright 2018-2020 TON DEV SOLUTIONS LTD.
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
use crate::crypto::load_keypair;
use crate::convert;
use crate::helpers::{
    TonClient,
    now,
    now_ms,
    create_client_verbose,
    create_client_local,
    query,
    load_ton_address,
    load_abi,
};
use ton_abi::{Contract, ParamType};
use chrono::{TimeZone, Local};
use hex;
use ton_client::abi::{
    encode_message,
    decode_message,
    ParamsOfDecodeMessage,
    ParamsOfEncodeMessage,
    Abi,
    CallSet,
    FunctionHeader,
    Signer
};
use ton_client::processing::{
    ParamsOfSendMessage,
    ParamsOfWaitForTransaction,
    ParamsOfProcessMessage,
    ProcessingEvent,
    wait_for_transaction,
    send_message,
};
use ton_client::tvm::{
    run_tvm,
    run_get,
    ParamsOfRunTvm,
    ParamsOfRunGet,
    run_executor,
    ParamsOfRunExecutor,
    AccountForExecutor,
};
use ton_block::{Account, Serializable, Deserializable};
use std::str::FromStr;
use serde_json::{Value, Map};

pub struct EncodedMessage {
    pub message_id: String,
    pub message: String,
    pub expire: Option<u32>,
    pub address: String,
}

pub async fn prepare_message(
    ton: TonClient,
    addr: &str,
    abi: Abi,
    method: &str,
    params: &str,
    header: Option<FunctionHeader>,
    keys: Option<String>,
    is_json: bool,
) -> Result<EncodedMessage, String> {
    if !is_json {
        println!("Generating external inbound message...");
    }

    let msg_params = prepare_message_params(addr, abi, method, params, header.clone(), keys)?;

    let msg = encode_message(ton, msg_params).await
        .map_err(|e| format!("failed to create inbound message: {}", e))?;

    Ok(EncodedMessage {
        message: msg.message,
        message_id: msg.message_id,
        expire: header.and_then(|h| h.expire),
        address: addr.to_owned(),
    })
}

pub fn prepare_message_params (
    addr: &str,
    abi: Abi,
    method: &str,
    params: &str,
    header: Option<FunctionHeader>,
    keys: Option<String>,
) -> Result<ParamsOfEncodeMessage, String> {
    let keys = keys.map(|k| load_keypair(&k)).transpose()?;
    let params = serde_json::from_str(&params)
        .map_err(|e| format!("arguments are not in json format: {}", e))?;

    let call_set = Some(CallSet {
        function_name: method.into(),
        input: Some(params),
        header: header.clone(),
    });

    Ok(ParamsOfEncodeMessage {
        abi,
        address: Some(addr.to_owned()),
        call_set,
        signer: if keys.is_some() {
            Signer::Keys { keys: keys.unwrap() }
        } else {
            Signer::None
        },
        ..Default::default()
    })
}

pub fn print_encoded_message(msg: &EncodedMessage) {
    println!();
    println!("MessageId: {}", msg.message_id);
    print!("Expire at: ");
    if msg.expire.is_some() {
        let expire_at = Local.timestamp(msg.expire.unwrap() as i64 , 0);
        println!("{}", expire_at.to_rfc2822());
    } else {
        println!("unknown");
    }
}

fn pack_message(msg: &EncodedMessage, method: &str, is_raw: bool) -> Result<Vec<u8>, String> {
    let res = if is_raw {
        base64::decode(&msg.message)
            .map_err(|e| format!("failed to decode message: {}", e))?
    } else {
        let json_msg = json!({
            "msg": {
                "message_id": msg.message_id,
                "message": msg.message,
                "expire": msg.expire,
                "address": msg.address,
            },
            "method": method,
        });
        serde_json::to_string(&json_msg)
            .map_err(|e| format!("failed to serialize message: {}", e))?
            .into_bytes()
    };
    Ok(res)
}

fn unpack_message(str_msg: &str) -> Result<(EncodedMessage, String), String> {
    let bytes = hex::decode(str_msg)
        .map_err(|e| format!("couldn't unpack message: {}", e))?;

    let str_msg = std::str::from_utf8(&bytes)
        .map_err(|e| format!("message is corrupted: {}", e))?;

    let json_msg: serde_json::Value = serde_json::from_str(str_msg)
        .map_err(|e| format!("couldn't decode message: {}", e))?;

    let method = json_msg["method"].as_str()
        .ok_or(r#"couldn't find "method" key in message"#)?
        .to_owned();
    let message_id = json_msg["msg"]["message_id"].as_str()
        .ok_or(r#"couldn't find "message_id" key in message"#)?
        .to_owned();
    let message = json_msg["msg"]["message"].as_str()
        .ok_or(r#"couldn't find "message" key in message"#)?
        .to_owned();
    let expire = json_msg["msg"]["expire"].as_u64().map(|x| x as u32);
    let address = json_msg["msg"]["address"].as_str()
        .ok_or(r#"couldn't find "address" key in message"#)?
        .to_owned();

    let msg = EncodedMessage {
        message_id, message, expire, address
    };
    Ok((msg, method))
}

async fn decode_call_parameters(ton: TonClient, msg: &EncodedMessage, abi: Abi) -> Result<(String, String), String> {
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
        serde_json::to_string_pretty(
            &result.value.unwrap_or(json!({}))
        ).map_err(|e| format!("failed to serialize result: {}", e))?
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

fn build_json_from_params(params_vec: Vec<&str>, abi: &str, method: &str) -> Result<String, String> {
    let abi_obj = Contract::load(abi.as_bytes()).map_err(|e| format!("failed to parse ABI: {}", e))?;
    let functions = abi_obj.functions();

    let func_obj = functions.get(method).ok_or("failed to load function from abi")?;
    let inputs = func_obj.input_params();

    let mut params_json = json!({ });
    for input in inputs {
        let mut iter = params_vec.iter();
        let _param = iter.find(|x| x.trim_start_matches('-') == input.name)
            .ok_or(format!(r#"argument "{}" of type "{}" not found"#, input.name, input.kind))?;

        let value = iter.next()
            .ok_or(format!(r#"argument "{}" of type "{}" has no value"#, input.name, input.kind))?
            .to_string();

        let value = match input.kind {
            ParamType::Uint(_) | ParamType::Int(_) => {
                json!(parse_integer_param(&value)?)
            },
            ParamType::Array(ref x) => {
                if let ParamType::Uint(_) = **x {
                    let mut result_vec: Vec<String> = vec![];
                    for i in value.split(|c| c == ',' || c == '[' || c == ']') {
                        if i != "" {
                            result_vec.push(parse_integer_param(i)?)
                        }
                    }
                    json!(result_vec)
                } else {
                    json!(value)
                }
            },
            _ => {
                json!(value)
            }
        };
        params_json[input.name.clone()] = value;
    }

    serde_json::to_string(&params_json).map_err(|e| format!("{}", e))
}

pub async fn query_account_boc(ton: TonClient, addr: &str) -> Result<String, String> {
    let accounts = query(
        ton,
            "accounts",
            json!({ "id": { "eq": addr } }),
            "boc",
            None,
        ).await
    .map_err(|e| format!("failed to query account: {}", e))?;

    if accounts.len() == 0 {
        return Err(format!("account not found"));
    }
    let boc = accounts[0]["boc"].as_str();
    if boc.is_none() {
        return Err(format!("account doesn't contain data"));
    }
    Ok(boc.unwrap().to_owned())
}

pub async fn emulate_locally(
    ton: TonClient,
    addr: &str,
    msg: String,
    is_fee: bool,
) -> Result<(), String> {
    let state: String;
    let state_boc = query_account_boc(ton.clone(), addr).await;
    if state_boc.is_err() {
        if is_fee {
            let addr = ton_block::MsgAddressInt::from_str(addr)
                .map_err(|e| format!("couldn't decode address: {}", e))?;
            state = base64::encode(
                &ton_types::cells_serialization::serialize_toc(
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
        println!("Local run succeeded. Executing onchain.");
    }
    Ok(())
}

pub async fn run_local_for_account(
    conf: Config,
    account: &str,
    abi: String,
    method: &str,
    params: &str,
) -> Result<(), String> {

    if !conf.is_json {
        println!("Running get-method...");
    }

    let ton = create_client_local()?;
    let abi = load_abi(&abi)?;

    let acc = Account::construct_from_file(account)
        .map_err(|e| format!(" failed to load account from the file {}: {}", account, e))?;

    let acc_bytes = acc.write_to_bytes()
        .map_err(|e| format!("failed to load data from the account: {}", e))?;
    let acc_boc = base64::encode(&acc_bytes);

    let addr = acc.get_addr()
        .ok_or("failed to load address from the account.")?
        .to_string();

    let now = now()?;
    let expire_at = conf.lifetime + now;
    let header = FunctionHeader {
        expire: Some(expire_at),
        ..Default::default()
    };

    let msg = prepare_message(
        ton.clone(),
        &addr,
        abi.clone(),
        method,
        params,
        Some(header),
        None,
        conf.is_json,
    ).await?;

    let res = run_local(
        ton,
        abi,
        msg.message,
        acc_boc
    ).await?;

    if !conf.is_json {
        println!("Succeeded.");
    }

    print_json_result(res, conf);
    Ok(())
}


async fn run_local(
    ton: TonClient,
    abi: Abi,
    msg: String,
    acc_boc: String,
) -> Result<serde_json::Value, String> {

    let result = run_tvm(
        ton.clone(),
        ParamsOfRunTvm {
            message: msg,
            account: acc_boc,
            abi: Some(abi.clone()),
            return_updated_account: Some(true),
            ..Default::default()
        },
    ).await
        .map_err(|e| format!("run failed: {:#}", e))?;
    let res = result.decoded.and_then(|d| d.output)
        .ok_or("Failed to decode the result. Check that abi matches the contract.")?;
    Ok(res)
}

pub async fn send_message_and_wait(
    ton: TonClient,
    abi: Option<Abi>,
    msg: String,
    conf: Config,
) -> Result<serde_json::Value, String> {

    if !conf.is_json {
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

    if !conf.async_call {
        let result = wait_for_transaction(
            ton.clone(),
            ParamsOfWaitForTransaction {
                abi,
                message: msg.clone(),
                shard_block_id: result.shard_block_id,
                send_events: true,
                ..Default::default()
            },
            callback.clone(),
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
) -> Result<serde_json::Value, String> {
    let callback = |event| { async move {
        match event {
            ProcessingEvent::DidSend { shard_block_id: _, message_id, message: _ } => println!("MessageId: {}", message_id),
            _ => (),
        }
    }};
    let res = ton_client::processing::process_message(
        ton,
        ParamsOfProcessMessage {
            message_encode_params: msg,
            send_events: true,
            ..Default::default()
        },
        callback,
    ).await
        .map_err(|e| format!("Failed: {:#}", e))?;

    Ok(res.decoded.and_then(|d| d.output).unwrap_or(json!({})))
}

pub async fn call_contract_with_result(
    conf: Config,
    addr: &str,
    abi: String,
    method: &str,
    params: &str,
    keys: Option<String>,
    local: bool,
    is_fee: bool,
) -> Result<serde_json::Value, String> {
    let ton = create_client_verbose(&conf)?;
    call_contract_with_client(ton, conf, addr, abi, method, params, keys, local, is_fee).await
}

pub async fn call_contract_with_client(
    ton: TonClient,
    conf: Config,
    addr: &str,
    abi: String,
    method: &str,
    params: &str,
    keys: Option<String>,
    local: bool,
    is_fee: bool,
) -> Result<serde_json::Value, String> {
    let abi = load_abi(&abi)?;

    let expire_at = conf.lifetime + now()?;
    let time = now_ms();
    let header = FunctionHeader {
        expire: Some(expire_at),
        time: Some(time),
        ..Default::default()
    };
    let msg_params = prepare_message_params(
        addr,
        abi.clone(),
        method,
        params,
        Some(header),
        keys.clone(),
    )?;

    let needs_encoded_msg = is_fee ||
        local ||
        conf.async_call ||
        conf.local_run;

    if needs_encoded_msg {
        let msg = encode_message(ton.clone(), msg_params.clone()).await
            .map_err(|e| format!("failed to create inbound message: {}", e))?;

        if local {
            if !conf.is_json {
                println!("Running get-method...");
            }
            let acc_boc = query_account_boc(ton.clone(), addr).await?;
            return run_local(ton.clone(), abi, msg.message.clone(), acc_boc).await;
        }
        if conf.local_run || is_fee {
            emulate_locally(ton.clone(), addr, msg.message.clone(), is_fee).await?;
            if is_fee {
                return Ok(Value::Null);
            }
        }
        if conf.async_call {
            return send_message_and_wait(ton,
                                         Some(abi),
                                         msg.message,
                                         conf).await;
        }
    }

    if !conf.is_json {
        print!("Expire at: ");
        let expire_at = Local.timestamp(expire_at as i64 , 0);
        println!("{}", expire_at.to_rfc2822());
    }
    process_message(ton.clone(), msg_params).await
}

fn print_json_result(result: Value, conf: Config) {
    if !result.is_null() {
        if !conf.is_json {
            println!("Result: {}", serde_json::to_string_pretty(&result).unwrap_or("failed to serialize the result".to_owned()));
        } else {
            println!("{}", serde_json::to_string_pretty(&result).unwrap_or("failed to serialize the result".to_owned()));
        }
    }
}

pub async fn call_contract(
    conf: Config,
    addr: &str,
    abi: String,
    method: &str,
    params: &str,
    keys: Option<String>,
    local: bool,
    is_fee: bool,
) -> Result<(), String> {
    let result = call_contract_with_result(conf.clone(), addr, abi, method, params, keys, local, is_fee).await?;
    if !conf.is_json {
        println!("Succeeded.");
    }
    print_json_result(result, conf);
    Ok(())
}

pub fn display_generated_message(
    msg: &EncodedMessage,
    method: &str,
    is_raw: bool,
    output: Option<&str>,
) -> Result<(), String> {
    print_encoded_message(msg);

    let msg_bytes = pack_message(msg, method, is_raw)?;
    if output.is_some() {
        let out_file = output.unwrap();
        std::fs::write(out_file, msg_bytes)
            .map_err(|e| format!("cannot write message to file: {}", e))?;
        println!("Message saved to file {}", out_file);
    } else {
        let msg_hex = hex::encode(&msg_bytes);
        println!("Message: {}", msg_hex);
        println!();
        qr2term::print_qr(msg_hex).map_err(|e| format!("failed to print QR code: {}", e))?;
        println!();
    }
    Ok(())
}

pub async fn generate_message(
    _conf: Config,
    addr: &str,
    abi: String,
    method: &str,
    params: &str,
    keys: Option<String>,
    lifetime: u32,
    is_raw: bool,
    output: Option<&str>,
) -> Result<(), String> {
    let ton = create_client_local()?;

    let ton_addr = load_ton_address(addr, &_conf)
        .map_err(|e| format!("failed to parse address: {}", e.to_string()))?;

    let abi = load_abi(&abi)?;

    let expire_at = lifetime + now()?;
    let header = FunctionHeader {
        expire: Some(expire_at),
        ..Default::default()
    };

    let msg = prepare_message(
        ton.clone(),
        &ton_addr,
        abi,
        method,
        params,
        Some(header),
        keys,
        false,
    ).await?;

    display_generated_message(&msg, method, is_raw, output)?;

    Ok(())
}

pub async fn call_contract_with_msg(conf: Config, str_msg: String, abi: String) -> Result<(), String> {
    let ton = create_client_verbose(&conf)?;
    let abi = load_abi(&abi)?;

    let (msg, _) = unpack_message(&str_msg)?;
    print_encoded_message(&msg);

    let params = decode_call_parameters(ton.clone(), &msg, abi.clone()).await?;

    println!("Calling method {} with parameters:", params.0);
    println!("{}", params.1);
    println!("Processing... ");

    let result = send_message_and_wait(ton, Some(abi), msg.message,  conf).await?;

    println!("Succeeded.");
    if !result.is_null() {
        println!("Result: {}", serde_json::to_string_pretty(&result)
            .map_err(|e| format!("failed to serialize result: {}", e))?);
    }
    Ok(())
}

pub fn parse_params(params_vec: Vec<&str>, abi: &str, method: &str) -> Result<String, String> {
    if params_vec.len() == 1 {
        // if there is only 1 parameter it must be a json string with arguments
        Ok(params_vec[0].to_owned())
    } else {
        build_json_from_params(params_vec, abi, method)
    }
}

pub async fn run_get_method(conf: Config, addr: &str, method: &str, params: Option<String>, is_boc:bool) -> Result<(), String> {
    let ton = create_client_verbose(&conf)?;

    let acc_boc = if is_boc {
        let acc = Account::construct_from_file(addr)
            .map_err(|e| format!(" failed to load account from the file {}: {}", addr, e))?;
        let acc_bytes = acc.write_to_bytes()
            .map_err(|e| format!("failed to load data from the account: {}", e))?;
        base64::encode(&acc_bytes)
    } else {
        let addr = load_ton_address(addr, &conf)
            .map_err(|e| format!("failed to parse address: {}", e.to_string()))?;
        query_account_boc(ton.clone(), addr.as_str()).await?
    };

    let params = params.map(|p| serde_json::from_str(&p))
        .transpose()
        .map_err(|e| format!("arguments are not in json format: {}", e))?;

    if !conf.is_json {
        println!("Running get-method...");
    }
    let result = run_get(
        ton,
        ParamsOfRunGet {
            account: acc_boc,
            function_name: method.to_owned(),
            input: params,
            ..Default::default()
        },
    ).await
    .map_err(|e| format!("run failed: {}", e.to_string()))?
    .output;

    if !conf.is_json {
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
