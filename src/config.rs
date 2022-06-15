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
use lazy_static::lazy_static;
use regex::Regex;

const TESTNET: &str = "net.ton.dev";
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
    40000
}

fn default_out_of_sync() -> u32 { 15 }

fn default_false() -> bool {
    false
}

fn default_true() -> bool { true }

fn default_lifetime() -> u32 {
    60
}

fn default_endpoints() -> Vec<String> {
    vec!()
}

fn default_trace() -> String { "None".to_string() }

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
    #[serde(default = "default_endpoints")]
    pub endpoints: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FullConfig {
    config: Config,
    endpoints_map: BTreeMap<String, Vec<String>>,
}

impl Default for Config {
    fn default() -> Self {
        let url = default_url();
        let endpoints = FullConfig::default_map()[&url].clone();
        Config {
            url,
            wc: default_wc(),
            addr: None,
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
            endpoints,
            out_of_sync_threshold: default_out_of_sync(),
            debug_fail: default_trace(),
        }
    }
}

impl Config {
    pub fn from_file(path: &str) -> Option<Self> {
        let conf_str = std::fs::read_to_string(path).ok()?;
        let config: serde_json::error::Result<FullConfig>  = serde_json::from_str(&conf_str);
        config.map(|c| c.config).or_else(|_| serde_json::from_str(&conf_str)).ok()
    }

    pub fn to_file(&self, path: &str) -> Result<(), String> {
        let mut fconf= FullConfig::from_file(path);
        fconf.config = self.clone();
        fconf.to_file(path)
    }
}


lazy_static! {
    static ref MAIN_ENDPOINTS: Vec<String> = vec![
        "https://eri01.main.everos.dev".to_string(),
        "https://gra01.main.everos.dev".to_string(),
        "https://gra02.main.everos.dev".to_string(),
        "https://lim01.main.everos.dev".to_string(),
        "https://rbx01.main.everos.dev".to_string(),
    ];

    static ref NET_ENDPOINTS: Vec<String> = vec![
        "https://eri01.net.everos.dev".to_string(),
        "https://rbx01.net.everos.dev".to_string(),
        "https://gra01.net.everos.dev".to_string(),
    ];

    static ref SE_ENDPOINTS: Vec<String> = vec![
        "http://0.0.0.0/".to_string(),
        "http://127.0.0.1/".to_string(),
        "http://localhost/".to_string(),
    ];
}

pub fn resolve_net_name(url: &str) -> Option<String> {
    let url_regex = Regex::new(r"^\s*(?:https?://)?(?P<net>\w+\.ton\.dev)\s*")
        .expect("Regex compilation error");
    if let Some(captures) = url_regex.captures(url) {
        let net = captures.name("net")
            .expect("Unexpected: capture <net> was not found")
            .as_str();
        if FullConfig::default_map().contains_key(net) {
            return Some(net.to_owned());
        }
    }
    if url.contains("127.0.0.1") ||
        url.contains("0.0.0.0") ||
        url.contains("localhost") {
        return Some("http://127.0.0.1/".to_string());
    }
    None
}

impl FullConfig {
    pub fn new() -> Self {
        FullConfig {
            config: Config::default(),
            endpoints_map: FullConfig::default_map(),
        }
    }
    pub fn default_map() -> BTreeMap<String, Vec<String>> {
        [("main.ton.dev".to_owned(), MAIN_ENDPOINTS.to_owned()),
            ("net.ton.dev".to_owned(), NET_ENDPOINTS.to_owned()),
            ("http://127.0.0.1/".to_owned(), SE_ENDPOINTS.to_owned()),
        ].iter().cloned().collect()
    }

    pub fn get_map(path: &str) -> BTreeMap<String, Vec<String>> {
        FullConfig::from_file(path).endpoints_map
    }

    pub fn from_file(path: &str) -> FullConfig {
        let conf_str = std::fs::read_to_string(path).ok().unwrap_or_default();
        serde_json::from_str(&conf_str).ok().unwrap_or(FullConfig::new())
    }

    pub fn to_file(&self, path: &str) -> Result<(), String>{
        let conf_str = serde_json::to_string_pretty(self)
            .map_err(|_| "failed to serialize config object".to_string())?;
        std::fs::write(path, conf_str).map_err(|e| format!("failed to write config file: {}", e))?;
        Ok(())
    }

    pub fn print_endpoints(path: &str) {
        let fconf = FullConfig::from_file(path);
        println!(
            "{}",
            serde_json::to_string_pretty(&fconf.endpoints_map).unwrap_or(
                "Failed to print endpoints map.".to_owned()
            )
        );
    }

    pub fn add_endpoint(path: &str, url: &str, endpoints: &str) -> Result<(), String> {
        let mut fconf = FullConfig::from_file(path);
        let mut new_endpoints : Vec<String> = endpoints
            .replace('[', "")
            .replace(']', "")
            .split(',')
            .map(|s| s.to_string())
            .collect();

        let old_endpoints = fconf.endpoints_map.entry(url.to_string()).or_insert(vec![]);
        old_endpoints.append(&mut new_endpoints);
        old_endpoints.sort();
        old_endpoints.dedup();
        fconf.to_file(path)
    }

    pub fn remove_endpoint(path: &str, url: &str) -> Result<(), String> {
        let mut fconf = FullConfig::from_file(path);
        if !fconf.endpoints_map.contains_key(url) {
            return Err("Endpoints map doesn't contain such url.".to_owned());
        }
        fconf.endpoints_map.remove(url);
        fconf.to_file(path)
    }

    pub fn reset_endpoints(path: &str) -> Result<(), String> {
        let mut fconf = FullConfig::from_file(path);
        fconf.endpoints_map = FullConfig::default_map();
        fconf.to_file(path)
    }
}

pub fn clear_config(
    mut config: Config,
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
    let is_json = config.is_json;
    if url {
        let url = default_url();
        config.endpoints = FullConfig::default_map()[&url].clone();
        config.url = url;
    }
    if addr {
        config.addr = None;
    }
    if wallet {
        config.wallet = None;
    }
    if abi {
        config.abi_path = None;
    }
    if keys {
        config.keys_path = None;
    }
    if retries {
        config.retries = default_retries();
    }
    if lifetime {
        config.lifetime = default_lifetime();
    }
    if timeout {
        config.timeout = default_timeout();
    }
    if wc {
        config.wc = default_wc();
    }
    if depool_fee {
        config.depool_fee = default_depool_fee();
    }
    if no_answer {
        config.no_answer = default_true();
    }
    if balance_in_tons {
        config.balance_in_tons = default_false();
    }
    if local_run {
        config.local_run = default_false();
    }

    if !(url || addr || wallet || abi || keys || retries || timeout || wc || depool_fee || lifetime
        || no_answer || balance_in_tons || local_run) {
        config = Config::default();
    }

    config.to_file(path)?;
    if !is_json {
        println!("Succeeded.");
    }
    Ok(())
}

pub fn set_config(
    mut config: Config,
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
    message_processing_timeout: Option<&str>,
    depool_fee: Option<&str>,
    lifetime:  Option<&str>,
    no_answer:  Option<&str>,
    balance_in_tons: Option<&str>,
    local_run: Option<&str>,
    async_call: Option<&str>,
    out_of_sync_threshold: Option<&str>,
    debug_fail: Option<&str>,
) -> Result<(), String> {
    if let Some(s) = url {
        let resolved_url = resolve_net_name(s).unwrap_or(s.to_owned());
        let empty : Vec<String> = Vec::new();
        config.endpoints = FullConfig::get_map(path).get(&resolved_url).unwrap_or(&empty).clone();
        config.url = resolved_url;
    }
    if let Some(s) = addr {
        config.addr = Some(s.to_string());
    }
    if let Some(s) = wallet {
        config.wallet = Some(s.to_string());
    }
    if let Some(s) = pubkey {
        config.pubkey = Some(s.to_string());
    }
    if let Some(s) = abi {
        config.abi_path = Some(s.to_string());
    }
    if let Some(s) = keys {
        config.keys_path = Some(s.to_string());
    }
    if let Some(retries) = retries {
        config.retries = u8::from_str_radix(retries, 10)
            .map_err(|e| format!(r#"failed to parse "retries": {}"#, e))?;
    }
    if let Some(lifetime) = lifetime {
        config.lifetime = u32::from_str_radix(lifetime, 10)
            .map_err(|e| format!(r#"failed to parse "lifetime": {}"#, e))?;
        if config.lifetime < 2 * config.out_of_sync_threshold {
            config.out_of_sync_threshold = config.lifetime >> 1;
        }
    }
    if let Some(timeout) = timeout {
        config.timeout = u32::from_str_radix(timeout, 10)
            .map_err(|e| format!(r#"failed to parse "timeout": {}"#, e))?;
    }
    if let Some(message_processing_timeout) = message_processing_timeout {
        config.message_processing_timeout = u32::from_str_radix(message_processing_timeout, 10)
            .map_err(|e| format!(r#"failed to parse "message_processing_timeout": {}"#, e))?;
    }
    if let Some(wc) = wc {
        config.wc = i32::from_str_radix(wc, 10)
            .map_err(|e| format!(r#"failed to parse "workchain id": {}"#, e))?;
    }
    if let Some(depool_fee) = depool_fee {
        config.depool_fee = depool_fee.parse::<f32>()
            .map_err(|e| format!(r#"failed to parse "depool_fee": {}"#, e))?;
    }
    if config.depool_fee < 0.5 {
        return Err("Minimal value for depool fee is 0.5".to_string());
    }
    if let Some(no_answer) = no_answer {
        config.no_answer = no_answer.parse::<bool>()
            .map_err(|e| format!(r#"failed to parse "no_answer": {}"#, e))?;
    }
    if let Some(balance_in_tons) = balance_in_tons {
        config.balance_in_tons = balance_in_tons.parse::<bool>()
            .map_err(|e| format!(r#"failed to parse "balance_in_tons": {}"#, e))?;
    }
    if let Some(local_run) = local_run {
        config.local_run = local_run.parse::<bool>()
            .map_err(|e| format!(r#"failed to parse "local_run": {}"#, e))?;
    }
    if let Some(async_call) = async_call {
        config.async_call = async_call.parse::<bool>()
            .map_err(|e| format!(r#"failed to parse "async_call": {}"#, e))?;
    }
    if let Some(out_of_sync_threshold) = out_of_sync_threshold {
        let time = u32::from_str_radix(out_of_sync_threshold, 10)
            .map_err(|e| format!(r#"failed to parse "out_of_sync_threshold": {}"#, e))?;
        if time * 2 > config.lifetime {
            return  Err("\"out_of_sync\" should not exceed 0.5 * \"lifetime\".".to_string());
        }
        config.out_of_sync_threshold = time;
    }
    if let Some(debug_fail) = debug_fail {
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

    config.to_file(path)?;
    if !config.is_json {
        println!("Succeeded.");
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::{resolve_net_name};

    #[test]
    fn test_endpoints_resolver() {
        assert_eq!(resolve_net_name(""), None);
        assert_eq!(resolve_net_name("http://os.ton.dev"), None);
        assert_eq!(resolve_net_name("https://rustnet.ton.dev"), None);
        assert_eq!(resolve_net_name("rustnet.ton.com"), None);
        assert_eq!(resolve_net_name("https://example.com"), None);
        assert_eq!(resolve_net_name("http://localhost"), Some("http://127.0.0.1/".to_owned()));
        assert_eq!(resolve_net_name("https://localhost"), Some("http://127.0.0.1/".to_owned()));
        assert_eq!(resolve_net_name("localhost"), Some("http://127.0.0.1/".to_owned()));
        assert_eq!(resolve_net_name("http://127.0.0.1"), Some("http://127.0.0.1/".to_owned()));
        assert_eq!(resolve_net_name("https://127.0.0.1"), Some("http://127.0.0.1/".to_owned()));
        assert_eq!(resolve_net_name("https://127.0.0.2"), None);
        assert_eq!(resolve_net_name("https://127.1.0.1"), None);
        assert_eq!(resolve_net_name("https://0.0.0.1"), None);
        assert_eq!(resolve_net_name("https://1.0.0.0"), None);

        assert_eq!(resolve_net_name("https://main.ton.dev"), Some("main.ton.dev".to_owned()));
        assert_eq!(resolve_net_name("http://main.ton.dev"), Some("main.ton.dev".to_owned()));
        assert_eq!(resolve_net_name("  http://main.ton.dev  "), Some("main.ton.dev".to_owned()));
        assert_eq!(resolve_net_name("  https://main.ton.dev  "), Some("main.ton.dev".to_owned()));
        assert_eq!(resolve_net_name("main.ton.dev"), Some("main.ton.dev".to_owned()));
        assert_eq!(resolve_net_name("main.ton.com"), None);

        assert_eq!(resolve_net_name("https://net.ton.dev"), Some("net.ton.dev".to_owned()));
        assert_eq!(resolve_net_name("http://net.ton.dev"), Some("net.ton.dev".to_owned()));
        assert_eq!(resolve_net_name("  http://net.ton.dev  "), Some("net.ton.dev".to_owned()));
        assert_eq!(resolve_net_name("  https://net.ton.dev  "), Some("net.ton.dev".to_owned()));
        assert_eq!(resolve_net_name("net.ton.dev"), Some("net.ton.dev".to_owned()));
        assert_eq!(resolve_net_name("net.ton.com"), None);
    }
}
