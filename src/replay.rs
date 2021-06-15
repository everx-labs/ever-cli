/*
 * Copyright 2021 TON DEV SOLUTIONS LTD.
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

use std::{collections::HashMap, fs::File, io::{self, BufRead, Lines, Write}, process::exit, sync::{Arc, atomic::AtomicU64}};
use clap::ArgMatches;
use serde_json::Value;

use ton_block::{Account, ConfigParamEnum, ConfigParams, Deserializable, Message, Serializable, Transaction, TransactionDescr};
use ton_client::{ClientConfig, ClientContext, abi::{Abi, CallSet, ParamsOfEncodeMessage, encode_message}, net::{AggregationFn, FieldAggregation, NetworkConfig, OrderBy, ParamsOfAggregateCollection, ParamsOfQueryCollection, SortDirection, aggregate_collection, query_collection}, tvm::{ParamsOfRunGet, ParamsOfRunTvm, run_get, run_tvm}};
use ton_executor::{BlockchainConfig, OrdinaryTransactionExecutor, TickTockTransactionExecutor, TransactionExecutor};
use ton_types::{HashmapE, UInt256, serialize_toc};

use crate::config::Config;

static CONFIG_ADDR: &str  = "-1:5555555555555555555555555555555555555555555555555555555555555555";
static ELECTOR_ADDR: &str = "-1:3333333333333333333333333333333333333333333333333333333333333333";

fn construct_blockchain_config(config_account: &Account) -> BlockchainConfig {
    let config_cell = config_account.get_data().unwrap().reference(0).ok();
    let config_params = ConfigParams::with_address_and_params(
        UInt256::with_array([0x55; 32]), config_cell);
    BlockchainConfig::with_config(config_params).unwrap()
}

async fn fetch(server_address: &str, account_address: &str, filename: &str) {
    let context = Arc::new(
        ClientContext::new(ClientConfig {
            network: NetworkConfig {
                server_address: Some(String::from(server_address)),
                ..Default::default()
            },
            ..Default::default()
        })
        .unwrap(),
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
    .unwrap();
    let tr_count = u64::from_str_radix(tr_count.values.as_array().unwrap().get(0).unwrap().as_str().unwrap(), 10).unwrap();

    let file = File::create(filename).unwrap();
    let mut writer = std::io::LineWriter::new(file);

    let zerostates = query_collection(
        context.clone(),
        ParamsOfQueryCollection {
            collection: "zerostates".to_owned(),
            filter: None,
            result: "accounts { id boc }".to_owned(),
            limit: None,
            order: None,
        },
    )
    .await;

    let mut zerostate_found = false;
    if let Ok(zerostates) = zerostates {
        let result = &zerostates.result.to_vec();
        let accounts = result[0]["accounts"].as_array().unwrap();
        for account in accounts {
            if account["id"] == account_address {
                let data = format!("{}\n", account);
                writer.write_all(data.as_bytes()).unwrap();
                zerostate_found = true;
                break;
            }
        }
    }

    if !zerostate_found {
        let data = format!("{{\"id\":\"{}\",\"boc\":\"{}\"}}\n",
            account_address, base64::encode(&Account::default().serialize().unwrap().write_to_bytes().unwrap()));
        writer.write_all(data.as_bytes()).unwrap();
    }

    let retry_strategy =
        tokio_retry::strategy::ExponentialBackoff::from_millis(10).take(5);

    let mut count = 0u64;
    let pb =indicatif::ProgressBar::new(tr_count);
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

        let transactions = tokio_retry::Retry::spawn(retry_strategy.clone(), action).await.unwrap();

        if transactions.result.is_empty() {
            break;
        }

        for txn in &transactions.result {
            let data = format!("{}\n", txn);
            writer.write_all(data.as_bytes()).unwrap();
        }

        let last = transactions.result.last().unwrap();
        lt = last["lt"].as_str().unwrap().to_owned();
        count += transactions.result.len() as u64;
        pb.set_position(std::cmp::min(count, tr_count));
    }
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
    fn new(filename: &str) -> Self {
        let file = File::open(filename).unwrap();
        let mut lines = io::BufReader::new(file).lines();

        let first_line = lines.next().unwrap().unwrap();
        let value = serde_json::from_str::<Value>(first_line.as_str()).unwrap();
        let boc = value["boc"].as_str().unwrap();
        let account = Account::construct_from_base64(boc).unwrap();
        let account_addr = String::from(value["id"].as_str().unwrap());

        Self { account, account_addr, tr: None, lines }
    }

    pub fn next_transaction(&mut self) {
        match self.lines.next() {
            Some(res) => {
                let value = serde_json::from_str::<Value>(res.unwrap().as_str()).unwrap();
                let id = String::from(value["id"].as_str().unwrap());
                let boc = value["boc"].as_str().unwrap();
                let tr = Transaction::construct_from_base64(boc).unwrap();
                let block_lt = u64::from_str_radix(&value["block"]["start_lt"].as_str().unwrap()[2..], 16).unwrap();
                self.tr = Some(TransactionExt { id, block_lt, tr });
            }
            None => {
                self.tr = None;
            }
        }
    }
}

fn choose<'a>(st1: &'a mut State, st2: &'a mut State) -> &'a mut State {
    let lt1 = st1.tr.as_ref().map_or(u64::MAX, |tr| tr.tr.lt);
    let lt2 = st2.tr.as_ref().map_or(u64::MAX, |tr| tr.tr.lt);
    if lt1 <= lt2 {
        st1
    } else {
        st2
    }
}

#[async_trait::async_trait]
trait ReplayTracker {
    async fn new() -> Self;
    async fn track(&mut self, account: &Account);
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
    async fn track(&mut self, account: &Account) {
        let account_bytes = serialize_toc(&account.serialize().unwrap()).unwrap();
        let output = run_tvm(self.ctx.clone(), ParamsOfRunTvm {
                account: base64::encode(&account_bytes),
                abi: Some(self.abi.clone()),
                message: self.message.clone(),
                ..Default::default()
            })
            .await
            .unwrap().decoded.unwrap().output.unwrap();
        //println!("{}", serde_json::to_string_pretty(&output).unwrap());
        let past_elections = output["past_elections"].as_object().unwrap();
        for (key, value) in past_elections {
            let elect_at = u32::from_str_radix(&key, 10).unwrap();
            let unfreeze = value["unfreeze_at"].as_str().unwrap();
            let t2 = u32::from_str_radix(unfreeze, 10).unwrap();
            let vset_hash = value["vset_hash"].as_str().unwrap();
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
    async fn track(&mut self, account: &Account) {
        let account_bytes = serialize_toc(&account.serialize().unwrap()).unwrap();
        let output = run_get(self.ctx.clone(), ParamsOfRunGet {
                account: base64::encode(&account_bytes),
                function_name: "past_elections_list".to_owned(),
                input: None,
                ..Default::default()
            })
            .await
            .unwrap().output;
        let mut list = output.as_array().unwrap().get(0).unwrap();
        loop {
            if list.is_null() {
                break;
            }
            let pair = list.as_array().unwrap();
            let head = pair.get(0).unwrap();

            let fields = head.as_array().unwrap();
            let key = fields.get(0).unwrap().as_str().unwrap();
            let elect_at = u32::from_str_radix(&key, 10).unwrap();
            let unfreeze = fields.get(1).unwrap().as_str().unwrap();
            let t2 = u32::from_str_radix(unfreeze, 10).unwrap();
            let vset_hash = fields.get(2).unwrap().as_str().unwrap();
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

            list = pair.get(1).unwrap();
        }
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
    async fn track(&mut self, account: &Account) {
        let config = construct_blockchain_config(&account);
        if let Some(ConfigParamEnum::ConfigParam34(cfg34)) = config.raw_config().config(34).unwrap() {
            let hash = cfg34.write_to_new_cell().unwrap().into_cell().unwrap().repr_hash();
            if self.cur_hash != UInt256::default() && self.cur_hash != hash {
                println!("DBG cfg34 hash changed to {}", &hash.to_hex_string()[..8]);
            }
            self.cur_hash = hash;
        }
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

async fn replay(input_filename: &str, config_filename: &str, txnid: &str,
    trace_execution: bool, track_elector_unfreeze: bool, track_config_param_34: bool) {
    let mut account_state = State::new(input_filename);
    let mut config_state = State::new(config_filename);
    assert!(config_state.account_addr == CONFIG_ADDR);

    let mut config = BlockchainConfig::default();
    let mut cur_block_lt = 0u64;

    if trace_execution {
        log::set_max_level(log::LevelFilter::Trace);
        log::set_logger(&LOGGER).unwrap();
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
        let tr = state.tr.as_ref().unwrap();

        print!("lt {: >26} {: >16x}, txn for {}, ", tr.tr.lt, tr.tr.lt, &state.account_addr[..8]);

        if cur_block_lt == 0 || cur_block_lt != tr.block_lt {
            assert!(tr.block_lt > cur_block_lt);
            cur_block_lt = tr.block_lt;
            config = construct_blockchain_config(&config_account);
        }

        let account_old_hash_local = state.account.serialize().unwrap().repr_hash();
        let account_old_hash_remote = tr.tr.read_state_update().unwrap().old_hash;
        if account_old_hash_local != account_old_hash_remote {
            println!("FAILURE\nOld hashes mismatch:\nremote {}\nlocal  {}",
                account_old_hash_remote.to_hex_string(),
                account_old_hash_local.to_hex_string());
            exit(1);
        }
        if tr.id == txnid {
            let account = state.account.serialize().unwrap();
            account.write_to_file(format!("{}-{}.boc", state.account_addr, txnid).as_str());

            let account = config_account.serialize().unwrap();
            account.write_to_file(format!("config-{}.boc", txnid).as_str());

            // config.boc suitable for creating ton-labs-executor tests
            let mut config_data = ton_types::SliceData::from(config_account.get_data().unwrap());
            let mut cfg = ton_types::BuilderData::new();
            cfg.append_raw(&config_data.get_next_bytes(32).unwrap(), 256).unwrap();
            cfg.append_reference_cell(config_data.reference(0).unwrap());
            cfg.into_cell().unwrap().write_to_file(format!("config-{}-test.boc", txnid).as_str());
        }
        let executor: Box<dyn TransactionExecutor> =
            match tr.tr.read_description().unwrap() {
                TransactionDescr::TickTock(desc) => {
                    Box::new(TickTockTransactionExecutor::new(config.clone(), desc.tt))
                }
                TransactionDescr::Ordinary(_) => {
                    Box::new(OrdinaryTransactionExecutor::new(config.clone()))
                }
                _ => {
                    panic!("Unknown transaction type");
                }
            };

        if track_elector_unfreeze && state.account_addr == ELECTOR_ADDR {
            tracker.as_mut().unwrap().track(&state.account).await;
        }

        let msg = tr.tr.in_msg_cell().map(|c| Message::construct_from_cell(c).unwrap());
        let tr_local = executor.execute_for_account(
            msg.as_ref(),
            &mut state.account,
            HashmapE::default(),
            tr.tr.now,
            tr.tr.lt,
            Arc::new(AtomicU64::new(tr.tr.lt)),
            trace_execution).unwrap();

        let account_new_hash_local = state.account.serialize().unwrap().repr_hash();
        let account_new_hash_remote = tr.tr.read_state_update().unwrap().new_hash;
        if account_new_hash_local != account_new_hash_remote {
            println!("FAILURE\nNew hashes mismatch:\nremote {}\nlocal  {}",
                account_new_hash_remote.to_hex_string(),
                account_new_hash_local.to_hex_string());
            println!("{:?}", tr_local.read_description().unwrap());
            exit(2);
        }

        println!("SUCCESS");

        if track_config_param_34 && state.account_addr == CONFIG_ADDR {
            cfg_tracker.as_mut().unwrap().track(&state.account).await;
        }

        if track_elector_unfreeze && state.account_addr == ELECTOR_ADDR {
            tracker.as_mut().unwrap().track(&state.account).await;
        }

        if &tr.id == txnid {
            println!("DONE");
            break;            
        }
        state.tr = None;
    }
}

pub async fn fetch_command(m: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    fetch(config.url.as_str(), m.value_of("ADDRESS").unwrap(), m.value_of("OUTPUT").unwrap()).await;
    Ok(())
}

pub async fn replay_command(m: &ArgMatches<'_>) -> Result<(), String> {
    replay(m.value_of("INPUT_TXNS").unwrap(),
        m.value_of("CONFIG_TXNS").unwrap(),
        m.value_of("TXNID").unwrap(),
        false, false, false).await;
    Ok(())
}
