use assert_cmd::Command;
use lazy_static::*;
use predicates::prelude::*;
use serde_json::{Map, Value};
use std::env;

#[allow(dead_code)]
pub mod create {
    use super::*;

    pub const BIN_NAME: &str = "ever-cli";
    pub const GIVER_ADDR: &str =
        "0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94";
    pub const GIVER_ABI: &str = "tests/samples/giver.abi.json";
    pub const GIVER_V2_ADDR: &str =
        "0:ece57bcc6c530283becbbd8a3b24d3c5987cdddc3c8b7b33be6e4a6312490415";
    pub const GIVER_V2_ABI: &str = "tests/samples/giver_v2.abi.json";
    pub const GIVER_V2_KEY: &str = "tests/samples/giver_v2.key";

    lazy_static! {
        pub static ref NETWORK: String =
            env::var("TON_NETWORK_ADDRESS").unwrap_or("http://127.0.0.1/".to_string());
    }

    pub fn get_config() -> Result<Map<String, Value>, Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin(BIN_NAME)?;
        let out = cmd
            .arg("config")
            .arg("--list")
            .output()
            .expect("Failed to get config.");

        let mut out = String::from_utf8_lossy(&out.stdout).to_string();
        out.replace_range(..out.find('\n').unwrap_or(0), "");
        let parsed: Value = serde_json::from_str(&out)?;
        let obj: Map<String, Value> = parsed.as_object().unwrap().clone();
        Ok(obj)
    }

    pub fn set_config(
        config: &[&str],
        argument: &[&str],
        config_path: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin(BIN_NAME)?;
        if config_path.is_some() {
            cmd.arg("--config").arg(config_path.unwrap());
        }
        cmd.arg("config");
        for i in 0..config.len() {
            cmd.arg(config[i]).arg(argument[i]);
        }
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("Succeeded"));
        Ok(())
    }

    pub fn giver(addr: &str) {
        let mut cmd = Command::cargo_bin(BIN_NAME).unwrap();
        cmd.arg("call")
            .arg("--abi")
            .arg(GIVER_ABI)
            .arg(GIVER_ADDR)
            .arg("sendGrams")
            .arg(format!(r#"{{"dest":"{}","amount":1000000000}}"#, addr));
        cmd.assert().success();
    }

    pub fn giver_v2(addr: &str) {
        let mut cmd = Command::cargo_bin(BIN_NAME).unwrap();
        cmd.arg("call")
            .arg("--abi")
            .arg(GIVER_V2_ABI)
            .arg(GIVER_V2_ADDR)
            .arg("--sign")
            .arg(GIVER_V2_KEY)
            .arg("sendTransaction")
            .arg(format!(
                r#"{{"dest":"{}","value":100000000000,"bounce":false}}"#,
                addr
            ));
        cmd.assert().success();
    }

    pub fn grep_address(output: &[u8]) -> String {
        let mut addr = String::from_utf8_lossy(output).to_string();
        addr.replace_range(..addr.find("0:").unwrap_or(0), "");
        addr.replace_range(addr.find("testnet").unwrap_or(addr.len()) - 1.., "");
        addr
    }

    pub fn grep_message_id(output: &[u8]) -> String {
        let mut message_id = String::from_utf8_lossy(output).to_string();
        let index = message_id
            .find("MessageId: ")
            .map(|i| i + "MessageId: ".len())
            .unwrap_or(0);
        message_id.replace_range(..index, "");
        if message_id.len() >= 64 {
            message_id.replace_range(64.., "");
        }
        message_id
    }

    pub fn generate_key_and_address(
        key_path: &str,
        tvc_path: &str,
        abi_path: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin(BIN_NAME)?;
        let out = cmd
            .arg("genaddr")
            .arg("--genkey")
            .arg(key_path)
            .arg(tvc_path)
            .arg("--abi")
            .arg(abi_path)
            .output()
            .expect("Failed to generate address.");

        Ok(grep_address(&out.stdout))
    }

    pub fn generate_phrase_and_key(key_path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin(BIN_NAME)?;
        let out = cmd
            .arg("genphrase")
            .arg("--dump")
            .arg(key_path)
            .output()
            .expect("Failed to generate a seed phrase.");
        let mut seed = String::from_utf8_lossy(&out.stdout).to_string();
        seed.replace_range(..seed.find('"').unwrap_or(0) + 1, "");
        seed.replace_range(seed.find("Keypair").unwrap_or(seed.len()) - 2.., "");

        Ok(seed)
    }
}
