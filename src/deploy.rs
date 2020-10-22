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
use crate::helpers::{create_client_verbose, load_abi, calc_acc_address};
use crate::config::Config;
use crate::crypto::load_keypair;
use ton_client::processing::{ParamsOfProcessMessage};
use ton_client::abi::{Signer, CallSet, DeploySet, ParamsOfEncodeMessage};

pub async fn deploy_contract(conf: Config, tvc: &str, abi: &str, params: &str, keys_file: &str, wc: i32) -> Result<(), String> {
    let ton = create_client_verbose(&conf)?;
    
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

    println!("Deploying...");
    let dset = DeploySet {
        tvc: tvc_base64,
        workchain_id: Some(wc),
        initial_data: None,
    };
    let params = serde_json::from_str(params)
        .map_err(|e| format!("function arguments is not a json: {}", e))?;

    let callback = |_event| { async move { } };
    ton_client::processing::process_message(
        ton.clone(),
        ParamsOfProcessMessage {
            message_encode_params: ParamsOfEncodeMessage {
                abi,
                address: Some(addr.clone()),
                deploy_set: Some(dset),
                call_set: CallSet::some_with_function_and_input("constructor", params),
                signer: Signer::Keys{ keys },
                processing_try_index: None,
            },
            send_events: true,
        },
        callback,
    ).await
    .map_err(|e| format!("deploy failed: {:#}", e))?;

    println!("Transaction succeeded.");
    println!("Contract deployed at address: {}", addr);
    Ok(())
}