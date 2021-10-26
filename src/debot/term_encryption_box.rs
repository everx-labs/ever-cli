use super::term_browser::input;
use crate::crypto::load_keypair;
use crate::helpers::{TonClient, HD_PATH};
use base64;
use std::io::{self};
use ton_client::crypto::{
    chacha20, nacl_box, nacl_box_open, nacl_secret_box, nacl_secret_box_open,
    register_encryption_box, EncryptionBoxHandle, EncryptionBoxInfo, ParamsOfChaCha20,
    ParamsOfNaclBox, ParamsOfNaclBoxOpen, ParamsOfNaclSecretBox, ParamsOfNaclSecretBoxOpen,
};
use ton_client::error::ClientResult;

#[derive(Clone, Copy)]
pub(crate) enum EncryptionBoxType {
    SecretNaCl,
    NaCl,
    ChaCha20,
}

pub(crate) struct ParamsOfTerminalEncryptionBox {
    pub box_type: EncryptionBoxType,
    pub their_pubkey: String,
    pub nonce: String,
    pub context: TonClient,
}

pub struct NaClSecretBox {
    /// 256-bit key - unprefixed 0-padded to 64 symbols hex string.
    pub key: String,
    /// 96-bit nonce, encoded in `hex`.
    pub nonce: String,
    /// Client params.context.
    pub client: TonClient,
}

pub struct ChaChaBox {
    /// 256-bit key, encoded with `base64`.
    pub key: String,
    /// 96-bit nonce, encoded in `hex`.
    pub nonce: String,
    /// Client params.context.
    pub client: TonClient,
}

pub struct NaClBox {
    /// Receiver's public key - unprefixed 0-padded to 64 symbols hex string.
    pub their_pubkey: String,
    /// Sender's private key - unprefixed 0-padded to 64 symbols hex string.
    pub secret: String,
    /// Nonce, encoded in `hex`.
    pub nonce: String,
    /// Client params.context.
    pub client: TonClient,
}

#[async_trait::async_trait]
impl ton_client::crypto::EncryptionBox for NaClSecretBox {
    async fn get_info(&self) -> ClientResult<EncryptionBoxInfo> {
        Ok(EncryptionBoxInfo {
            hdpath: Some(String::from(HD_PATH)),
            algorithm: Some("NaclSecretBox".to_string()),
            options: Some(json!({"nonce": self.nonce.clone()})),
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
            hdpath: Some(String::from(HD_PATH)),
            algorithm: Some("ChaCha20".to_string()),
            options: Some(json!({"nonce": self.nonce.clone()})),
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
            hdpath: Some(String::from(HD_PATH)),
            algorithm: Some("NaclBox".to_string()),
            options: Some(json!({
                "their_public": self.their_pubkey.clone(),
                "nonce": self.nonce.clone()})),
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
                secret: self.secret.clone(),
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
                secret: self.secret.clone(),
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
    pub async fn new(params: ParamsOfTerminalEncryptionBox) -> Result<Self, String> {
        let key: String;

        {
            let stdio = io::stdin();
            let mut reader = stdio.lock();
            let mut writer = io::stdout();
            let enter_str = "enter seed phrase or path to keypair file";
            let value = input(enter_str, &mut reader, &mut writer);
            let pair = load_keypair(&value).map_err(|e| e.to_string())?;
            key = format!("{:064}", pair.secret);
        }

        let registered_box = match params.box_type {
            EncryptionBoxType::SecretNaCl => {
                register_encryption_box(
                    params.context.clone(),
                    NaClSecretBox {
                        key: hex::encode(&key),
                        nonce: hex::encode(&params.nonce),
                        client: params.context.clone(),
                    },
                )
                .await
                .map_err(|e| e.to_string())?
                .handle
            }
            EncryptionBoxType::NaCl => {
                let padded_pubkey = format!("{:064}", params.their_pubkey);
                register_encryption_box(
                    params.context.clone(),
                    NaClBox {
                        their_pubkey: hex::encode(&padded_pubkey),
                        secret: hex::encode(&key),
                        nonce: hex::encode(&params.nonce),
                        client: params.context.clone(),
                    },
                )
                .await
                .map_err(|e| e.to_string())?
                .handle
            }
            EncryptionBoxType::ChaCha20 => {
                register_encryption_box(
                    params.context.clone(),
                    ChaChaBox {
                        key: base64::encode(&key),
                        nonce: hex::encode(&params.nonce),
                        client: params.context.clone(),
                    },
                )
                .await
                .map_err(|e| e.to_string())?
                .handle
            }
        };
        Ok(Self {
            handle: registered_box,
            box_type: params.box_type,
            client: params.context.clone(),
        })
    }
    pub fn handle(&self) -> EncryptionBoxHandle {
        self.handle.clone()
    }
}
