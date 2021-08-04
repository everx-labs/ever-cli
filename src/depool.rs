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
use crate::depool_abi::{DEPOOL_ABI, PARTICIPANT_ABI};
use crate::helpers::
    {
    create_client_local,
    create_client_verbose,
    load_abi,
    load_ton_address,
    now,
    TonClient,
    answer_filter,
    events_filter,
    print_message,
    };
use crate::multisig::{send_with_body, MSIG_ABI};
use clap::{App, ArgMatches, SubCommand, Arg, AppSettings};
use serde_json;
use ton_client::abi::{ParamsOfEncodeMessageBody, CallSet, ParamsOfDecodeMessageBody};
use ton_client::net::{OrderBy, ParamsOfQueryCollection, ParamsOfWaitForCollection, SortDirection};
use crate::call::{prepare_message, print_encoded_message};
use ton_client::processing::{
    ParamsOfSendMessage,
    ParamsOfWaitForTransaction,
    wait_for_transaction,
    send_message,
};
use std::collections::HashMap;

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
        .help("Seed phrase or path to file with keypair which must be used to sign message to multisig wallet.");
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
        .help("Address of the destination smart contract.");
    let wait_answer = Arg::with_name("WAIT_ANSWER")
        .long("--wait-answer")
        .short("-a")
        .help("Wait for depool answer when calling a depool function.");
    SubCommand::with_name("depool")
        .about("DePool commands.")
        .setting(AppSettings::AllowLeadingHyphen)
        .setting(AppSettings::DontCollapseArgsInUsage)
        .arg(Arg::with_name("ADDRESS")
            .takes_value(true)
            .long("--addr")
            .help("DePool contract address. If parameter is omitted, then value `addr` from the config is used"))
        .arg(wait_answer.clone())
        .subcommand(SubCommand::with_name("donor")
            .about(r#"Top level command for specifying donor for exotic stakes in depool."#)
            .subcommand(SubCommand::with_name("vesting")
                .about("Set the address from which participant can receive a vesting stake.")
                .setting(AppSettings::AllowLeadingHyphen)
                .arg(wallet_arg.clone())
                .arg(keys_arg.clone())
                .arg(donor_arg.clone())
                .arg(wait_answer.clone()))
            .subcommand(SubCommand::with_name("lock")
                .about("Set the address from which participant can receive a lock stake.")
                .setting(AppSettings::AllowLeadingHyphen)
                .arg(wallet_arg.clone())
                .arg(keys_arg.clone())
                .arg(donor_arg.clone())
                .arg(wait_answer.clone())))
        .subcommand(SubCommand::with_name("answers")
            .about("Prints depool answers to the wallet")
            .setting(AppSettings::AllowLeadingHyphen)
            .arg(wallet_arg.clone())
            .arg(Arg::with_name("SINCE")
                .takes_value(true)
                .long("--since")
                .short("-s")
                .help("Prints answers since this unixtime.")) )
        .subcommand(SubCommand::with_name("stake")
            .about(r#"Top level command for managing stakes in depool. Uses a supplied multisignature wallet to send internal message with stake to depool."#)
            .subcommand(SubCommand::with_name("ordinary")
                .about("Deposits an ordinary stake in the depool from the multisignature wallet.")
                .setting(AppSettings::AllowLeadingHyphen)
                .arg(wallet_arg.clone())
                .arg(value_arg.clone())
                .arg(keys_arg.clone())
                .arg(wait_answer.clone()))
            .subcommand(SubCommand::with_name("vesting")
                .about("Deposits a vesting stake in the depool from the multisignature wallet.")
                .setting(AppSettings::AllowLeadingHyphen)
                .arg(wallet_arg.clone())
                .arg(value_arg.clone())
                .arg(keys_arg.clone())
                .arg(wait_answer.clone())
                .arg(total_period_arg.clone())
                .arg(withdrawal_period_arg.clone())
                .arg(beneficiary_arg.clone()))
            .subcommand(SubCommand::with_name("lock")
                .about("Deposits a lock stake in the depool from the multisignature wallet.")
                .setting(AppSettings::AllowLeadingHyphen)
                .arg(wallet_arg.clone())
                .arg(value_arg.clone())
                .arg(keys_arg.clone())
                .arg(wait_answer.clone())
                .arg(total_period_arg.clone())
                .arg(withdrawal_period_arg.clone())
                .arg(beneficiary_arg.clone()))
            .subcommand(SubCommand::with_name("transfer")
                .about("Transfers ownership of the wallet stake to another contract.")
                .setting(AppSettings::AllowLeadingHyphen)
                .arg(wallet_arg.clone())
                .arg(value_arg.clone())
                .arg(keys_arg.clone())
                .arg(wait_answer.clone())
                .arg(dest_arg.clone()))
            .subcommand(SubCommand::with_name("remove")
                .about("Withdraws an ordinary stake from the current pooling round of the depool to the multisignature wallet.")
                .setting(AppSettings::AllowLeadingHyphen)
                .arg(wallet_arg.clone())
                .arg(value_arg.clone())
                .arg(keys_arg.clone())
                .arg(wait_answer.clone()))
            .subcommand(SubCommand::with_name("withdrawPart")
                .about("Withdraws part of the stake after round completion.")
                .setting(AppSettings::AllowLeadingHyphen)
                .arg(wallet_arg.clone())
                .arg(value_arg.clone())
                .arg(wait_answer.clone())
                .arg(keys_arg.clone())))
        .subcommand(SubCommand::with_name("replenish")
            .about("Transfers funds from the multisignature wallet to the depool contract (NOT A STAKE).")
            .setting(AppSettings::AllowLeadingHyphen)
            .arg(wallet_arg.clone())
            .arg(value_arg.clone())
            .arg(keys_arg.clone()))
        .subcommand(SubCommand::with_name("ticktock")
            .about("Calls depool 'ticktock()' function to update its state. 1 ton is attached to this call (change will be returned).")
            .setting(AppSettings::AllowLeadingHyphen)
            .arg(wallet_arg.clone())
            .arg(keys_arg.clone()))
        .subcommand(SubCommand::with_name("withdraw")
            .about("Allows to disable auto investment of the stake into the next round and withdraw all the stakes after round completion.")
            .setting(AppSettings::AllowLeadingHyphen)
            .subcommand(SubCommand::with_name("on")
                .setting(AppSettings::AllowLeadingHyphen)
                .arg(wallet_arg.clone())
                .arg(wait_answer.clone())
                .arg(keys_arg.clone()))
            .subcommand(SubCommand::with_name("off")
                .setting(AppSettings::AllowLeadingHyphen)
                .arg(wallet_arg.clone())
                .arg(wait_answer.clone())
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
    let wallet = load_ton_address(&wallet, conf)
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
        .ok_or("depool address is not defined. Supply it in the config file or in command line.".to_string())?;
    let depool = load_ton_address(&depool, &conf)
        .map_err(|e| format!("invalid depool address: {}", e))?;

    let mut conf = conf;
    let mut set_wait_answer = |m: &ArgMatches|  {
        if m.is_present("WAIT_ANSWER") {
            conf.no_answer = false;
        }
    };
    set_wait_answer(m);
    if let Some(m) = m.subcommand_matches("donor") {
        let matches = m.subcommand_matches("vesting").or(m.subcommand_matches("lock"));
        if let Some(matches) = matches {
            let is_vesting = m.subcommand_matches("vesting").is_some();
            set_wait_answer(matches);
            let (wallet, keys) = parse_wallet_data(&matches, &conf)?;
            return set_donor_command(matches, conf, depool.as_str(), &wallet, &keys, is_vesting).await;
        }
    }

    if let Some(m) = m.subcommand_matches("stake") {
        if let Some(m) = m.subcommand_matches("ordinary") {
            set_wait_answer(m);
            return ordinary_stake_command(
                CommandData::from_matches_and_conf(m, conf, depool)?,
            ).await;
        }
        if let Some(m) = m.subcommand_matches("vesting") {
            set_wait_answer(m);
            return exotic_stake_command(m,
                CommandData::from_matches_and_conf(m, conf, depool)?,
                true,
            ).await;
        }
        if let Some(m) = m.subcommand_matches("lock") {
            set_wait_answer(m);
            return exotic_stake_command(m,
                CommandData::from_matches_and_conf(m, conf, depool)?,
                false,
            ).await;
        }
        if let Some(m) = m.subcommand_matches("remove") {
            set_wait_answer(m);
            return remove_stake_command(
                CommandData::from_matches_and_conf(m, conf, depool)?,
            ).await;
        }
        if let Some(m) = m.subcommand_matches("withdrawPart") {
            set_wait_answer(m);
            return withdraw_stake_command(
                CommandData::from_matches_and_conf(m, conf, depool)?,
            ).await;
        }
        if let Some(m) = m.subcommand_matches("transfer") {
            set_wait_answer(m);
            return transfer_stake_command(m,
                CommandData::from_matches_and_conf(m, conf, depool)?,
            ).await;
        }
    }
    if let Some(m) = m.subcommand_matches("withdraw") {
        let matches = m.subcommand_matches("on").or(m.subcommand_matches("off"));
        if let Some(matches) = matches {
            set_wait_answer(matches);
            let (wallet, keys) = parse_wallet_data(&matches, &conf)?;
            let enable_withdraw = m.subcommand_matches("on").is_some();
            return set_withdraw_command(conf, &depool, &wallet, &keys, enable_withdraw).await;
        }
    }
    if let Some(m) = m.subcommand_matches("events") {
        return events_command(m, conf, &depool).await
    }
    if let Some(m) = m.subcommand_matches("answers") {
        return answer_command(m, conf, &depool).await
    }
    if let Some(m) = m.subcommand_matches("replenish") {
        return replenish_command(
            CommandData::from_matches_and_conf(m, conf, depool)?,
        ).await;
    }
    if let Some(m) = m.subcommand_matches("ticktock") {
        let (wallet, keys) = parse_wallet_data(&m, &conf)?;
        return ticktock_command(conf, &depool, &wallet, &keys).await;
    }
    Err("unknown depool command".to_owned())
}

async fn answer_command(m: &ArgMatches<'_>, conf: Config, depool: &str) -> Result<(), String> {
    let wallet = m.value_of("MSIG")
        .map(|s| s.to_string())
        .or(conf.wallet.clone())
        .ok_or("multisig wallet address is not defined.".to_string())?;
    let since = m.value_of("SINCE")
            .map(|s| {
                u32::from_str_radix(s, 10)
                .map_err(|e| format!(r#"cannot parse "since" option: {}"#, e))
            })
            .transpose()?
            .unwrap_or(0);

    let ton = create_client_verbose(&conf)?;
    let wallet = load_ton_address(&wallet, &conf)
        .map_err(|e| format!("invalid depool address: {}", e))?;

    let messages = ton_client::net::query_collection(
        ton.clone(),
        ParamsOfQueryCollection {
            collection: "messages".to_owned(),
            filter: Some(answer_filter(depool, &wallet, since)),
            result: "id value body created_at created_at_string".to_owned(),
            order: Some(vec![OrderBy{ path: "created_at".to_owned(), direction: SortDirection::DESC }]),
            ..Default::default()
        },
    ).await.map_err(|e| format!("failed to query depool messages: {}", e))?;
    println!("{} answers found", messages.result.len());
    for messages in &messages.result {
        print_answer(ton.clone(), messages).await;
    }
    println!("Done");
    Ok(())
}

async fn print_answer(ton: TonClient, message: &serde_json::Value) {
    println!("Answer:");
    print_message(ton, message, PARTICIPANT_ABI, true).await;
}

/*
 * Events command
 */

async fn events_command(m: &ArgMatches<'_>, conf: Config, depool: &str) -> Result<(), String> {
    let since = m.value_of("SINCE");
    let wait_for = m.is_present("WAITONE");
    let depool = Some(depool);
    print_args!(depool, since);
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

async fn print_event(ton: TonClient, event: &serde_json::Value) {
    println!("event {}", event["id"].as_str().unwrap());

    let body = event["body"].as_str().unwrap();
    let result = ton_client::abi::decode_message_body(
        ton.clone(),
        ParamsOfDecodeMessageBody {
            abi: load_abi(DEPOOL_ABI).unwrap(),
            body: body.to_owned(),
            is_internal: false,
            ..Default::default()
        },
    ).await;
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
    let _addr = load_ton_address(depool, &conf)?;

    let events = ton_client::net::query_collection(
        ton.clone(),
        ParamsOfQueryCollection {
            collection: "messages".to_owned(),
            filter: Some(events_filter(depool, since)),
            result: "id body created_at created_at_string".to_owned(),
            order: Some(vec![OrderBy{ path: "created_at".to_owned(), direction: SortDirection::DESC }]),
            ..Default::default()
        },
    ).await.map_err(|e| format!("failed to query depool events: {}", e))?;
    println!("{} events found", events.result.len());
    for event in &events.result {
        print_event(ton.clone(), event).await;
    }
    println!("Done");
    Ok(())
}

async fn wait_for_event(conf: Config, depool: &str) -> Result<(), String> {
    let ton = create_client_verbose(&conf)?;
    let _addr = load_ton_address(depool, &conf)?;
    println!("Waiting for a new event...");
    let event = ton_client::net::wait_for_collection(
        ton.clone(),
        ParamsOfWaitForCollection {
            collection: "messages".to_owned(),
            filter: Some(events_filter(depool, now())),
            result: "id body created_at created_at_string".to_owned(),
            timeout: Some(conf.timeout),
            ..Default::default()
        },

    ).await.map_err(|e| println!("failed to query event: {}", e.to_string()));
    if event.is_ok() {
        print_event(ton.clone(), &event.unwrap().result).await;
    }
    Ok(())
}
/*
 * Stake commands
 */

async fn ordinary_stake_command(
    cmd: CommandData<'_>,
) -> Result<(), String> {
    let (depool, wallet, stake, keys) =
        (Some(&cmd.depool), Some(&cmd.wallet), Some(cmd.stake), Some(&cmd.keys));
    print_args!(depool, wallet, stake, keys);
    add_ordinary_stake(cmd).await
}

async fn replenish_command(
    cmd: CommandData<'_>,
) -> Result<(), String> {
    let (depool, wallet, stake, keys) =
        (Some(&cmd.depool), Some(&cmd.wallet), Some(cmd.stake), Some(&cmd.keys));
    print_args!(depool, wallet, stake, keys);
    replenish_stake(cmd).await
}

async fn ticktock_command(
    conf: Config,
    depool: &str,
    wallet: &str,
    keys: &str,
) -> Result<(), String> {
    let (depool, wallet, keys) = (Some(depool), Some(wallet), Some(keys));
    print_args!(depool, wallet, keys);
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
    print_args!(depool, wallet, stake, keys, dest);
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
    print_args!(depool, wallet, keys, donor);
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
    print_args!(depool, wallet, stake, keys, beneficiary, withdrawal_period, total_period);
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
    cmd: CommandData<'_>,
) -> Result<(), String> {
    let (depool, wallet, stake, keys) = (Some(&cmd.depool), Some(&cmd.wallet), Some(cmd.stake), Some(&cmd.keys));
    print_args!(depool, wallet, stake, keys);
   remove_stake(cmd).await
}

async fn withdraw_stake_command(
    cmd: CommandData<'_>,
) -> Result<(), String> {
    let (depool, wallet, stake, keys) = (Some(&cmd.depool), Some(&cmd.wallet), Some(cmd.stake), Some(&cmd.keys));
    print_args!(depool, wallet, stake, keys);
   withdraw_stake(cmd).await
}

async fn set_withdraw_command(
    conf: Config,
    depool: &str,
    wallet: &str,
    keys: &str,
    enable: bool,
) -> Result<(), String> {
    let (depool, wallet, keys) = (Some(depool), Some(wallet), Some(keys));
    let withdraw = Some(if enable { "true" } else { "false" });
    print_args!(depool, wallet, keys, withdraw);
    set_withdraw(conf, depool.unwrap(), wallet.unwrap(), keys.unwrap(), enable).await
}

async fn add_ordinary_stake(cmd: CommandData<'_>) -> Result<(), String> {
    let stake = u64::from_str_radix(&convert::convert_token(cmd.stake)?, 10)
        .map_err(|e| format!(r#"failed to parse stake value: {}"#, e))?;
    let body = encode_add_ordinary_stake(stake).await?;
    let fee = u64::from_str_radix(&convert::convert_token(&cmd.depool_fee)?, 10)
        .map_err(|e| format!(r#"failed to parse depool fee value: {}"#, e))?;
    let value = (fee + stake) as f64 * 1.0 / 1e9;
    call_contract(cmd.conf.clone(), &cmd.wallet, &cmd.depool, &format!("{}", value), &cmd.keys, &body, true).await
}

async fn replenish_stake(cmd: CommandData<'_>) -> Result<(), String> {
    let body = encode_replenish_stake().await?;
    call_contract(cmd.conf.clone(), &cmd.wallet, &cmd.depool, cmd.stake, &cmd.keys, &body, false).await
}

async fn call_ticktock(
    conf: Config,
    depool: &str,
    wallet: &str,
    keys: &str,
) -> Result<(), String> {
    let body = encode_ticktock().await?;
    call_contract(conf.clone(), wallet, depool, "1", keys, &body, false).await
}

async fn add_exotic_stake(
    cmd: CommandData<'_>,
    beneficiary: &str,
    wp: u32,
    tp: u32,
    is_vesting: bool,
) -> Result<(), String> {
    let beneficiary = load_ton_address(beneficiary, &cmd.conf)?;
    let stake = u64::from_str_radix(&convert::convert_token(cmd.stake)?, 10)
        .map_err(|e| format!(r#"failed to parse stake value: {}"#, e))?;
    let body = if is_vesting {
        encode_add_vesting_stake(stake, beneficiary.as_str(), tp, wp).await?
    } else {
        encode_add_lock_stake(stake, beneficiary.as_str(), tp, wp).await?
    };
    let fee = u64::from_str_radix(&convert::convert_token(&cmd.depool_fee)?, 10)
        .map_err(|e| format!(r#"failed to parse depool fee value: {}"#, e))?;
    let value = (fee + stake) as f64 * 1.0 / 1e9;
    call_contract(cmd.conf.clone(), &cmd.wallet, &cmd.depool, &format!("{}", value), &cmd.keys, &body, true).await
}

async fn remove_stake(
    cmd: CommandData<'_>,
) -> Result<(), String> {
    let stake = u64::from_str_radix(
        &convert::convert_token(cmd.stake)?, 10,
    ).unwrap();
    let body = encode_remove_stake(stake).await?;
    call_contract(cmd.conf.clone(), &cmd.wallet, &cmd.depool, &cmd.depool_fee, &cmd.keys, &body, true).await
}

async fn withdraw_stake(
    cmd: CommandData<'_>,
) -> Result<(), String> {
    let stake = u64::from_str_radix(
        &convert::convert_token(cmd.stake)?, 10,
    ).unwrap();
    let body = encode_withdraw_stake(stake).await?;
    call_contract(cmd.conf.clone(), &cmd.wallet, &cmd.depool, &cmd.depool_fee, &cmd.keys, &body, true).await
}

async fn transfer_stake(cmd: CommandData<'_>, dest: &str) -> Result<(), String> {
    let dest = load_ton_address(dest, &cmd.conf)?;
    let stake = u64::from_str_radix(
        &convert::convert_token(cmd.stake)?, 10,
    ).unwrap();
    let body = encode_transfer_stake(dest.as_str(), stake).await?;
    call_contract(cmd.conf.clone(), &cmd.wallet, &cmd.depool, &cmd.depool_fee, &cmd.keys, &body, true).await
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
    call_contract(conf.clone(), &wallet, &depool, &format!("{}", value), &keys, &body, true).await
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
    call_contract(conf.clone(), &wallet, &depool, &format!("{}", value), &keys, &body, true).await
}

async fn encode_body(func: &str, params: serde_json::Value) -> Result<String, String> {
    let client = create_client_local()?;
    ton_client::abi::encode_message_body(
        client.clone(),
        ParamsOfEncodeMessageBody {
            abi: load_abi(DEPOOL_ABI)?,
            call_set: CallSet::some_with_function_and_input(func, params).unwrap(),
            is_internal: true,
            ..Default::default()
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

async fn call_contract(
    conf: Config,
    wallet: &str,
    depool: &str,
    value: &str,
    keys: &str,
    body: &str,
    answer_is_expected: bool
) -> Result<(), String> {
    if conf.no_answer {
        send_with_body(conf.clone(), wallet, depool, value, keys, body).await
    } else {
        call_contract_and_get_answer(
            conf.clone(),
            wallet,
            depool,
            value,
            keys,
            body,
            answer_is_expected
        ).await
    }
}

async fn call_contract_and_get_answer(
    conf: Config,
    src_addr: &str,
    dest_addr: &str,
    value: &str,
	keys: &str,
    body: &str,
    answer_is_expected: bool
) -> Result<(), String> {
    let ton = create_client_verbose(&conf)?;
    let abi = load_abi(MSIG_ABI)?;
    let start = now();

    let params = json!({
        "dest": dest_addr,
        "value": convert::convert_token(value)?,
        "bounce": true,
        "allBalance": false,
        "payload": body,
    }).to_string();

    let msg = prepare_message(
        ton.clone(),
        src_addr,
        abi.clone(),
        "submitTransaction",
        &params,
        None,
        Some(keys.to_owned()),
        false
    ).await?;

    print_encoded_message(&msg);

    println!("Multisig message processing... ");
    let callback = |_| {
        async move {}
    };

    let result = send_message(
        ton.clone(),
        ParamsOfSendMessage {
            message: msg.message.clone(),
            abi: Some(abi.clone()),
            send_events: false,
            ..Default::default()
        },
        callback,
    ).await
    .map_err(|e| format!("Failed: {:#}", e))?;

    wait_for_transaction(
        ton.clone(),
        ParamsOfWaitForTransaction {
            abi: Some(abi.clone()),
            message: msg.message.clone(),
            shard_block_id: result.shard_block_id,
            send_events: true,
            ..Default::default()
        },
        callback.clone(),
    ).await
    .map_err(|e| format!("Failed: {:#}", e))?;

    println!("\nMessage was successfully sent to the multisig, waiting for message to be sent to the depool...");

    let message = ton_client::net::wait_for_collection(
        ton.clone(),
        ParamsOfWaitForCollection {
            collection: "messages".to_owned(),
            filter: Some(answer_filter(src_addr, dest_addr, start)),
            result: "id body created_at created_at_string".to_owned(),
            timeout: Some(conf.timeout),
            ..Default::default()
        },
    ).await.map_err(|e| println!("failed to query message: {}", e.to_string()));

    if message.is_err() {
        println!("\nRequest failed. Check the contract balance to be great enough to cover transfer value with possible fees.");
        return Ok(());
    }
    println!("\nRequest was successfully sent to depool.");
    if answer_is_expected {
        println!("\nWaiting for depool answer...");

        let mut statuses: HashMap<u32, &str> = HashMap::new();
        statuses.insert(0, "SUCCESS");
        statuses.insert(1, "STAKE_TOO_SMALL");
        statuses.insert(3, "DEPOOL_CLOSED");
        statuses.insert(6, "NO_PARTICIPANT");
        statuses.insert(9, "PARTICIPANT_ALREADY_HAS_VESTING");
        statuses.insert(10, "WITHDRAWAL_PERIOD_GREATER_TOTAL_PERIOD");
        statuses.insert(11, "TOTAL_PERIOD_MORE_18YEARS");
        statuses.insert(12, "WITHDRAWAL_PERIOD_IS_ZERO");
        statuses.insert(13, "TOTAL_PERIOD_IS_NOT_DIVISIBLE_BY_WITHDRAWAL_PERIOD");
        statuses.insert(16, "REMAINING_STAKE_LESS_THAN_MINIMAL");
        statuses.insert(17, "PARTICIPANT_ALREADY_HAS_LOCK");
        statuses.insert(18, "TRANSFER_AMOUNT_IS_TOO_BIG");
        statuses.insert(19, "TRANSFER_SELF");
        statuses.insert(20, "TRANSFER_TO_OR_FROM_VALIDATOR");
        statuses.insert(21, "FEE_TOO_SMALL");
        statuses.insert(22, "INVALID_ADDRESS");
        statuses.insert(23, "INVALID_DONOR");
        statuses.insert(24, "NO_ELECTION_ROUND");
        statuses.insert(25, "INVALID_ELECTION_ID");
        statuses.insert(26, "TRANSFER_WHILE_COMPLETING_STEP");
        statuses.insert(27, "NO_POOLING_STAKE");

        let message = ton_client::net::wait_for_collection(
            ton.clone(),
            ParamsOfWaitForCollection {
                collection: "messages".to_owned(),
                filter: Some(answer_filter(dest_addr, src_addr, start)),
                result: "id body created_at created_at_string value".to_owned(),
                timeout: Some(conf.timeout),
                ..Default::default()
            },
        ).await.map_err(|e| println!("failed to query answer: {}", e.to_string()));
        if message.is_ok() {
            let message = message.unwrap().result;
            println!("\nAnswer: ");
            let (name, args) = print_message(ton.clone(), &message, PARTICIPANT_ABI, true).await;
            if name == "receiveAnswer" {
                let args: serde_json::Value = serde_json::from_str(&args).unwrap();
                let status = args["errcode"].as_str().unwrap().parse::<u32>().unwrap();
                let comment = args["comment"].as_str().unwrap();
                if statuses.contains_key(&status) {
                    println!("Answer status: {}\nComment: {}", statuses[&status], comment);
                } else {
                    println!("Answer status: Unknown({})\nComment: {}", status, comment);
                }

            }
            println!();
        } else {
            println!("\nThere were no answer messages during the timeout period.\n");
        }
    }
    println!("Done");
    Ok(())
}