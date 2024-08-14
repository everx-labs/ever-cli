/*
 * Copyright 2018-2023 EverX.
 *
 * Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
 * this file except in compliance with the License.
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific EVERX DEV software governing permissions and
 * limitations under the License.
 */
use crate::call::{emulate_locally, process_message, send_message_and_wait};
use crate::config::FullConfig;
use crate::crypto::load_keypair;
use crate::helpers::{create_client_verbose, create_client_with_signature_id, load_abi, now_ms};
use crate::message::{display_generated_message, EncodedMessage};
use crate::{Config, SignatureIDType};
use ever_client::abi::{
    encode_message, Abi, CallSet, DeploySet, FunctionHeader, ParamsOfEncodeMessage, Signer,
};
use ever_client::crypto::KeyPair;

pub async fn deploy_contract(
    full_config: &mut FullConfig,
    tvc: &str,
    abi: &str,
    params: &str,
    keys_file: Option<String>,
    wc: i32,
    is_fee: bool,
    alias: Option<&str>,
    method: String,
) -> Result<(), String> {
    let config = &full_config.config;
    let ton = create_client_verbose(config)?;

    if !is_fee && !config.is_json {
        println!("Deploying...");
    }

    let (msg, addr) = prepare_deploy_message(
        tvc,
        abi,
        params,
        keys_file.clone(),
        wc,
        &full_config.config,
        None,
        method,
    )
    .await?;

    let enc_msg = encode_message(ton.clone(), msg.clone())
        .await
        .map_err(|e| format!("failed to create inbound message: {}", e))?;

    if config.local_run || is_fee {
        emulate_locally(ton.clone(), addr.as_str(), enc_msg.message.clone(), is_fee).await?;
        if is_fee {
            return Ok(());
        }
    }

    if config.async_call {
        let abi = load_abi(abi, config).await?;
        send_message_and_wait(ton, Some(abi), enc_msg.message, config).await?;
    } else {
        process_message(ton.clone(), msg, config)
            .await
            .map_err(|e| format!("{:#}", e))?;
    }

    if !config.is_json {
        if !config.async_call {
            println!("Transaction succeeded.");
        }
        println!("Contract deployed at address: {}", addr);
    } else {
        println!("{{}}");
    }
    if let Some(alias) = alias {
        full_config.add_alias(alias, Some(addr), Some(abi.to_string()), keys_file)?;
    }
    Ok(())
}

pub async fn generate_deploy_message(
    tvc: &str,
    abi: &str,
    params: &str,
    keys_file: Option<String>,
    wc: i32,
    is_raw: bool,
    output: Option<&str>,
    config: &Config,
    signature_id: Option<SignatureIDType>,
    method: String,
) -> Result<(), String> {
    let (client, signature_id) = create_client_with_signature_id(config, signature_id)?;

    let (msg, addr) = prepare_deploy_message(
        tvc,
        abi,
        params,
        keys_file,
        wc,
        config,
        signature_id,
        method.clone(),
    )
    .await?;
    let msg = encode_message(client, msg)
        .await
        .map_err(|e| format!("failed to create inbound message: {}", e))?;

    let msg = EncodedMessage {
        message: msg.message,
        message_id: msg.message_id,
        expire: None,
        address: addr.to_owned(),
    };
    display_generated_message(&msg, method.as_str(), is_raw, output, config.is_json)?;
    if !config.is_json {
        println!("Contract's address: {}", addr);
        println!("Succeeded.");
    }
    Ok(())
}

pub async fn prepare_deploy_message(
    tvc: &str,
    abi: &str,
    params: &str,
    keys_file: Option<String>,
    wc: i32,
    config: &Config,
    signature_id: Option<i32>,
    method: String,
) -> Result<(ParamsOfEncodeMessage, String), String> {
    let abi = load_abi(abi, config).await?;

    let keys = keys_file.map(|k| load_keypair(&k)).transpose()?;

    let tvc_bytes =
        std::fs::read(tvc).map_err(|e| format!("failed to read smart contract file {tvc}: {e}"))?;

    prepare_deploy_message_params(
        &tvc_bytes,
        abi,
        method,
        now_ms(),
        params,
        keys,
        wc,
        signature_id,
    )
    .await
}

pub async fn prepare_deploy_message_params(
    tvc_bytes: &[u8],
    abi: Abi,
    function_name: String,
    time: u64,
    params: &str,
    keys: Option<KeyPair>,
    wc: i32,
    signature_id: Option<i32>,
) -> Result<(ParamsOfEncodeMessage, String), String> {
    let tvc = base64::encode(tvc_bytes);

    let data_map_supported = abi.abi().unwrap().data_map_supported();
    let address = if data_map_supported {
        crate::helpers::calc_acc_address(
            tvc_bytes,
            wc,
            keys.as_ref().map(|k| k.public.clone()),
            None,
            abi.clone(),
        )
        .await?
    } else {
        let tvc_cell = ever_block::boc::read_single_root_boc(tvc_bytes).unwrap();
        let tvc_hash = tvc_cell.repr_hash();
        format!("{}:{}", wc, tvc_hash.as_hex_string())
    };

    let header = Some(FunctionHeader {
        time: Some(time),
        ..Default::default()
    });
    let deploy_set = Some(DeploySet {
        tvc: Some(tvc),
        workchain_id: Some(wc),
        ..Default::default()
    });
    let params = serde_json::from_str(params)
        .map_err(|e| format!("function arguments is not a json: {}", e))?;
    let call_set = Some(CallSet {
        function_name,
        input: Some(params),
        header,
    });
    let signer = if let Some(keys) = keys {
        Signer::Keys { keys }
    } else {
        Signer::None
    };
    Ok((
        ParamsOfEncodeMessage {
            abi,
            address: Some(address.clone()),
            deploy_set,
            call_set,
            signer,
            signature_id,
            ..Default::default()
        },
        address,
    ))
}
