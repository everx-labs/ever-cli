use super::dinterface::decode_answer_id;
use crate::config::Config;
use crate::debot::term_signing_box::TerminalSigningBox;
use crate::helpers::TonClient;
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
		},
        {
			"name": "getSigningBox",
			"inputs": [
				{"name":"answerId","type":"uint32"}
			],
			"outputs": [
				{"name":"handle","type":"uint32"}
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
    client: TonClient,
    config: Config,
}
impl UserInfo {
    pub fn new(client: TonClient, config: Config) -> Self {
        Self { client, config }
    }

    fn get_account(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let value = self
            .config
            .wallet
            .clone()
            .unwrap_or_else(|| format!("0:{:064}", 0));
        Ok((answer_id, json!({ "value": value })))
    }

    fn get_public_key(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let value = self
            .config
            .pubkey
            .clone()
            .unwrap_or_else(|| format!("0x{:064}", 0));
        Ok((answer_id, json!({ "value": value })))
    }

    async fn get_signing_box(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let mut signing_box = TerminalSigningBox::new_with_keypath(
            self.client.clone(),
            self.config.keys_path.clone().unwrap_or_default(),
        )
        .await?;
        let handle = signing_box.leak();
        Ok((answer_id, json!({ "handle": handle.0})))
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
            "getSigningBox" => self.get_signing_box(args).await,
            _ => Err(format!("function \"{}\" is not implemented", func)),
        }
    }
}
