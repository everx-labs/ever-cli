use crate::debot::term_browser::terminal_input;
use crate::helpers::load_ton_address;
use serde_json::Value;
use ton_client::abi::Abi;
use ton_client::debot::{DebotInterface, InterfaceResult};
use super::dinterface::decode_answer_id;
use crate::config::Config;

const ID: &'static str = "d7ed1bd8e6230871116f4522e58df0a93c5520c56f4ade23ef3d8919a984653b";

pub const ABI: &str = r#"
{
	"ABI version": 2,
	"header": ["time"],
	"functions": [
		{
			"name": "select",
			"inputs": [
				{"name":"answerId","type":"uint32"}
			],
			"outputs": [
				{"name":"value","type":"address"}
			]
		},
		{
			"name": "constructor",
			"inputs": [
			],
			"outputs": [
			]
		}
	],
	"data": [
	],
	"events": [
	]
}
"#;

pub struct AddressInput {
    conf: Config
}
impl AddressInput {
    pub fn new(conf: Config) -> Self {
        Self {conf}
    }
    fn select(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let value = terminal_input("", |val| {
            let _ = load_ton_address(val, &self.conf).map_err(|e| format!("Invalid address: {}", e))?;
            Ok(())
        });
        Ok((answer_id, json!({ "value": value })))
    }
}

#[async_trait::async_trait]
impl DebotInterface for AddressInput {
    fn get_id(&self) -> String {
        ID.to_string()
    }

    fn get_abi(&self) -> Abi {
        Abi::Json(ABI.to_owned())
    }

    async fn call(&self, func: &str, args: &Value) -> InterfaceResult {
        match func {
            "select" => self.select(args),
            _ => Err(format!("function \"{}\" is not implemented", func)),
        }
    }
}
