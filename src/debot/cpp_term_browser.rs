/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/
use super::term_signing_box::TerminalSigningBox;
use crate::config::Config;
use crate::helpers::{create_client, TonClient, load_abi};
use std::io::{self, Write};
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use ton_client::crypto::{SigningBoxHandle, decode_public_key};
use ton_client::debot::{CppBrowserCallbacks, cpprun_exec};
use ton_block::MsgAddressInt;
use ton_types::UInt256;
use ed25519_dalek::PublicKey;
use crate::convert::convert_token;
use chrono::{Utc, NaiveDateTime, DateTime};
use console::{Term, Key};
use ton_client::abi::{CallSet, encode_message, ParamsOfEncodeMessage, DeploySet};

pub fn read_line_with_hotkeys(term: &Term) -> io::Result<Option<String>> {
    if !term.features().is_attended() {
        return Ok(Some(String::new()));
    }

    let mut chars: Vec<char> = Vec::<char>::new();

    loop {
        match term.read_key()? {
            Key::Escape => {
                return Ok(None);
            }
            Key::Backspace => {
                if chars.pop().is_some() {
                    term.clear_chars(1)?;
                }
                term.flush()?;
            }
            Key::Char(chr) => {
                chars.push(chr);
                let mut bytes_char = [0; 4];
                chr.encode_utf8(&mut bytes_char);
                term.write_str(chr.encode_utf8(&mut bytes_char))?;
                term.flush()?;
            }
            Key::Enter => break,
            Key::Unknown => {
                return Err(io::Error::new(
                    io::ErrorKind::NotConnected,
                    "Not a terminal",
                ))
            }
            _ => (),
        }
    }
    term.write_line("")?;
    term.flush()?;
    Ok(Some(chars.iter().collect::<String>()))
}

struct CppTerminalBrowser {
    client: TonClient
}

impl CppTerminalBrowser {
    pub fn new(client: TonClient) -> Self {
        Self {
            client,
        }
    }
    pub fn get_client(&self) -> TonClient {
       return self.client.clone();
    }
}

struct CppCallbacks {
    browser: Arc<RwLock<CppTerminalBrowser>>,
}

impl CppCallbacks {
    pub fn new(browser: Arc<RwLock<CppTerminalBrowser>>) -> Self {
        Self { browser }
    }
}

#[async_trait::async_trait]
impl CppBrowserCallbacks for CppCallbacks {
    /// Debot asks browser to print message to user
    async fn log(&self, msg: String) {
        println!("{}", msg);
    }

    // Debot engine asks user to enter argument for an action.
    async fn input(&self, prompt: String) -> Option<String> {
        print!("{} > ", prompt);
        io::stdout().flush().unwrap();

        let term = Term::stdout();
        return read_line_with_hotkeys(&term).expect("failed to read from stdin");
    }

    /// Debot engine requests keys to sign something
    async fn get_signing_box(&self) -> Result<SigningBoxHandle, String> {
        let terminal_box = TerminalSigningBox::new()?;
        let client = self.browser.read().unwrap().client.clone();
        let handle = ton_client::crypto::get_signing_box(
            client,
            terminal_box.keys,
        )
        .await
        .map(|r| r.handle)
        .map_err(|e| e.to_string())?;
        Ok(handle)
    }

    async fn input_address(&self, prompt: String) -> Option<MsgAddressInt> {
        let val = (||
        loop {
            print!("{} > ", prompt);
            io::stdout().flush().unwrap();
            let term = Term::stdout();
            let input_text = read_line_with_hotkeys(&term).expect("failed to read from stdin");
            if input_text.is_none() { return None; }
            let input_text = input_text.unwrap();

            let addr_int = MsgAddressInt::from_str(&input_text.trim());
            if addr_int.is_ok() { return Some(addr_int.unwrap()); }
            println!("It is not a correct address, repeat:");
        })();
        return val;
    }
    async fn input_uint256(&self, prompt: String) -> Option<UInt256> {
        let val = (||
        loop {
            print!("{} > ", prompt);
            io::stdout().flush().unwrap();
            let term = Term::stdout();
            let input_text = read_line_with_hotkeys(&term).expect("failed to read from stdin");
            if input_text.is_none() { return None; }
            let input_text = input_text.unwrap();
            let v = UInt256::from_str(input_text.trim());
            if v.is_ok() { return Some(v.unwrap()); }
            let v = input_text.trim().parse::<u64>();
            if v.is_ok() {
                let mut vec = vec![0u8; (32 - 8)];
                vec.extend(v.unwrap().to_be_bytes().to_vec());
                return Some(UInt256::from(vec));
            }
            println!("It is not a correct u256 number, repeat:");
        })();
        return val;
    }
    async fn input_pubkey(&self, prompt: String) -> Option<PublicKey> {
        let val = (||
        loop {
            print!("{} > ", prompt);
            io::stdout().flush().unwrap();

            let term = Term::stdout();
            let input_text = read_line_with_hotkeys(&term).expect("failed to read from stdin");
            if input_text.is_none() { return None; }
            let input_text = input_text.unwrap();
            let v = decode_public_key(&input_text.trim().to_string());
            if v.is_ok() { return Some(v.unwrap()); }
            println!("It is not a correct pubkey, repeat:");
        })();
        return val;
    }
    async fn input_tons(&self, prompt: String) -> Option<String> {
        let val = (||
        loop {
            print!("{} > ", prompt);
            io::stdout().flush().unwrap();

            let term = Term::stdout();
            let input_text = read_line_with_hotkeys(&term).expect("failed to read from stdin");
            if input_text.is_none() { return None; }
            let input_text = input_text.unwrap();
            let v = convert_token(input_text.trim());
            if v.is_ok() { return Some(v.unwrap()); }
            println!("It is not a correct tons value, repeat:");
        })();
        return val;
    }
    async fn input_yes_or_no(&self, prompt: String) -> Option<bool> {
        let val = (||
        loop {
            print!("{} > ", prompt);
            io::stdout().flush().unwrap();

            let term = Term::stdout();
            let input_text = read_line_with_hotkeys(&term).expect("failed to read from stdin");
            if input_text.is_none() { return None; }
            let input_text = input_text.unwrap();
            if input_text.trim().to_lowercase() == "y" { return Some(true); }
            if input_text.trim().to_lowercase() == "n" { return Some(false); }
            println!("It is not a correct answer (y/n), repeat:");
        })();
        return val;
    }
    async fn input_datetime(&self, prompt: String) -> Option<NaiveDateTime> {
        let now: DateTime<Utc> = Utc::now();
        println!("UTC now is: {}", now);

        let val = (||
        loop {
            print!("{} > ", prompt);
            io::stdout().flush().unwrap();

            let term = Term::stdout();
            let input_text = read_line_with_hotkeys(&term).expect("failed to read from stdin");
            if input_text.is_none() { return None; }
            let input_text = input_text.unwrap();
            let v = NaiveDateTime::parse_from_str(&input_text.trim(), "%Y-%m-%d %H:%M:%S");
            if v.is_ok() { return Some(v.unwrap()); }
            println!("It is not a correct datetime, repeat:");
        })();
        return val;
    }

    async fn input_deploy_message(&self, prompt: String) -> Option<String> {
        // TODO: implement contract deploy message preparation
        println!("{}", prompt);
        //io::stdout().flush().unwrap();

        print!("TVC file path > ");
        io::stdout().flush().unwrap();
        let term = Term::stdout();
        let tvc_path = read_line_with_hotkeys(&term).expect("failed to read from stdin");
        if tvc_path.is_none() { return None; }
        let tvc_bytes = &std::fs::read(tvc_path.unwrap())
            .map_err(|e| format!("failed to read smart contract file: {}", e)).unwrap();
        let tvc_base64 = base64::encode(&tvc_bytes);

        print!("abi file path > ");
        io::stdout().flush().unwrap();
        let abi_path = read_line_with_hotkeys(&term).expect("failed to read from stdin");
        if abi_path.is_none() { return None; }
        let abi_str = std::fs::read_to_string(abi_path.unwrap())
            .map_err(|e| format!("failed to read ABI file: {}", e)).unwrap();
        let abi = load_abi(&abi_str).unwrap();
        let abi_contract: ton_abi::Contract = ton_abi::Contract::load(abi_str.as_bytes()).unwrap();

        print!("function to call on deploy (constructor?) > ");
        io::stdout().flush().unwrap();

        let func_name = read_line_with_hotkeys(&term).expect("failed to read from stdin");
        if func_name.is_none() { return None; }
        let func_name = func_name.unwrap();
        let callset: Option<CallSet>;
        if !abi_contract.function(&func_name).unwrap().input_params().is_empty() {
            println!("function parameters: {:?}",
                abi_contract.function(&func_name).unwrap().input_params());
            let params = read_line_with_hotkeys(&term).expect("failed to read from stdin");
            if params.is_none() { return None; }

            let params = serde_json::from_str(&params.unwrap())
                .map_err(|e| format!("function arguments is not a json: {}", e)).unwrap();
            callset = CallSet::some_with_function_and_input(&func_name, params);
        } else {
            callset = CallSet::some_with_function(&func_name);
        }
        let client = self.browser.read().unwrap().get_client();

        let message = encode_message(
            client,
            ParamsOfEncodeMessage {
                abi: abi.clone(),
                address: None,
                call_set: callset,
                deploy_set: Some(DeploySet {
                    initial_data: None,
                    tvc: tvc_base64,
                    workchain_id: None,
                    initial_pubkey: None
                }),
                processing_try_index: None,
                signer: ton_client::abi::Signer::None,
            }).await.unwrap();

        return Some(message.message);
    }
}

pub async fn run_cpp_debot_browser(
    tvc_path: &str,
    config: Config,
) -> Result<(), String> {
    println!("Connecting to {}", config.url);
    let ton = create_client(&config)?;
    let browser = Arc::new(RwLock::new(CppTerminalBrowser::new(ton.clone())));
    let callbacks = Arc::new(CppCallbacks::new(Arc::clone(&browser)));

    // "core_dumps = true" to have core dump for debot crashes
    let core_dumps = true;
    cpprun_exec(ton, callbacks, tvc_path.to_string(), core_dumps).await?;
    Ok(())
}

#[cfg(test)]
mod tests {}
