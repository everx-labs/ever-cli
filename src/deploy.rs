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
use crate::helpers::{create_client_verbose, load_abi, calc_acc_address, now};
use crate::config::Config;
use crate::crypto::load_keypair;
use ton_client::processing::{ParamsOfProcessMessage};
use ton_client::abi::{Signer, CallSet, DeploySet, ParamsOfEncodeMessage};
use ton_client::net::{OrderBy, ParamsOfQueryCollection, SortDirection};

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
        ..Default::default()
    };
    let params: serde_json::Value = serde_json::from_str(params)
        .map_err(|e| format!("function arguments is not a json: {}", e))?;

    let callback = |_event| { async move { } };

    let mut attempts = conf.retries + 1; // + 1 (first try)
    while attempts != 0 {
        attempts -= 1;
        let start = now();
        ton_client::processing::process_message(
            ton.clone(),
            ParamsOfProcessMessage {
                message_encode_params: ParamsOfEncodeMessage {
                    abi: abi.clone(),
                    address: Some(addr.clone()),
                    deploy_set: Some(dset.clone()),
                    call_set: CallSet::some_with_function_and_input("constructor", params.clone()),
                    signer: Signer::Keys{
                        keys: keys.clone()
                    },
                    // processing_try_index
                    ..Default::default()
                },
                send_events: true,
                ..Default::default()
            },
            callback,
        ).await
        .map_err(|e| format!("deploy failed: {:#}", e))?;

        let messages = ton_client::net::query_collection(
            ton.clone(),
            ParamsOfQueryCollection {
                collection: "messages".to_owned(),
                filter: Some(json!({
                    "dst": { "eq": addr },
                    "created_at": {"ge": start }
                })),
                result: "id created_at".to_owned(),
                order: Some(vec![OrderBy{ path: "created_at".to_owned(), direction: SortDirection::DESC }]),
                ..Default::default()
            },
        ).await.map_err(|e| format!("failed to query deploy message: {}", e))?;

        if messages.result.len() > 0 {
            println!("Transaction succeeded.");
            println!("Contract deployed at address: {}", addr);
            return Ok(());
        }

        println!("Message query result didn't find any. Performing next attempt...");
    }
    Err("All attempts has failed.".to_owned())
}
    