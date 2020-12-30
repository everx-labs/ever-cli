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
use crate::helpers::create_client_verbose;
use crate::config::Config;
use serde_json::json;
use ton_client::net::{ParamsOfQueryCollection, query_collection};

const ACCOUNT_FIELDS: &str = r#"
    acc_type_name
    balance(format: DEC)
    last_paid
    last_trans_lt
    data
"#;

pub async fn get_account(conf: Config, addr: &str) -> Result<(), String> {
    let ton = create_client_verbose(&conf)?;

    println!("Processing...");
    let query_result = query_collection(
        ton.clone(),
        ParamsOfQueryCollection {
            collection: "accounts".to_owned(),
            filter: Some(json!({ "id": { "eq": addr } })),
            result: ACCOUNT_FIELDS.to_string(),
            order: None,
            limit: Some(1),
        },
    ).await.map_err(|e| format!("failed to query account info: {}", e))?;
    let accounts = query_result.result;
    println!("Succeeded.");

    if accounts.len() == 1 {
        let acc = &accounts[0];
        let acc_type = acc["acc_type_name"].as_str().unwrap();
        if acc_type != "NonExist" {
            println!("acc_type:      {}", acc_type);
            let balance_str = &acc["balance"].as_str().unwrap();
            println!("balance:       {}", u64::from_str_radix(balance_str, 10).unwrap());
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
            println!("Account does not exist.");    
        }
    } else {
        println!("Account not found.");
    }
    Ok(())
}