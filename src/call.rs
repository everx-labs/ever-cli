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
use crate::helpers::{create_client_verbose, now, load_ton_address, load_tvc};
use ton_abi::{Contract, ParamType};
use chrono::{TimeZone, Local};
use hex;
use ton_client_rs::{
    TonClient, TonAddress, EncodedMessage
};
use ton_types::cells_serialization::{BagOfCells};
use std::io::Cursor;

const MAX_LEVEL: log::LevelFilter = log::LevelFilter::Warn;

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() < MAX_LEVEL
    }

    fn log(&self, record: &log::Record) {
		match record.level() {
			log::Level::Error | log::Level::Warn => {
				eprintln!("{} - {}", record.level(), record.args());
			}
			_ => {
				println!("{} - {}", record.level(), record.args());
			}
		}
    }

    fn flush(&self) {}
}

fn prepare_message(
    ton: &TonClient,
    addr: &TonAddress,
    abi: &str,
    method: &str,
    params: &str,
    header: Option<String>,
    keys: Option<String>,
) -> Result<EncodedMessage, String> {    
    
    let keys = keys.map(|k| load_keypair(&k)).transpose()?;

    ton.contracts.create_run_message(
        addr,
        abi.into(),
        method,
        header.map(|v| v.into()),
        params.into(),
        keys.as_ref(),
        None,
    )
    .map_err(|e| format!("failed to create inbound message: {}", e))
}

fn print_encoded_message(msg: &EncodedMessage) {
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

fn pack_message(msg: &EncodedMessage, method: &str) -> String {
    let json_msg = json!({
        "msg": {
            "message_id": msg.message_id,
            "message_body": hex::encode(&msg.message_body),
            "expire": msg.expire,
            "address": msg.address,
        },
        "method": method,
    });

    hex::encode(serde_json::to_string(&json_msg).unwrap())
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
    let message_body = json_msg["msg"]["message_body"].as_str()
        .ok_or(r#"couldn't find "message_body" key in message"#)?;
    let message_body = hex::decode(message_body).unwrap();
    let expire = json_msg["msg"]["expire"].as_u64().map(|x| x as u32);
    let address = json_msg["msg"]["address"].as_str()
        .ok_or(r#"couldn't find "address" key in message"#)?;
    let address = TonAddress::from_str(address)
        .map_err(|e| format!("failed to parse address: {}", e.to_string()))?;
    
    let msg = EncodedMessage {
        message_id, message_body, expire, address
    };
    Ok((msg, method))
}

fn pack_state(mut msg: EncodedMessage, state: Option<Vec<u8>>) -> Result<EncodedMessage, String> {
    if state.is_some() {
        let mut buff = Cursor::new(state.unwrap());
        let image = ton_sdk::ContractImage::from_state_init(&mut buff)
            .map_err(|e| format!("unable to load contract image: {}", e))?;
        let state_init = image.state_init();
        let mut raw_msg = ton_sdk::Contract::deserialize_message(&msg.message_body[..])
            .map_err(|e| format!("cannot deserialize buffer to msg: {}", e))?;
        raw_msg.set_state_init(state_init);
        let (msg_bytes, message_id) = ton_sdk::Contract::serialize_message(&raw_msg)
            .map_err(|e| format!("cannot serialize msg with state: {}", e))?;
        msg.message_body = msg_bytes;
        msg.message_id = message_id.to_string();
    }
    Ok(msg)
}

pub fn decode_call_parameters(ton: &TonClient, msg: &EncodedMessage, abi: &str) -> Result<(String, String), String> {
    let tvm_msg = ton_sdk::Contract::deserialize_message(&msg.message_body[..]).unwrap();
    let body_slice = tvm_msg.body().unwrap();

    let mut data = Vec::new();
    let bag = BagOfCells::with_root(&body_slice.cell());
    bag.write_to(&mut data, false)
        .map_err(|e| format!("couldn't create body BOC: {}", e))?;
        
    let result = ton.contracts.decode_input_message_body(
        abi.into(),
        &data[..],
        false
    ).map_err(|e| format!("couldn't decode message body: {}", e))?;

    Ok((
        result.function,
        serde_json::to_string_pretty(&result.output).unwrap()
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
        
    let func_obj = functions.get(method).unwrap();
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

pub fn call_contract_with_result(
    conf: Config,
    addr: &str,
    abi: String,
    method: &str,
    params: &str,
    keys: Option<String>,
    local: bool,
    tvc_file: Option<&str>
) -> Result<serde_json::Value, String> {
    let ton = create_client_verbose(&conf)?;
    let ton_addr = load_ton_address(addr)?;

    let result = if local {
        println!("Running get-method...");
        ton.contracts.run_local(
            &ton_addr,
            None,
            abi.into(),
            method,
            None,
            params.into(),
            None,
            None,
            false
        )
        .map_err(|e| format!("run failed: {}", e.to_string()))?
        .output
        
    } else {
        
        println!("Generating external inbound message...");
        let msg = prepare_message(
            &ton,
            &ton_addr,
            &abi,
            method,
            params,
            None,
            keys,
        )?;

        let state = tvc_file.map(|v| load_tvc(v)).transpose()?;
        let msg = pack_state(msg, state)?;
        
        print_encoded_message(&msg);
        println!("Processing... ");

        ton.contracts.process_message(msg, Some(abi.into()), Some(method), true)
            .map_err(|e| format!("Failed: {}", e.to_string()))?
            .output
    };
    Ok(result)
}

pub fn call_contract(
    conf: Config,
    addr: &str,
    abi: String,
    method: &str,
    params: &str,
    keys: Option<String>,
    local: bool,
    tvc_file: Option<&str>,
) -> Result<(), String> {
    let result = call_contract_with_result(conf, addr, abi, method, params, keys, local, tvc_file)?;

    println!("Succeeded.");
    if !result.is_null() {
        println!("Result: {}", serde_json::to_string_pretty(&result).unwrap());
    }
    Ok(())
}

pub fn generate_message(
    _conf: Config,
    addr: &str,
    abi: String,
    method: &str,
    params: &str,
    keys: Option<String>,
    lifetime: u32,
    tvc_file: Option<&str>,
) -> Result<(), String> {
    let ton = TonClient::default()
        .map_err(|e| format!("failed to create tonclient: {}", e.to_string()))?;

    let ton_addr = TonAddress::from_str(addr)
        .map_err(|e| format!("failed to parse address: {}", e.to_string()))?;

    let expire_at = lifetime + now();
    let header = json!({
        "expire": expire_at
    });

    let msg = prepare_message(
        &ton,
        &ton_addr,
        &abi,
        method,
        params,
        Some(serde_json::to_string(&header).unwrap()),
        keys,
    )?;
    let state = tvc_file.map(|v| load_tvc(v)).transpose()?;
    let msg = pack_state(msg, state)?;
    print_encoded_message(&msg);

    let str_msg = pack_message(&msg, method);
    println!("Message: {}", &str_msg);
    println!();
    let res = qr2term::print_qr(&str_msg)
        .map_err(|e| format!("cannot generate QR-code: {}", e));
    if res.is_err() {
        println!("{}", res.unwrap_err());
    }
    println!();
    Ok(())
}

pub fn call_contract_with_msg(conf: Config, str_msg: String, abi: String) -> Result<(), String> {
    let ton = create_client_verbose(&conf)?;

    let (msg, method) = unpack_message(&str_msg)?;
    print_encoded_message(&msg);

    let params = decode_call_parameters(&ton, &msg, &abi)?;

    println!("Calling method {} with parameters:", params.0);
    println!("{}", params.1);
    println!("Processing... ");
    let result = ton.contracts.process_message(
        msg,
        Some(abi.into()),
        Some(&method),
        true
    )
    .map_err(|e| format!("Failed: {}", e.to_string()))?;

    println!("Succeded.");
    if !result.output.is_null() {
        println!("Result: {}", serde_json::to_string_pretty(&result.output).unwrap());
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

pub fn run_get_method(conf: Config, addr: &str, method: &str, params: Option<String>) -> Result<(), String> {
    let ton = create_client_verbose(&conf)?;

    let ton_addr = TonAddress::from_str(addr)
        .map_err(|e| format!("failed to parse address: {}", e.to_string()))?;

    println!("Running get-method...");
    let result = ton.contracts.run_get(
            Some(&ton_addr),
            None,
            method,
            params.map(|p| p.into()),
        )
        .map_err(|e| format!("run failed: {}", e.to_string()))?
        .output;
    
    println!("Succeded.");
    println!("Result: {}", result);
    Ok(())
}