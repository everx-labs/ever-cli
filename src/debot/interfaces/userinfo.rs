use super::dinterface::decode_answer_id;
use crate::config::Config;
use serde_json::Value;
use ton_client::abi::Abi;
use ton_client::debot::{DebotInterface, InterfaceResult};

const ID: &'static str = "a56115147709ed3437efb89460b94a120b7fe94379c795d1ebb0435a847ee580";

const ABI: &str = r#"
{
	"ABI version": 2,
	"header": ["time"],
	"functions": [
		{
			"name": "getAccount",
			"inputs": [
				{"name":"answerId","type":"uint32"}
			],
			"outputs": [
				{"name":"value","type":"address"}
			]
		},
		{
			"name": "getPublicKey",
			"inputs": [
				{"name":"answerId","type":"uint32"}
			],
			"outputs": [
				{"name":"value","type":"uint256"}
			]
		}
	],
	"data": [
	],
	"events": [
	]
}
"#;

pub struct UserInfo {
    config: Config,
}
impl UserInfo {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    fn get_account(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let value = self.config.wallet.clone().unwrap_or_else(|| format!("0:{:064}", 0));
        Ok((answer_id, json!({ "value": value })))
    }

    fn get_public_key(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let value = self.config.pubkey.clone().unwrap_or_else(|| format!("0x{:064}", 0));
        Ok((answer_id, json!({ "value": value })))
    }
}

#[async_trait::async_trait]
impl DebotInterface for UserInfo {
    fn get_id(&self) -> String {
        ID.to_string()
    }

    fn get_abi(&self) -> Abi {
        Abi::Json(ABI.to_owned())
    }

    async fn call(&self, func: &str, args: &Value) -> InterfaceResult {
        match func {
            "getAccount" => self.get_account(args),
            "getPublicKey" => self.get_public_key(args),
            _ => Err(format!("function \"{}\" is not implemented", func)),
        }
    }
}
