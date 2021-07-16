use super::dinterface::{decode_string_arg};
use crate::debot::term_browser::{action_input};
use serde_json::Value;
use serde::{de, Deserialize, Deserializer};
use ton_client::abi::Abi;
use ton_client::debot::{DebotInterface, InterfaceResult};
use std::fmt::Display;
use std::str::FromStr;
use ton_client::encoding::decode_abi_number;

const ID: &'static str = "ac1a4d3ecea232e49783df4a23a81823cdca3205dc58cd20c4db259c25605b48";

const ABI: &str = r#"
{
	"ABI version": 2,
	"header": ["time"],
	"functions": [
		{
			"name": "select",
			"inputs": [
				{"name":"title","type":"bytes"},
				{"name":"description","type":"bytes"},
				{"components":[{"name":"title","type":"bytes"},{"name":"description","type":"bytes"},{"name":"handlerId","type":"uint32"}],"name":"items","type":"tuple[]"}
			],
			"outputs": [
				{"name":"index","type":"uint32"}
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

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MenuItem {
    #[serde(deserialize_with = "from_hex_to_utf8_str")]
    title: String,
    #[serde(deserialize_with = "from_hex_to_utf8_str")]
    description: String,
    #[serde(deserialize_with = "from_abi_num")]
    pub handler_id: u32,
}

fn str_hex_to_utf8(s: &str) -> Option<String> {
    String::from_utf8(hex::decode(s).ok()?).ok()
}

fn from_hex_to_utf8_str<'de, S, D>(des: D) -> Result<S, D::Error>
where
    S: FromStr,
    S::Err: Display,
    D: Deserializer<'de>
{
    let s: String = Deserialize::deserialize(des)?;
    let s = str_hex_to_utf8(&s)
        .ok_or(format!("failed to convert bytes to utf8 string")).unwrap();
    S::from_str(&s).map_err(de::Error::custom)
}

fn from_abi_num<'de, D>(des: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>
{
    let s: String = Deserialize::deserialize(des)?;
    decode_abi_number(&s).map_err(de::Error::custom)
}

pub struct Menu {}
impl Menu {

	pub fn new() -> Self {
		Self{}
	}

    fn select(&self, args: &Value) -> InterfaceResult {
		let menu_items: Vec<MenuItem> = serde_json::from_value(args["items"].clone()).unwrap();
        let title = decode_string_arg(args, "title")?;
        let description = decode_string_arg(args, "description")?;
        if title.len() > 0 {
            println!("{}", title);
        }
        if description.len() > 0 {
            println!("{}", description);
        }
        for (i, menu) in menu_items.iter().enumerate() {
            println!("{}) {}", i + 1, menu.title);
            if menu.description != "" {
                println!("   {}", menu.description);
            }
        }
        loop {
            let res = action_input(menu_items.len());
            if res.is_err() {
                println!("{}", res.unwrap_err());
                continue;
            }
            let (n, _, _) = res.unwrap();
            let menu = menu_items.get(n - 1);
            if menu.is_none() {
                println!("Invalid menu. Try again.");
                continue;
            }

            return Ok(( menu.unwrap().handler_id, json!({ "index": n - 1 }) ));
        }

    }
}

#[async_trait::async_trait]
impl DebotInterface for Menu {
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
