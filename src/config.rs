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
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use clap::ArgMatches;
use lazy_static::lazy_static;
use regex::Regex;
use crate::global_config_path;
use crate::helpers::default_config_name;

const TESTNET: &str = "net.evercloud.dev";
const MAINNET: &str = "main.evercloud.dev";
const GOSH: &str = "gosh.sh";
pub const LOCALNET: &str = "http://127.0.0.1/";

fn default_wc() -> i32 {
    0
}

fn default_retries() -> u8 {
    0
}

fn default_depool_fee() -> f32 {
    0.5
}

fn default_timeout() -> u32 {
    40000
}

fn default_out_of_sync() -> u32 { 15 }

fn default_false() -> bool {
    false
}

fn default_true() -> bool { true }

fn default_lifetime() -> u32 {
    40
}

fn default_endpoints() -> Vec<String> {
    Vec::new()
}

fn default_aliases() -> BTreeMap<String, ContractData> {
    BTreeMap::new()
}

fn default_trace() -> String { "None".to_string() }

fn default_config() -> Config {
    Config::new()
}

fn default_global_timeout() -> u32 {
    300
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Config {
    #[serde(default = "default_wc")]
    pub wc: i32,
    pub addr: Option<String>,
    pub method: Option<String>,
    pub parameters: Option<String>,
    pub wallet: Option<String>,
    pub pubkey: Option<String>,
    pub abi_path: Option<String>,
    pub keys_path: Option<String>,
    #[serde(default = "default_retries")]
    pub retries: u8,
    #[serde(default = "default_timeout")]
    pub timeout: u32,
    #[serde(default = "default_timeout")]
    pub message_processing_timeout: u32,
    #[serde(default = "default_out_of_sync")]
    pub out_of_sync_threshold: u32,
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
    #[serde(default = "default_false")]
    pub async_call: bool,
    #[serde(default = "default_trace")]
    pub debug_fail: String,
    #[serde(default = "default_global_timeout")]
    pub global_timeout: u32,

    // SDK authentication parameters
    pub project_id: Option<String>,
    pub access_key: Option<String>,
    ////////////////////////////////

    #[serde(default = "default_endpoints")]
    pub endpoints: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ContractData {
    pub abi_path: Option<String>,
    pub address: Option<String>,
    pub key_path: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FullConfig {
    #[serde(default = "default_config")]
    pub config: Config,
    #[serde(default = "default_aliases")]
    pub aliases: BTreeMap<String, ContractData>,
    #[serde(default = "default_config_name")]
    pub path: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            wc: default_wc(),
            addr: None,
            method: None,
            parameters: None,
            wallet: None,
            pubkey: None,
            abi_path: None,
            keys_path: None,
            retries: default_retries(),
            timeout: default_timeout(),
            message_processing_timeout: default_timeout(),
            is_json: default_false(),
            depool_fee: default_depool_fee(),
            lifetime: default_lifetime(),
            no_answer: default_true(),
            balance_in_tons: default_false(),
            local_run: default_false(),
            async_call: default_false(),
            endpoints: default_endpoints(),
            out_of_sync_threshold: default_out_of_sync(),
            debug_fail: default_trace(),
            global_timeout: default_global_timeout(),
            project_id: None,
            access_key: None,
        }
    }
}

impl Default for FullConfig {
    fn default() -> Self {
        FullConfig {
            config: default_config(),
            aliases: default_aliases(),
            path: default_config_name(),
        }
    }
}

impl Config {
    fn new() -> Self {
        Config {
            wc: default_wc(),
            addr: None,
            method: None,
            parameters: None,
            wallet: None,
            pubkey: None,
            abi_path: None,
            keys_path: None,
            retries: default_retries(),
            timeout: default_timeout(),
            message_processing_timeout: default_timeout(),
            is_json: default_false(),
            depool_fee: default_depool_fee(),
            lifetime: default_lifetime(),
            no_answer: default_true(),
            balance_in_tons: default_false(),
            local_run: default_false(),
            async_call: default_false(),
            endpoints: default_endpoints(),
            out_of_sync_threshold: default_out_of_sync(),
            debug_fail: default_trace(),
            global_timeout: default_global_timeout(),
            project_id: None,
            access_key: None,
        }
    }
}


lazy_static! {
    static ref MAIN_ENDPOINTS: Vec<String> = vec![
        "https://mainnet.evercloud.dev".to_string()
    ];

    static ref NET_ENDPOINTS: Vec<String> = vec![
        "https://devnet.evercloud.dev".to_string()
    ];

    static ref SE_ENDPOINTS: Vec<String> = vec![
        "http://0.0.0.0".to_string(),
        "http://127.0.0.1".to_string(),
        "http://localhost".to_string(),
    ];
    static ref GOSH_ENDPOINTS: Vec<String> = vec![
        "https://bhs01.network.gosh.sh".to_string(),
        "https://eri01.network.gosh.sh".to_string(),
        "https://gra01.network.gosh.sh".to_string()
    ];
}

pub fn resolve_net_name(url: &str) -> Option<Vec<String>> {
    if url == "main" {
        return Some(MAIN_ENDPOINTS.to_owned());
    }
    if url == "dev" || url == "devnet" {
        return Some(NET_ENDPOINTS.to_owned());
    }
    if url.contains("127.0.0.1") ||
        url.contains("0.0.0.0") ||
        url.contains("localhost") {
        return Some(SE_ENDPOINTS.to_owned());
    }
    if url == "network.gosh.sh" || url == "gosh.sh" || url == "gosh" {
        return Some(GOSH_ENDPOINTS.to_owned());
    }
    None
}

impl FullConfig {
    fn new(config: Config,path: String) -> Self {
        FullConfig {
            config,
            aliases: BTreeMap::new(),
            path,
        }
    }

    pub fn from_file(path: &str) -> FullConfig {
        let conf_str = std::fs::read_to_string(path).ok().unwrap_or_default();
        let config: serde_json::error::Result<Config>  = serde_json::from_str(&conf_str);
        if config.is_ok() && config.as_ref().unwrap() != &Config::default() {
            return FullConfig::new(config.unwrap(), path.to_string());
        }
        let full_config: serde_json::error::Result<FullConfig> = serde_json::from_str(&conf_str);
        let mut full_config = if full_config.is_err() {
            let conf_str = std::fs::read_to_string(&global_config_path()).ok()
                .unwrap_or_default();
            let mut global_config = serde_json::from_str::<FullConfig>(&conf_str)
                .unwrap_or(FullConfig::default());
            global_config.path = path.to_string();
            global_config
        } else {
            full_config.unwrap()
        };
        full_config.path = path.to_string();
        full_config
    }

    pub fn to_file(&self, path: &str) -> Result<(), String>{
        let conf_str = serde_json::to_string_pretty(self)
            .map_err(|_| "failed to serialize config object".to_string())?;
        std::fs::write(path, conf_str).map_err(|e| format!("failed to write config file {}: {}", path, e))?;
        Ok(())
    }

    pub fn print_aliases(&self) {
        println!(
            "{}",
            serde_json::to_string_pretty(&self.aliases).unwrap_or(
                "Failed to print aliases map.".to_owned()
            )
        );
    }

    pub fn add_alias(&mut self, alias: &str, address: Option<String>, abi: Option<String>, key_path: Option<String>) -> Result<(), String> {
        self.aliases.insert(alias.to_owned(), ContractData {abi_path: abi, address, key_path} );
        self.to_file(&self.path)
    }

    pub fn remove_alias(&mut self, alias: &str) -> Result<(), String> {
        self.aliases.remove(alias);
        self.to_file(&self.path)
    }
}

pub fn clear_config(
    full_config: &mut FullConfig,
    matches: &ArgMatches,
    is_json: bool,
) -> Result<(), String> {
    let mut config = &mut full_config.config;
    let is_json = config.is_json || is_json;
    if matches.is_present("ADDR") {
        config.addr = None;
    }
    if matches.is_present("WALLET") {
        config.wallet = None;
    }
    if matches.is_present("ABI") {
        config.abi_path = None;
    }
    if matches.is_present("KEYS") {
        config.keys_path = None;
    }
    if matches.is_present("METHOD") {
        config.method = None;
    }
    if matches.is_present("PARAMETERS") {
        config.parameters = None;
    }
    if matches.is_present("PUBKEY") {
        config.pubkey = None;
    }
    if matches.is_present("RETRIES") {
        config.retries = default_retries();
    }
    if matches.is_present("LIFETIME") {
        config.lifetime = default_lifetime();
    }
    if matches.is_present("TIMEOUT") {
        config.timeout = default_timeout();
    }
    if matches.is_present("MSG_TIMEOUT") {
        config.timeout = default_timeout();
    }
    if matches.is_present("WC") {
        config.wc = default_wc();
    }
    if matches.is_present("DEPOOL_FEE") {
        config.depool_fee = default_depool_fee();
    }
    if matches.is_present("NO_ANSWER") {
        config.no_answer = default_true();
    }
    if matches.is_present("BALANCE_IN_TONS") {
        config.balance_in_tons = default_false();
    }
    if matches.is_present("LOCAL_RUN") {
        config.local_run = default_false();
    }
    if matches.is_present("ASYNC_CALL") {
        config.async_call = default_false();
    }
    if matches.is_present("DEBUG_FAIL") {
        config.debug_fail = default_trace();
    }
    if matches.is_present("OUT_OF_SYNC") {
        config.out_of_sync_threshold = default_out_of_sync();
    }
    if matches.is_present("IS_JSON") {
        config.is_json = default_false();
    }
    if matches.is_present("PROJECT_ID") {
        config.project_id = None;
        if config.access_key.is_some() && !config.is_json {
            println!("Warning: You have access_key set without project_id. It has no sense in case of authentication.");
        }
    }
    if matches.is_present("ACCESS_KEY") {
        config.access_key = None;
    }

    if matches.args.is_empty() {
        *config = Config::new();
    }

    full_config.to_file(&full_config.path)?;
    if !is_json {
        println!("Succeeded.");
    }
    Ok(())
}

pub fn parse_endpoints(
    config: &mut Config,
    endpoints_str: &str
) {
    let new_endpoints : Vec<String> = endpoints_str
        .replace('[', "")
        .replace(']', "")
        .split(',')
        .map(|s| s.to_string())
        .collect();

    if new_endpoints.len() == 1 {
        match resolve_net_name(&new_endpoints[0]) {
            Some(endpoints) => { config.endpoints = endpoints },
            _ => { config.endpoints = new_endpoints }
        }
    } else {
        config.endpoints = new_endpoints;
    }
}

pub fn set_config(
    full_config: &mut FullConfig,
    matches: &ArgMatches,
    is_json: bool,
) -> Result<(), String> {
    let mut config= &mut full_config.config;
    if let Some(endpoints) = matches.value_of("ENDPOINTS") {
        parse_endpoints(config, endpoints);
    }
    if let Some(s) = matches.value_of("ADDR") {
        config.addr = Some(s.to_string());
    }
    if let Some(method) = matches.value_of("METHOD") {
        config.method = Some(method.to_string());
    }
    if let Some(parameters) = matches.value_of("PARAMETERS") {
        config.parameters = Some(parameters.to_string());
    }
    if let Some(s) = matches.value_of("WALLET") {
        config.wallet = Some(s.to_string());
    }
    if let Some(s) = matches.value_of("PUBKEY") {
        config.pubkey = Some(s.to_string());
    }
    if let Some(s) = matches.value_of("ABI") {
        config.abi_path = Some(s.to_string());
    }
    if let Some(s) = matches.value_of("KEYS") {
        config.keys_path = Some(s.to_string());
    }
    if let Some(retries) = matches.value_of("RETRIES") {
        config.retries = u8::from_str_radix(retries, 10)
            .map_err(|e| format!(r#"failed to parse "retries": {}"#, e))?;
    }
    if let Some(lifetime) = matches.value_of("LIFETIME") {
        config.lifetime = u32::from_str_radix(lifetime, 10)
            .map_err(|e| format!(r#"failed to parse "lifetime": {}"#, e))?;
        if config.lifetime < 2 * config.out_of_sync_threshold {
            config.out_of_sync_threshold = config.lifetime >> 1;
        }
    }
    if let Some(timeout) = matches.value_of("TIMEOUT") {
        config.timeout = u32::from_str_radix(timeout, 10)
            .map_err(|e| format!(r#"failed to parse "timeout": {}"#, e))?;
    }
    if let Some(timeout) = matches.value_of("GLOBAL_TIMEOUT") {
        config.global_timeout = u32::from_str_radix(timeout, 10)
            .map_err(|e| format!(r#"failed to parse "global_timeout": {}"#, e))?;
    }
    if let Some(message_processing_timeout) = matches.value_of("MSG_TIMEOUT") {
        config.message_processing_timeout = u32::from_str_radix(message_processing_timeout, 10)
            .map_err(|e| format!(r#"failed to parse "message_processing_timeout": {}"#, e))?;
    }
    if let Some(wc) = matches.value_of("WC") {
        config.wc = i32::from_str_radix(wc, 10)
            .map_err(|e| format!(r#"failed to parse "workchain id": {}"#, e))?;
    }
    if let Some(depool_fee) = matches.value_of("DEPOOL_FEE") {
        config.depool_fee = depool_fee.parse::<f32>()
            .map_err(|e| format!(r#"failed to parse "depool_fee": {}"#, e))?;
        if config.depool_fee < 0.5 {
            return Err("Minimal value for depool fee is 0.5".to_string());
        }
    }
    if let Some(no_answer) = matches.value_of("NO_ANSWER") {
        config.no_answer = no_answer.parse::<bool>()
            .map_err(|e| format!(r#"failed to parse "no_answer": {}"#, e))?;
    }
    if let Some(balance_in_tons) = matches.value_of("BALANCE_IN_TONS") {
        config.balance_in_tons = balance_in_tons.parse::<bool>()
            .map_err(|e| format!(r#"failed to parse "balance_in_tons": {}"#, e))?;
    }
    if let Some(local_run) = matches.value_of("LOCAL_RUN") {
        config.local_run = local_run.parse::<bool>()
            .map_err(|e| format!(r#"failed to parse "local_run": {}"#, e))?;
    }
    if let Some(async_call) = matches.value_of("ASYNC_CALL") {
        config.async_call = async_call.parse::<bool>()
            .map_err(|e| format!(r#"failed to parse "async_call": {}"#, e))?;
    }
    if let Some(out_of_sync_threshold) = matches.value_of("OUT_OF_SYNC") {
        let time = u32::from_str_radix(out_of_sync_threshold, 10)
            .map_err(|e| format!(r#"failed to parse "out_of_sync_threshold": {}"#, e))?;
        if time * 2 > config.lifetime {
            return  Err("\"out_of_sync\" should not exceed 0.5 * \"lifetime\".".to_string());
        }
        config.out_of_sync_threshold = time;
    }
    if let Some(debug_fail) = matches.value_of("DEBUG_FAIL") {
        let debug_fail = debug_fail.to_lowercase();
        config.debug_fail = if debug_fail == "full" {
            "Full".to_string()
        } else if debug_fail == "minimal" {
            "Minimal".to_string()
        } else if debug_fail == "none" {
            "None".to_string()
        } else {
            return Err(r#"Wrong value for "debug_fail" config."#.to_string())
        };
    }
    if let Some(is_json) = matches.value_of("IS_JSON") {
        config.is_json = is_json.parse::<bool>()
            .map_err(|e| format!(r#"failed to parse "is_json": {}"#, e))?;
    }
    if let Some(s) = matches.value_of("PROJECT_ID") {
        config.project_id = Some(s.to_string());
    }
    if let Some(s) = matches.value_of("ACCESS_KEY") {
        config.access_key = Some(s.to_string());
        if config.project_id.is_none() && !(config.is_json || is_json) {
            println!("Warning: You have access_key set without project_id. It has no sense in case of authentication.");
        }
    }

    full_config.to_file(&full_config.path)?;
    if !(full_config.config.is_json || is_json) {
        println!("Succeeded.");
    }
    Ok(())
}