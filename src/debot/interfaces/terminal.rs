use super::dinterface::{decode_answer_id, decode_bool_arg, decode_prompt, decode_string_arg};
use crate::debot::term_browser::terminal_input;
use serde_json::Value;
use ton_client::abi::Abi;
use ton_client::debot::{DebotInterface, InterfaceResult};
use crate::convert::convert_token;
use ton_types::cells_serialization::deserialize_tree_of_cells;
use ton_client::encoding::decode_abi_bigint;
use std::io::{Read};

const ID: &'static str = "8796536366ee21852db56dccb60bc564598b618c865fc50c8b1ab740bba128e3";

const ABI: &str = r#"
{
	"ABI version": 2,
	"header": ["time"],
	"functions": [
		{
			"name": "inputStr",
			"inputs": [
				{"name":"answerId","type":"uint32"},
                {"name":"prompt","type":"bytes"},
                {"name":"multiline","type":"bool"}
			],
			"outputs": [
				{"name":"value","type":"bytes"}
			]
		},
		{
			"name": "inputInt",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"prompt","type":"bytes"}
			],
			"outputs": [
				{"name":"value","type":"int256"}
			]
		},
		{
			"name": "inputUint",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"prompt","type":"bytes"}
			],
			"outputs": [
				{"name":"value","type":"uint256"}
			]
		},
		{
			"name": "inputTons",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"prompt","type":"bytes"}
			],
			"outputs": [
				{"name":"value","type":"uint128"}
			]
		},
		{
			"name": "inputBoolean",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"prompt","type":"bytes"}
			],
			"outputs": [
				{"name":"value","type":"bool"}
			]
		},
		{
			"name": "print",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"message","type":"bytes"}
			],
			"outputs": [
			]
		},
		{
			"name": "printf",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"fmt","type":"bytes"},
				{"name":"fargs","type":"cell"}
			],
			"outputs": [
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

pub struct Terminal {}
impl Terminal {
    pub fn new() -> Self {
        Self {}
    }
    fn input_str(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let prompt = decode_prompt(args)?;
        let multiline = decode_bool_arg(args, "multiline")?;
        let mut value = String::new();
        if multiline {
            println!("{}", &prompt);
            println!("(Ctrl+D to exit)");
            std::io::stdin().read_to_string(&mut value)
                .map_err(|e| format!("input error: {}", e))?;
        } else {
            value = terminal_input(&prompt, |_val| Ok(()));
        }
        Ok((answer_id, json!({ "value": hex::encode(value.as_bytes()) })))
    }

    fn input_int(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let value = terminal_input(&decode_prompt(args)?, |val| {
            let _ = decode_abi_bigint(val).map_err(|e| format!("{}", e))?;
            Ok(())
        });
        Ok((answer_id, json!({ "value": value })))
    }

    fn input_uint(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let value = terminal_input(&decode_prompt(args)?, |val| {
            let _ = decode_abi_bigint(val).map_err(|e| format!("{}", e))?;
            /* number.is negative(){
                Err(format!("number must be positive"))?;
            }*/
            Ok(())
        });
        Ok((answer_id, json!({ "value": value })))
    }

    fn input_tokens(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let mut nanotokens = String::new();
        let _ = terminal_input(&decode_prompt(args)?, |val| {
            nanotokens = convert_token(val)?;
            Ok(())
        });
        Ok((answer_id, json!({ "value": nanotokens })))
    }

    fn input_boolean(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        println!("{}", decode_prompt(args)?);
        let mut yes_no = false;
        let _ = terminal_input("(y/n)", |val| {
            yes_no = match val.as_str() {
                "y" => true,
                "n" => false,
                _ => Err(format!("invalid enter"))?,
            };
            Ok(())
        });
        Ok((answer_id, json!({ "value": yes_no })))
    }

    pub fn print(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let message = decode_string_arg(args, "message")?;
		println!("{}", message);
		Ok((answer_id, json!({})))
    }

    pub fn printf(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let fmt = decode_string_arg(args, "fmt")?;
        let fargs = args["fargs"].as_str().ok_or(format!(r#"argument "fargs" not found"#))?;
        let boc_bytes = base64::decode(&fargs)
            .map_err(|e| format!("failed to decode cell from base64: {}", e))?;
        let _args_cell = deserialize_tree_of_cells(&mut boc_bytes.as_slice())
            .map_err(|e| format!("failed to deserialize cell: {}", e))?;
        
        let message = printf(&fmt, |_arg| {
            "1".to_string()
        });

		println!("{}", message);
		Ok((answer_id, json!({})))
    }
}

#[async_trait::async_trait]
impl DebotInterface for Terminal {
    fn get_id(&self) -> String {
        ID.to_string()
    }

    fn get_abi(&self) -> Abi {
        Abi::Json(ABI.to_owned())
    }

    async fn call(&self, func: &str, args: &Value) -> InterfaceResult {
        match func {
            "inputStr" => self.input_str(args),
            "inputInt" => self.input_int(args),
            "inputUint" => self.input_uint(args),
            "inputTons" => self.input_tokens(args),
            "inputBoolean" => self.input_boolean(args),
            "print" => self.print(args),
            "printf" => self.printf(args),
            _ => Err(format!("function \"{}\" is not implemented", func)),
        }
    }
}

fn printf<F>(fmt: &str, formatter: F) -> String 
where
    F: Fn(&str) -> String,
{
    let mut message = String::new();
    let mut cursor = fmt;
    while let Some(i) = cursor.find("{") {
        let left = cursor.get(..i).unwrap();
        let right = cursor.get(i+1..).unwrap();
        message += left;
        //println!("right: {}", right);
        if left.ends_with("\\") {
            message += "{";
            cursor = right;
            continue;
        }
        if let Some(i) = right.find("}") {
            let arg = right.get(..i).unwrap();
            let right = right.get(i+1..).unwrap();
            message += &formatter(arg);
            cursor = right;
        }
    }
    message += cursor;
    message
} 

#[cfg(test)]
mod tests {
    use super::printf;
    #[test]
    fn test_printf() {
        let result = printf("Hello, \\{string {}! My \\age\\ is {uint32} and {}", |arg| {
            println!("arg: {}", arg);
            "TYPE".to_owned()
        });
        println!("{}", result);
    }
}