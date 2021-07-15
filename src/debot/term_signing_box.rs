use super::term_browser::input;
use crate::crypto::load_keypair;
use crate::helpers::TonClient;
use std::io::{self, BufRead, Read, Write, BufReader};
use ton_client::crypto::{
    get_signing_box, remove_signing_box, KeyPair, RegisteredSigningBox, SigningBoxHandle,
};

pub(super) struct TerminalSigningBox {
    handle: SigningBoxHandle,
    client: TonClient,
}

impl TerminalSigningBox {
    pub async fn new<R: Read>(client: TonClient, possible_keys: Vec<String>, reader: Option<BufReader<R>>) -> Result<Self, String> 
    {
        let keys = {
            if let Some(mut reader) = reader {
                let mut writer = io::stdout();
                input_keys(None, possible_keys, &mut reader, &mut writer, 3)?
            } else {
                let stdio = std::io::stdin();
                let mut reader = stdio.lock();
                let mut writer = io::stdout();
                input_keys(None, possible_keys, &mut reader, &mut writer, 3)?
            }
        };
        let handle = get_signing_box(client.clone(), keys)
            .await
            .map(|r| r.handle)
            .map_err(|e| e.to_string())?;

        Ok(Self { handle, client })
    }

    pub fn handle(&self) -> SigningBoxHandle {
        self.handle.clone()
    }

    pub fn leak(&mut self) -> SigningBoxHandle {
        let handle = self.handle.clone();
        self.handle = SigningBoxHandle(0);
        handle
    }
}

impl Drop for TerminalSigningBox {
    fn drop(&mut self) {
        if self.handle.0 != 0 {
            let _ = remove_signing_box(
                self.client.clone(),
                RegisteredSigningBox {
                    handle: self.handle.clone(),
                },
            );
        }
    }
}

pub(super) fn input_keys<R, W>(
    prompt: Option<&str>,
    possible_keys: Vec<String>,
    reader: &mut R,
    writer: &mut W,
    tries: u8,
) -> Result<KeyPair, String>
where
    R: BufRead,
    W: Write,
{
    let enter_str = prompt.unwrap_or("enter seed phrase or path to keypair file");
    let mut pair = Err("no keypair".to_string());
    let mut format_pubkeys = String::new();
    possible_keys
        .iter()
        .for_each(|x| format_pubkeys += &format!(" {},", x));
    for _ in 0..tries {
        let value = input(enter_str, reader, writer);
        pair = load_keypair(&value).map_err(|e| {
            println!("Invalid keys: {}. Try again.", e);
            e.to_string()
        });
        if let Ok(ref keys) = pair {
            if possible_keys.len() != 0 {
                if let None = possible_keys
                    .iter()
                    .find(|x| x.get(2..).unwrap() == keys.public.as_str())
                {
                    println!("Unexpected keys.");
                    println!(
                        "Hint: enter keypair which contains one of the following public keys: {}",
                        format_pubkeys
                    );
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }
    pair
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    const PUBLIC: &'static str = "9711a04f0b19474272bc7bae5472a8fbbb6ef71ce9c193f5ec3f5af808069a41";
    const PRIVATE: &'static str =
        "cdf2a820517fa783b9b6094d15e650af92d485084ab217fc2c859f02d49623f3";
    const SEED: &'static str =
        "episode polar pistol excite essence van cover fox visual gown yellow minute";
    const KEYS_FILE: &'static str = "./keys.json";

    fn create_keypair_file(name: &str) {
        let mut file = File::create(name).unwrap();
        file.write_all(
            format!(
                r#"{{
            "public": "{}",
            "secret": "{}"
        }}"#,
                PUBLIC, PRIVATE
            )
            .as_bytes(),
        )
        .unwrap();
    }

    #[test]
    fn load_key_from_file() {
        let mut in_data = KEYS_FILE.as_bytes();
        let mut out_data = vec![];

        create_keypair_file(KEYS_FILE);
        let keys = input_keys(None, vec![], &mut in_data, &mut out_data, 1).unwrap();
        assert_eq!(format!("{}", keys.public), PUBLIC);
        assert_eq!(format!("{}", keys.secret), PRIVATE);
    }

    #[test]
    fn load_key_from_seed() {
        let mut in_data = SEED.as_bytes();
        let mut out_data = vec![];

        let keys = input_keys(None, vec![], &mut in_data, &mut out_data, 1).unwrap();
        assert_eq!(format!("{}", keys.public), PUBLIC);
        assert_eq!(format!("{}", keys.secret), PRIVATE);
    }
}
