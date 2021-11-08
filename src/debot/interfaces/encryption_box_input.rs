use super::dinterface::{decode_answer_id, decode_nonce, decode_prompt, decode_arg, decode_num_arg};
use crate::debot::term_encryption_box::{
    EncryptionBoxType, ParamsOfTerminalEncryptionBox, TerminalEncryptionBox,
};
use ton_client::crypto::EncryptionBoxHandle;
use crate::helpers::TonClient;
use serde_json::Value;
use tokio::sync::RwLock;
use ton_client::abi::Abi;
use ton_client::debot::{DebotInterface, InterfaceResult};

const ID: &'static str = "5b5f76b54d976d72f1ada3063d1af2e5352edaf1ba86b3b311170d4d81056d61";

const ABI: &str = r#"
{
	"ABI version": 2,
	"header": ["time"],
	"functions": [
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
        Self {
            handles: RwLock::new(vec![]),
            client: client,
        }
    }

    async fn get_nacl_box(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let prompt = decode_prompt(args)?;
        let nonce = decode_nonce(args)?;
        let their_pubkey = decode_arg(args, "theirPubkey")?;
        println!("{}", prompt);
        let result = TerminalEncryptionBox::new(ParamsOfTerminalEncryptionBox {
            context: self.client.clone(),
            box_type: EncryptionBoxType::NaCl,
            their_pubkey: their_pubkey,
            nonce: nonce,
        }).await;
        let handle = result.map(|r| r.handle()).unwrap_or(EncryptionBoxHandle(0));
        self.handles.write().await.push(TerminalEncryptionBox{handle, client: self.client.clone()});
        Ok((answer_id, json!({ "handle": self.handles.read().await.last().unwrap().handle().0})))
    }
    async fn get_nacl_secret_box(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let prompt = decode_prompt(args)?;
        let nonce = decode_nonce(args)?;
        println!("{}", prompt);
        let result = TerminalEncryptionBox::new(ParamsOfTerminalEncryptionBox {
            context: self.client.clone(),
            box_type: EncryptionBoxType::SecretNaCl,
            their_pubkey: String::new(),
            nonce: nonce,
        })
        .await;
        let handle = result.map(|r| r.handle()).unwrap_or(EncryptionBoxHandle(0));
        self.handles.write().await.push(TerminalEncryptionBox{handle, client: self.client.clone()});
        Ok((answer_id, json!({ "handle": self.handles.read().await.last().unwrap().handle().0})))
    }
    async fn get_chacha20_box(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let nonce = decode_nonce(args)?;
        let prompt = decode_prompt(args)?;
        println!("{}", prompt);
        let result = TerminalEncryptionBox::new(ParamsOfTerminalEncryptionBox {
            context: self.client.clone(),
            box_type: EncryptionBoxType::ChaCha20,
            their_pubkey: String::new(),
            nonce,
        })
        .await;
        let handle = result.map(|r| r.handle()).unwrap_or(EncryptionBoxHandle(0));
        self.handles.write().await.push(TerminalEncryptionBox{handle, client: self.client.clone()});
        Ok((answer_id, json!({ "handle": self.handles.read().await.last().unwrap().handle().0})))
    }
    async fn remove_handle(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let handle = decode_num_arg::<u32>(args, "handle")?;
        let mut handles = self.handles.write().await;
        let initial_size = handles.len();
        handles.retain(|value| (*value).handle().0 != handle);
        let removed: bool = initial_size != handles.len();
        Ok((answer_id, json!({ "removed": removed })))
    }
    async fn get_supported_algorithms(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        Ok((
            answer_id,
            json!({
                "names":
                    vec![
                        hex::encode("NaclBox"),
                        hex::encode("NaclSecretBox"),
                        hex::encode("ChaCha20"),
                    ]
            }),
        ))
    }
}

#[async_trait::async_trait]
impl DebotInterface for EncryptionBoxInput {
    fn get_id(&self) -> String {
        ID.to_string()
    }

    fn get_abi(&self) -> Abi {
        Abi::Json(ABI.to_owned())
    }

    async fn call(&self, func: &str, args: &Value) -> InterfaceResult {
        match func {
            "getNaclBox" => self.get_nacl_box(args).await,
            "getNaclSecretBox" => self.get_nacl_secret_box(args).await,
            "getChaCha20Box" => self.get_chacha20_box(args).await,
            "remove" => self.remove_handle(args).await,
            "getSupportedAlgorithms" => self.get_supported_algorithms(args).await,
            _ => Err(format!("function \"{}\" is not implemented", func)),
        }
    }
}

