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
use ton_client_rs::{TonClient, TonClientConfig, Ed25519KeyPair, TonAddress};
use std::time::SystemTime;

const MAX_LEVEL: log::LevelFilter = log::LevelFilter::Warn;

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() < MAX_LEVEL
    }

    fn log(&self, record: &log::Record) {
		match record.level() {
			log::Level::Error | log::Level::Warn => {
				eprintln!("{}", record.args());
			}
			_ => {
				println!("{}", record.args());
			}
		}
    }

    fn flush(&self) {}
}

pub fn read_keys(filename: &str) -> Result<Ed25519KeyPair, String> {
    let keys_str = std::fs::read_to_string(filename)
        .map_err(|e| format!("failed to read keypair file: {}", e.to_string()))?;
    serde_json::from_str(&keys_str)
        .map_err(|e| format!("failed to parse keypair file: {}", e))
}

pub fn load_ton_address(addr: &str) -> Result<TonAddress, String> {
    TonAddress::from_str(addr)
        .map_err(|e| format!("failed to parse address: {}", e.to_string()))
}

pub fn now() -> u32 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as u32
}

pub fn create_client_local() -> Result<TonClient, String> {
    TonClient::default()
        .map_err(|e| format!("failed to create tonclient: {}", e.to_string()))
}

pub fn create_client(conf: &Config) -> Result<TonClient, String> {
    TonClient::new(&TonClientConfig{
        base_url: Some(conf.url.clone()),
        message_retries_count: Some(conf.retries),
        message_expiration_timeout: Some(conf.timeout),
        message_expiration_timeout_grow_factor: Some(1.5),
        message_processing_timeout: Some(conf.timeout),
        wait_for_timeout: Some(60 * 60000),
        access_key: None,
        out_of_sync_threshold: None,
    })
    .map_err(|e| format!("failed to create tonclient: {}", e.to_string()))
}

pub fn create_client_verbose(conf: &Config) -> Result<TonClient, String> {
    println!("Connecting to {}", conf.url);

    log::set_max_level(MAX_LEVEL);
    log::set_boxed_logger(Box::new(SimpleLogger))
        .map_err(|e| format!("failed to init logger: {}", e))?;
    
    create_client(conf)
}
