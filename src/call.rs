/*
 * Copyright 2018-2019 TON DEV SOLUTIONS LTD.
 *
 * Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
 * this file except in compliance with the License.  You may obtain a copy of the
 * License at: https://ton.dev/licenses
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific TON DEV software governing permissions and
 * limitations under the License.
 */
use crate::config::Config;
use crate::crypto::{generate_keypair_from_mnemonic, KeyPair};
use crate::helpers::read_keys;
use chrono::{TimeZone, Local};
use hex;
use std::time::SystemTime;
use ton_client_rs::{
    TonClient, TonClientConfig, TonAddress, Ed25519KeyPair, EncodedMessage
};
use ton_types::cells_serialization::{BagOfCells};

fn now() -> u32 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as u32
}

fn keypair_to_ed25519pair(pair: KeyPair) -> Result<Ed25519KeyPair, String> {
    let mut buffer = [0u8; 64];
    let public_vec = hex::decode(&pair.public)
        .map_err(|e| format!("failed to decode public key: {}", e))?;
    let private_vec = hex::decode(&pair.secret)
        .map_err(|e| format!("failed to decode private key: {}", e))?;

    buffer[..32].copy_from_slice(&private_vec);
    buffer[32..].copy_from_slice(&public_vec);

    Ok(Ed25519KeyPair::zero().from_bytes(buffer))
}

fn load_keypair(keys: Option<String>) -> Result<Option<Ed25519KeyPair>, String> {
    match keys {
        Some(keys) => {
            if keys.find(' ').is_none() {
                let keys = read_keys(&keys)?;
                Ok(Some(keys))
            } else {
                let pair = generate_keypair_from_mnemonic(&keys)?;
                Ok(Some(keypair_to_ed25519pair(pair)?))
            }
        },
        None => Ok(None),
    }
}

fn create_client(url: String) -> Result<TonClient, String> {
    TonClient::new(&TonClientConfig{
        base_url: Some(url),
        message_retries_count: Some(0),
        message_expiration_timeout: Some(20000),
        message_expiration_timeout_grow_factor: Some(1.5),
        message_processing_timeout: Some(20000),
        message_processing_timeout_grow_factor: Some(1.5),
        wait_for_timeout: None,
        access_key: None,
    })
    .map_err(|e| format!("failed to create tonclient: {}", e.to_string()))
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
    
    let keys = load_keypair(keys)?;

    ton.contracts.create_run_message(
        addr,
        abi,
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
            "expire": msg.expire
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
    
    let msg = EncodedMessage {
        message_id, message_body, expire
    };
    Ok((msg, method))
}

fn decode_call_parameters(ton: &TonClient, msg: &EncodedMessage, abi: &str) -> Result<(String, String), String> {
    let tvm_msg = ton_sdk::Contract::deserialize_message(&msg.message_body[..]).unwrap();
    let body_slice = tvm_msg.body().unwrap();

    let mut data = Vec::new();
    let bag = BagOfCells::with_root(&body_slice.cell());
    bag.write_to(&mut data, false)
        .map_err(|e| format!("couldn't create body BOC: {}", e))?;
        
    let result = ton.contracts.decode_input_message_body(
        &abi,
        &data[..]
    ).map_err(|e| format!("couldn't decode message body: {}", e))?;

    Ok((
        result.function,
        serde_json::to_string_pretty(&result.output).unwrap()
    ))
}

pub fn call_contract(
    conf: Config,
    addr: &str,
    abi: String,
    method: &str,
    params: &str,
    keys: Option<String>,
    local: bool,
) -> Result<(), String> {
    let ton = create_client(conf.url.clone())?;

    let ton_addr = TonAddress::from_str(addr)
        .map_err(|e| format!("failed to parse address: {}", e.to_string()))?;

    let result = if local {
        println!("Running get-method...");
        ton.contracts.run_local(
            &ton_addr,
            None,
            &abi,
            method,
            None,
            params.into(),
            None
        )
        .map_err(|e| format!("run failed: {}", e.to_string()))?
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

        print_encoded_message(&msg);
        println!("Processing... ");

        ton.contracts.process_message(msg, Some(&abi), Some(method), None)
            .map_err(|e| format!("Failed: {}", e.to_string()))?
            .output
    };

    println!("Succeded.");
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
    print_encoded_message(&msg);

    let str_msg = pack_message(&msg, method);
    println!("Message: {}", &str_msg);
    println!();
    qr2term::print_qr(&str_msg).unwrap();
    println!();
    Ok(())
}

pub fn call_contract_with_msg(conf: Config, str_msg: String, abi: String) -> Result<(), String> {
    let ton = create_client(conf.url.clone())?;

    let (msg, method) = unpack_message(&str_msg)?;
    print_encoded_message(&msg);

    let params = decode_call_parameters(&ton, &msg, &abi)?;

    println!("Calling method {} with parameters:", params.0);
    println!("{}", params.1);
    println!("Processing... ");
    let result = ton.contracts.process_message(
        msg,
        Some(&abi),
        Some(&method),
        None
    )
    .map_err(|e| format!("Failed: {}", e.to_string()))?;

    println!("Succeded.");
    if !result.output.is_null() {
        println!("Result: {}", serde_json::to_string_pretty(&result.output).unwrap());
    }
    Ok(())
}