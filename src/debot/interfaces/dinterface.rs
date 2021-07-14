use super::{Menu, AddressInput, AmountInput, ConfirmInput, NumberInput, SigningBoxInput, Terminal, UserInfo};
use super::echo::Echo;
use super::stdout::Stdout;
use crate::config::Config;
use crate::helpers::TonClient;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use ton_client::debot::{DebotInterface, DebotInterfaceExecutor, InterfaceResult};
use ton_client::encoding::{decode_abi_number, decode_abi_bigint};
use ton_client::abi::Abi;
use num_traits::cast::NumCast;
use num_bigint::BigInt;

pub struct SupportedInterfaces {
    client: TonClient,
    interfaces: HashMap<String, Arc<dyn DebotInterface + Send + Sync>>,
}

#[async_trait::async_trait]
impl DebotInterfaceExecutor for SupportedInterfaces {
    fn get_interfaces<'a>(&'a self) -> &'a HashMap<String, Arc<dyn DebotInterface + Send + Sync>> {
        &self.interfaces
    }
    fn get_client(&self) -> TonClient {
        self.client.clone()
    }
}

impl SupportedInterfaces {
    pub fn new(client: TonClient, conf: &Config) -> Self {
        let mut interfaces = HashMap::new();

        let iface: Arc<dyn DebotInterface + Send + Sync> = Arc::new(AddressInput::new(conf.clone()));
        interfaces.insert(iface.get_id(), iface);

        let iface: Arc<dyn DebotInterface + Send + Sync> = Arc::new(AmountInput::new());
        interfaces.insert(iface.get_id(), iface);

        let iface: Arc<dyn DebotInterface + Send + Sync> = Arc::new(NumberInput::new());
        interfaces.insert(iface.get_id(), iface);

        let iface: Arc<dyn DebotInterface + Send + Sync> = Arc::new(ConfirmInput::new());
        interfaces.insert(iface.get_id(), iface);

        let iface: Arc<dyn DebotInterface + Send + Sync> = Arc::new(Stdout::new());
        interfaces.insert(iface.get_id(), iface);

        let iface: Arc<dyn DebotInterface + Send + Sync> = Arc::new(Echo::new());
        interfaces.insert(iface.get_id(), iface);

        let iface: Arc<dyn DebotInterface + Send + Sync> = Arc::new(Terminal::new());
        interfaces.insert(iface.get_id(), iface);

        let iface: Arc<dyn DebotInterface + Send + Sync> = Arc::new(Menu::new());
        interfaces.insert(iface.get_id(), iface);

        let iface: Arc<dyn DebotInterface + Send + Sync> = Arc::new(SigningBoxInput::new(client.clone()));
        interfaces.insert(iface.get_id(), iface);

        let iface: Arc<dyn DebotInterface + Send + Sync> = Arc::new(UserInfo::new(conf.clone()));
        interfaces.insert(iface.get_id(), iface);

        Self { client, interfaces }
    }
}

#[async_trait::async_trait]
pub trait BrowserInterface {
    fn get_id(&self) -> String;
    fn get_abi(&self) -> Abi;
    async fn call(&self, func: &str, args: &Value) -> InterfaceResult;
}

#[async_trait::async_trait]
impl<T> DebotInterface for T where T: BrowserInterface {
    fn get_id(&self) -> String {
        self.get_id()
    }

    fn get_abi(&self) -> Abi {
        self.get_abi()
    }

    async fn call(&self, func: &str, args: &Value) -> InterfaceResult {
        self.call(func, args).await
    }
}

pub fn decode_answer_id(args: &Value) -> Result<u32, String> {
    u32::from_str_radix(
        args["answerId"]
            .as_str()
            .ok_or(format!("answer id not found in argument list"))?,
        10,
    )
    .map_err(|e| format!("{}", e))
}

pub fn decode_arg(args: &Value, name: &str) -> Result<String, String> {
    args[name]
        .as_str()
        .ok_or(format!("\"{}\" not found", name))
        .map(|x| x.to_string())
}

pub fn decode_bool_arg(args: &Value, name: &str) -> Result<bool, String> {
    args[name]
        .as_bool()
        .ok_or(format!("\"{}\" not found", name))
}

pub fn decode_string_arg(args: &Value, name: &str) -> Result<String, String> {
    let bytes = hex::decode(&decode_arg(args, name)?)
        .map_err(|e| format!("{}", e))?;
    std::str::from_utf8(&bytes)
        .map_err(|e| format!("{}", e))
        .map(|x| x.to_string())
}

pub fn decode_prompt(args: &Value) -> Result<String, String> {
    decode_string_arg(args, "prompt")
}

pub fn decode_num_arg<T>(args: &Value, name: &str) -> Result<T, String>
where
    T: NumCast,
{
    let num_str = decode_arg(args, name)?;
    decode_abi_number::<T>(&num_str)
        .map_err(|e| format!("failed to parse integer \"{}\": {}", num_str, e))
}

pub fn decode_int256(args: &Value, name: &str) -> Result<BigInt, String> {
    let num_str = decode_arg(args, name)?;
    decode_abi_bigint(&num_str)
        .map_err(|e| format!("failed to decode integer \"{}\": {}", num_str, e))
}

pub fn decode_array<F, T>(args: &Value, name: &str, validator: F) -> Result<Vec<T>, String> 
    where F: Fn(&Value) -> Option<T>
{
    let array = args[name]
        .as_array()
        .ok_or(format!("\"{}\" is invalid: must be array", name))?;
    let mut strings = vec![];
    for elem in array {
        strings.push(
            validator(&elem).ok_or(format!("invalid array element type"))?
        );
    }
    Ok(strings)
}