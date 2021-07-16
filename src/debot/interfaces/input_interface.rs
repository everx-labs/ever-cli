use ton_client::abi::Abi;
use crate::debot::{ManifestProcessor, ProcessorError};
use ton_client::debot::{DebotInterface, InterfaceResult};
use std::sync::{Arc};
use tokio::sync::RwLock;
use serde_json::Value;
use super::dinterface::{decode_answer_id, decode_prompt, decode_string_arg};
use super::menu::MenuItem;

pub struct InputInterface {
    processor: Arc<RwLock<ManifestProcessor>>,
    inner_interface: Arc<dyn DebotInterface + Send + Sync>,
}

impl InputInterface {
    pub fn new(
        inner_interface: Arc<dyn DebotInterface + Send + Sync>,
        processor: Arc<RwLock<ManifestProcessor>>,
    ) -> Self {
        Self {
            inner_interface,
            processor,
        }
    }
}

#[async_trait::async_trait]
impl DebotInterface for InputInterface {
    fn get_id(&self) -> String {
        self.inner_interface.get_id()
    }

    fn get_abi(&self) -> Abi {
        self.inner_interface.get_abi()
    }

    async fn call(&self, func: &str, args: &Value) -> InterfaceResult {
        let result = self
            .processor
            .write().await
            .next_input(&self.get_id(), func, args);
        match result {
            Err(ProcessorError::InterfaceCallNeeded) => self.inner_interface.call(func, args).await,
            Err(e) => Err(format!("{:?}", e))?,
            Ok(params) => {
                let prompt = decode_prompt(args);
                let title = decode_string_arg(args, "title");
                let processor = self.processor.read().await;
                if let Ok(prompt) = prompt {
                    processor.print(&prompt);
                }
                if let Ok(prompt) = title {
                    processor.print(&prompt);
                }
                let params = params.unwrap_or(json!({}));
                for arg in params.as_object().unwrap() {
                    processor.print(&format!("{}", arg.1));
                }
                let answer_id = if self.get_id() == "ac1a4d3ecea232e49783df4a23a81823cdca3205dc58cd20c4db259c25605b48" {
                    let n = params["index"].as_u64().unwrap();
                    let menu_items: Vec<MenuItem> = serde_json::from_value(args["items"].clone()).unwrap();
                    let menu = menu_items.get(n as usize);
                    menu.unwrap().handler_id
                } else {
                    decode_answer_id(args)?
                };
                
                Ok((answer_id, params))
            }
        }
    }
}