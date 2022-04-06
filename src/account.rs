/*
* Copyright 2018-2021 TON DEV SOLUTIONS LTD.
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
use crate::helpers::{check_dir, create_client_verbose, json_account, print_account, query_account_field};
use crate::config::Config;
use serde_json::{json, Value};
use ton_client::net::{ParamsOfQueryCollection, query_collection};
use ton_client::utils::{calc_storage_fee, ParamsOfCalcStorageFee};
use ton_block::{Account, Deserializable, Serializable};

const ACCOUNT_FIELDS: &str = r#"
    id
    acc_type_name
    balance(format: DEC)
    last_paid
    last_trans_lt
    data
    boc
    code_hash
"#;

const DEFAULT_PATH: &str = ".";

async fn query_accounts(conf: Config, addresses: Vec<String>, fields: &str) -> Result<Vec<Value>, String> {
    let ton = create_client_verbose(&conf, true)?;

    if !conf.is_json {
        println!("Processing...");
    }

    let mut res = vec![];
    let mut it = 0;
    loop {
        if it >= addresses.len() {
            break;
        }
        let mut filter = json!({ "id": { "eq": addresses[it] } });
        let mut cnt = 1;
        for address in addresses.iter().skip(it).take(50) {
            cnt += 1;
            filter = json!({ "id": { "eq": address },
                "OR": filter
            });
        }
        it += cnt;
        let mut query_result = query_collection(
            ton.clone(),
            ParamsOfQueryCollection {
                collection: "accounts".to_owned(),
                filter: Some(filter),
                result: fields.to_string(),
                limit: Some(cnt as u32),
                ..Default::default()
            },
        ).await.map_err(|e| format!("failed to query account info: {}", e))?;
        res.append(query_result.result.as_mut());
    }
    Ok(res)
}

pub async fn get_account(conf: Config, addresses: Vec<String>, dumpfile: Option<&str>, dumpboc: Option<&str>) -> Result<(), String> {
    let accounts = query_accounts(conf.clone(), addresses.clone(), ACCOUNT_FIELDS).await?;
    if !conf.is_json {
        println!("Succeeded.");
    }
    let mut found_addresses = vec![];
    if !accounts.is_empty() {
        let mut json_res = json!({ });
        for acc in accounts.iter() {
            let address = acc["id"].as_str().unwrap_or("Undefined").to_owned();
            found_addresses.push(address.clone());
            let acc_type = acc["acc_type_name"].as_str().unwrap_or("Undefined").to_owned();
            if acc_type != "NonExist" {
                let bal = acc["balance"].as_str();
                let balance =
                    if bal.is_some() {
                        let bal = bal.unwrap();
                        if conf.balance_in_tons {
                            let bal = u64::from_str_radix(bal, 10)
                                .map_err(|e| format!("failed to decode balance: {}", e))?;
                            let int_bal = bal as f64 / 1e9;
                            let frac_balance = (bal as f64 / 1e6 + 0.5) as u64 % 1000;
                            let balance_str = format!("{}", int_bal as u64);
                            format!("{}.{}{}", balance_str.chars()
                                .collect::<Vec<char>>()
                                .rchunks(3)
                                .map(|c| c.iter().collect::<String>())
                                .rev()
                                .collect::<Vec<String>>()
                                .join(" "),
                                    frac_balance,
                                    if conf.is_json {
                                        ""
                                    } else {
                                        " ton"
                                    }
                            )
                        } else {
                            format!("{}{}", bal,
                                    if conf.is_json {
                                        ""
                                    } else {
                                        " nanoton"
                                    }
                            )
                        }
                    } else {
                        "Undefined".to_owned()
                    };
                let last_paid = format!("{}", acc["last_paid"].as_u64().ok_or("failed to decode last_paid".to_owned())?);
                let last_trans_id = acc["last_trans_lt"].as_str().unwrap_or("Undefined").to_owned();
                let data = acc["data"].as_str();
                let data_boc = if data.is_some() {
                    hex::encode(base64::decode(data.unwrap()).map_err(|e| format!("failed to decode account data: {}", e))?)
                } else {
                    "null".to_owned()
                };
                let code_hash = acc["code_hash"].as_str().unwrap_or("null").to_owned();
                if conf.is_json {
                    json_res[address.clone()] = json_account(
                        Some(acc_type),
                        Some(address.clone()),
                        Some(balance),
                        Some(last_paid),
                        Some(last_trans_id),
                        Some(data_boc),
                        Some(code_hash),
                        None,
                    );
                } else {
                    print_account(
                        &conf,
                        Some(acc_type),
                        Some(address.clone()),
                        Some(balance),
                        Some(last_paid),
                        Some(last_trans_id),
                        Some(data_boc),
                        Some(code_hash),
                        None,
                    );
                }
            } else if conf.is_json {
                json_res[address.clone()] = json_account(Some(acc_type), Some(address.clone()), None, None, None, None, None, None);
            } else {
                print_account(&conf, Some(acc_type), Some(address.clone()), None, None, None, None, None, None);
            }
            if !conf.is_json {
                println!();
            }
        }
        for address in addresses.iter() {
            if !found_addresses.contains(address) {
                if conf.is_json {
                    json_res[address.clone()] = json!({
                       "acc_type": "NonExist"
                    });
                } else {
                    println!("{} not found", address);
                    println!();
                }
            }
        }
        if conf.is_json {
            println!("{}", serde_json::to_string_pretty(&json_res)
                .map_err(|e| format!("Failed to serialize result: {}", e))?);
        }
    } else if conf.is_json {
        println!("{{\n}}");
    } else {
        println!("Account not found.");
    }

    if dumpfile.is_some() || dumpboc.is_some() && addresses.len() == 1 && accounts.len() == 1 {
        let acc = &accounts[0];
        let boc = acc["boc"].as_str()
            .ok_or("failed to get boc of the account".to_owned())?;
        let account = Account::construct_from_base64(boc)
            .map_err(|e| format!("failed to load account from the boc: {}", e))?;
        if dumpfile.is_some() {
            if account.state_init().is_some() {
                account.state_init().unwrap()
                    .write_to_file(dumpfile.unwrap())
                    .map_err(|e| format!("failed to write data to the file {}: {}", dumpfile.unwrap(), e))?;
            } else {
                return Err("account doesn't contain state init.".to_owned());
            }
            if !conf.is_json {
                println!("Saved contract to file {}", &dumpfile.unwrap());
            }
        }
        if dumpboc.is_some() {
            account.write_to_file(dumpboc.unwrap())
                .map_err(|e| format!("failed to write data to the file {}: {}", dumpboc.unwrap(), e))?;
            if !conf.is_json {
                println!("Saved account to file {}", &dumpboc.unwrap());
            }
        }
    }
    Ok(())
}

pub async fn calc_storage(conf: Config, addr: &str, period: u32) -> Result<(), String> {
    let ton = create_client_verbose(&conf, true)?;

    if !conf.is_json {
        println!("Processing...");
    }

    let boc = query_account_field(
        ton.clone(),
        addr,
        "boc",
    ).await.map_err(|e| e)?;

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

pub async fn dump_accounts(conf:Config, addresses: Vec<String>, path: Option<&str>) -> Result<(), String> {
    let accounts = query_accounts(conf.clone(), addresses.clone(), "id boc").await?;
    let mut addresses = addresses.clone();
    check_dir(path.unwrap_or(""))?;
    for account in accounts.iter() {
        let mut address = account["id"].as_str()
            .ok_or("Failed to parse address in the query result".to_owned())?
            .to_owned();
        match addresses.iter().position(|el| el == &address) {
            Some(index) => { addresses.remove(index) },
            None => { return Err("Query contains an unexpected address.".to_string()); }
        };

        address.replace_range(..address.find(':').unwrap_or(0) + 1, "");
        let path = format!("{}/{}.boc", path.unwrap_or(DEFAULT_PATH), address);
        let boc = account["boc"].as_str()
            .ok_or("Failed to parse boc in the query result".to_owned())?;
        Account::construct_from_base64(boc)
            .map_err(|e| format!("Failed to load account from the boc: {}", e))?
            .write_to_file(path.clone())
            .map_err(|e| format!("Failed to write data to the file {}: {}", path.clone(), e))?;
        if !conf.is_json {
            println!("{} successfully dumped.", path);
        }
    }

    if !conf.is_json {
        if !addresses.is_empty() {
            for address in addresses.iter() {
                println!("{} was not found.", address);
            }
        }
        println!("Succeeded.");
    } else {
        println!("{{}}");
    }
    Ok(())
}
