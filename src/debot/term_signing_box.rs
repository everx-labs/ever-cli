use crate::crypto::load_keypair;
use super::term_browser::input;
use ton_client::crypto::KeyPair;
use std::io::{self, BufRead, Write};

pub(super) struct TerminalSigningBox {
    pub keys: KeyPair
}

impl TerminalSigningBox {
    pub fn new() -> Result<Self, String> {
        let stdio = io::stdin();
        let mut reader = stdio.lock();    
        let mut writer = io::stdout();
        let keys = input_keys(&mut reader, &mut writer, 3)?;
        Ok(Self {
            keys
        })
    }
}

pub(super) fn input_keys<R, W>(reader: &mut R, writer: &mut W, tries: u8) -> Result<KeyPair, String>
where
    R: BufRead,
    W: Write,
{
    let enter_str = "enter seed phrase or path to keypair file";
    let mut pair = Err("no keypair".to_string());
    for _ in 0..tries {
        let value = input(enter_str, reader, writer);
        pair = load_keypair(&value).map_err(|e| {
            println!("Invalid keys: {}. Try again.", e);
            e.to_string()
        });
        if pair.is_ok() {
            break;
        }
    }
    pair
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    const PUBLIC: &'static str = "9711a04f0b19474272bc7bae5472a8fbbb6ef71ce9c193f5ec3f5af808069a41";
    const PRIVATE: &'static str = "cdf2a820517fa783b9b6094d15e650af92d485084ab217fc2c859f02d49623f3";
    const SEED: &'static str = "episode polar pistol excite essence van cover fox visual gown yellow minute";
    const KEYS_FILE: &'static str = "./keys.json";

    fn create_keypair_file(name: &str) {
        let mut file = File::create(name).unwrap();
        file.write_all(format!(r#"{{
            "public": "{}",
            "secret": "{}"
        }}"#, PUBLIC, PRIVATE).as_bytes()).unwrap();
    }

    #[test]
    fn load_key_from_file() {
        let mut in_data = KEYS_FILE.as_bytes();
        let mut out_data = vec![];

        create_keypair_file(KEYS_FILE);
        let keys = input_keys(&mut in_data, &mut out_data, 1).unwrap();
        
        assert_eq!(format!("{}", keys.public), PUBLIC);
        assert_eq!(format!("{}", keys.secret), PRIVATE);        
    }

    #[test]
    fn load_key_from_seed() {
        let mut in_data = SEED.as_bytes();
        let mut out_data = vec![];

        let keys = input_keys(&mut in_data, &mut out_data, 1).unwrap();
        
        assert_eq!(format!("{}", keys.public), PUBLIC);
        assert_eq!(format!("{}", keys.secret), PRIVATE);
    }
}