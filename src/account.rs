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
use serde_json::json;
use ton_client_rs::{TonClient, TonAddress};

const ACCOUNT_FIELDS: &str = r#"
    acc_type_name
    balance
    last_paid
    last_trans_lt
    data
"#;

pub fn get_account(conf: Config, addr: &str) -> Result<(), String> {
    let ton = TonClient::new_with_base_url(&conf.url)
        .map_err(|e| format!("failed to create tonclient: {}", e.to_string()))?;

    TonAddress::from_str(addr)
        .map_err(|e| format!("failed to parse address: {}", e.to_string()))?;
    
    println!("Processing...");
    let query_result = ton.queries.accounts.query(
        &json!({
            "id": { "eq": addr }
        }).to_string(),
        ACCOUNT_FIELDS,
        None,
        None,
    ).map_err(|e| format!("failed to query account info: {}", e.to_string()))?;
    println!("Succeded.");

    if query_result.len() == 1 {
        let acc = &query_result[0];
        println!("acc_type:      {}", acc["acc_type_name"].as_str().unwrap());
        let balance_str = &acc["balance"].as_str().unwrap()[2..];
        println!("balance:       {}", u64::from_str_radix(balance_str, 16).unwrap());
        println!("last_paid:     {}", acc["last_paid"].as_u64().unwrap());
        println!("last_trans_lt: {}", acc["last_trans_lt"].as_str().unwrap());
        let data_str = acc["data"].as_str();
        if data_str.is_some() {
            let data_vec = base64::decode(data_str.unwrap()).unwrap();
            println!("data(boc): {}", hex::encode(&data_vec));
        } else {
            println!("data(boc): null");
        }
    } else {
        println!("Account not found.");
    }
    Ok(())
}