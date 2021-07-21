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
use ton_client::utils::{calc_storage_fee, ParamsOfCalcStorageFee};
use ton_block::{Account, Deserializable, Serializable};

use crate::call::query_account_boc;

const ACCOUNT_FIELDS: &str = r#"
    acc_type_name
    balance(format: DEC)
    last_paid
    last_trans_lt
    data
    boc
    code_hash
"#;

pub async fn get_account(conf: Config, addr: &str, dumpfile: Option<&str>) -> Result<(), String> {
    let ton = create_client_verbose(&conf)?;

    if !conf.is_json {
        println!("Processing...");
    }

    let query_result = query_collection(
        ton.clone(),
        ParamsOfQueryCollection {
            collection: "accounts".to_owned(),
            filter: Some(json!({ "id": { "eq": addr } })),
            result: ACCOUNT_FIELDS.to_string(),
            limit: Some(1),
            ..Default::default()
        },
    ).await.map_err(|e| format!("failed to query account info: {}", e))?;
    let accounts = query_result.result;
    if !conf.is_json {
        println!("Succeeded.");
    }

    if conf.is_json {
        println!("{{");
        if accounts.len() == 1 {
            let acc = &accounts[0];
            let acc_type = acc["acc_type_name"].as_str().unwrap();
            if acc_type != "NonExist" {
                println!("  \"acc_type\": \"{}\",", acc_type);
                let balance_str = &acc["balance"].as_str().unwrap();
                println!("  \"balance\": \"{}\",", u64::from_str_radix(balance_str, 10).unwrap());
                println!("  \"last_paid\": \"{}\",", acc["last_paid"].as_u64().unwrap());
                println!("  \"last_trans_lt\": \"{}\",", acc["last_trans_lt"].as_str().unwrap());
                let data_str = acc["data"].as_str();
                if data_str.is_some() {
                    let data_vec = base64::decode(data_str.unwrap()).unwrap();
                    println!("  \"data(boc)\": \"{}\",", hex::encode(&data_vec));
                } else {
                    println!("  \"data(boc)\": \"null\",");
                }
                println!("  \"code_hash\": \"{}\"", acc["code_hash"].as_str().unwrap_or("null"));
            } else {
                println!("  \"acc_type\": \"{}\"", acc_type);
            }
        }
        println!("}}");
    } else {
        if accounts.len() == 1 {
            let acc = &accounts[0];
            let acc_type = acc["acc_type_name"].as_str().unwrap();
            if acc_type != "NonExist" {
                println!("acc_type:      {}", acc_type);
                let balance_str = &acc["balance"].as_str().unwrap();
                let balance_num = u64::from_str_radix(balance_str, 10).unwrap();
                if conf.balance_in_tons {
                    let int_balance = balance_num as f64/ 1e9;
                    let frac_balance = (balance_num as f64 / 1e6 + 0.5) as u64 % 1000;
                    let balance_str = format!("{}", int_balance as u64);
                    println!("balance:       {}.{} ton", balance_str.chars()
                        .collect::<Vec<char>>()
                        .rchunks(3)
                        .map(|c| c.iter().collect::<String>())
                        .rev()
                        .collect::<Vec<String>>()
                        .join(" "),
                        frac_balance
                    );
                } else {
                    println!("balance:       {} nanoton", balance_num);
                }
                println!("last_paid:     {}", acc["last_paid"].as_u64().unwrap());
                println!("last_trans_lt: {}", acc["last_trans_lt"].as_str().unwrap());
                let data_str = acc["data"].as_str();
                if data_str.is_some() {
                    let data_vec = base64::decode(data_str.unwrap()).unwrap();
                    println!("data(boc): {}", hex::encode(&data_vec));
                } else {
                    println!("data(boc): null");
                }
                println!("code_hash: {}", acc["code_hash"].as_str().unwrap_or("null"));
            } else {
                println!("Account does not exist.");
            }
        } else {
            println!("Account not found.");
        }
    }
    if dumpfile.is_some() {
        if accounts.len() == 1 {
            let acc = &accounts[0];
            let boc = acc["boc"].as_str()
                .ok_or("failed to get boc of the account")
                .map_err(|e| format!("{}", e))?;
            let account = Account::construct_from_base64(boc)
                .map_err(|e| format!("failed to load account from the boc: {}", e))?;
            if account.state_init().is_some() {
                account.state_init().unwrap()
                    .write_to_file(dumpfile.unwrap())
                    .map_err(|e| format!("failed to write data to the file: {}", e))?;
            } else {
                return Err("account doesn't contain state init.".to_owned());
            }
            if !conf.is_json {
                println!("Saved contract to file {}", &dumpfile.unwrap());
            }
        }
    }
    Ok(())
}

pub async fn calc_storage(conf: Config, addr: &str, period: u32) -> Result<(), String> {
    let ton = create_client_verbose(&conf)?;

    if !conf.is_json {
        println!("Processing...");
    }

    let boc = query_account_boc(
        ton.clone(),
        addr
    ).await.map_err(|e| format!("{}", e))?;

    let res = calc_storage_fee(
        ton.clone(),
        ParamsOfCalcStorageFee {
            account: boc,
            period,
        }
    ).await.map_err(|e| format!("failed to calculate storage fee: {}", e))?;

    if !conf.is_json {
        println!("Storage fee per {} seconds: {} nanotons", period, res.fee);
    } else {
        println!("{{");
        println!("  \"storage_fee\": \"{}\",", res.fee);
        println!("  \"period\": \"{}\"", period);
        println!("}}");
    }
    Ok(())
}
