/*
 * Copyright 2021-2022 TON DEV SOLUTIONS LTD.
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

use std::{
    collections::HashMap, fs::File,
    io::{self, BufRead, Lines, Write, BufReader, Read},
    process::exit,
    sync::{Arc, atomic::AtomicU64}
};
use clap::ArgMatches;
use failure::err_msg;
use serde_json::Value;

use ton_block::{Account, ConfigParamEnum, ConfigParams, Deserializable, Message, Serializable, Transaction, TransactionDescr, Block, HashmapAugType};
use ton_client::{
    ClientConfig, ClientContext,
    abi::{Abi, CallSet, ParamsOfEncodeMessage, encode_message},
    net::{AggregationFn, FieldAggregation, NetworkConfig, OrderBy, ParamsOfAggregateCollection, ParamsOfQueryCollection, SortDirection, aggregate_collection, query_collection},
    tvm::{ParamsOfRunGet, ParamsOfRunTvm, run_get, run_tvm}
};
use ton_executor::{BlockchainConfig, ExecuteParams, OrdinaryTransactionExecutor, TickTockTransactionExecutor, TransactionExecutor};
use ton_types::{HashmapE, UInt256, serialize_toc, Cell, serialize_tree_of_cells};

use crate::config::Config;
use crate::debug_executor::{TraceLevel, DebugTransactionExecutor};

pub static CONFIG_ADDR: &str  = "-1:5555555555555555555555555555555555555555555555555555555555555555";
static ELECTOR_ADDR: &str = "-1:3333333333333333333333333333333333333333333333333333333333333333";

pub const DUMP_NONE:  u8 = 0x00;
pub const DUMP_ACCOUNT:  u8 = 0x01;
pub const DUMP_CONFIG:   u8 = 0x02;
pub const DUMP_EXECUTOR_CONFIG: u8 = 0x04;
pub const DUMP_ALL:   u8 = 0xFF;

pub fn construct_blockchain_config(config_account: &Account) -> Result<BlockchainConfig, String> {
    construct_blockchain_config_err(config_account).map_err(|e| format!("Failed to construct config: {}", e))
}

fn construct_blockchain_config_err(config_account: &Account) -> Result<BlockchainConfig, failure::Error> {
    let config_cell = config_account
        .get_data().ok_or(err_msg("Failed to get account's data"))?
        .reference(0).ok();
    let config_params = ConfigParams::with_address_and_params(
        UInt256::with_array([0x55; 32]), config_cell);
    BlockchainConfig::with_config(config_params)
}

pub async fn fetch(server_address: &str, account_address: &str, filename: &str, fast_stop: bool) -> Result<(), String> {
    if std::path::Path::new(filename).exists() {
        println!("File exists");
        return Ok(())
    }

    let context = Arc::new(
        ClientContext::new(ClientConfig {
            network: NetworkConfig {
                server_address: Some(String::from(server_address)),
                ..Default::default()
            },
            ..Default::default()
        }).map_err(|e| format!("Failed to create ctx: {}", e))?,
    );

    let tr_count = aggregate_collection(
        context.clone(),
        ParamsOfAggregateCollection {
            collection: "transactions".to_owned(),
            filter: Some(serde_json::json!({
                "account_addr": {
                    "eq": account_address
                },
            })),
            fields: Some(vec![
                FieldAggregation {
                    field: "fn".to_owned(),
                    aggregation_fn: AggregationFn::COUNT
                },
            ]),
        },
    )
    .await
    .map_err(|e| format!("Failed to fetch txns count: {}", e))?;
    let tr_count = u64::from_str_radix(
        tr_count.values.as_array().ok_or(format!("Failed to parse value"))?
        .get(0).ok_or(format!("Failed to parse value"))?
        .as_str().ok_or(format!("Failed to parse value"))?, 10)
        .map_err(|e| format!("Failed to parse decimal int: {}", e))?;

    let file = File::create(filename)
        .map_err(|e| format!("Failed to create file: {}", e))?;
    let mut writer = std::io::LineWriter::new(file);

    let zerostates = query_collection(
        context.clone(),
        ParamsOfQueryCollection {
            collection: "zerostates".to_owned(),
            filter: None,
            result: "accounts { id boc }".to_owned(),
            limit: Some(1),
            order: None,
        },
    )
    .await;

    let mut zerostate_found = false;
    if let Ok(zerostates) = zerostates {
        let result = &zerostates.result.to_vec();
        let accounts = result[0]["accounts"].as_array().ok_or(format!("Failed to parse value"))?;
        for account in accounts {
            if account["id"] == account_address {
                let data = format!("{}\n", account);
                writer.write_all(data.as_bytes()).map_err(|e| format!("Failed to write to file: {}", e))?;
                zerostate_found = true;
                break;
            }
        }
    }

    if !zerostate_found {
        println!("account {}: zerostate not found, writing out default initial state", account_address);
        let data = format!("{{\"id\":\"{}\",\"boc\":\"{}\"}}\n",
            account_address, base64::encode(&Account::default().write_to_bytes()
                .map_err(|e| format!("failed to serialize account: {}", e))?));
        writer.write_all(data.as_bytes()).map_err(|e| format!("Failed to write to file: {}", e))?;
    }
    if fast_stop {
        return Ok(());
    }
    let retry_strategy =
        tokio_retry::strategy::ExponentialBackoff::from_millis(10).take(5);

    let mut count = 0u64;
    let pb = indicatif::ProgressBar::new(tr_count);
    let mut lt = String::from("0x0");
    loop {
        let action = || async {
            let query = query_collection(
                context.clone(),
                ParamsOfQueryCollection {
                    collection: "transactions".to_owned(),
                    filter: Some(serde_json::json!({
                        "account_addr": {
                            "eq": account_address
                        },
                        "lt": { "gt": lt },
                    })),
                    result: "id lt block { start_lt } boc".to_owned(),
                    limit: None,
                    order: Some(vec![
                        OrderBy { path: "lt".to_owned(), direction: SortDirection::ASC }
                    ]),
                },
            );
            query.await
        };

        let transactions = tokio_retry::Retry::spawn(retry_strategy.clone(), action).await
            .map_err(|e| format!("Failed to fetch transactions: {}", e))?;

        if transactions.result.is_empty() {
            break;
        }

        for txn in &transactions.result {
            let data = format!("{}\n", txn);
            writer.write_all(data.as_bytes()).map_err(|e| format!("Failed to write to file: {}", e))?;
        }

        let last = transactions.result.last().ok_or(format!("Failed to get last txn"))?;
        lt = last["lt"].as_str().ok_or(format!("Failed to parse value"))?.to_owned();
        count += transactions.result.len() as u64;
        pb.set_position(std::cmp::min(count, tr_count));
    }
    Ok(())
}

struct TransactionExt {
    id: String,
    block_lt: u64,
    tr: Transaction,
}

struct State {
    account: Account,
    account_addr: String,
    tr: Option<TransactionExt>,
    lines: Lines<std::io::BufReader<File>>,
}

impl State {
    fn new(filename: &str) -> Result<Self, String> {
        let file = File::open(filename)
            .map_err(|e| format!("failed to open file {}: {}", filename, e))?;
        let mut lines = io::BufReader::new(file).lines();

        let first_line = lines.next()
            .ok_or("file is empty")?
            .map_err(|e| format!("failed to read first line: {}", e))?;
        let value = serde_json::from_str::<Value>(first_line.as_str())
            .map_err(|e| format!("failed to deserialize value: {}", e))?;
        let boc = value["boc"].as_str().ok_or("failed to decode boc")?;
        let account = Account::construct_from_base64(boc)
            .map_err(|e| format!("failed to load account from the boc: {}", e))?;
        let account_addr = String::from(value["id"].as_str()
                                            .ok_or("failed to load account address")?);

        Ok(Self { account, account_addr, tr: None, lines })
    }

    pub fn next_transaction(&mut self) -> Option<()> {
        match self.lines.next() {
            Some(res) => {
                let value = serde_json::from_str::<Value>(res.ok()?.as_str()).ok()?;
                let id = String::from(value["id"].as_str()?);
                let boc = value["boc"].as_str()?;
                let tr = Transaction::construct_from_base64(boc).ok()?;
                let block_lt = u64::from_str_radix(&value["block"]["start_lt"].as_str()?[2..], 16).ok()?;
                self.tr = Some(TransactionExt { id, block_lt, tr });
            }
            None => {
                self.tr = None;
            }
        }
        Some(())
    }
}

fn choose<'a>(st1: &'a mut State, st2: &'a mut State) -> &'a mut State {
    let lt1 = st1.tr.as_ref().map_or(u64::MAX, |tr| tr.tr.logical_time());
    let lt2 = st2.tr.as_ref().map_or(u64::MAX, |tr| tr.tr.logical_time());
    if lt1 <= lt2 {
        st1
    } else {
        st2
    }
}

#[async_trait::async_trait]
trait ReplayTracker {
    async fn new() -> Self;
    async fn track(&mut self, account: &Account) -> Result<(), String>;
}

#[derive(Clone)]
struct ElectorUnfreezeTracker {
    ctx: Arc<ClientContext>,
    abi: Abi,
    message: String,
    unfreeze_map: HashMap<u32, u32>,
}

impl Default for ElectorUnfreezeTracker {
    fn default() -> Self {
        Self {
            ctx: Arc::new(ClientContext::new(ClientConfig::default()).unwrap()),
            abi: Abi::Json(String::new()),
            message: String::new(),
            unfreeze_map: HashMap::new()
        }
    }
}

#[async_trait::async_trait]
impl ReplayTracker for ElectorUnfreezeTracker {
    async fn new() -> Self {
        let ctx = Arc::new(ClientContext::new(ClientConfig::default()).unwrap());
        let abi = Abi::Json(std::fs::read_to_string("Elector.abi.json").unwrap());
        let message = encode_message(ctx.clone(), ParamsOfEncodeMessage {
            address: Some(ELECTOR_ADDR.into()),
            abi: abi.clone(),
            call_set: CallSet::some_with_function("get"),
            ..Default::default()
        }).await.unwrap().message;
        Self { ctx, abi, message, unfreeze_map: HashMap::new() }
    }
    async fn track(&mut self, account: &Account) -> Result<(), String> {
        let account_bytes = serialize_toc(&account.serialize()
            .map_err(|e| format!("Failed to serialize account: {}", e))?)
            .map_err(|e| format!("Failed to serialize tree of cells: {}", e))?;
        let output = run_tvm(self.ctx.clone(), ParamsOfRunTvm {
                account: base64::encode(&account_bytes),
                abi: Some(self.abi.clone()),
                message: self.message.clone(),
                ..Default::default()
            })
            .await.map_err(|e| format!("Failed to execute run_tvm: {}", e))?
            .decoded.ok_or("Failed to decode run_tvm result")?
            .output.ok_or("Empty body")?;
        let past_elections = output["past_elections"].as_object().ok_or(format!("Failed to parse value"))?;
        for (key, value) in past_elections {
            let elect_at = u32::from_str_radix(&key, 10)
                .map_err(|e| format!("Failed to parse decimal int: {}", e))?;
            let unfreeze = value["unfreeze_at"].as_str().ok_or(format!("Failed to parse value"))?;
            let t2 = u32::from_str_radix(unfreeze, 10)
                .map_err(|e| format!("Failed to parse decimal int: {}", e))?;
            let vset_hash = value["vset_hash"].as_str().ok_or(format!("Failed to parse value"))?;
            match self.unfreeze_map.insert(elect_at, t2) {
                Some(t1) => {
                    if t1 != t2 {
                        println!("DBG past election {} vset {}: unfreeze time changed from {} to {} (+{})",
                            elect_at, &vset_hash[2..10], t1, t2, t2 - t1);
                    }
                }
                None => {
                    println!("DBG past election {} {}: unfreeze time set to {}",
                        elect_at, &vset_hash[2..10], t2);
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
struct ElectorOrigUnfreezeTracker {
    ctx: Arc<ClientContext>,
    unfreeze_map: HashMap<u32, u32>,
}

impl Default for ElectorOrigUnfreezeTracker {
    fn default() -> Self {
        Self {
            ctx: Arc::new(ClientContext::new(ClientConfig::default()).unwrap()),
            unfreeze_map: HashMap::new()
        }
    }
}

#[async_trait::async_trait]
impl ReplayTracker for ElectorOrigUnfreezeTracker {
    async fn new() -> Self {
        let ctx = Arc::new(ClientContext::new(ClientConfig::default()).unwrap());
        Self { ctx, unfreeze_map: HashMap::new() }
    }
    async fn track(&mut self, account: &Account) -> Result<(), String> {
        let account_bytes = serialize_toc(&account.serialize()
            .map_err(|e| format!("Failed to serialize account: {}", e))?)
            .map_err(|e| format!("Failed to serialize tree of cells: {}", e))?;
        let output = run_get(self.ctx.clone(), ParamsOfRunGet {
                account: base64::encode(&account_bytes),
                function_name: "past_elections_list".to_owned(),
                input: None,
                ..Default::default()
            })
            .await.map_err(|e| format!("Failed to execute run_tvm: {}", e))?.output;
        let mut list = output.as_array().ok_or(format!("Failed to parse value"))?
            .get(0).ok_or(format!("Failed to parse value"))?;
        loop {
            if list.is_null() {
                break;
            }
            let pair = list.as_array().ok_or(format!("Failed to parse value"))?;
            let head = pair.get(0).ok_or(format!("Failed to parse value"))?;

            let fields = head.as_array().ok_or(format!("Failed to parse value"))?;
            let key = fields.get(0).ok_or(format!("Failed to parse value"))?.as_str().ok_or(format!("Failed to parse value"))?;
            let elect_at = u32::from_str_radix(&key, 10).map_err(|e| format!("Failed to parse decimal int: {}", e))?;
            let unfreeze = fields.get(1).ok_or(format!("Failed to parse value"))?.as_str().ok_or(format!("Failed to parse value"))?;
            let t2 = u32::from_str_radix(unfreeze, 10).map_err(|e| format!("Failed to parse decimal int: {}", e))?;
            let vset_hash = fields.get(2).ok_or(format!("Failed to parse value"))?.as_str().ok_or(format!("Failed to parse value"))?;
            match self.unfreeze_map.insert(elect_at, t2) {
                Some(t1) => {
                    if t1 != t2 {
                        println!("DBG past election {} vset {}: unfreeze time changed from {} to {} (+{})",
                            elect_at, &vset_hash[2..10], t1, t2, t2 - t1);
                    }
                }
                None => {
                    println!("DBG past election {} {}: unfreeze time set to {}",
                        elect_at, &vset_hash[2..10], t2);
                }
            }

            list = pair.get(1).ok_or(format!("Failed to parse value"))?;
        }
        Ok(())
    }
}

#[derive(Clone)]
struct ConfigParam34Tracker {
    cur_hash: UInt256,
}

#[async_trait::async_trait]
impl ReplayTracker for ConfigParam34Tracker {
    async fn new() -> Self {
        Self { cur_hash: UInt256::default() }
    }
    async fn track(&mut self, account: &Account) -> Result<(), String> {
        let config = construct_blockchain_config(&account)?;
        let param = config.raw_config().config(34).map_err(
            |e| format!("Failed to get config param 34: {}", e))?;
        if let Some(ConfigParamEnum::ConfigParam34(cfg34)) = param {
            let hash = cfg34.write_to_new_cell()
                .map_err(|e| format!("failed to serialize config param 32: {}", e))?
                .into_cell()
                .map_err(|e| format!("failed to finalize cell: {}", e))?
                .repr_hash();
            if self.cur_hash != UInt256::default() && self.cur_hash != hash {
                println!("DBG cfg34 hash changed to {}", &hash.to_hex_string()[..8]);
            }
            self.cur_hash = hash;
        }
        Ok(())
    }
}

struct TrivialLogger;
static LOGGER: TrivialLogger = TrivialLogger;

impl log::Log for TrivialLogger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }
    fn log(&self, record: &log::Record) {
        println!("{}", record.args());
    }
    fn flush(&self) {}
}

pub async fn replay(
    input_filename: &str,
    config_filename: &str,
    txnid: &str,
    trace_execution: bool,
    track_elector_unfreeze: bool,
    track_config_param_34: bool,
    trace_last_transaction: TraceLevel,
    init_trace_last_logger: impl Fn() -> Result<(), String>,
    debug_info:  Option<String>,
    dump_mask: u8,
) -> Result<Transaction, String> {
    let mut account_state = State::new(input_filename)?;
    let mut config_state = State::new(config_filename)?;
    assert_eq!(config_state.account_addr, CONFIG_ADDR);

    let mut config = BlockchainConfig::default();
    let mut cur_block_lt = 0u64;

    if trace_execution {
        log::set_max_level(log::LevelFilter::Trace);
        log::set_logger(&LOGGER).map_err(|e| format!("Failed to set logger: {}", e))?;
    }

    let mut cfg_tracker = if track_config_param_34 {
        Some(ConfigParam34Tracker::new().await)
    } else {
        None
    };

    let mut tracker = if track_elector_unfreeze {
        Some(ElectorUnfreezeTracker::new().await)
    } else {
        None
    };
    loop {
        if account_state.tr.is_none() {
            account_state.next_transaction();
            if account_state.tr.is_none() {
                break;
            }
        }
        if config_state.tr.is_none() {
            config_state.next_transaction();
        }

        let config_account = config_state.account.clone();
        let state = choose(&mut account_state, &mut config_state);
        let tr = state.tr.as_ref().ok_or("failed to obtain state transaction")?;

        //print!("lt {: >26} {: >16x}, txn for {}, ", tr.tr.lt, tr.tr.lt, &state.account_addr[..8]);

        if cur_block_lt == 0 || cur_block_lt != tr.block_lt {
            assert!(tr.block_lt > cur_block_lt);
            cur_block_lt = tr.block_lt;
            config = construct_blockchain_config(&config_account)?;
        }

        //println!("{} {}", state.account_addr, tr.id);

        let mut account_root = state.account.serialize()
            .map_err(|e| format!("Failed to serialize: {}", e))?;

        let account_old_hash_local = account_root.repr_hash();
        let account_old_hash_remote = tr.tr.read_state_update()
            .map_err(|e| format!("failed to read state update: {}", e))?.old_hash;
        if account_old_hash_local != account_old_hash_remote {
            println!("FAILURE\nOld hashes mismatch:\nremote {}\nlocal  {}",
                account_old_hash_remote.to_hex_string(),
                account_old_hash_local.to_hex_string());
            exit(1);
        }
        if tr.id == txnid {
            if dump_mask & DUMP_ACCOUNT != 0 {
                let path = format!("{}-{}.boc", state.account_addr, txnid);
                account_root.write_to_file(&path);
                println!("Contract account was dumped to {}", path);
            }
            if dump_mask & DUMP_CONFIG != 0 {
                let path = format!("config-{}.boc", txnid);
                let account = config_account.serialize()
                    .map_err(|e| format!("Failed to serialize config account: {}", e))?;
                account.write_to_file(&path);
                println!("Config account was dumped to {}", path);
            }
            if dump_mask & DUMP_EXECUTOR_CONFIG != 0 {
                // config.boc suitable for creating ton-labs-executor tests
                let mut config_data = ton_types::SliceData::from(config_account.get_data()
                    .ok_or("Failed to get config data")?);
                let mut cfg = ton_types::BuilderData::new();
                cfg.append_raw(&config_data.get_next_bytes(32)
                    .map_err(|e| format!("Failed to read config data: {}", e))?, 256)
                    .map_err(|e| format!("Failed to append config data: {}", e))?;
                cfg.append_reference_cell(config_data.reference(0)
                    .map_err(|e| format!("Failed to get config zero reference: {}", e))?);
                let path = format!("config-{}-test.boc", txnid);
                cfg.into_cell().map_err(|e| format!("Failed to finalize builder: {}", e))?
                    .write_to_file(&path);
                println!("Config for executor was dumped to {}", path);
            }
        }
        let trace_last = (trace_last_transaction != TraceLevel::None) && tr.id == txnid;
        if trace_last {
            init_trace_last_logger()?;
        }
        let executor: Box<dyn TransactionExecutor> =
            match tr.tr.read_description()
                .map_err(|e| format!("failed to read transaction: {}", e))? {
                TransactionDescr::TickTock(desc) => {
                    Box::new(TickTockTransactionExecutor::new(config.clone(), desc.tt))
                }
                TransactionDescr::Ordinary(_) => {
                    if trace_last {
                        Box::new(DebugTransactionExecutor::new(config.clone(), debug_info.clone(), trace_last_transaction.clone(), false))
                    } else {
                        Box::new(OrdinaryTransactionExecutor::new(config.clone()))
                    }
                }
                _ => {
                    panic!("Unknown transaction type");
                }
            };

        if track_elector_unfreeze && state.account_addr == ELECTOR_ADDR {
            tracker.as_mut().unwrap().track(&state.account).await?;
        }

        let msg = tr.tr.in_msg_cell().map(|c| Message::construct_from_cell(c)
            .map_err(|e| format!("failed to construct message: {}", e))).transpose()?;

        let params = ExecuteParams {
            state_libs: HashmapE::default(),
            block_unixtime: tr.tr.now(),
            block_lt: tr.tr.logical_time(),
            last_tr_lt: Arc::new(AtomicU64::new(tr.tr.logical_time())),
            seed_block: UInt256::default(),
            debug: trace_execution,
            ..ExecuteParams::default()
        };
        let tr_local = executor.execute_with_libs_and_params(
            msg.as_ref(),
            &mut account_root,
            params).map_err(|e| format!("Failed to execute txn: {}", e))?;
        state.account = Account::construct_from_cell(account_root.clone())
            .map_err(|e| format!("Failed to construct account: {}", e))?;
        state.account.update_storage_stat()
            .map_err(|e| format!("failed to update account: {}", e))?;

        let account_new_hash_local = account_root.repr_hash();
        let account_new_hash_remote = tr.tr.read_state_update()
            .map_err(|e| format!("failed to read state update: {}", e))?
            .new_hash;
        if account_new_hash_local != account_new_hash_remote {
            println!("FAILURE\nNew hashes mismatch:\nremote {}\nlocal  {}",
                account_new_hash_remote.to_hex_string(),
                account_new_hash_local.to_hex_string());
            println!("{:?}", tr_local.read_description()
                .map_err(|e| format!("failed to read description: {}", e))?);
            exit(2);
        }

        if track_config_param_34 && state.account_addr == CONFIG_ADDR {
            cfg_tracker.as_mut().unwrap().track(&state.account).await?;
        }

        if track_elector_unfreeze && state.account_addr == ELECTOR_ADDR {
            tracker.as_mut().unwrap().track(&state.account).await?;
        }

        if tr.id == txnid {
            println!("DONE");
            return Ok(tr_local);
        }
        state.tr = None;
    }
    Err("Specified transaction was not found.".to_string())
}

pub async fn fetch_block(server_address: &str, block_id: &str, filename: &str) -> Result<(), failure::Error> {
    let context = Arc::new(
        ClientContext::new(ClientConfig {
            network: NetworkConfig {
                server_address: Some(String::from(server_address)),
                ..Default::default()
            },
            ..Default::default()
        })?
    );

    let block = query_collection(
        context.clone(),
        ParamsOfQueryCollection {
            collection: "blocks".to_owned(),
            filter: Some(serde_json::json!({
                "id": {
                    "eq": block_id
                },
            })),
            result: "workchain_id boc".to_owned(),
            limit: None,
            order: None,
        },
    ).await?;

    if block.result.len() != 1 {
        return Err(err_msg("Failed to fetch the block"))
    }

    let mut accounts = vec!();

    let wid = block.result.get(0).unwrap()["workchain_id"].as_i64().unwrap();
    let block = Block::construct_from_base64(block.result.get(0).unwrap()["boc"].as_str().unwrap())?;
    let extra = block.read_extra()?;
    let account_blocks = extra.read_account_blocks()?;

    account_blocks.iterate_objects(|account_block| {
        let mut slice = account_block.account_id().clone();
        let id = UInt256::construct_from(&mut slice)?;
        let account_name = format!("{}:{}", wid, id.to_hex_string());
        let mut txns = vec!();
        account_block.transaction_iterate(|tr| {
            let cell = tr.serialize()?;
            let mut bytes = vec!();
            serialize_tree_of_cells(&cell, &mut bytes)?;
            txns.push((cell.repr_hash().to_hex_string(), base64::encode(&bytes)));
            Ok(true)
        })?;
        accounts.push((account_name.clone(), txns));
        Ok(true)
    })?;

    if accounts.is_empty() {
        return Err(err_msg("The block is empty"))
    }

    for (account, _) in &accounts {
        println!("Fetching transactions of {}", account);
        fetch(server_address,
            account.as_str(),
            format!("{}.txns", account).as_str(),
            false).await.map_err(|err| err_msg(err))?;
    }

    let config_txns_path = format!("{}.txns", CONFIG_ADDR);
    if !std::path::Path::new(config_txns_path.as_str()).exists() {
        println!("Fetching transactions of {}", CONFIG_ADDR);
        fetch(server_address,
            CONFIG_ADDR,
            config_txns_path.as_str(),
            false).await.map_err(|err| err_msg(err))?;
    }

    let acc = accounts[0].0.as_str();
    let txnid = accounts[0].1[0].0.as_str();

    let config_path = format!("config-{}.boc", txnid);
    if !std::path::Path::new(config_path.as_str()).exists() {
        println!("Computing config: replaying {} up to {}", acc, txnid);
        replay(format!("{}.txns", acc).as_str(),
            config_txns_path.as_str(), txnid,
            false, false, false, TraceLevel::None,
            || Ok(()), None, DUMP_CONFIG).await.map_err(|err| err_msg(err))?;
    } else {
        println!("Using pre-computed config {}", config_path);
    }

    let mut writer = std::io::BufWriter::new(File::create(filename)?);
    let mut config_data = Vec::new();
    let mut config_file = File::open(format!("config-{}.boc", txnid))?;
    config_file.read_to_end(&mut config_data)?;
    writer.write(format!("{{\"config-boc\":\"{}\",\"accounts\":[", base64::encode(&config_data)).as_bytes())?;

    let mut index1 = 0;
    for (account, txns) in &accounts {
        println!("Pre-replaying {}", account);
        let txnid = txns[0].0.as_str();
        if !std::path::Path::new(format!("{}-{}.boc", account, txnid).as_str()).exists() {
            replay(format!("{}.txns", account).as_str(),
                format!("{}.txns", CONFIG_ADDR).as_str(), txnid,
                false, false, false, TraceLevel::None,
                || Ok(()), None, DUMP_ACCOUNT).await.map_err(|err| err_msg(err))?;
        }
        let mut account_file = File::open(format!("{}-{}.boc", account, txnid))?;
        let mut account_data = Vec::new();
        account_file.read_to_end(&mut account_data)?;
        writer.write(format!("{{\"account-boc\":\"{}\",\"transactions\":[", base64::encode(&account_data)).as_bytes())?;
        let mut index2 = 0;
        for (_, txn) in txns {
            writer.write(format!("\"{}\"{}", txn, delim(index2, txns)).as_bytes())?;
            index2 += 1;
        }
        writer.write(format!("]}}{}", delim(index1, &accounts)).as_bytes())?;
        index1 += 1;
    }
    writer.write(b"]}")?;
    println!("Wrote {}", filename);
    Ok(())
}

fn delim<T>(index: usize, vec: &Vec<T>) -> &'static str {
    if index == vec.len() - 1 { "" } else { "," }
}

// {
//   "config-boc": "<base64>",
//   "accounts": [
//     {
//       "account-boc": "<base64>",
//       "transactions": [
//         "<base64>",
//         "<base64>",
//         "<base64>"
//       ]
//     },
//     {
//       ...
//     },
//     ...
//   ]
// }

struct AccountReplayData {
    account_cell: Cell,
    transactions: Vec<Transaction>,
}

struct BlockReplayData {
    config: BlockchainConfig,
    accounts: Vec<AccountReplayData>,
}

#[inline(never)]
fn replay_block_impl(data: BlockReplayData) -> Result<(), failure::Error> {
    let mut acc_count = 0;
    let mut txn_count = 0;
    let start = std::time::Instant::now();
    for acc in data.accounts {
        let mut account = acc.account_cell;
        for tr in acc.transactions {
            let executor: Box<dyn TransactionExecutor> =
                match tr.read_description()? {
                    TransactionDescr::TickTock(desc) => {
                        Box::new(TickTockTransactionExecutor::new(data.config.clone(), desc.tt))
                    }
                    TransactionDescr::Ordinary(_) => {
                        Box::new(OrdinaryTransactionExecutor::new(data.config.clone()))
                    }
                    _ => panic!("Unknown transaction type")
                };
            let msg_cell = tr.in_msg_cell();
            let msg = msg_cell.map(|c| Message::construct_from_cell(c)).transpose()?;

            let params = ExecuteParams {
                state_libs: HashmapE::default(),
                block_unixtime: tr.now,
                block_lt: tr.lt,
                last_tr_lt: Arc::new(AtomicU64::new(tr.lt)),
                seed_block: UInt256::default(),
                debug: false,
            };
            let tr_local = executor.execute_with_libs_and_params(
                msg.as_ref(),
                &mut account,
                params)?;

            let account_new_hash_local = account.repr_hash();
            let account_new_hash_remote = tr.read_state_update()?.new_hash;
            if account_new_hash_local != account_new_hash_remote {
                println!("FAILURE\nNew hashes mismatch:\nremote {}\nlocal  {}",
                    account_new_hash_remote.to_hex_string(),
                    account_new_hash_local.to_hex_string());
                println!("{:?}", tr_local.read_description()?);
                return Err(err_msg("hash mismatch"));
            }
            txn_count += 1;
        }
        acc_count += 1;
    }
    let duration = start.elapsed();
    println!("Replayed {} transactions of {} accounts", txn_count, acc_count);
    println!("Time elapsed is {:?}", duration);
    Ok(())
}

pub async fn replay_block(block_filename: &str) -> Result<(), failure::Error> {
    let block_file = File::open(block_filename)?;
    let block_json: Value = serde_json::from_reader(BufReader::new(block_file))?;

    let toplevel = block_json.as_object()
        .ok_or(err_msg("Failed to read block description"))?;
    let config_boc = toplevel.get("config-boc")
        .ok_or(err_msg("No config-boc field found"))?.as_str()
        .ok_or(err_msg("config-boc field is not a string"))?;
    let config_account = Account::construct_from_base64(config_boc)?;
    let config = construct_blockchain_config_err(&config_account)?;
    let accounts_json = toplevel.get("accounts")
        .ok_or(err_msg("No accounts field found"))?.as_array()
        .ok_or(err_msg("accounts field is not a string"))?;

    let mut accounts = Vec::new();

    for acc in accounts_json {
        let account_boc = acc.get("account-boc")
            .ok_or(err_msg("No account-boc field found"))?.as_str()
            .ok_or(err_msg("account-boc field is not a string"))?;
        let account = Account::construct_from_base64(account_boc)?;
        let account_cell = account.serialize()?;

        let mut transactions = Vec::new();

        let txns = acc.get("transactions")
            .ok_or(err_msg("No transactions field found"))?.as_array()
            .ok_or(err_msg("transactions field is not an array"))?;
        for txn in txns {
            let txn_boc = txn.as_str()
                .ok_or(err_msg("transaction boc is not a string"))?;
            let tr = Transaction::construct_from_base64(txn_boc)?;
            transactions.push(tr);
        }
        accounts.push(AccountReplayData { account_cell, transactions });
    }

    // all required data has been loaded into memory, now do the desired replay
    replay_block_impl(BlockReplayData { config, accounts })
}

pub async fn fetch_block_command(m: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    fetch_block(config.url.as_str(),
        m.value_of("BLOCKID").ok_or("Missing block id")?,
        m.value_of("OUTPUT").ok_or("Missing output filename")?
    ).await.map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn fetch_command(m: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    fetch(config.url.as_str(),
        m.value_of("ADDRESS").ok_or("Missing account address")?,
        m.value_of("OUTPUT").ok_or("Missing output filename")?,
        false
    ).await?;
    if config.is_json {
        println!("{{}}");
    } else {
        println!("Succeeded");
    }
    Ok(())
}

pub async fn replay_block_command(m: &ArgMatches<'_>) -> Result<(), String> {
    replay_block(m.value_of("BLOCK").ok_or("Missing block filename")?).await
        .map_err(|e| format!("Replay failed: {}", e))
}

pub async fn replay_command(m: &ArgMatches<'_>) -> Result<(), String> {
    let _ = replay(m.value_of("INPUT_TXNS").ok_or("Missing input txns filename")?,
        m.value_of("CONFIG_TXNS").ok_or("Missing config txns filename")?,
        m.value_of("TXNID").ok_or("Missing final txn id")?,
        false, false, false, TraceLevel::None, ||{Ok(())}, None, DUMP_ALL
    ).await?;
    Ok(())
}
