use serde::{Deserialize, Serialize};
use serde_json::Value;

fn default_init_method() -> String { "start".to_owned() }
fn default_mandatory() -> bool { false }
fn default_interactive() -> bool { true }

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum ApproveKind {
    ApproveOnchainCall,
    ApproveNetwork,
    ApproveMessageLimit,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct DebotManifest {
    pub debot_address: String,
    #[serde(default = "default_init_method")]
    pub init_method: String,
    pub init_args: Option<Value>,
    pub init_msg: Option<String>,
    pub auto_approve: Option<Vec<ApproveKind>>,
    #[serde(default = "default_interactive")]
    pub interactive: bool,
    pub chain: Vec<ChainLink>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ChainLink {
    Input {
        interface: String,
        method: String,
        params: Option<Value>,
        #[serde(default = "default_mandatory")]
        mandatory: bool,
    },
    OnchainCall {
        approve: bool,
        iflq: Option<String>,
        ifeq: Option<String>,
    },
    Signature {
        handle: u32
    },
}