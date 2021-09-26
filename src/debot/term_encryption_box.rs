use super::term_browser::input;
use crate::helpers::TonClient;
use base64;
use serde_json::Value;
use std::io::{self, BufRead, Write};
use ton_client::crypto::{
    chacha20, nacl_box, nacl_box_open, nacl_secret_box, nacl_secret_box_open,
    register_encryption_box, EncryptionBoxHandle, EncryptionBoxInfo, ParamsOfChaCha20,
    ParamsOfNaclBox, ParamsOfNaclBoxOpen, ParamsOfNaclSecretBox, ParamsOfNaclSecretBoxOpen,
    ResultOfNaclBox, ResultOfNaclBoxOpen,
};
use ton_client::error::ClientResult;

#[derive(Clone, Copy)]
pub(super) enum EncryptionBoxType {
    SecretNaCl,
    NaCl,
    ChaCha20,
}

pub struct NaClSecretBox {
    /// 256-bit key - unprefixed 0-padded to 64 symbols hex string.
    pub key: String,
    /// 96-bit nonce, encoded in `hex`.
    pub nonce: String,
    /// Client context.
    pub client: TonClient,
}

pub struct ChaChaBox {
    /// 256-bit key, encoded with `base64`.
    pub key: String,
    /// 96-bit nonce, encoded in `hex`.
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
impl ton_client::crypto::EncryptionBox for NaClSecretBox {
    async fn get_info(&self) -> ClientResult<EncryptionBoxInfo> {
        Ok(EncryptionBoxInfo {
            hdpath: None,
            algorithm: Some("SecretNaCl".to_string()),
            options: Some(
                json!({/*"key": hex::encode(self.key.clone()), */"nonce": self.nonce.clone()}),
            ),
            public: None,
        })
    }
    async fn encrypt(&self, data: &String) -> ClientResult<String> {
        Ok(nacl_secret_box(
            self.client.clone(),
            ParamsOfNaclSecretBox {
                decrypted: base64::encode(data),
                key: self.key.clone(),
                nonce: self.nonce.clone(),
            },
        )
        .unwrap()
        .encrypted)
    }
    async fn decrypt(&self, data: &String) -> ClientResult<String> {
        Ok(nacl_secret_box_open(
            self.client.clone(),
            ParamsOfNaclSecretBoxOpen {
                encrypted: data.clone(),
                key: self.key.clone(),
                nonce: self.nonce.clone(),
            },
        )
        .unwrap()
        .decrypted)
    }
}

#[async_trait::async_trait]
impl ton_client::crypto::EncryptionBox for ChaChaBox {
    async fn get_info(&self) -> ClientResult<EncryptionBoxInfo> {
        Ok(EncryptionBoxInfo {
            hdpath: None,
            algorithm: Some("ChaCha20".to_string()),
            options: Some(
                json!({/*"key": hex::encode(self.key.clone()), */"nonce": self.nonce.clone()}),
            ),
            public: None,
        })
    }
    async fn encrypt(&self, data: &String) -> ClientResult<String> {
        Ok(chacha20(
            self.client.clone(),
            ParamsOfChaCha20 {
                data: base64::encode(data),
                key: self.key.clone(),
                nonce: self.nonce.clone(),
            },
        )
        .unwrap()
        .data)
    }
    async fn decrypt(&self, data: &String) -> ClientResult<String> {
        Ok(chacha20(
            self.client.clone(),
            ParamsOfChaCha20 {
                data: data.clone(),
                key: self.key.clone(),
                nonce: self.nonce.clone(),
            },
        )
        .unwrap()
        .data)
    }
}

#[async_trait::async_trait]
impl ton_client::crypto::EncryptionBox for NaClBox {
    async fn get_info(&self) -> ClientResult<EncryptionBoxInfo> {
        Ok(EncryptionBoxInfo {
            hdpath: None,
            algorithm: Some("NaCl".to_string()),
            options: Some(json!({
                "their_pubkey": hex::encode(self.their_pubkey.clone()),
                "nonce": hex::encode(self.nonce.clone())})),
            public: None,
        })
    }
    async fn encrypt(&self, data: &String) -> ClientResult<String> {
        Ok(nacl_box(
            self.client.clone(),
            ParamsOfNaclBox {
                decrypted: base64::encode(data),
                nonce: self.nonce.clone(),
                their_public: self.their_pubkey.clone(),
                secret: self.ssecret.clone(),
            },
        )
        .unwrap()
        .encrypted)
    }
    async fn decrypt(&self, data: &String) -> ClientResult<String> {
        Ok(nacl_box_open(
            self.client.clone(),
            ParamsOfNaclBoxOpen {
                encrypted: data.clone(),
                nonce: self.nonce.clone(),
                their_public: self.their_pubkey.clone(),
                secret: self.rsecret.clone(),
            },
        )
        .unwrap()
        .decrypted)
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
            EncryptionBoxType::SecretNaCl => {
                let key = String::from("");
                let nonce = String::from("");
                let registered_box = register_encryption_box(
                    client.clone(),
                    NaClSecretBox {
                        key: hex::encode(&key), //TODO: HAS TO BE 0-padded to 64 symbols hex string
                        nonce: hex::encode(&nonce),
                        client: client.clone(),
                    },
                )
                .await
                .unwrap()
                .handle;
                Self {
                    handle: registered_box,
                    box_type: box_type,
                    client: client.clone(),
                }
            }
            EncryptionBoxType::NaCl => {
                let their_pubkey = String::from("");
                let ssecret = String::from("");
                let rsecret = String::from("");
                let nonce = String::from("");
                let registered_box = register_encryption_box(
                    client.clone(),
                    NaClBox {
                        their_pubkey: hex::encode(&their_pubkey), //TODO: HAS TO BE 0-padded to 64 symbols hex string
                        ssecret: hex::encode(&ssecret), //TODO: HAS TO BE 0-padded to 64 symbols hex string
                        rsecret: hex::encode(&rsecret), //TODO: HAS TO BE 0-padded to 64 symbols hex string
                        nonce: hex::encode(&nonce),
                        client: client.clone(),
                    },
                )
                .await
                .unwrap()
                .handle;
                Self {
                    handle: registered_box,
                    box_type: box_type,
                    client: client.clone(),
                }
            }
            EncryptionBoxType::ChaCha20 => {
                let key = String::from("");
                let nonce = String::from("");
                let registered_box = register_encryption_box(
                    client.clone(),
                    ChaChaBox {
                        key: base64::encode(&key),
                        nonce: hex::encode(&nonce),
                        client: client.clone(),
                    },
                )
                .await
                .unwrap()
                .handle;
                Self {
                    handle: registered_box,
                    box_type: box_type,
                    client: client.clone(),
                }
            }
        }
    }
    pub fn handle(&self) -> EncryptionBoxHandle {
        self.handle.clone()
    }
}
