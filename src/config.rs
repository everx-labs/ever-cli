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
use serde::{Deserialize, Serialize};

const TESTNET: &'static str = "https://net.ton.dev";
fn default_url() -> String {
    TESTNET.to_string()
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_url")]
    pub url: String,
    #[serde(default)]
    pub wc: i8,
    pub addr: Option<String>,
    pub abi_path: Option<String>,
    pub keys_path: Option<String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            url: default_url(),
            wc: 0,
            addr: None,
            abi_path: None,
            keys_path: None,
        }
    }

    pub fn from_file(path: &str) -> Option<Self> {
        let conf_str = std::fs::read_to_string(path).ok()?;
        let conf: Config = serde_json::from_str(&conf_str).ok()?;
        Some(conf)
    }
}

pub fn set_config(
    mut conf: Config,
    path: &str,
    url: Option<&str>,
    addr: Option<&str>,
    abi: Option<&str>,
    keys: Option<&str>,
) -> Result<(), String> {
        if let Some(s) = url {
            conf.url = s.to_string();
        }
        if let Some(s) = addr {
            conf.addr = Some(s.to_string());
        }
        if let Some(s) = abi {
            conf.abi_path = Some(s.to_string());
        }
        if let Some(s) = keys {
            conf.keys_path = Some(s.to_string());
        }
        let conf_str = serde_json::to_string(&conf)
            .map_err(|_| "failed to serialize config object".to_string())?;

        std::fs::write(path, conf_str).map_err(|e| format!("failed to write config file: {}", e))?;
        println!("Succeeded.");
        Ok(())
    }