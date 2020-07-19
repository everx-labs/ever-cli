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
use std::time::SystemTime;
use ton_client_rs::{Ed25519KeyPair, TonAddress, TonClient, TonClientConfig};

pub fn now() -> u32 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as u32
}

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

#[allow(dead_code)]
pub fn load_tvc(tvc_file: &str) -> Result<Vec<u8>, String> {
    std::fs::read(tvc_file)
        .map_err(|e| format!("failed to read image file: {}", e.to_string()))
}

pub fn create_default_client() -> Result<TonClient, String> {
    TonClient::default()
        .map_err(|e| format!("failed to create ton client: {}", e.to_string()))
}

pub fn create_client(conf: &Config) -> Result<TonClient, String> {
    TonClient::new(&TonClientConfig{
        base_url: Some(conf.url.clone()),
        message_retries_count: Some(conf.retries),
        message_expiration_timeout: Some(conf.timeout),
        message_expiration_timeout_grow_factor: Some(1.5),
        message_processing_timeout: Some(conf.timeout),
        wait_for_timeout: None,
        access_key: None,
        out_of_sync_threshold: None,
    })
    .map_err(|e| format!("failed to create tonclient: {}", e.to_string()))
}

pub fn create_client_verbose(conf: &Config) -> Result<TonClient, String> {
    println!("Connecting to {}", conf.url);
    create_client(conf)
}