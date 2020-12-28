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
use crate::{print_args, VERBOSE_MODE};
use crate::config::Config;
use crate::convert;
use crate::depool_abi::DEPOOL_ABI;
use crate::helpers::{create_client_local, create_client_verbose, load_abi, load_ton_address, now, TonClient};
use crate::multisig::send_with_body;
use clap::{App, ArgMatches, SubCommand, Arg, AppSettings};
use serde_json;
use ton_client::abi::{ParamsOfEncodeMessageBody, ParamsOfDecodeMessageBody, CallSet, Signer};
use ton_client::net::{OrderBy, ParamsOfQueryCollection, ParamsOfWaitForCollection, SortDirection};

pub fn create_depool_command<'a, 'b>() -> App<'a, 'b> {
    let wallet_arg = Arg::with_name("MSIG")
        .takes_value(true)
        .long("--wallet")
        .short("-w")
        .help("Multisig wallet address.");
    let value_arg = Arg::with_name("VALUE")
        .takes_value(true)
        .long("--value")
        .short("-v")
        .help("Value in tons.");
    let keys_arg = Arg::with_name("SIGN")
        .takes_value(true)
        .long("--sign")
        .short("-s")
        .help("Path to keypair file or seed phrase which must be used to sign message to multisig wallet.");
    let total_period_arg = Arg::with_name("TPERIOD")
        .takes_value(true)
        .long("--total")
        .short("-t")
        .help("Total period of vesting stake (days).");
    let withdrawal_period_arg = Arg::with_name("WPERIOD")
        .takes_value(true)
        .long("--withdrawal")
        .short("-i")
        .help("Payment period of vesting stake (days).");
    let beneficiary_arg = Arg::with_name("BENEFICIARY")
        .takes_value(true)
        .long("--beneficiary")
        .short("-b")
        .help("Smart contract address which will own lock stake rewards.");
    let donor_arg = Arg::with_name("DONOR")
        .takes_value(true)
        .long("--donor")
        .short("-d")
        .help("Donor smart contract address.");
    let dest_arg = Arg::with_name("DEST")
        .takes_value(true)
        .long("--dest")
        .short("-d")
        .help("Address of destination smart contract.");
    SubCommand::with_name("depool")
        .about("DePool commands.")
        .setting(AppSettings::AllowLeadingHyphen)
        .setting(AppSettings::DontCollapseArgsInUsage)
        .arg(Arg::with_name("ADDRESS")
            .takes_value(true)
            .long("--addr")
            .help("DePool contract address. if the parameter is omitted, then the value `addr` from the config is used"))
        .subcommand(SubCommand::with_name("donor")
            .about(r#"Top level command for specifying donor for exotic stakes in depool."#)
            .subcommand(SubCommand::with_name("vesting")
                .about("Set the address from which participant can receive a vesting stake.")
                .setting(AppSettings::AllowLeadingHyphen)
                .arg(wallet_arg.clone())
                .arg(keys_arg.clone())
                .arg(donor_arg.clone()))
            .subcommand(SubCommand::with_name("lock")
                .about("Set the address from which participant can receive a lock stake.")
                .setting(AppSettings::AllowLeadingHyphen)
                .arg(wallet_arg.clone())
                .arg(keys_arg.clone())
                .arg(donor_arg.clone())))
        .subcommand(SubCommand::with_name("stake")
            .about(r#"Top level command for managing stakes in depool. Uses a supplied multisignature wallet to send internal message with stake to depool."#)
            .subcommand(SubCommand::with_name("ordinary")
                .about("Deposits ordinary stake in depool from multisignature wallet.")
                .setting(AppSettings::AllowLeadingHyphen)
                .arg(wallet_arg.clone())
                .arg(value_arg.clone())
                .arg(keys_arg.clone()))
            .subcommand(SubCommand::with_name("vesting")
                .about("Deposits vesting stake in depool from multisignature wallet.")
                .setting(AppSettings::AllowLeadingHyphen)
                .arg(wallet_arg.clone())
                .arg(value_arg.clone())
                .arg(keys_arg.clone())
                .arg(total_period_arg.clone())
                .arg(withdrawal_period_arg.clone())
                .arg(beneficiary_arg.clone()))
            .subcommand(SubCommand::with_name("lock")
                .about("Deposits lock stake in depool from multisignature wallet.")
                .setting(AppSettings::AllowLeadingHyphen)
                .arg(wallet_arg.clone())
                .arg(value_arg.clone())
                .arg(keys_arg.clone())
                .arg(total_period_arg.clone())
                .arg(withdrawal_period_arg.clone())
                .arg(beneficiary_arg.clone()))
            .subcommand(SubCommand::with_name("transfer")
                .about("Transfers ownership of wallet stake to another contract.")
                .setting(AppSettings::AllowLeadingHyphen)
                .arg(wallet_arg.clone())
                .arg(value_arg.clone())
                .arg(keys_arg.clone())
                .arg(dest_arg.clone()))
            .subcommand(SubCommand::with_name("remove")
                .about("Withdraws ordinary stake from current pooling round of depool to the multisignature wallet.")
                .setting(AppSettings::AllowLeadingHyphen)
                .arg(wallet_arg.clone())
                .arg(value_arg.clone())
                .arg(keys_arg.clone()))
            .subcommand(SubCommand::with_name("withdrawPart")
                .about("Withdraws part of the stake after round completion.")
                .setting(AppSettings::AllowLeadingHyphen)
                .arg(wallet_arg.clone())
                .arg(value_arg.clone())
                .arg(keys_arg.clone())))
        .subcommand(SubCommand::with_name("replenish")
            .about("Transfers funds from the multisignature wallet to the depool contract (NOT A STAKE).")
            .setting(AppSettings::AllowLeadingHyphen)
            .arg(wallet_arg.clone())
            .arg(value_arg.clone())
            .arg(keys_arg.clone()))
        .subcommand(SubCommand::with_name("ticktock")
            .about("Call DePool 'ticktock()' function to update its state. 1 ton is attached to this call (change will be returned).")
            .setting(AppSettings::AllowLeadingHyphen)
            .arg(wallet_arg.clone())
            .arg(keys_arg.clone()))
        .subcommand(SubCommand::with_name("withdraw")
            .about("Allows to disable auto investment of the stake into next round and withdraw all the stakes after round completion.")
            .setting(AppSettings::AllowLeadingHyphen)
            .subcommand(SubCommand::with_name("on")
                .setting(AppSettings::AllowLeadingHyphen)
                .arg(wallet_arg.clone())
                .arg(keys_arg.clone()))
            .subcommand(SubCommand::with_name("off")
                .setting(AppSettings::AllowLeadingHyphen)
                .arg(wallet_arg.clone())
                .arg(keys_arg.clone())))
        .subcommand(SubCommand::with_name("events")
            .about("Prints depool events.")
            .setting(AppSettings::AllowLeadingHyphen)
            .arg(Arg::with_name("SINCE")
                .takes_value(true)
                .long("--since")
                .short("-s")
                .help("Prints events since this unixtime."))
            .arg(Arg::with_name("WAITONE")
                .long("--wait-one")
                .short("-w")
                .help("Waits until new event will be emitted.")) )
}

struct CommandData<'a> {
    conf: Config,
    depool: String,
    wallet: String,
    keys: String,
    stake: &'a str,
    depool_fee: String,
}

impl<'a> CommandData<'a> {
    pub fn from_matches_and_conf(m: &'a ArgMatches, conf: Config, depool: String) -> Result<Self, String> {
        let (wallet, stake, keys) = parse_stake_data(m, &conf)?;
        let depool_fee = conf.depool_fee.clone().to_string();
        Ok(CommandData {conf, depool, wallet, stake, keys, depool_fee})
    }
}

fn parse_wallet_data(m: &ArgMatches, conf: &Config) -> Result<(String, String), String> {
    let wallet = m.value_of("MSIG")
        .map(|s| s.to_string())
        .or(conf.wallet.clone())
        .ok_or("multisig wallet address is not defined.".to_string())?;
    load_ton_address(&wallet)
        .map_err(|e| format!("invalid multisig address: {}", e))?;
    let keys = m.value_of("SIGN")
        .map(|s| s.to_string())
        .or(conf.keys_path.clone())
        .ok_or("keypair is not defined.".to_string())?;
    Ok((wallet, keys))
}

fn parse_stake_data<'a>(m: &'a ArgMatches, conf: &Config) -> Result<(String, &'a str, String), String> {
    let (wallet, keys) = parse_wallet_data(m, conf)?;
    let stake = m.value_of("VALUE")
        .ok_or("stake value is not defined.".to_string())?;
    Ok((wallet, stake, keys))
}

pub async fn depool_command(m: &ArgMatches<'_>, conf: Config) -> Result<(), String> {
    let depool = m.value_of("ADDRESS")
        .map(|s| s.to_string())
        .or(conf.addr.clone())
        .ok_or("depool address is not defined. Supply it in config file or in command line.".to_string())?;
    load_ton_address(&depool)
        .map_err(|e| format!("invalid depool address: {}", e))?;

    if let Some(m) = m.subcommand_matches("donor") {
        let matches = m.subcommand_matches("vesting").or(m.subcommand_matches("lock"));
        if let Some(matches) = matches {
            let is_vesting = m.subcommand_matches("vesting").is_some();
            let (wallet, keys) = parse_wallet_data(&matches, &conf)?;
            return set_donor_command(matches, conf, &depool, &wallet, &keys, is_vesting).await;
        }
    }

    if let Some(m) = m.subcommand_matches("stake") {
        if let Some(m) = m.subcommand_matches("ordinary") {
            return ordinary_stake_command(m,
                CommandData::from_matches_and_conf(m, conf, depool)?,
            ).await;
        }
        if let Some(m) = m.subcommand_matches("vesting") {
            return exotic_stake_command(m,
                CommandData::from_matches_and_conf(m, conf, depool)?,
                true
            ).await;
        }
        if let Some(m) = m.subcommand_matches("lock") {
            return exotic_stake_command(m,
                CommandData::from_matches_and_conf(m, conf, depool)?,
                false
            ).await;
        }
        if let Some(m) = m.subcommand_matches("remove") {
            return remove_stake_command(m,
                CommandData::from_matches_and_conf(m, conf, depool)?,
            ).await;
        }
        if let Some(m) = m.subcommand_matches("withdrawPart") {
            return withdraw_stake_command(m,
                CommandData::from_matches_and_conf(m, conf, depool)?,
            ).await;
        }
        if let Some(m) = m.subcommand_matches("transfer") {
            return transfer_stake_command(m,
                CommandData::from_matches_and_conf(m, conf, depool)?,
            ).await;
        }
    }
    if let Some(m) = m.subcommand_matches("withdraw") {
        let matches = m.subcommand_matches("on").or(m.subcommand_matches("off"));
        if let Some(matches) = matches {
            let (wallet, keys) = parse_wallet_data(&matches, &conf)?;
            let enable_withdraw = m.subcommand_matches("on").is_some();
            return set_withdraw_command(matches, conf, &depool, &wallet, &keys, enable_withdraw).await;
        }
    }
    if let Some(m) = m.subcommand_matches("events") {
        return events_command(m, conf, &depool).await
    }
    if let Some(m) = m.subcommand_matches("replenish") {
        return replenish_command(m,
            CommandData::from_matches_and_conf(m, conf, depool)?,
        ).await;
    }
    if let Some(m) = m.subcommand_matches("ticktock") {
        let (wallet, keys) = parse_wallet_data(&m, &conf)?;
        return ticktock_command(m, conf, &depool, &wallet, &keys).await;
    }
    Err("unknown depool command".to_owned())
}

/*
 * Events command
 */

async fn events_command(m: &ArgMatches<'_>, conf: Config, depool: &str) -> Result<(), String> {
    let since = m.value_of("SINCE");
    let wait_for = m.is_present("WAITONE");
    let depool = Some(depool);
    print_args!(m, depool, since);
    if !wait_for {
        let since = since.map(|s| {
                u32::from_str_radix(s, 10)
                    .map_err(|e| format!(r#"cannot parse "since" option: {}"#, e))
            })
            .transpose()?
            .unwrap_or(0);
        get_events(conf, depool.unwrap(), since).await
    } else {
        wait_for_event(conf, depool.unwrap()).await
    }
}

fn events_filter(addr: &str, since: u32) -> serde_json::Value {
    json!({ 
        "src": { "eq": addr },
        "msg_type": {"eq": 2 },
        "created_at": {"ge": since }
    })
}

fn print_event(ton: TonClient, event: &serde_json::Value) {
    println!("event {}", event["id"].as_str().unwrap());
    
    let body = event["body"].as_str().unwrap();
    let result = ton_client::abi::decode_message_body(
        ton.clone(),
        ParamsOfDecodeMessageBody {
            abi: load_abi(DEPOOL_ABI).unwrap(),
            body: body.to_owned(),
            is_internal: false,
        },
    );
    let (name, args) = if result.is_err() {
        ("unknown".to_owned(), "{}".to_owned())
    } else {
        let result = result.unwrap();
        (result.name, serde_json::to_string(&result.value).unwrap())
    };

    println!("{} {} ({})\n{}\n", 
        name,
        event["created_at"].as_u64().unwrap(),
        event["created_at_string"].as_str().unwrap(),
        args
    );
}

async fn get_events(conf: Config, depool: &str, since: u32) -> Result<(), String> {
    let ton = create_client_verbose(&conf)?;
    let _addr = load_ton_address(depool)?;

    let events = ton_client::net::query_collection(
        ton.clone(),
        ParamsOfQueryCollection {
            collection: "messages".to_owned(),
            filter: Some(events_filter(depool, since)),
            result: "id body created_at created_at_string".to_owned(),
            order: Some(vec![OrderBy{ path: "created_at".to_owned(), direction: SortDirection::DESC }]),
            limit: None,
        },
    ).await.map_err(|e| format!("failed to query depool events: {}", e))?;
    println!("{} events found", events.result.len());
    for event in &events.result {
        print_event(ton.clone(), event);
    }
    println!("Done");
    Ok(())
}

async fn wait_for_event(conf: Config, depool: &str) -> Result<(), String> {
    let ton = create_client_verbose(&conf)?;
    let _addr = load_ton_address(depool)?;
    println!("Waiting for a new event...");
    let event = ton_client::net::wait_for_collection(
        ton.clone(),
        ParamsOfWaitForCollection {
            collection: "messages".to_owned(),
            filter: Some(events_filter(depool, now())),
            result: "id body created_at created_at_string".to_owned(),
            timeout: None,
        },
        
    ).await.map_err(|e| format!("failed to query event: {}", e.to_string()))?;
    print_event(ton.clone(), &event.result);
    Ok(())
}
/*
 * Stake commands
 */

async fn ordinary_stake_command(
    m: &ArgMatches<'_>,
    cmd: CommandData<'_>,
) -> Result<(), String> {
    let (depool, wallet, stake, keys) = 
        (Some(&cmd.depool), Some(&cmd.wallet), Some(cmd.stake), Some(&cmd.keys));
    print_args!(m, depool, wallet, stake, keys);
    add_ordinary_stake(cmd).await
}

async fn replenish_command(
    m: &ArgMatches<'_>,
    cmd: CommandData<'_>,
) -> Result<(), String> {
    let (depool, wallet, stake, keys) = 
        (Some(&cmd.depool), Some(&cmd.wallet), Some(cmd.stake), Some(&cmd.keys));
    print_args!(m, depool, wallet, stake, keys);
    replenish_stake(cmd).await
}

async fn ticktock_command(
    m: &ArgMatches<'_>,
    conf: Config,
    depool: &str,
    wallet: &str,
    keys: &str,
) -> Result<(), String> {
    let (depool, wallet, keys) = (Some(depool), Some(wallet), Some(keys));
    print_args!(m, depool, wallet, keys);
    call_ticktock(conf, depool.unwrap(), wallet.unwrap(), keys.unwrap()).await
}

async fn transfer_stake_command(
    m: &ArgMatches<'_>,
    cmd: CommandData<'_>,
) -> Result<(), String> {
    let dest = Some(m.value_of("DEST")
        .ok_or("destination address is not defined.".to_string())?);
    let (depool, wallet, stake, keys) = 
        (Some(&cmd.depool), Some(&cmd.wallet), Some(cmd.stake), Some(&cmd.keys));
    print_args!(m, depool, wallet, stake, keys, dest);
    transfer_stake(cmd, dest.unwrap()).await
}

async fn set_donor_command(
    m: &ArgMatches<'_>,
    conf: Config,
    depool: &str,
    wallet: &str,
    keys: &str,
    is_vesting: bool,
) -> Result<(), String> {
    let (depool, wallet, keys) = (Some(depool), Some(wallet), Some(keys));
    let donor = Some(m.value_of("DONOR")
        .ok_or("donor is not defined.".to_string())?);
    print_args!(m, depool, wallet, keys, donor);
    set_donor(conf, depool.unwrap(), wallet.unwrap(), keys.unwrap(), is_vesting, donor.unwrap()).await
}

async fn exotic_stake_command(
    m: &ArgMatches<'_>,
    cmd: CommandData<'_>,
    is_vesting: bool,
) -> Result<(), String> {
    let withdrawal_period = Some(m.value_of("WPERIOD")
        .ok_or("withdrawal period is not defined.".to_string())?);
    let total_period = Some(m.value_of("TPERIOD")
        .ok_or("total period is not defined.".to_string())?);
    let beneficiary = Some(m.value_of("BENEFICIARY")
        .ok_or("beneficiary is not defined.".to_string())?);
    let (depool, wallet, stake, keys) = (Some(&cmd.depool), Some(&cmd.wallet), Some(cmd.stake), Some(&cmd.keys));
    print_args!(m, depool, wallet, stake, keys, beneficiary, withdrawal_period, total_period);
    let period_checker = |v| {
        if v > 0 && v <= 36500 { 
            Ok(v) 
        } else { 
            Err(format!("period cannot be more than 36500 days")) 
        }
    };
    let wperiod = u32::from_str_radix(withdrawal_period.unwrap(), 10)
        .map_err(|e| format!("invalid withdrawal period: {}", e))
        .and_then(period_checker)?;
    let tperiod = u32::from_str_radix(total_period.unwrap(), 10)
        .map_err(|e| format!("invalid total period: {}", e))
        .and_then(period_checker)?;
    let wperiod = wperiod * 86400;
    let tperiod = tperiod * 86400;
    add_exotic_stake(cmd, beneficiary.unwrap(), wperiod, tperiod, is_vesting).await
}

async fn remove_stake_command(
    m: &ArgMatches<'_>,
    cmd: CommandData<'_>,
) -> Result<(), String> {
    let (depool, wallet, stake, keys) = (Some(&cmd.depool), Some(&cmd.wallet), Some(cmd.stake), Some(&cmd.keys));
    print_args!(m, depool, wallet, stake, keys);
   remove_stake(cmd).await
}

async fn withdraw_stake_command(
    m: &ArgMatches<'_>,
    cmd: CommandData<'_>,
) -> Result<(), String> {
    let (depool, wallet, stake, keys) = (Some(&cmd.depool), Some(&cmd.wallet), Some(cmd.stake), Some(&cmd.keys));
    print_args!(m, depool, wallet, stake, keys);
   withdraw_stake(cmd).await
}

async fn set_withdraw_command(
    m: &ArgMatches<'_>,
    conf: Config,
    depool: &str,
    wallet: &str,
    keys: &str,
    enable: bool,
) -> Result<(), String> {
    let (depool, wallet, keys) = (Some(depool), Some(wallet), Some(keys));
    let withdraw = Some(if enable { "true" } else { "false" });
    print_args!(m, depool, wallet, keys, withdraw);
    set_withdraw(conf, depool.unwrap(), wallet.unwrap(), keys.unwrap(), enable).await
}

async fn add_ordinary_stake(cmd: CommandData<'_>) -> Result<(), String> {
    let stake = u64::from_str_radix(&convert::convert_token(cmd.stake)?, 10)
        .map_err(|e| format!(r#"failed to parse stake value: {}"#, e))?;
    let body = encode_add_ordinary_stake(stake).await?;
    let fee = u64::from_str_radix(&convert::convert_token(&cmd.depool_fee)?, 10)
        .map_err(|e| format!(r#"failed to parse depool fee value: {}"#, e))?;
    let value = (fee + stake) as f64 * 1.0 / 1e9;

    send_with_body(cmd.conf, &cmd.wallet, &cmd.depool, &format!("{}", value), &cmd.keys, &body).await
}

async fn replenish_stake(cmd: CommandData<'_>) -> Result<(), String> {
    let body = encode_replenish_stake().await?;
    send_with_body(cmd.conf, &cmd.wallet, &cmd.depool, cmd.stake, &cmd.keys, &body).await
}

async fn call_ticktock(
    conf: Config,
    depool: &str,
    wallet: &str,
    keys: &str,
) -> Result<(), String> {
    let body = encode_ticktock().await?;
    send_with_body(conf, wallet, depool, "1", keys, &body).await
}

async fn add_exotic_stake(
    cmd: CommandData<'_>,
    beneficiary: &str,
    wp: u32,
    tp: u32,
    is_vesting: bool,
) -> Result<(), String> {
    load_ton_address(beneficiary)?;
    let stake = u64::from_str_radix(&convert::convert_token(cmd.stake)?, 10)
        .map_err(|e| format!(r#"failed to parse stake value: {}"#, e))?;
    let body = if is_vesting {
        encode_add_vesting_stake(stake, beneficiary, tp, wp).await?
    } else {
        encode_add_lock_stake(stake, beneficiary, tp, wp).await?
    };
    let fee = u64::from_str_radix(&convert::convert_token(&cmd.depool_fee)?, 10)
        .map_err(|e| format!(r#"failed to parse depool fee value: {}"#, e))?;
    let value = (fee + stake) as f64 * 1.0 / 1e9;
    send_with_body(cmd.conf, &cmd.wallet, &cmd.depool, &format!("{}", value), &cmd.keys, &body).await
}

async fn remove_stake(
    cmd: CommandData<'_>,
) -> Result<(), String> {
    let stake = u64::from_str_radix(
        &convert::convert_token(cmd.stake)?, 10,
    ).unwrap();
    let body = encode_remove_stake(stake).await?;
    send_with_body(cmd.conf, &cmd.wallet, &cmd.depool, &cmd.depool_fee, &cmd.keys, &body).await
}

async fn withdraw_stake(
    cmd: CommandData<'_>,
) -> Result<(), String> {
    let stake = u64::from_str_radix(
        &convert::convert_token(cmd.stake)?, 10,
    ).unwrap();
    let body = encode_withdraw_stake(stake).await?;
    send_with_body(cmd.conf, &cmd.wallet, &cmd.depool, &cmd.depool_fee, &cmd.keys, &body).await
}

async fn transfer_stake(cmd: CommandData<'_>, dest: &str) -> Result<(), String> {
    load_ton_address(dest)?;
    let stake = u64::from_str_radix(
        &convert::convert_token(cmd.stake)?, 10,
    ).unwrap();
    let body = encode_transfer_stake(dest, stake).await?;
    send_with_body(cmd.conf, &cmd.wallet, &cmd.depool, &cmd.depool_fee, &cmd.keys, &body).await
}

async fn set_withdraw(
    conf: Config,
    depool: &str,
    wallet: &str,
    keys: &str,
    enable: bool,
) -> Result<(), String> {
    let body = encode_set_withdraw(enable).await?;
    let value = conf.depool_fee.to_string();
    send_with_body(conf, wallet, depool, &value, keys, &body).await
}

async fn set_donor(
    conf: Config,
    depool: &str,
    wallet: &str,
    keys: &str,
    is_vesting: bool,
    donor: &str,
) -> Result<(), String> {
    let body = encode_set_donor(is_vesting, donor).await?;
    let value = conf.depool_fee.to_string();
    send_with_body(conf, wallet, depool, &value, keys, &body).await
}

async fn encode_body(func: &str, params: serde_json::Value) -> Result<String, String> {
    let client = create_client_local()?;
    ton_client::abi::encode_message_body(
        client.clone(),
        ParamsOfEncodeMessageBody {
            abi: load_abi(DEPOOL_ABI)?,
            call_set: CallSet::some_with_function_and_input(func, params).unwrap(),
            is_internal: true,
            signer: Signer::None,
            processing_try_index: None,
        },
    ).await
    .map_err(|e| format!("failed to encode body: {}", e))
    .map(|r| r.body)
}

async fn encode_set_withdraw(flag: bool) -> Result<String, String> {
    if flag {
        encode_body("withdrawAll", json!({}))
    } else {
        encode_body("cancelWithdrawal", json!({}))
    }.await
}

async fn encode_add_ordinary_stake(stake: u64) -> Result<String, String> {
	encode_body("addOrdinaryStake", json!({
        "stake": stake
    })).await
}

async fn encode_replenish_stake() -> Result<String, String> {
	encode_body("receiveFunds", json!({})).await
}

async fn encode_ticktock() -> Result<String, String> {
	encode_body("ticktock", json!({})).await
}

async fn encode_add_vesting_stake(
    stake: u64,
    beneficiary: &str,
    tperiod: u32,
    wperiod: u32,
) -> Result<String, String> {
	encode_body("addVestingStake", json!({
        "stake": stake,
        "beneficiary": beneficiary,
        "withdrawalPeriod": wperiod,
        "totalPeriod": tperiod
    })).await
}

async fn encode_set_donor(is_vesting: bool, donor: &str) -> Result<String, String> {
    if is_vesting {
        encode_body("setVestingDonor", json!({
            "donor": donor
        }))
    } else {
        encode_body("setLockDonor", json!({
            "donor": donor
        }))
    }.await
}

async fn encode_add_lock_stake(
    stake: u64,
    beneficiary: &str,
    tperiod: u32,
    wperiod: u32,
) -> Result<String, String> {
	encode_body("addLockStake", json!({
        "stake": stake,
        "beneficiary": beneficiary,
        "withdrawalPeriod": wperiod,
        "totalPeriod": tperiod
    })).await
}

async fn encode_remove_stake(target_value: u64) -> Result<String, String> {
	encode_body("withdrawFromPoolingRound", json!({
        "withdrawValue": target_value
    })).await
}

async fn encode_withdraw_stake(target_value: u64) -> Result<String, String> {
	encode_body("withdrawPart", json!({
        "withdrawValue": target_value
    })).await
}

async fn encode_transfer_stake(dest: &str, amount: u64) -> Result<String, String> {
	encode_body("transferStake", json!({
        "dest": dest,
        "amount": amount
    })).await
}
