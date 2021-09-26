use base64;
use super::term_browser::input;
use crate::helpers::TonClient;
use serde_json::Value;
use std::io::{self, BufRead, Write};
use ton_client::crypto::{
    register_encryption_box, chacha20,
    EncryptionBoxHandle,
    EncryptionBoxInfo, ParamsOfChaCha20};
use ton_client::crypto::nacl::{
    nacl_box, nacl_box_open,
    nacl_secret_box, nacl_secret_box_open,
    ParamsOfNaclBox, ResultOfNaclBox,
    ParamsOfNaclBoxOpen, ResultOfNaclBoxOpen,
    ParamsOfNaclSecretBox, ParamsOfNaclSecretBoxOpen
};
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
    /// Receiver's public key - unprefixed 0-padded to 64 symbols hex string.
    pub their_pubkey: String,
    /// Sender's private key - unprefixed 0-padded to 64 symbols hex string.
    pub ssecret: String,
    /// Receiver's private key - unprefixed 0-padded to 64 symbols hex string.
    pub rsecret: String,
    /// Nonce, encoded in `hex`.
    pub nonce: String,
    /// Client context.
    pub client: TonClient,
}

#[async_trait::async_trait]
impl ton_client::crypto::EncryptionBox for ChaChaBox{
    async fn get_info(&self) -> ClientResult<EncryptionBoxInfo> {
        Ok(EncryptionBoxInfo {hdpath: None, algorithm: Some("ChaCha20".to_string()),
            options: Some(json!({/*"key": hex::encode(self.key.clone()), */"nonce": hex::encode(self.nonce.clone())})), public: None})
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
        let decrypted_data = chacha20(self.client.clone(), ParamsOfChaCha20{data: data, key: self.key.clone(), nonce: self.nonce.clone()}
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
    async fn get_info(&self) -> ClientResult<EncryptionBoxInfo> {
        Ok(EncryptionBoxInfo {hdpath: None, algorithm: Some("NaCl".to_string()),
            options: Some(json!({
                "their_pubkey": hex::encode(self.their_pubkey.clone()),
                "nonce": hex::encode(self.nonce.clone())})
            ), public: None})
    }
    async fn encrypt(&self, data: &String) -> ClientResult<String> {
        let encrypted_data = nacl_box(self.client.clone(), ParamsOfNaclBox{decrypted: base64::encode(data), nonce: self.nonce, their_public: self.their_pubkey, secret: self.ssecret}
        ).unwrap().encrypted;
        Ok(
            String::from_utf8(
                base64::decode(
                    &encrypted_data
                ).unwrap()
            ).unwrap()
        )
    }
    async fn decrypt(&self, data: &String) -> ClientResult<String> {
        let decrypted_data = nacl_box_open(self.client.clone(), ParamsOfNaclBoxOpen{encrypted: data, nonce: self.nonce, their_public: self.their_pubkey, secret: self.rsecret});
        Ok(
            String::from_utf8(
                base64::decode(
                   &decrypted_data
                ).unwrap()
            ).unwrap()
        )
    }
}

pub(super) struct TerminalEncryptionBox {
    handle: EncryptionBoxHandle,
    box_type: EncryptionBoxType,
    client: TonClient,
}

impl TerminalEncryptionBox {
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