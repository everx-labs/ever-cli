use base64;
use super::term_browser::input;
use crate::helpers::TonClient;
use serde_json::Value;
use std::io::{self, BufRead, Write};
use ton_client::crypto::{
    register_encryption_box,
    RegisteredEncryptionBox, EncryptionBoxHandle, EncryptionAlgorithm,
};
use ton_client::crypto::{EncryptionBoxInfo, register_encryption_box,
    chacha20, ParamsOfChaCha20, ResultOfChaCha20};
use ton_client::error::ClientResult;

pub(super) enum EncryptionBoxType {
    NaCl,
    ChaCha20,
}

pub struct ChaChaBox {
    /// 256-bit key.
    pub key: String,
    /// 96-bit nonce.
    pub nonce: String,
    /// Client context.
    pub client: TonClient,
}

pub struct NaClBox {
    /// Receiver's public key.
    pub their_pubkey: String,
    /// Sender's private key.
    pub ssecret: String,
    /// Receiver's private key.
    pub rsecret: String,
    /// Nonce.
    pub nonce: String,
    /// Client context.
    pub client: TonClient,
}

#[async_trait::async_trait]
impl ton_client::crypto::EncryptionBox for ChaChaBox{
    async fn get_info(&self) -> ClientResult<EncryptionBoxInfo> {
        Ok(EncryptionBoxInfo {hdpath: None, algorithm: Some("ChaCha20".to_string()),
            options: Some(json!({"key": hex::encode(self.key.clone()), "nonce": hex::encode(self.nonce.clone())})), public: None})
    }
    async fn encrypt(&self, data: &String) -> ClientResult<String> {
        let encrypted_data = chacha20(
            self.client.clone(), ParamsOfChaCha20{data: base64::encode(data), key: self.key.clone(), nonce: self.nonce.clone()}
        ).unwrap().data;
        Ok(
            String::from_utf8(
                base64::decode(
                    &encrypted_data
                ).unwrap()
            ).unwrap()
        )
    }
    async fn decrypt(&self, data: &String) -> ClientResult<String> {
        let decrypted_data = chacha20(self.client.clone(), ParamsOfChaCha20{data: base64::encode(data), key: self.key.clone(), nonce: self.nonce.clone()}
        ).unwrap().data;
        Ok(
            String::from_utf8(
                base64::decode(
                   &decrypted_data
                ).unwrap()
            ).unwrap()
        )
    }
}

#[async_trait::async_trait]
impl ton_client::crypto::EncryptionBox for NaClBox{
    /// Gets encryption box information
    async fn get_info(&self) -> ClientResult<EncryptionBoxInfo> {
        EncryptionBoxInfo {hdpath: None, algorithm: Some("NaCl".to_string()),
            options: Some(json!({"their_pubkey": hex::encode(self.their_pubkey.clone()), "secret": hex::encode(self.secret.clone()), "nonce": hex::encode(self.nonce.clone())})), public: None})
    }
    /// Encrypts data
    async fn encrypt(&self, data: &String) -> ClientResult<String> {}
    /// Decrypts data
    async fn decrypt(&self, data: &String) -> ClientResult<String> {}
}

pub(super) struct TerminalEncryptionBox {
    handle: EncryptionBoxHandle,
    box_type: EncryptionBoxType,
    client: TonClient,
}

impl TerminalEncryptionBox {
    /*pub async fn new(client: TonClient, box_type: EncryptionBoxType, box_args: Ok()keys = {
            let mut reader = stdio.lock();
            let mut writer = io::stdout();
            input_keys(None, vec![], &mut reader, &mut writer, 3)?
        };
        Ok(Self { handle, box_type, client })
    }*/
    pub async fn new(client: TonClient, box_type: EncryptionBoxType, box_args: Value) -> Self {
        match box_type {
            Nacl => {},
            ChaCha20 => {
                let registered_box = register_encryption_box(
                    client,
                    ChaChaBox{key: String::from(""), nonce: String::from(""), client}
                ).await.unwrap().handle;
                Self{handle: registered_box, box_type: box_type, client: client}
            },
        }
    }
    pub fn handle(&self) -> EncryptionBoxHandle {
        self.handle.clone()
    }
}