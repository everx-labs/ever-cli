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
use crate::call::create_client_verbose;
use crate::config::Config;
use crate::convert;
use crate::crypto::{SdkClient};
use crate::depool_abi::DEPOOL_ABI;
use crate::helpers::{load_ton_address, now};
use crate::multisig::send_with_body;
use clap::{App, ArgMatches, SubCommand, Arg, AppSettings};
use serde_json;
use ton_client_rs::{OrderBy, SortDirection, TonClient};

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
        .help("Stake value.");
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
    let reinvest_arg = Arg::with_name("AUTORESUME")
        .long("--autoresume-off")
        .help("Disables autoresume flag for participant. In this case stake will be invested only to one round.");
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
        .subcommand(SubCommand::with_name("stake")
            .about(r#"Top level command for managing stakes in depool. Uses a supplied multisignature wallet to send internal message with stake to depool."#)
            .subcommand(SubCommand::with_name("ordinary")
                .about("Deposits ordinary stake in depool from multisignature wallet.")
                .setting(AppSettings::AllowLeadingHyphen)
                .arg(wallet_arg.clone())
                .arg(value_arg.clone())
                .arg(keys_arg.clone())
                .arg(reinvest_arg))
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
        .subcommand(SubCommand::with_name("withdraw")
            .about("Allows to disable auto investment of the stake into next round and withdraw all the stakes after round completion.")
            .setting(AppSettings::AllowLeadingHyphen)
            .subcommand(SubCommand::with_name("on")
                .arg(wallet_arg.clone())
                .arg(keys_arg.clone()))
            .subcommand(SubCommand::with_name("off")
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
}

impl<'a> CommandData<'a> {
    pub fn from_matches_and_conf(m: &'a ArgMatches, conf: Config, depool: String) -> Result<Self, String> {
        let (wallet, stake, keys) = parse_stake_data(m, &conf)?;
        Ok(CommandData {conf, depool, wallet, stake, keys})
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

pub fn depool_command(m: &ArgMatches, conf: Config) -> Result<(), String> {
    let depool = m.value_of("ADDRESS")
        .map(|s| s.to_string())
        .or(conf.addr.clone())
        .ok_or("depool address is not defined. Supply it in config file or in command line.".to_string())?;
    load_ton_address(&depool)
        .map_err(|e| format!("invalid depool address: {}", e))?;

    if let Some(m) = m.subcommand_matches("stake") {
        if let Some(m) = m.subcommand_matches("ordinary") {
            return ordinary_stake_command(m,
                CommandData::from_matches_and_conf(m, conf, depool)?,
            );
        }
        if let Some(m) = m.subcommand_matches("vesting") {
            return exotic_stake_command(m,
                CommandData::from_matches_and_conf(m, conf, depool)?,
                true
            );
        }
        if let Some(m) = m.subcommand_matches("lock") {
            return exotic_stake_command(m,
                CommandData::from_matches_and_conf(m, conf, depool)?,
                false
            );
        }
        if let Some(m) = m.subcommand_matches("remove") {
            return remove_stake_command(m,
                CommandData::from_matches_and_conf(m, conf, depool)?,
            );
        }
        if let Some(m) = m.subcommand_matches("withdrawPart") {
            return withdraw_stake_command(m,
                CommandData::from_matches_and_conf(m, conf, depool)?,
            );
        }
        if let Some(m) = m.subcommand_matches("transfer") {
            return transfer_stake_command(m,
                CommandData::from_matches_and_conf(m, conf, depool)?,
            );
        }
    }
    if let Some(m) = m.subcommand_matches("withdraw") {
        let matches = m.subcommand_matches("on").or(m.subcommand_matches("off"));
        if let Some(matches) = matches {
            let (wallet, keys) = parse_wallet_data(&matches, &conf)?;
            let enable_withdraw = m.subcommand_matches("on").is_some();
            return set_withdraw_command(matches, conf, &depool, &wallet, &keys, enable_withdraw);
        }
    }
    if let Some(m) = m.subcommand_matches("events") {
        return events_command(m, conf, &depool)
    }
    if let Some(m) = m.subcommand_matches("replenish") {
        return replenish_command(m,
            CommandData::from_matches_and_conf(m, conf, depool)?,
        );
    }
    Err("unknown depool command".to_owned())
}

/*
 * Events command
 */

fn events_command(m: &ArgMatches, conf: Config, depool: &str) -> Result<(), String> {
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
        get_events(conf, depool.unwrap(), since)
    } else {
        wait_for_event(conf, depool.unwrap())
    }
}

fn events_filter(addr: &str, since: u32) -> serde_json::Value {
    json!({ 
        "src": { "eq": addr },
        "msg_type": {"eq": 2 },
        "created_at": {"ge": since }
    })
}

fn print_event(ton: &TonClient, event: &serde_json::Value) {
    println!("event {}", event["id"].as_str().unwrap());
    
    let body = base64::decode(event["body"].as_str().unwrap()).unwrap();
    let result = ton.contracts.decode_output_message_body(
        DEPOOL_ABI.into(),
        &body[..],
    );
    let (name, args) = if result.is_err() {
        ("unknown".to_owned(), "{}".to_owned())
    } else {
        let result = result.unwrap();
        (result.function, serde_json::to_string(&result.output).unwrap())
    };

    println!("{} {} ({})\n{}\n", 
        name,
        event["created_at"].as_u64().unwrap(),
        event["created_at_string"].as_str().unwrap(),
        args
    );
}

fn get_events(conf: Config, depool: &str, since: u32) -> Result<(), String> {
    let ton = create_client_verbose(&conf)?;
    let _addr = load_ton_address(depool)?;

    let events = ton.queries.messages.query(
        events_filter(depool, since).into(),
        "id body created_at created_at_string",
        Some(OrderBy{ path: "created_at".to_owned(), direction: SortDirection::Descending }),
        None,
    ).map_err(|e| format!("failed to query depool events: {}", e.to_string()))?;
    println!("{} events found", events.len());
    for event in &events {
        print_event(&ton, event);
    }
    println!("Done");
    Ok(())
}

fn wait_for_event(conf: Config, depool: &str) -> Result<(), String> {
    let ton = create_client_verbose(&conf)?;
    let _addr = load_ton_address(depool)?;
    println!("Waiting for a new event...");
    let event = ton.queries.messages.wait_for(
        events_filter(depool, now()).into(),
        "id body created_at created_at_string",
    ).map_err(|e| format!("failed to query event: {}", e.to_string()))?;
    print_event(&ton, &event);
    Ok(())
}
/*
 * Stake commands
 */

fn ordinary_stake_command<'a>(
    m: &ArgMatches,
    cmd: CommandData
) -> Result<(), String> {
    let disable_reinvest = m.is_present("AUTORESUME");
    let autoresume = Some(if disable_reinvest { "false" } else { "true" });
    let (depool, wallet, stake, keys) = (Some(&cmd.depool), Some(&cmd.wallet), Some(cmd.stake), Some(&cmd.keys));
    print_args!(m, depool, wallet, stake, keys, autoresume);
    add_ordinary_stake(cmd, !disable_reinvest)
}

fn replenish_command<'a>(
    m: &ArgMatches,
    cmd: CommandData
) -> Result<(), String> {
    let (depool, wallet, stake, keys) = (Some(&cmd.depool), Some(&cmd.wallet), Some(cmd.stake), Some(&cmd.keys));
    print_args!(m, depool, wallet, stake, keys);
    replenish_stake(cmd)
}

fn transfer_stake_command<'a>(
    m: &ArgMatches,
    cmd: CommandData
) -> Result<(), String> {
    let dest = Some(m.value_of("DEST")
        .ok_or("destination address is not defined.".to_string())?);
    let (depool, wallet, stake, keys) = (Some(&cmd.depool), Some(&cmd.wallet), Some(cmd.stake), Some(&cmd.keys));
    print_args!(m, depool, wallet, stake, keys, dest);
    transfer_stake(cmd, dest.unwrap())
}

fn exotic_stake_command(
    m: &ArgMatches,
    cmd: CommandData,
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
    add_exotic_stake(cmd, beneficiary.unwrap(), wperiod, tperiod, is_vesting)
}

fn remove_stake_command(
    m: &ArgMatches,
    cmd: CommandData,
) -> Result<(), String> {
    let (depool, wallet, stake, keys) = (Some(&cmd.depool), Some(&cmd.wallet), Some(cmd.stake), Some(&cmd.keys));
    print_args!(m, depool, wallet, stake, keys);
   remove_stake(cmd)
}

fn withdraw_stake_command(
    m: &ArgMatches,
    cmd: CommandData,
) -> Result<(), String> {
    let (depool, wallet, stake, keys) = (Some(&cmd.depool), Some(&cmd.wallet), Some(cmd.stake), Some(&cmd.keys));
    print_args!(m, depool, wallet, stake, keys);
   withdraw_stake(cmd)
}

fn set_withdraw_command(
    m: &ArgMatches,
    conf: Config,
    depool: &str,
    wallet: &str,
    keys: &str,
    enable: bool,
) -> Result<(), String> {
    let (depool, wallet, keys) = (Some(depool), Some(wallet), Some(keys));
    let withdraw = Some(if enable { "true" } else { "false" });
    print_args!(m, depool, wallet, keys, withdraw);
    set_withdraw(conf, depool.unwrap(), wallet.unwrap(), keys.unwrap(), enable)
}

fn add_ordinary_stake(
    cmd: CommandData,
    autoresume: bool,
) -> Result<(), String> {
    let body = encode_add_ordinary_stake(autoresume)?;
    send_with_body(cmd.conf, &cmd.wallet, &cmd.depool, cmd.stake, &cmd.keys, &body)
}

fn replenish_stake(
    cmd: CommandData
) -> Result<(), String> {
    let body = encode_replenish_stake()?;
    send_with_body(cmd.conf, &cmd.wallet, &cmd.depool, cmd.stake, &cmd.keys, &body)
}

fn add_exotic_stake(
    cmd: CommandData,
    beneficiary: &str,
    wp: u32,
    tp: u32,
    is_vesting: bool,
) -> Result<(), String> {
    load_ton_address(beneficiary)?;
    let body = if is_vesting {
        encode_add_vesting_stake(beneficiary, tp, wp)?
    } else {
        encode_add_lock_stake(beneficiary, tp, wp)?
    };
    send_with_body(cmd.conf, &cmd.wallet, &cmd.depool, cmd.stake, &cmd.keys, &body)
}

fn remove_stake(
    cmd: CommandData,
) -> Result<(), String> {
    let stake = u64::from_str_radix(
        &convert::convert_token(cmd.stake)?, 10,
    ).unwrap();
    let body = encode_remove_stake(stake)?;
    send_with_body(cmd.conf, &cmd.wallet, &cmd.depool, "0.05", &cmd.keys, &body)
}

fn withdraw_stake(
    cmd: CommandData,
) -> Result<(), String> {
    let stake = u64::from_str_radix(
        &convert::convert_token(cmd.stake)?, 10,
    ).unwrap();
    let body = encode_withdraw_stake(stake)?;
    send_with_body(cmd.conf, &cmd.wallet, &cmd.depool, "0.05", &cmd.keys, &body)
}

fn transfer_stake(cmd: CommandData, dest: &str) -> Result<(), String> {
    load_ton_address(dest)?;
    let stake = u64::from_str_radix(
        &convert::convert_token(cmd.stake)?, 10,
    ).unwrap();
    let body = encode_transfer_stake(dest, stake)?;
    send_with_body(cmd.conf, &cmd.wallet, &cmd.depool, "0.1", &cmd.keys, &body)
}

fn set_withdraw(
    conf: Config,
    depool: &str,
    wallet: &str,
    keys: &str,
    enable: bool,
) -> Result<(), String> {
    let body = encode_set_withdraw(enable)?;
    send_with_body(conf, wallet, depool, "0.03", keys, &body)
}

fn encode_body(func: &str, params: serde_json::Value) -> Result<String, String> {
    let client = SdkClient::new();
	let abi: serde_json::Value = serde_json::from_str(DEPOOL_ABI).unwrap();
    let result = client.request(
        "contracts.run.body",
        json!({
            "abi": abi,
            "function": func,
            "params": params,
			"internal": true,
        })
    )?;
    let json_body: serde_json::Value = serde_json::from_str(&result)
        .map_err(|e| format!("failed to encode msg body: {}", e))?;
    json_body.get("bodyBase64")
        .ok_or(format!(r#"internal error: "bodyBase64" not found in sdk call result"#))?
        .as_str()
        .ok_or(format!(r#"internal error: "bodyBase64" field is not a string"#))
        .map(|s| s.to_owned())
}

fn encode_set_withdraw(flag: bool) -> Result<String, String> {
    encode_body("withdrawAllAfterCompleting", json!({
        "doWithdrawAll": flag
    }))
}

fn encode_add_ordinary_stake(reinvest: bool) -> Result<String, String> {
	encode_body("addOrdinaryStake", json!({
        "reinvest": reinvest
    }))
}

fn encode_replenish_stake() -> Result<String, String> {
	encode_body("receiveFunds", json!({
    }))
}

fn encode_add_vesting_stake(beneficiary: &str, tperiod: u32, wperiod: u32) -> Result<String, String> {
	encode_body("addVestingStake", json!({
        "beneficiary": beneficiary,
        "withdrawalPeriod": wperiod,
        "totalPeriod": tperiod
    }))
}

fn encode_add_lock_stake(beneficiary: &str, tperiod: u32, wperiod: u32) -> Result<String, String> {
	encode_body("addLockStake", json!({
        "beneficiary": beneficiary,
        "withdrawalPeriod": wperiod,
        "totalPeriod": tperiod
    }))
}

fn encode_remove_stake(target_value: u64) -> Result<String, String> {
	encode_body("removeOrdinaryStake", json!({
        "withdrawValue": target_value
    }))
}

fn encode_withdraw_stake(target_value: u64) -> Result<String, String> {
	encode_body("withdrawPartAfterCompleting", json!({
        "withdrawValue": target_value
    }))
}

fn encode_transfer_stake(dest: &str, amount: u64) -> Result<String, String> {
	encode_body("transferStake", json!({
        "dest": dest,
        "amount": amount
    }))
}
