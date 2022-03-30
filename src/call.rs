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
use crate::crypto::load_keypair;
use crate::convert;
use crate::helpers::{TonClient, now, now_ms, create_client_verbose, create_client_local, load_ton_address, load_abi, construct_account_from_tvc, query_account_field};
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
    ExecutionOptions
};
use ton_block::{Account, Serializable, Deserializable, Message, MsgAddressInt, ExternalInboundMessageHeader, Grams};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use serde_json::{Value, Map};
use ton_block::MsgAddressExt::AddrNone;
use ton_types::{BuilderData, Cell, IBitstring, SliceData};

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

pub fn serialize_config_param(config_str: String) -> Result<(Cell, u32), String> {
    let config_json: serde_json::Value = serde_json::from_str(&*config_str)
        .map_err(|e| format!(r#"failed to parse "new_param_file": {}"#, e))?;
    let config_json = config_json.as_object()
        .ok_or(format!(r#""new_param_file" is not json object"#))?;
    if config_json.len() != 1 {
        Err(r#""new_param_file" is not a valid json"#.to_string())?;
    }

    let mut key_number = None;
    for key in config_json.keys() {
        if !key.starts_with("p") {
            Err(r#""new_param_file" is not a valid json"#.to_string())?;
        }
        key_number = Some(key.trim_start_matches("p").to_string());
        break;
    }

    let key_number = key_number
        .ok_or(format!(r#""new_param_file" is not a valid json"#))?
        .parse::<u32>()
        .map_err(|e| format!(r#""new_param_file" is not a valid json: {}"#, e))?;

    let config_params = ton_block_json::parse_config(config_json)
        .map_err(|e| format!(r#"failed to parse config params from "new_param_file": {}"#, e))?;

    let config_param = config_params.config(key_number)
        .map_err(|e| format!(r#"failed to parse config params from "new_param_file": {}"#, e))?
        .ok_or(format!(r#"Not found config number {} in parsed config_params"#, key_number))?;

    let mut cell = BuilderData::default();
    config_param.write_to_cell(&mut cell)
        .map_err(|e| format!(r#"failed to serialize config param": {}"#, e))?;
    let config_cell = cell.references()[0].clone();

    Ok((config_cell, key_number))
}

pub fn prepare_message_new_config_param(
    config_param: Cell,
    seqno: u32,
    key_number: u32,
    config_account: SliceData,
    private_key_of_config_account: Vec<u8>
) -> Result<Message, String> {
    let prefix = hex::decode("43665021").unwrap();
    let since_the_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32 + 100; // timestamp + 100 secs

    let mut cell = BuilderData::default();
    cell.append_raw(prefix.as_slice(), 32).unwrap();
    cell.append_u32(seqno).unwrap();
    cell.append_u32(since_the_epoch).unwrap();
    cell.append_i32(key_number as i32).unwrap();
    cell.append_reference_cell(config_param.clone());

    let exp_key = ed25519_dalek::ExpandedSecretKey::from(
        &ed25519_dalek::SecretKey::from_bytes(private_key_of_config_account.as_slice()
    )
        .map_err(|e| format!(r#"failed to read private key from config-master file": {}"#, e))?);
    let pub_key = ed25519_dalek::PublicKey::from(&exp_key);
    let msg_signature = exp_key.sign(cell.finalize(0).unwrap().repr_hash().into_vec().as_slice(), &pub_key).to_bytes().to_vec();

    let mut cell = BuilderData::default();
    cell.append_raw(msg_signature.as_slice(), 64*8).unwrap();
    cell.append_raw(prefix.as_slice(), 32).unwrap();
    cell.append_u32(seqno).unwrap();
    cell.append_u32(since_the_epoch).unwrap();
    cell.append_i32(key_number as i32).unwrap();
    cell.append_reference_cell(config_param);

    let config_contract_address = MsgAddressInt::with_standart(None, -1, config_account).unwrap();
    let mut header = ExternalInboundMessageHeader::new(AddrNone, config_contract_address);
    header.import_fee = Grams::zero();
    let message = Message::with_ext_in_header_and_body(header, cell.into());

    Ok(message)
}

pub fn print_encoded_message(msg: &EncodedMessage, is_json:bool) {
    let expire = if msg.expire.is_some() {
        let expire_at = Local.timestamp(msg.expire.unwrap() as i64, 0);
        expire_at.to_rfc2822()
    } else {
        "unknown".to_string()
    };
    if !is_json {
        println!();
        println!("MessageId: {}", msg.message_id);
        println!("Expire at: {}", expire);
    } else {
        println!("  \"MessageId\": \"{}\",", msg.message_id);
        println!("  \"Expire at\": \"{}\",", expire);
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

pub fn load_account(path: &str, from_tvc: bool) -> Result<Account, String> {
    Ok(if from_tvc {
        construct_account_from_tvc(path, None, None)?
    } else {
        Account::construct_from_file(path)
            .map_err(|e| format!(" failed to load account from the file {}: {}", path, e))?
    })
}

pub async fn run_local_for_account(
    conf: Config,
    account: &str,
    abi: String,
    method: &str,
    params: &str,
    bc_config: Option<&str>,
    is_tvc: bool,
) -> Result<(), String> {

    if !conf.is_json {
        println!("Running get-method...");
    }

    let ton = create_client_local()?;
    let abi = load_abi(&abi)?;

    let acc = load_account(account, is_tvc)?;

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
        acc_boc,
        bc_config
    ).await?;

    if !conf.is_json {
        println!("Succeeded.");
    }

    print_json_result(res, conf)?;
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

async fn run_local(
    ton: TonClient,
    abi: Abi,
    msg: String,
    acc_boc: String,
    bc_config: Option<&str>,
) -> Result<serde_json::Value, String> {
    let execution_options = prepare_execution_options(bc_config)?;
    let result = run_tvm(
        ton.clone(),
        ParamsOfRunTvm {
            message: msg,
            account: acc_boc,
            abi: Some(abi.clone()),
            return_updated_account: Some(true),
            execution_options,
            ..Default::default()
        },
    ).await
        .map_err(|e| format!("{:#}", e))?;
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
    is_json: bool,
) -> Result<serde_json::Value, String> {
    let callback = |event| { async move {
        match event {
            ProcessingEvent::DidSend { shard_block_id: _, message_id, message: _ } => println!("MessageId: {}", message_id),
            _ => (),
        }
    }};
    let res = if !is_json {
        ton_client::processing::process_message(
            ton,
            ParamsOfProcessMessage {
                message_encode_params: msg,
                send_events: true,
                ..Default::default()
            },
            callback,
        ).await
            .map_err(|e| format!("{:#}", e))?
    } else {
        ton_client::processing::process_message(
            ton,
            ParamsOfProcessMessage {
                message_encode_params: msg,
                send_events: true,
                ..Default::default()
            },
            |_| { async move {} },
        ).await
            .map_err(|e| format!("{:#}", e))?
    };

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
            let acc_boc = query_account_field(ton.clone(), addr, "boc").await?;
            return run_local(ton.clone(), abi, msg.message.clone(), acc_boc, None).await;
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
    process_message(ton.clone(), msg_params, conf.is_json).await
}

fn print_json_result(result: Value, conf: Config) -> Result<(), String> {
    if !result.is_null() {
        let result = serde_json::to_string_pretty(&result)
            .map_err(|e| format!("Failed to serialize the result: {}", e))?;
        if !conf.is_json {
            println!("Result: {}", result);
        } else {
            println!("{}", result);
        }
    }
    Ok(())
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
    print_json_result(result, conf)?;
    Ok(())
}

pub fn display_generated_message(
    msg: &EncodedMessage,
    method: &str,
    is_raw: bool,
    output: Option<&str>,
    is_json: bool,
) -> Result<(), String> {
    if is_json {
        println!("{{");
    }
    print_encoded_message(msg, is_json);

    let msg_bytes = pack_message(msg, method, is_raw)?;
    if output.is_some() {
        let out_file = output.unwrap();
        std::fs::write(out_file, msg_bytes)
            .map_err(|e| format!("cannot write message to file: {}", e))?;
        if !is_json {
            println!("Message saved to file {}", out_file);
        } else {
            println!("  \"Message\": \"saved to file {}\"", out_file);
        }
    } else {
        let msg_hex = hex::encode(&msg_bytes);
        if !is_json {
            println!("Message: {}", msg_hex);
            println!();
            qr2term::print_qr(msg_hex).map_err(|e| format!("failed to print QR code: {}", e))?;
            println!();
        } else {
            println!("  \"Message\": \"{}\"", msg_hex);
        }
    }
    if is_json {
        println!("}}");
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
        _conf.is_json,
    ).await?;

    display_generated_message(&msg, method, is_raw, output, _conf.is_json)?;

    Ok(())
}

pub async fn call_contract_with_msg(conf: Config, str_msg: String, abi: String) -> Result<(), String> {
    let ton = create_client_verbose(&conf)?;
    let abi = load_abi(&abi)?;

    let (msg, _) = unpack_message(&str_msg)?;
    if conf.is_json {
        println!("{{");
    }
    print_encoded_message(&msg, conf.is_json);

    let params = decode_call_parameters(ton.clone(), &msg, abi.clone()).await?;

    if !conf.is_json {
        println!("Calling method {} with parameters:", params.0);
        println!("{}", params.1);
        println!("Processing... ");
    } else {
        println!("  \"Method\": \"{}\",", params.0);
        println!("  \"Parameters\": {},", params.1);
        println!("}}");
    }
    let result = send_message_and_wait(ton, Some(abi), msg.message,  conf.clone()).await?;

    if !conf.is_json {
        println!("Succeeded.");
        if !result.is_null() {
            println!("Result: {}", serde_json::to_string_pretty(&result)
                .map_err(|e| format!("failed to serialize result: {}", e))?);
        }
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

pub async fn run_get_method(conf: Config, addr: &str, method: &str, params: Option<String>, is_local: bool, is_tvc: bool, bc_config: Option<&str>) -> Result<(), String> {
    let ton = if !is_local {
        create_client_verbose(&conf)?
    } else {
        create_client_local()?
    };

    let acc_boc = if is_local {
        let acc = load_account(addr, is_tvc)?;
        let acc_bytes = acc.write_to_bytes()
            .map_err(|e| format!("failed to load data from the account: {}", e))?;
        base64::encode(&acc_bytes)
    } else {
        let addr = load_ton_address(addr, &conf)
            .map_err(|e| format!("failed to parse address: {}", e.to_string()))?;
        query_account_field(ton.clone(), addr.as_str(), "boc").await?
    };

    let params = params.map(|p| serde_json::from_str(&p))
        .transpose()
        .map_err(|e| format!("arguments are not in json format: {}", e))?;

    if !conf.is_json {
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
