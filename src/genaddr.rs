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
use crate::config::Config;
use crate::helpers::{create_client_local, read_keys, load_abi, calc_acc_address};
use ed25519_dalek::PublicKey;
use std::fs::OpenOptions;
use ton_sdk;
use crate::crypto::{gen_seed_phrase, generate_keypair_from_mnemonic};
use ton_client::utils::{convert_address, ParamsOfConvertAddress, AddressStringFormat};

pub async fn generate_address(
    conf: Config,
    tvc: &str,
    abi: &str,
    wc_str: Option<&str>,
    keys_file: Option<&str>,
    new_keys: bool,
    initial_data: Option<&str>,
    update_tvc: bool,
) -> Result<(), String> {
    let contract = std::fs::read(tvc)
        .map_err(|e| format!("failed to read smart contract file: {}", e))?;

    let abi_str = std::fs::read_to_string(abi)
        .map_err(|e| format!("failed to read ABI file: {}", e))?;

    let abi = load_abi(&abi_str)?;

    let (phrase, keys) = if keys_file.is_some() && !new_keys {
        (None, read_keys(keys_file.unwrap())?)
    } else {
        let seed_phr = gen_seed_phrase()?;
        let pair = generate_keypair_from_mnemonic(&seed_phr)?;
        (Some(seed_phr), pair)
    };
    
    let wc = wc_str.map(|wc| i32::from_str_radix(wc, 10))
        .transpose()
        .map_err(|e| format!("failed to parse workchain id: {}", e))?
        .unwrap_or(conf.wc);
        
    let addr = calc_acc_address(
        &contract,
        wc,
        keys.public.clone(),
        initial_data.clone(),
        abi.clone()
    ).await?;
    
    println!();
    if let Some(phr) = phrase {
        println!(r#"Seed phrase: "{}""#, phr);
        println!();
    }
    println!("Raw address: {}", addr);
        
    if update_tvc {
        let initial_data = initial_data.map(|s| s.to_string());
        let key_bytes = hex::decode(&keys.public).unwrap();
        update_contract_state(tvc, &key_bytes, initial_data, &abi_str)?;
    }
    
    if new_keys && keys_file.is_some() {
        let keys_json = serde_json::to_string_pretty(&keys).unwrap();
        std::fs::write(keys_file.unwrap(), &keys_json).unwrap();
    }
    
    
    println!("testnet:");
    println!("Non-bounceable address (for init): {}", calc_userfriendly_address(&addr, false, true)?);
    println!("Bounceable address (for later access): {}", calc_userfriendly_address(&addr, true, true)?);
    println!("mainnet:");
    println!("Non-bounceable address (for init): {}", calc_userfriendly_address(&addr, false, false)?);
    println!("Bounceable address (for later access): {}", calc_userfriendly_address(&addr, true, false)?);

    println!("Succeeded");
    Ok(())
}

fn calc_userfriendly_address(address: &str, bounce: bool, test: bool) -> Result<String, String> {
    convert_address(
        create_client_local().unwrap(),
        ParamsOfConvertAddress {
            address: address.to_owned(),
            output_format: AddressStringFormat::Base64{ url: true, bounce, test },
        }
    )
    .map(|r| r.address)
    .map_err(|e| format!("failed to convert address to base64 form: {}", e))
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