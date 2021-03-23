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
use crate::helpers::{create_client_verbose, create_client_local, load_abi, calc_acc_address};
use crate::config::Config;
use crate::crypto::load_keypair;
use crate::call::{EncodedMessage, display_generated_message};
use ton_client::processing::{ParamsOfProcessMessage};
use ton_client::abi::{
    encode_message, Signer, CallSet, DeploySet, ParamsOfEncodeMessage
};

pub async fn deploy_contract(
    conf: Config,
    tvc: &str,
    abi: &str,
    params: &str,
    keys_file: &str,
    wc: i32
) -> Result<(), String> {
    let ton = create_client_verbose(&conf)?;

    println!("Deploying...");

    let (msg, addr) = prepare_deploy_message(tvc, abi, params, keys_file, wc).await?;

    let callback = |_event| { async move { } };
    ton_client::processing::process_message(
        ton.clone(),
        ParamsOfProcessMessage {
            message_encode_params: msg,
            send_events: true,
            ..Default::default()
        },
        callback,
    ).await
    .map_err(|e| format!("deploy failed: {:#}", e))?;

    println!("Transaction succeeded.");
    println!("Contract deployed at address: {}", addr);
    Ok(())
}

pub async fn generate_deploy_message(
    tvc: &str,
    abi: &str,
    params: &str,
    keys_file: &str, wc: i32,
    is_raw: bool,
    output: Option<&str>,
) -> Result<(), String> {

    let ton = create_client_local()?;

    let (msg, addr) = prepare_deploy_message(tvc, abi, params, keys_file, wc).await?;
    let msg = encode_message(ton, msg).await
        .map_err(|e| format!("failed to create inbound message: {}", e))?;
    let msg = EncodedMessage {
        message: msg.message,
        message_id: msg.message_id,
        expire: None,
        address: addr.to_owned(),
    };
    display_generated_message(&msg, "constructor", is_raw, output)?;
    println!("Contract's address: {}", addr);
    println!("Succeeded.");

    Ok(())
}

pub async fn prepare_deploy_message(
    tvc: &str,
    abi: &str,
    params: &str,
    keys_file: &str,
    wc: i32
) -> Result<(ParamsOfEncodeMessage, String), String> {

    let abi = std::fs::read_to_string(abi)
        .map_err(|e| format!("failed to read ABI file: {}", e))?;
    let abi = load_abi(&abi)?;

    let keys = load_keypair(keys_file)?;

    let tvc_bytes = &std::fs::read(tvc)
        .map_err(|e| format!("failed to read smart contract file: {}", e))?;

    let tvc_base64 = base64::encode(&tvc_bytes);

    let addr = calc_acc_address(
        &tvc_bytes,
        wc,
        keys.public.clone(),
        None,
        abi.clone()
    ).await?;

    let dset = DeploySet {
        tvc: tvc_base64,
        workchain_id: Some(wc),
        ..Default::default()
    };
    let params = serde_json::from_str(params)
        .map_err(|e| format!("function arguments is not a json: {}", e))?;

    Ok((ParamsOfEncodeMessage {
        abi,
        address: Some(addr.clone()),
        deploy_set: Some(dset),
        call_set: CallSet::some_with_function_and_input("constructor", params),
        signer: Signer::Keys{ keys },
        ..Default::default()
    }, addr))
}
    