use super::address_input::AddressInput;
use super::echo::Echo;
use super::stdout::Stdout;
use super::terminal::Terminal;
use super::menu::Menu;
use crate::helpers::TonClient;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use ton_client::debot::{DebotInterface, DebotInterfaceExecutor};

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
    pub fn new(client: TonClient) -> Self {
        let mut interfaces = HashMap::new();

        let iface: Arc<dyn DebotInterface + Send + Sync> = Arc::new(AddressInput::new());
        interfaces.insert(iface.get_id(), iface);

        let iface: Arc<dyn DebotInterface + Send + Sync> = Arc::new(Stdout::new());
        interfaces.insert(iface.get_id(), iface);

        let iface: Arc<dyn DebotInterface + Send + Sync> = Arc::new(Echo::new());
        interfaces.insert(iface.get_id(), iface);

        let iface: Arc<dyn DebotInterface + Send + Sync> = Arc::new(Terminal::new());
        interfaces.insert(iface.get_id(), iface);

        let iface: Arc<dyn DebotInterface + Send + Sync> = Arc::new(Menu::new());
        interfaces.insert(iface.get_id(), iface);

        Self { client, interfaces }
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
