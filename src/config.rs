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

fn default_true() -> bool { true }

fn default_lifetime() -> u32 {
    60
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_url")]
    pub url: String,
    #[serde(default = "default_wc")]
    pub wc: i32,
    pub addr: Option<String>,
    pub wallet: Option<String>,
    pub pubkey: Option<String>,
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
    #[serde(default = "default_lifetime")]
    pub lifetime: u32,
    #[serde(default = "default_true")]
    pub no_answer: bool,
    #[serde(default = "default_false")]
    pub balance_in_tons: bool,
    #[serde(default = "default_false")]
    pub local_run: bool,
    #[serde(default = "default_true")]
    pub wait_for_transaction: bool,
}

impl Config {
    pub fn new() -> Self {
        Config {
            url: default_url(),
            wc: default_wc(),
            addr: None,
            wallet: None,
            pubkey: None,
            abi_path: None,
            keys_path: None,
            retries: default_retries(),
            timeout: default_timeout(),
            is_json: default_false(),
            depool_fee: default_depool_fee(),
            lifetime: default_lifetime(),
            no_answer: default_true(),
            balance_in_tons: default_false(),
            local_run: default_false(),
            wait_for_transaction: default_true(),
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
    lifetime: bool,
    no_answer: bool,
    balance_in_tons: bool,
    local_run: bool,
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
    if lifetime {
        conf.lifetime = default_lifetime();
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
    if no_answer {
        conf.no_answer = default_true();
    }
    if balance_in_tons {
        conf.balance_in_tons = default_false();
    }
    if local_run {
        conf.local_run = default_false();
    }

    if (url || addr || wallet || abi || keys || retries || timeout || wc || depool_fee || lifetime
        || no_answer || balance_in_tons || local_run) == false {
        conf = Config::new();
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
    pubkey: Option<&str>,
    abi: Option<&str>,
    keys: Option<&str>,
    wc: Option<&str>,
    retries: Option<&str>,
    timeout: Option<&str>,
    depool_fee: Option<&str>,
    lifetime:  Option<&str>,
    no_answer:  Option<&str>,
    balance_in_tons: Option<&str>,
    local_run: Option<&str>,
    wait_for_transaction: Option<&str>,
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
    if let Some(s) = pubkey {
        conf.pubkey = Some(s.to_string());
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
    if let Some(lifetime) = lifetime {
        conf.lifetime = u32::from_str_radix(lifetime, 10)
            .map_err(|e| format!(r#"failed to parse "lifetime": {}"#, e))?;
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
    if let Some(no_answer) = no_answer {
        conf.no_answer = no_answer.parse::<bool>()
            .map_err(|e| format!(r#"failed to parse "no_answer": {}"#, e))?;
    }
    if let Some(balance_in_tons) = balance_in_tons {
        conf.balance_in_tons = balance_in_tons.parse::<bool>()
            .map_err(|e| format!(r#"failed to parse "balance_in_tons": {}"#, e))?;
    }
    if let Some(local_run) = local_run {
        conf.local_run = local_run.parse::<bool>()
            .map_err(|e| format!(r#"failed to parse "local_run": {}"#, e))?;
    }
    if let Some(wait_for_transaction) = wait_for_transaction {
        conf.wait_for_transaction = wait_for_transaction.parse::<bool>()
            .map_err(|e| format!(r#"failed to parse "wait_for_transaction": {}"#, e))?;
    }

    let conf_str = serde_json::to_string(&conf)
        .map_err(|_| "failed to serialize config object".to_string())?;
    std::fs::write(path, conf_str).map_err(|e| format!("failed to write config file: {}", e))?;
    if !conf.is_json {
        println!("Succeeded.");
    }
    Ok(())
}