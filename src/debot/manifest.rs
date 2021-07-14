use serde::{Deserialize, Serialize};
use serde_json::Value;  

fn default_init_method() -> String { "start".to_owned() }
fn default_mandatory() -> bool { false }

#[derive(Serialize, Deserialize, Clone)]
enum ApproveKind {
    ApproveOnchainCall,
    ApproveNetwork,
    ApproveMessageLimit,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct DebotManifest {
    debot_address: String,
    #[serde(default = "default_init_method")]
    init_method: String,
    init_args: Option<String>,
    auto_approve: Option<Vec<ApproveKind>>,
    chain: Vec<ChainLink>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) enum ChainLink {
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

pub(crate) struct ChainProcessor {
    manifest: DebotManifest,
}

impl ChainProcessor {
    pub fn new(manifest: DebotManifest) -> Self {
        Self { manifest }
    }

    pub fn next() {

    }
}