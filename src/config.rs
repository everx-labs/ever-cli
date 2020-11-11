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
use serde::{Deserialize, Serialize};

const TESTNET: &'static str = "https://net.ton.dev";
fn default_url() -> String {
    TESTNET.to_string()
}

fn default_wc() -> i32 {
    0
}

fn default_retries() -> u8 {
    5
}

fn default_depool_fee() -> f32 {
    0.5
}

fn default_timeout() -> u32 {
    60000
}

fn default_false() -> bool {
    false
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_url")]
    pub url: String,
    #[serde(default = "default_wc")]
    pub wc: i32,
    pub addr: Option<String>,
    pub wallet: Option<String>,
    pub abi_path: Option<String>,
    pub keys_path: Option<String>,
    #[serde(default = "default_retries")]
    pub retries: u8,
    #[serde(default = "default_timeout")]
    pub timeout: u32,
    #[serde(default = "default_false")]
    pub is_json: bool,
    #[serde(default = "default_depool_fee")]
    pub depool_fee: f32,
}

impl Config {
    pub fn new() -> Self {
        Config {
            url: default_url(),
            wc: default_wc(),
            addr: None,
            wallet: None,
            abi_path: None,
            keys_path: None,
            retries: default_retries(),
            timeout: default_timeout(),
            is_json: default_false(),
            depool_fee: default_depool_fee(),
        }
    }

    pub fn from_file(path: &str) -> Option<Self> {
        let conf_str = std::fs::read_to_string(path).ok()?;
        let conf: Config = serde_json::from_str(&conf_str).ok()?;
        Some(conf)
    }
}

pub fn clear_config(
    mut conf: Config,
    path: &str,
    url: bool,
    addr: bool,
    wallet: bool,
    abi: bool,
    keys: bool,
    wc: bool,
    retries: bool,
    timeout: bool,
    depool_fee: bool, 
) -> Result<(), String> {
    if url {
        conf.url = default_url();
    }
    if addr {
        conf.addr = None;
    }
    if wallet {
        conf.wallet = None;
    }
    if abi {
        conf.abi_path = None;
    }
    if keys {
        conf.keys_path = None;
    }
    if retries {
        conf.retries = default_retries();
    }
    if timeout {
        conf.timeout = default_timeout();
    }
    if wc {
        conf.wc = default_wc();
    }
    if depool_fee {
        conf.depool_fee = default_depool_fee();
    }
    if (url || addr || wallet || abi || keys || retries || timeout || wc || depool_fee) == false {
        conf = Config {
            url: default_url(),
            wc: default_wc(),
            addr: None,
            wallet: None,
            abi_path: None,
            keys_path: None,
            retries: default_retries(),
            timeout: default_timeout(),
            is_json: default_false(),
            depool_fee: default_depool_fee(),
        };
    }
    let conf_str = serde_json::to_string(&conf)
        .map_err(|_| "failed to serialize config object".to_string())?;

    std::fs::write(path, conf_str).map_err(|e| format!("failed to write config file: {}", e))?;
    println!("Succeeded.");
    Ok(())
}

pub fn set_config(
    mut conf: Config,
    path: &str,
    url: Option<&str>,
    addr: Option<&str>,
    wallet: Option<&str>,
    abi: Option<&str>,
    keys: Option<&str>,
    wc: Option<&str>,
    retries: Option<&str>,
    timeout: Option<&str>,
    depool_fee: Option<&str>, 
) -> Result<(), String> {
        if let Some(s) = url {
            conf.url = s.to_string();
        }
        if let Some(s) = addr {
            conf.addr = Some(s.to_string());
        }
        if let Some(s) = wallet {
            conf.wallet = Some(s.to_string());
        }
        if let Some(s) = abi {
            conf.abi_path = Some(s.to_string());
        }
        if let Some(s) = keys {
            conf.keys_path = Some(s.to_string());
        }
        if let Some(retries) = retries {
            conf.retries = u8::from_str_radix(retries, 10)
                .map_err(|e| format!(r#"failed to parse "retries": {}"#, e))?;
        }
        if let Some(timeout) = timeout {
            conf.timeout = u32::from_str_radix(timeout, 10)
                .map_err(|e| format!(r#"failed to parse "timeout": {}"#, e))?;
        }
        if let Some(wc) = wc {
            conf.wc = i32::from_str_radix(wc, 10)
                .map_err(|e| format!(r#"failed to parse "workchain id": {}"#, e))?;
        }
        if let Some(depool_fee) = depool_fee {
            conf.depool_fee = depool_fee.parse::<f32>()
                .map_err(|e| format!(r#"failed to parse "depool_fee": {}"#, e))?;
        }
        if conf.depool_fee < 0.5 {
            return Err("Minimal value for depool fee is 0.5".to_string());
        }
        let conf_str = serde_json::to_string(&conf)
            .map_err(|_| "failed to serialize config object".to_string())?;

        std::fs::write(path, conf_str).map_err(|e| format!("failed to write config file: {}", e))?;
        println!("Succeeded.");
        Ok(())
    }