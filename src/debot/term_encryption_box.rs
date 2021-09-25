use base64;
use super::term_browser::input;
use crate::helpers::TonClient;
use serde_json::Value;
use std::io::{self, BufRead, Write};
use ton_client::crypto::{
    encryption_box_encrypt, encryption_box_decrypt, remove_signing_box, create_encryption_box,
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
    pub key: String,
    pub nonce: String,
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
    pub fn handle(&self) -> EncryptionBoxHandle {
        self.handle.clone()
    }
}