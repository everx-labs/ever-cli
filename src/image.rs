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
use crate::call::decode_call_parameters;
use crate::config::Config;
use crate::helpers::{create_client, create_default_client, load_ton_address};
use serde_json;
use std::fs::OpenOptions;
use std::io::Seek;
use std::io::Write;
use ton_block::{AccountStatus, GetRepresentationHash, MsgAddressInt, Serializable, StateInit};
use ton_client_rs::{EncodedMessage, LocalRunContext, OrderBy, SortDirection, TonAddress, TonClient};
use ton_sdk;
use ton_types::cells_serialization::BagOfCells;

const DEFAULT_ACCOUNT_BALANCE: u64 = 1_000_000_000000000;

#[derive(Debug)]
struct TxnInfo {
    id: String,
    aborted: bool,
    now: u32,
    lt: String,
    in_msg_id: String,
    in_msg_boc: String,
    tvm_status: bool,
    tvm_exit_code: i64,
}

impl TxnInfo {
    pub fn from_json(txn: &serde_json::Value) -> TxnInfo {
        let id = txn["id"].as_str().unwrap().to_owned();
        let in_msg_id = txn["in_msg"].as_str().unwrap().to_owned();
        let in_msg_boc = txn["in_message"]["boc"].as_str().unwrap().to_owned();
        let now = txn["now"].as_u64().unwrap() as u32;
        let lt = txn["lt"].as_str().unwrap().to_owned();
        let aborted = txn["aborted"].as_bool().unwrap();
        // TODO: think how this can be used in transaction verification
        let tvm_exit_code = 0;//txn["compute"]["exit_code"].as_i64().unwrap();
        let tvm_status = true;//txn["compute"]["success"].as_bool().unwrap();
        TxnInfo {
            id,
            aborted,
            now,
            lt,
            in_msg_id,
            in_msg_boc,
            tvm_status,
            tvm_exit_code,
        }
    }
}

fn open_tvc_file_rw(tvc_file: &str) -> Result<std::fs::File, String> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .open(tvc_file)
        .map_err(|e| format!("unable to open tvc file: {}", e))
}

fn rewrite_tvc_file(mut tvc: std::fs::File, data: Vec<u8>) -> Result<(), String> {
    tvc.seek(std::io::SeekFrom::Start(0))
        .and_then(|_| tvc.write_all(&data))
        .map_err(|e| format!("failed to rewrite tvc file: {}", e))
}

fn query_transactions(
    conf: &Config,
    filter: serde_json::Value,
) -> Result<Vec<serde_json::Value>, String> {
    let ton = create_client(conf)?;
    ton.queries.transactions.query(
        filter.into(),
        "id tr_type_name in_msg in_message { boc } now lt aborted compute { exit_code success }",
        Some(OrderBy{ path: "lt".to_owned(), direction: SortDirection::Ascending }),
        None,
    ).map_err(|e| format!("failed to query transactions : {}", e))
}

fn query_all_account_transactions(
    conf: &Config,
    addr: &str,
    last_tr_id: Option<&str>,
) -> Result<Vec<TxnInfo>, String> {
    let mut query = match last_tr_id {
        Some(id) => {
            let last_txn = query_one_transaction(conf, addr, id)?;
            json!({ "account_addr": { "eq" : addr }, "lt": { "lt" : last_txn.lt } })
        }
        None => json!({ "account_addr": { "eq" : addr } }),
    };

    let mut txn_vec = vec![];
    loop {
        let txns = query_transactions(conf, query.clone())?;
        for txn in &txns {
            txn_vec.push(TxnInfo::from_json(&txn));
        }
        if txns.len() == 50 {
            query["lt"]["gt"] = json!(txn_vec.last().unwrap().lt);
        } else {
            break;
        }
    }

    if txn_vec.len() != 0 {
        Ok(txn_vec)
    } else {
        Err(format!("transactions not found"))
    }
}

fn query_one_transaction(conf: &Config, addr: &str, tr_id: &str) -> Result<TxnInfo, String> {
    let txns = query_transactions(
        conf,
        json!({
            "id": { "eq": tr_id },
            "account_addr": { "eq" : addr }
        }),
    )?;
    if txns.len() != 0 {
        Ok(TxnInfo::from_json(&txns[0]))
    } else {
        Err(format!("transactions not found"))
    }
}

fn serialize_state_init(state: StateInit) -> Result<Vec<u8>, String> {
    let base_err = "failed to serialize account state: ";
    let cell = state
        .write_to_new_cell()
        .map_err(|e| format!("{} {}", base_err, e))?;

    let mut data = Vec::new();
    let bag = BagOfCells::with_root(&cell.into());
    bag.write_to(&mut data, false)
        .map_err(|e| format!("{}{}", base_err, e))?;
    Ok(data)
}

fn execute_all(
    addr: &str,
    tvc: &mut std::fs::File,
    txn_list: Vec<TxnInfo>,
    abi: Option<String>,
) -> Result<StateInit, String> {
    let image = ton_sdk::ContractImage::from_state_init(tvc)
        .map_err(|e| format!("unable to load contract image: {}", e))?;
    // TODO: get workchain id from string addr
    let raw_addr = image.msg_address(-1);
    let mut state_init = image.state_init();
    let ton_addr = load_ton_address(addr)?;
    let ton = create_default_client()?;
    let mut state_hash = String::new();
    let mut last_paid = 0;
    for (i, txn) in txn_list.iter().enumerate() {
        state_hash = execute_one(
            i + 1,
            &ton,
            &txn,
            raw_addr.clone(),
            &ton_addr,
            last_paid,
            abi.clone(),
            &mut state_init,
        )?;
        last_paid = txn.now;
    }
    println!("Last state hash: {}", state_hash);
    Ok(state_init)
}

fn execute_one(
    i: usize,
    ton: &TonClient,
    txn: &TxnInfo,
    raw_addr: MsgAddressInt,
    addr: &TonAddress,
    last_paid: u32,
    abi: Option<String>,
    state: &mut StateInit,
) -> Result<String, String> {
    println!("Executing txn #{}: id = {}", i, txn.id);
    let hash = state.hash().unwrap();
    println!("Old state hash: {:?}", hash);

    let account = ton_sdk::Contract {
        id: raw_addr,
        acc_type: AccountStatus::AccStateActive,
        balance: DEFAULT_ACCOUNT_BALANCE,
        balance_other: None,
        code: state.code.clone(),
        data: state.data.clone(),
        last_paid: last_paid,
    };

    let msg = EncodedMessage {
        address: addr.clone(),
        message_id: txn.in_msg_id.clone(),
        message_body: base64::decode(&txn.in_msg_boc).unwrap(),
        expire: None,
    };

    let context = LocalRunContext {
        config_boc: None,
        time: Some(txn.now),
        transaction_lt: Some(u64::from_str_radix(&txn.lt[2..], 16).unwrap()),
        block_lt: None,
    };

    let mut func_name = String::new();
    if abi.is_some() {
        let params = decode_call_parameters(ton, &msg, &abi.as_ref().unwrap())?;
        println!("Calling method {} with parameters:", params.0);
        println!("{}", params.1);
        func_name = params.0;
    }

    let result = ton.contracts.run_local_msg(
        addr,
        Some(serde_json::to_value(account).unwrap().into()),
        msg,
        abi.map(|v| v.into()),
        if func_name.is_empty() {
            None
        } else {
            Some(&func_name)
        },
        Some(context),
        true,
    );

    if result.is_err() {
        let err = result.unwrap_err();
        println!("{}", err);
        if !txn.aborted {
            Err(format!("transaction must succeeded, but it aborted"))?;
        }
        return Ok(format!("{:?}", hash));
    }

    let result = result.unwrap();
    println!("Result: {}", result.output);

    let new_account: ton_sdk::Contract = serde_json::from_value(result.account.unwrap())
        .map_err(|e| format!("failed to parse account data returned from sdk: {}", e))?;
    state.data = new_account.data;
    state.code = new_account.code;

    let hash = state.hash().unwrap();
    println!("New state hash: {:?}", hash);

    if txn.aborted {
        Err(format!("transaction must aborted"))?;
    }
    Ok(format!("{:?}", hash))
}

pub fn replay_transaction(
    conf: Config,
    addr: &str,
    tvc_file: &str,
    tr_id: &str,
) -> Result<(), String> {
    let txn = query_one_transaction(&conf, addr, tr_id)?;
    println!("Transaction received");
    println!("Inbound message id: {}", txn.in_msg_id);

    let mut tvc = open_tvc_file_rw(tvc_file)?;
    let state = execute_all(addr, &mut tvc, vec![txn], None)?;
    rewrite_tvc_file(tvc, serialize_state_init(state)?)
}

pub fn replay_state(
    conf: Config,
    addr: &str,
    tvc_file: &str,
    tr_id: Option<&str>,
    abi: Option<String>,
) -> Result<(), String> {
    let txns = query_all_account_transactions(&conf, addr, tr_id)?;
    println!("{} transactions received", txns.len());

    let mut tvc = open_tvc_file_rw(tvc_file)?;
    let state = execute_all(addr, &mut tvc, txns, abi)?;
    rewrite_tvc_file(tvc, serialize_state_init(state)?)
}