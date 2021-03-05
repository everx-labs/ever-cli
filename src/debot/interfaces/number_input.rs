use crate::debot::term_browser::terminal_input;
use serde_json::Value;
use ton_client::abi::Abi;
use ton_client::debot::{DebotInterface, InterfaceResult};
use super::dinterface::{decode_answer_id, decode_int256, decode_prompt};
use ton_client::encoding::decode_abi_bigint;

const ID: &'static str = "c5a9558b2664aed7dc3e6123436d544f13ffe69ab0e259412f48c6d1c8588401";

pub const ABI: &str = r#"
{
	"ABI version": 2,
	"header": ["time"],
	"functions": [
		{
			"name": "get",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"prompt","type":"bytes"},
				{"name":"min","type":"int256"},
				{"name":"max","type":"int256"}
			],
			"outputs": [
				{"name":"value","type":"int256"}
			]
		}
	],
	"data": [
	],
	"events": [
	]
}

"#;

pub struct NumberInput {}

impl NumberInput {
    pub fn new() -> Self {
        Self {}
    }
    fn get(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let prompt = decode_prompt(args)?;
        let min = decode_int256(args, "min")?;
        let max = decode_int256(args, "max")?;
        let prompt = format!("{}\n(>= {} and <= {})", prompt, min, max);
        let value = terminal_input(&prompt, |val| {
            let number = decode_abi_bigint(val.as_str())
                .map_err(|e| format!("input is not a valid number: {}", e))?;
            if number < min || number > max {
                return Err(format!("number is out of range"));
            }
            Ok(())
        });
        Ok((answer_id, json!({ "value": value })))
    }
}

#[async_trait::async_trait]
impl DebotInterface for NumberInput {
    fn get_id(&self) -> String {
        ID.to_string()
    }

    fn get_abi(&self) -> Abi {
        Abi::Json(ABI.to_owned())
    }

    async fn call(&self, func: &str, args: &Value) -> InterfaceResult {
        match func {
            "get" => self.get(args),
            _ => Err(format!("function \"{}\" is not implemented", func)),
        }
    }
}