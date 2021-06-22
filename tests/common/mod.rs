use assert_cmd::Command;
use lazy_static::*;
use std::env;
use serde_json::{Value, Map};

pub const BIN_NAME: &str = "tonos-cli";

lazy_static! {
    pub static ref NETWORK: String = env::var("TON_NETWORK_ADDRESS")
        .unwrap_or("http://127.0.0.1".to_string());
}

#[allow(dead_code)]
pub fn get_config() -> Result<Map<String, Value>, Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    let out = cmd.arg("config")
        .arg("--list")
        .output()
        .expect("Failed to get config.");

    let mut out = String::from_utf8_lossy(&out.stdout).to_string();
    out.replace_range(..out.find('\n').unwrap_or(0), "");
    let parsed: Value = serde_json::from_str(&out)?;
    let obj: Map<String, Value> = parsed.as_object().unwrap().clone();
    Ok(obj)
}

#[allow(dead_code)]
pub fn giver(addr: &str) {
    let giver_abi_name = "tests/samples/giver.abi.json";
    let mut cmd = Command::cargo_bin(BIN_NAME).unwrap();
    cmd.arg("call")
        .arg("--abi")
        .arg(giver_abi_name)
        .arg("0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94")
        .arg("sendGrams")
        .arg(format!(r#"{{"dest":"{}","amount":1000000000}}"#, addr));
    cmd.assert()
        .success();
}

#[allow(dead_code)]
pub fn grep_address(output: &[u8]) -> String {
    let mut addr = String::from_utf8_lossy(output).to_string();
    addr.replace_range(..addr.find("0:").unwrap_or(0), "");
    addr.replace_range(addr.find("testnet").unwrap_or(addr.len())-1.., "");
    addr
}