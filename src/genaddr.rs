/*
 * Copyright 2018-2019 TON DEV SOLUTIONS LTD.
 *
 * Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
 * this file except in compliance with the License.  You may obtain a copy of the
 * License at: https://ton.dev/licenses
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific TON DEV software governing permissions and
 * limitations under the License.
 */
use crate::config::Config;
use crate::helpers::read_keys;
use crc16::*;
use base64;
use ed25519_dalek::PublicKey;
use ton_client_rs::{TonClient, TonAddress};
use serde_json;
use std::fs::OpenOptions;
use ton_sdk;
use crate::crypto::{gen_seed_phrase, generate_keypair_from_mnemonic, keypair_to_ed25519pair};

pub fn generate_address(
    _conf: Config,
    tvc: &str,
    abi: &str,
    wc_str: Option<&str>,
    keys_file: Option<&str>,
    new_keys: bool,
    initial_data: Option<&str>,
    update_tvc: bool,
) -> Result<(), String> {
    let ton = TonClient::default()
        .map_err(|e| format!("failed to create tonclient: {}", e.to_string()))?;

    let contract = std::fs::read(tvc)
        .map_err(|e| format!("failed to read smart contract file: {}", e.to_string()))?;

    let abi = std::fs::read_to_string(abi)
        .map_err(|e| format!("failed to read ABI file: {}", e.to_string()))?;

    let (phrase, keys) = if keys_file.is_some() && !new_keys {
        (None, read_keys(keys_file.unwrap())?)
    } else {
        let seed_phr = gen_seed_phrase()?;
        let pair = generate_keypair_from_mnemonic(&seed_phr)?;
        (Some(seed_phr), keypair_to_ed25519pair(pair)?)
    };
    
    let initial_data = initial_data.map(|s| s.to_string());
    
    let wc = i32::from_str_radix(wc_str.unwrap_or("0"), 10)
    .map_err(|e| format!("failed to parse workchain id: {}", e))?;
    
    let addr = ton.contracts.get_deploy_address(
        &abi,
        &contract,
        initial_data.clone().map(|d| d.into()),
        &keys.public,
        wc,
    ).map_err(|e| format!("failed to generate address: {}", e.to_string()))?;
    
    println!();
    if let Some(phr) = phrase {
        println!(r#"Seed phrase: "{}""#, phr);
        println!();
    }
    println!("Raw address: {}", addr);
    
    if update_tvc {
        update_contract_state(tvc, &keys.public.0, initial_data, &abi)?;
    }
    
    if new_keys && keys_file.is_some() {
        let keys_json = serde_json::to_string_pretty(&keys).unwrap();
        std::fs::write(keys_file.unwrap(), &keys_json).unwrap();
    }
    
    if let TonAddress::Std(wc, addr256) = addr {
        println!("testnet:");
        println!("Non-bounceable address (for init): {}", &calc_userfriendly_address(wc, &addr256, false, true));
        println!("Bounceable address (for later access): {}", &calc_userfriendly_address(wc, &addr256, true, true));
        println!("mainnet:");
        println!("Non-bounceable address (for init): {}", &calc_userfriendly_address(wc, &addr256, false, false));
        println!("Bounceable address (for later access): {}", &calc_userfriendly_address(wc, &addr256, true, false));
    }

    println!("Succeeded");
    Ok(())
}

fn calc_userfriendly_address(wc: i8, addr: &[u8], bounce: bool, testnet: bool) -> String {
    let mut bytes: Vec<u8> = vec![];
    bytes.push(if bounce { 0x11 } else { 0x51 } + if testnet { 0x80 } else { 0 });
    bytes.push(wc as u8);
    bytes.extend_from_slice(addr);
    let crc = State::<XMODEM>::calculate(&bytes);
    bytes.extend_from_slice(&crc.to_be_bytes());
    base64::encode(&bytes)
}

fn update_contract_state(tvc_file: &str, pubkey: &[u8], data: Option<String>, abi: &str) -> Result<(), String> {
    use std::io::{Seek, Write};
    let mut state_init = OpenOptions::new().read(true).write(true).open(tvc_file)
        .map_err(|e| format!("unable to open contract file: {}", e))?;

    let pubkey_object = PublicKey::from_bytes(pubkey)
        .map_err(|e| format!("unable to load public key: {}", e))?;

    let mut contract_image = ton_sdk::ContractImage::from_state_init_and_key(&mut state_init, &pubkey_object)
        .map_err(|e| format!("unable to load contract image: {}", e))?;

    if data.is_some() {
        contract_image.update_data(&data.unwrap(), abi)
            .map_err(|e| format!("unable to update contract image data: {}", e))?;
    }

    let vec_bytes = contract_image.serialize()
        .map_err(|e| format!("unable to serialize contract image: {}", e))?;

    state_init.seek(std::io::SeekFrom::Start(0)).unwrap();
    state_init.write_all(&vec_bytes).unwrap();
    println!("TVC file updated");

    Ok(())
}