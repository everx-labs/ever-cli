use ton_client::debot::{DebotInterfaceExecutor, DebotInterface};
use std::collections::HashMap;
use crate::helpers::{TonClient};
use std::sync::Arc;
use super::echo::Echo;
use super::stdout::Stdout;

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

        /*let iface: Arc<dyn DebotInterface + Send + Sync> =
            Arc::new(AddressInputInterface::new());
        interfaces.insert(iface.get_id(), iface);*/

        let iface: Arc<dyn DebotInterface + Send + Sync> =
            Arc::new(Stdout::new());
        interfaces.insert(iface.get_id(), iface);

        let iface: Arc<dyn DebotInterface + Send + Sync> =
            Arc::new(Echo::new());
        interfaces.insert(iface.get_id(), iface);

        Self { client, interfaces }
    }
}