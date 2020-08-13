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
use ton_client_rs::{Ed25519KeyPair, TonAddress};
use std::time::SystemTime;

pub fn read_keys(filename: &str) -> Result<Ed25519KeyPair, String> {
    let keys_str = std::fs::read_to_string(filename)
        .map_err(|e| format!("failed to read keypair file: {}", e.to_string()))?;
    let keys: Ed25519KeyPair = serde_json::from_str(&keys_str).unwrap();
    Ok(keys)
}

pub fn load_ton_address(addr: &str) -> Result<TonAddress, String> {
    TonAddress::from_str(addr)
        .map_err(|e| format!("failed to parse address: {}", e.to_string()))
}

pub fn now() -> u32 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as u32
}
