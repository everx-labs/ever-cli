/*
 * Copyright 2022 TON DEV SOLUTIONS LTD.
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
use crate::helpers::{
    create_client_verbose,
    load_abi,
    load_ton_address,
    now,
    TonClient,
    events_filter, abi_from_matches_or_config,
};
use clap::ArgMatches;

use ton_client::abi::{ParamsOfDecodeMessageBody, Abi};
use ton_client::net::{OrderBy, ParamsOfQueryCollection, ParamsOfWaitForCollection, SortDirection};

pub async fn events_command(m: &ArgMatches<'_>, config: &Config) -> Result<(), String> {
    let addr = m.value_of("ADDRESS")
        .map(|s| s.to_string())
        .or(config.addr.clone())
        .ok_or("ADDRESS is not defined. Supply it in the config file or command line."
            .to_string())?;
    let abi_path = abi_from_matches_or_config(m, config)?;
    let abi = load_abi(&abi_path, config).await?;
        
    let since = m.value_of("SINCE");
    let wait_for = m.is_present("WAITONE");

    if !wait_for {
        let since = since.map(|s| {
                u32::from_str_radix(s, 10)
                    .map_err(|e| format!(r#"cannot parse "since" option: {}"#, e))
            })
            .transpose()?
            .unwrap_or(0);
        get_events(config, abi, &addr, since).await
    } else {
        wait_for_event(config, abi, &addr).await
    }
}

async fn print_event(ton: TonClient, abi: &Abi, event: &serde_json::Value) -> Result<(), String> {
    println!("event {}", event["id"].as_str()
        .ok_or("failed to serialize event id")?);

    let body = event["body"].as_str()
        .ok_or("failed to serialize event body")?;
    let result = ton_client::abi::decode_message_body(
        ton.clone(),
        ParamsOfDecodeMessageBody {
            abi: abi.clone(),
            body: body.to_owned(),
            is_internal: false,
            ..Default::default()
        },
    ).await;
    let (name, args) = if result.is_err() {
        ("unknown".to_owned(), "{}".to_owned())
    } else {
        let result = result.unwrap();
        (result.name, serde_json::to_string(&result.value)
            .map_err(|e| format!("failed to serialize the result: {}", e))?)
    };

    println!("{} {} ({})\n{}\n",
        name,
        event["created_at"].as_u64().ok_or("failed to serialize event field")?,
        event["created_at_string"].as_str().ok_or("failed to serialize event field")?,
        args
    );
    Ok(())
}

async fn get_events(config: &Config, abi: Abi, addr: &str, since: u32) -> Result<(), String> {
    let ton = create_client_verbose(&config)?;
    let _addr = load_ton_address(addr, &config)?;

    let events = ton_client::net::query_collection(
        ton.clone(),
        ParamsOfQueryCollection {
            collection: "messages".to_owned(),
            filter: Some(events_filter(addr, since)),
            result: "id body created_at created_at_string".to_owned(),
            order: Some(vec![OrderBy{ path: "created_at".to_owned(), direction: SortDirection::DESC }]),
            ..Default::default()
        },
    ).await.map_err(|e| format!("failed to query account events: {}", e))?;
    println!("{} events found", events.result.len());
    for event in &events.result {
        print_event(ton.clone(), &abi, event).await?;
    }
    println!("Done");
    Ok(())
}

async fn wait_for_event(config: &Config, abi: Abi, addr: &str) -> Result<(), String> {
    let ton = create_client_verbose(&config)?;
    let _addr = load_ton_address(addr, &config)?;
    println!("Waiting for a new event...");
    let event = ton_client::net::wait_for_collection(
        ton.clone(),
        ParamsOfWaitForCollection {
            collection: "messages".to_owned(),
            filter: Some(events_filter(addr, now()?)),
            result: "id body created_at created_at_string".to_owned(),
            timeout: Some(config.timeout),
            ..Default::default()
        },

    ).await.map_err(|e| println!("failed to query event: {}", e));
    if event.is_ok() {
        print_event(ton.clone(), &abi, &event.unwrap().result).await?;
    }
    Ok(())
}
