use super::dinterface::{decode_answer_id, decode_prompt, decode_nonce};
use serde_json::Value;
use ton_client::abi::Abi;
use ton_client::debot::{DebotInterface, InterfaceResult};
use ton_client::encoding::decode_abi_bigint;
use crate::helpers::TonClient;
use ton_client::crypto::boxes::encryption_box::{EncryptionBox};
use crate::debot::term_encryption_box::{TerminalEncryptionBox, EncryptionBoxType};
use tokio::sync::RwLock;

const ID: &'static str = "c13024e101c95e71afb1f5fa6d72f633d51e721de0320d73dfd6121a54e4d40b";

const ABI: &str = r#"
{
	"ABI version": 2,
	"header": ["time"],
	"functions": "functions": [
        {
            "name": "getNaclBox",
            "inputs": [
                {"name":"answerId","type":"uint32"},
                {"name":"prompt","type":"bytes"},
                {"name":"nonce","type":"bytes"},
                {"name":"theirPubkey","type":"uint256"}
            ],
            "outputs": [
                {"name":"handle","type":"uint32"}
            ]
        },
        {
            "name": "getNaclSecretBox",
            "inputs": [
                {"name":"answerId","type":"uint32"},
                {"name":"prompt","type":"bytes"},
                {"name":"nonce","type":"bytes"}
            ],
            "outputs": [
                {"name":"handle","type":"uint32"}
            ]
        },
        {
            "name": "getChaCha20Box",
            "inputs": [
                {"name":"answerId","type":"uint32"},
                {"name":"prompt","type":"bytes"},
                {"name":"nonce","type":"bytes"}
            ],
            "outputs": [
                {"name":"handle","type":"uint32"}
            ]
        },
        {
            "name": "remove",
            "inputs": [
                {"name":"answerId","type":"uint32"},
                {"name":"handle","type":"uint32"}
            ],
            "outputs": [
                {"name":"removed","type":"bool"}
            ]
        },
        {
            "name": "getSupportedAlgorithms",
            "inputs": [
                {"name":"answerId","type":"uint32"}
            ],
            "outputs": [
                {"name":"names","type":"bytes[]"}
            ]
        }
    ],
	"data": [
	],
	"events": [
	]
}
"#;

pub struct EncryptionBoxInput {
    handles: RwLock<Vec<TerminalEncryptionBox>>,
    client: TonClient,
}

impl EncryptionBoxInput {
    pub fn new(client: TonClient) -> Self {
        Self {handles: RwLock::new(vec![]), cliemt}
    }

    async fn getNaclBox($self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let prompt = decode_prompt(args)?;
        Ok((answer_id, json!({})));
    }
    async fn getNaclSecretBox($self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let prompt = decode_prompt(args)?;
        Ok((answer_id, json!({})));
    }
    async fn getChaCha20Box($self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let prompt = decode_prompt(args)?;
        Ok((answer_id, json!({})));
    }
    async fn remove($self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        Ok((answer_id, json!({})));
    }
    async fn getSupportedAlgorithms($self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        Ok((answer_id, json!({})));
    }
}

#[async_trait::async_trait]
impl DebotInterface for SigningBoxInput {
    fn get_id(&self) -> String {
        ID.to_string()
    }

    fn get_abi(&self) -> Abi {
        Abi::Json(ABI.to_owned())
    }

    async fn call(&self, func: &str, args: &Value) -> InterfaceResult {
        match func {
            "getNaclBox" => self.getNaclBox(args).await,
            "getNaclSecretBox" => self.getNaclSecretBox(args).await,
            "getChaCha20Box" => self.getChaCha20Box(args).await,
            "remove" => self.removeHandle(args).await,
            "getSupportedAlgorithms" => self.getSupportedAlgorithms(args).await,
            _ => Err(format!("function \"{}\" is not implemented", func)),
        }
    }
}

