use ton_client::abi::Abi;
use crate::debot::{ManifestProcessor, ProcessorError};
use ton_client::debot::{DebotInterface, InterfaceResult};
use std::sync::{Arc};
use tokio::sync::RwLock;
use serde_json::Value;
use super::dinterface::{decode_answer_id, decode_prompt};

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
                let answer_id = decode_answer_id(args)?;
                let prompt = decode_prompt(args);
                if let Ok(prompt) = prompt {
                    self.processor.read().await.print(&prompt);
                }
                Ok((answer_id, params.unwrap_or(json!({}))))
            }
        }
    }
}