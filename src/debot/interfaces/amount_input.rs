use crate::debot::term_browser::terminal_input;
use serde_json::Value;
use ton_client::abi::Abi;
use ton_client::debot::{DebotInterface, InterfaceResult};
use super::dinterface::{decode_answer_id, decode_num_arg, decode_prompt};
use crate::config::Config;
use ton_client::encoding::decode_abi_number;
use crate::convert;

const ID: &'static str = "a1d347099e29c1624c8890619daf207bde18e92df5220a54bcc6d858309ece84";

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
				{"name":"decimals","type":"uint8"},
				{"name":"min","type":"uint128"},
				{"name":"max","type":"uint128"}
			],
			"outputs": [
				{"name":"value","type":"uint128"}
			]
		}
	],
	"data": [
	],
	"events": [
	]
}
"#;

pub struct AmountInput {
    conf: Config
}
impl AmountInput {
    pub fn new(conf: Config) -> Self {
        Self {conf}
    }
    fn get(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let prompt = decode_prompt(args)?;
        let decimals = decode_num_arg::<u128>(args, "decimals")?;
        let min = decode_num_arg::<u128>(args, "min")?;
        let max = decode_num_arg::<u128>(args, "max")?;
        let mut value = String::new();
        let _ = terminal_input(&prompt, |val| {
            value = convert::convert_token(val.as_str())?;
            Ok(())
        });
        Ok((answer_id, json!({ "value": value })))
    }
}

#[async_trait::async_trait]
impl DebotInterface for AmountInput {
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
