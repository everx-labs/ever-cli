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
use crate::call::create_client_verbose;
use crate::config::Config;
use ton_client_rs::{EncodedMessage, TonAddress};

pub fn sendfile(conf: Config, msg_boc: &str) -> Result<(), String> {
    let ton = create_client_verbose(&conf)?;
    let boc_vec = std::fs::read(msg_boc)
        .map_err(|e| format!("failed to read boc file: {}", e))?;

    let tvm_msg = ton_sdk::Contract::deserialize_message(&boc_vec[..])
        .map_err(|e| format!("failed to parse message from boc: {}", e))?;
    let dst = tvm_msg.dst()
        .ok_or(format!("failed to parse dst address"))?;

    let ton_addr = TonAddress::from_str(&format!("{}", dst))
        .map_err(|e| format!("failed to parse address: {}", e.to_string()))?;
        
    println!("Sending message to account {}", ton_addr);
    let msg = EncodedMessage {
        message_id: String::new(),
        message_body: boc_vec,
        expire: None,
        address: ton_addr,
    };
    ton.contracts.send_message(msg)
        .map_err(|e| format!("Failed: {}", e.to_string()))?;
    println!("Succeded.");
    Ok(())
}