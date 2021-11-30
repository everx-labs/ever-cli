use crate::debot::term_browser::terminal_input;
use serde_json::Value;
use ton_client::abi::Abi;
use ton_client::debot::{DebotInterface, InterfaceResult};
use super::dinterface::{decode_answer_id, decode_prompt, decode_num_arg};
use ton_client::encoding::decode_abi_number;

const ID: &'static str = "4e862a9df81183ab425bdf0fbd76bd0b558c7f44c24887b4354bf1c26c74a623";

pub const ABI: &str = r#"
{
    "ABI version": 2,
    "version": "2.2",
    "header": ["time"],
    "functions": [
        {
            "name": "getDate",
            "inputs": [
                {"name":"answerId","type":"uint32"},
                {"name":"prompt","type":"string"},
                {"name":"defaultDate","type":"int128"},
                {"name":"minDate","type":"int128"},
                {"name":"maxDate","type":"int128"}
            ],
            "outputs": [
                {"name":"date","type":"int128"}
            ]
        },
        {
            "name": "getTime",
            "inputs": [
                {"name":"answerId","type":"uint32"},
                {"name":"prompt","type":"string"},
                {"name":"defaultTime","type":"uint32"},
                {"name":"minTime","type":"uint32"},
                {"name":"maxTime","type":"uint32"},
                {"name":"minuteInterval","type":"uint8"}
            ],
            "outputs": [
                {"name":"time","type":"uint32"}
            ]
        },
        {
            "name": "getDateTime",
            "inputs": [
                {"name":"answerId","type":"uint32"},
                {"name":"prompt","type":"string"},
                {"name":"defaultDatetime","type":"int128"},
                {"name":"minDatetime","type":"int128"},
                {"name":"maxDatetime","type":"int128"},
                {"name":"minuteInterval","type":"uint8"},
                {"name":"inTimeZoneOffset","type":"int16"}
            ],
            "outputs": [
                {"name":"datetime","type":"int128"},
                {"name":"timeZoneOffset","type":"int16"}
            ]
        },
        {
            "name": "getTimeZoneOffset",
            "inputs": [
                {"name":"answerId","type":"uint32"}
            ],
            "outputs": [
                {"name":"timeZoneOffset","type":"int16"}
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
    ],
    "fields": [
        {"name":"_pubkey","type":"uint256"},
        {"name":"_timestamp","type":"uint64"},
        {"name":"_constructorFlag","type":"bool"}
    ]
}
"#;

pub struct DateTimeInput {}

impl DateTimeInput {
    pub fn new() -> Self {
        Self {}
    }
    fn getDate(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let prompt = decode_prompt(args)?;
        println!("{}", prompt);
        let default_date = decode_num_arg::<i128>(args, "defaultDate")?;
        let min_date = decode_num_arg::<i128>(args, "minDate")?;
        let max_date = decode_num_arg::<i128>(args, "maxDate")?;
        let mut value = String::new();
        let prompt = format!(
            "{}\n(>= {} and <= {})",
            prompt,
            min_date,   //here and below to convert to real readable dates?
            max_date);
        let _ = terminal_input(&prompt, |val| {
            value = val.clone();
            let number = decode_abi_number::<i128>(&value)
                .map_err(|e| format!("input is not a valid date: {}", e))?;
            if number < min_date || number > max_date {
                return Err(format!("date is out of range"));
            }
            Ok(())
        });
        Ok((answer_id, json!({"time": value})))

    }
    fn getTime(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let prompt = decode_prompt(args)?;
        println!("{}", prompt);
        let defaultTime = decode_num_arg::<u32>(args, "defaultTime")?;
        let minTime = decode_num_arg::<u32>(args, "minTime")?;
        let maxTime = decode_num_arg::<u32>(args, "maxTime")?;
        let minuteInterval = decode_num_arg::<u8>(args, "minuteInterval")?;
        let time : u32 = 1;
        Ok((answer_id, json!({"time": time})))
    }
    fn getDateTime(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let prompt = decode_prompt(args)?;
        println!("{}", prompt);
        let defaultDatetime = decode_num_arg::<i128>(args, "defaultDatetime")?;
        let minDatetime = decode_num_arg::<i128>(args, "minDatetime")?;
        let maxDatetime = decode_num_arg::<i128>(args, "maxDatetime")?;
        let minuteInterval = decode_num_arg::<u8>(args, "minuteInterval")?;
        let inTimeZoneOffset = decode_num_arg::<i16>(args, "inTimeZoneOffset")?;
        let datetime : i128 = 1;
        let timeZoneOffset : i16 = 1;
        Ok((answer_id, json!({"datetime": datetime, "timeZoneOffset": timeZoneOffset})))
    }
    fn getTimeZoneOffset(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let timeZoneOffset : i16 = 1;
        Ok((answer_id, json!({"timeZoneOffset": timeZoneOffset})))
    }
}

#[async_trait::async_trait]
impl DebotInterface for DateTimeInput {
    fn get_id(&self) -> String {
        ID.to_string()
    }

    fn get_abi(&self) -> Abi {
        Abi::Json(ABI.to_owned())
    }

    async fn call(&self, func: &str, args: &Value) -> InterfaceResult {
        match func {
            "getDate" => self.getDate(args),
            "getTime" => self.getTime(args),
            "getDateTime" => self.getDateTime(args),
            "getTimeZoneOffset" => self.getTimeZoneOffset(args),
            _ => Err(format!("function \"{}\" is not implemented", func)),
        }
    }
}
