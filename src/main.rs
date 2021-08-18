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
#[macro_use] extern crate clap;
#[macro_use] extern crate log;
#[macro_use] extern crate serde_json;

mod account;
mod call;
mod config;
mod convert;
mod crypto;
mod decode;
mod debot;
mod deploy;
mod depool;
mod depool_abi;
mod genaddr;
mod getconfig;
mod helpers;
mod multisig;
mod sendfile;
mod voting;
mod replay;

use account::{get_account, calc_storage};
use call::{call_contract, call_contract_with_msg, generate_message, parse_params, run_get_method, run_local_for_account};
use clap::{ArgMatches, SubCommand, Arg, AppSettings};
use config::{Config, set_config, clear_config};
use crypto::{generate_mnemonic, extract_pubkey, generate_keypair};
use debot::{create_debot_command, debot_command};
use decode::{create_decode_command, decode_command};
use deploy::{deploy_contract, generate_deploy_message};
use depool::{create_depool_command, depool_command};
use helpers::{load_ton_address, load_abi, create_client_local};
use genaddr::generate_address;
use getconfig::query_global_config;
use multisig::{create_multisig_command, multisig_command};
use std::{env, path::PathBuf};
use voting::{create_proposal, decode_proposal, vote};
use replay::{fetch_command, replay_command};
use ton_client::abi::{ParamsOfEncodeMessageBody, CallSet};
use crate::config::FullConfig;

pub const VERBOSE_MODE: bool = true;
const DEF_MSG_LIFETIME: u32 = 30;
const CONFIG_BASE_NAME: &'static str = "tonos-cli.conf.json";
const DEF_STORAGE_PERIOD: u32 = 60 * 60 * 24 * 365;

enum CallType {
    Run,
    Call,
    Msg,
    Fee,
}

enum DeployType {
    Full,
    MsgOnly,
    Fee,
}

#[macro_export]
macro_rules! print_args {
    ($( $arg:ident ),* ) => {
        if (VERBOSE_MODE) {
            println!("Input arguments:");
            $(
                println!(
                    "{:>width$}: {}",
                    stringify!($arg),
                    if $arg.is_some() { $arg.as_ref().unwrap() } else { "None" },
                    width=8
                );
            )*
        }
    };
}

fn default_config_name() -> Result<String, String> {
    env::current_dir()
        .map_err(|e| format!("cannot get current dir: {}", e))
        .map(|dir| {
            dir.join(PathBuf::from(CONFIG_BASE_NAME))
                .to_str().unwrap().to_string()
        })
}

#[tokio::main]
async fn main() -> Result<(), i32> {
    main_internal().await.map_err(|err_str| {
        println!("Error: {}", err_str);
        1
    })
}

async fn main_internal() -> Result <(), String> {
    let callex_sub_command = SubCommand::with_name("callex")
        .about("Sends an external message with encoded function call to the contract (alternative syntax).")
        .setting(AppSettings::AllowMissingPositional)
        .setting(AppSettings::AllowLeadingHyphen)
        .setting(AppSettings::TrailingVarArg)
        .setting(AppSettings::DontCollapseArgsInUsage)
        .arg(Arg::with_name("METHOD")
            .help("Name of the function being called."))
        .arg(Arg::with_name("ADDRESS")
            .help("Contract address."))
        .arg(Arg::with_name("ABI")
            .help("Path to the contract ABI file."))
        .arg(Arg::with_name("SIGN")
            .help("Seed phrase or path to the file with keypair used to sign the message."))
        .arg(Arg::with_name("PARAMS")
            .help("Function arguments. Must be a list of `--name value` pairs or a json string with all arguments.")
            .multiple(true));

    let runget_sub_command = SubCommand::with_name("runget")
        .about("Runs get-method of a FIFT contract.")
        .setting(AppSettings::AllowLeadingHyphen)
        .setting(AppSettings::TrailingVarArg)
        .setting(AppSettings::DontCollapseArgsInUsage)
        .arg(Arg::with_name("ADDRESS")
            .required(true)
            .help("Contract address."))
        .arg(Arg::with_name("METHOD")
            .required(true)
            .help("Name of the function being called."))
        .arg(Arg::with_name("PARAMS")
            .help("Function arguments.")
            .multiple(true));

    let no_answer_with_value = Arg::with_name("NO_ANSWER")
        .long("--no-answer")
        .takes_value(true)
        .help("Flag whether to wait for depool answer when calling a depool function.");
    let no_answer = Arg::with_name("NO_ANSWER")
        .long("--no-answer")
        .help("Flag whether to wait for depool answer when calling a depool function.");

    let matches = clap_app! (tonos_cli =>
        (version: &*format!("{}\nCOMMIT_ID: {}\nBUILD_DATE: {}\nCOMMIT_DATE: {}\nGIT_BRANCH: {}",
        env!("CARGO_PKG_VERSION"),
            env!("BUILD_GIT_COMMIT"),
            env!("BUILD_TIME") ,
            env!("BUILD_GIT_DATE"),
            env!("BUILD_GIT_BRANCH")
        ))
        (author: "TONLabs")
        (about: "TONLabs console tool for TON")
        (@arg NETWORK: -u --url +takes_value "Network to connect.")
        (@arg CONFIG: -c --config +takes_value "Path to the tonos-cli configuration file.")
        (@arg JSON: -j --json "Cli prints output in json format.")
        (@subcommand version =>
            (about: "Prints build and version info.")
        )
        (@subcommand convert =>
            (about: "Converts tokens to nanotokens.")
            (@subcommand tokens =>
                (about: "Converts tokens to nanotokens.")
                (@arg AMOUNT: +required +takes_value "Token amount value")
            )
        )
        (@subcommand genphrase =>
            (about: "Generates a seed phrase for keypair.")
            (author: "TONLabs")
        )
        (@subcommand genpubkey =>
            (about: "Generates a public key from the seed phrase.")
            (author: "TONLabs")
            (@arg PHRASE: +required +takes_value "Seed phrase (12 words). Should be specified in quotes.")
        )
        (@subcommand getkeypair =>
            (about: "Generates a keypair from the seed phrase and saves it to the file.")
            (author: "TONLabs")
            (@arg KEY_FILE: +required +takes_value "Path to the file where to store the keypair.")
            (@arg PHRASE: +required +takes_value "Seed phrase (12 words). Should be specified in quotes.")
        )
        (@subcommand genaddr =>
            (@setting AllowNegativeNumbers)
            (about: "Calculates smart contract address in different formats. By default, input tvc file isn't modified.")
            (version: &*format!("{}", env!("CARGO_PKG_VERSION")))
            (author: "TONLabs")
            (@arg TVC: +required +takes_value "Path to the compiled smart contract (tvc file).")
            (@arg ABI: +required +takes_value "Path to the contract ABI file.")
            (@arg WC: --wc +takes_value "Workchain id used to generate addresses (default value is taken from the config).")
            (@arg GENKEY: --genkey +takes_value conflicts_with[SETKEY] "Path to the file, where a new generated keypair for the contract will be saved.")
            (@arg SETKEY: --setkey +takes_value conflicts_with[GENKEY] "Seed phrase or path to the file with keypair.")
            (@arg DATA: --data +takes_value "Initial data to insert into the contract. Should be specified in json format.")
            (@arg SAVE: --save "If this flag is specified, modifies the tvc file with the keypair and initial data")
        )
        (@subcommand deploy =>
            (@setting AllowNegativeNumbers)
            (@setting AllowLeadingHyphen)
            (about: "Deploys a smart contract to the blockchain.")
            (version: &*format!("{}", env!("CARGO_PKG_VERSION")))
            (author: "TONLabs")
            (@arg TVC: +required +takes_value "Path to the compiled smart contract (tvc file).")
            (@arg PARAMS: +required +takes_value "Constructor arguments. Can be specified with a filename, which contains json data.")
            (@arg ABI: --abi +takes_value "Path to the contract ABI file.")
            (@arg SIGN: --sign +takes_value "Seed phrase or path to the file with keypair used to sign 'constructor message'.")
            (@arg WC: --wc +takes_value "Workchain id of the smart contract (default value is taken from the config).")
        )
        (@subcommand deploy_message =>
            (@setting AllowNegativeNumbers)
            (@setting AllowLeadingHyphen)
            (about: "Generates a signed message to deploy a smart contract to the blockchain.")
            (version: &*format!("{}", env!("CARGO_PKG_VERSION")))
            (author: "TONLabs")
            (@arg TVC: +required +takes_value "Path to the compiled smart contract (tvc file)")
            (@arg PARAMS: +required +takes_value "Constructor arguments. Can be specified with a filename, which contains json data.")
            (@arg ABI: --abi +takes_value "Path to the contract ABI file.")
            (@arg SIGN: --sign +takes_value "Seed phrase or path to the file with keypair used to sign 'constructor message'.")
            (@arg WC: --wc +takes_value "Workchain id of the smart contract (default value is taken from the config).")
            (@arg OUTPUT: -o --output +takes_value "Path to the file where to store the message.")
            (@arg RAW: --raw "Creates raw message boc.")
        )
        (subcommand: callex_sub_command)
        (@subcommand call =>
            (@setting AllowLeadingHyphen)
            (about: "Sends an external message with encoded function call to the contract.")
            (version: &*format!("{}", env!("CARGO_PKG_VERSION")))
            (author: "TONLabs")
            (@arg ADDRESS: +required +takes_value "Contract address.")
            (@arg METHOD: +required +takes_value "Name of the function being called.")
            (@arg PARAMS: +required +takes_value "Function arguments. Can be specified with a filename, which contains json data.")
            (@arg ABI: --abi +takes_value "Path to the contract ABI file.")
            (@arg SIGN: --sign +takes_value "Seed phrase or path to the file with keypair used to sign the message.")
        )
        (@subcommand send =>
            (about: "Sends a prepared message to the contract.")
            (version: &*format!("{}", env!("CARGO_PKG_VERSION")))
            (author: "TONLabs")
            (@arg MESSAGE: +required +takes_value "Message to send. Message data should be specified in quotes.")
            (@arg ABI: --abi +takes_value "Path to the contract ABI file.")
        )
        (@subcommand message =>
            (@setting AllowLeadingHyphen)
            (about: "Generates a signed message with encoded function call.")
            (author: "TONLabs")
            (@arg ADDRESS: +required +takes_value "Contract address.")
            (@arg METHOD: +required +takes_value "Name of the function being called.")
            (@arg PARAMS: +required +takes_value "Function arguments. Can be specified with a filename, which contains json data.")
            (@arg ABI: --abi +takes_value "Path to the contract ABI file.")
            (@arg SIGN: --sign +takes_value "Seed phrase or path to the file with keypair used to sign the message.")
            (@arg LIFETIME: --lifetime +takes_value "Period of time in seconds while message is valid.")
            (@arg OUTPUT: -o --output +takes_value "Path to the file where to store the message.")
            (@arg RAW: --raw "Creates raw message boc.")
        )
        (@subcommand body =>
            (@setting AllowLeadingHyphen)
            (about: "Generates a payload for internal function call.")
            (version: &*format!("{}", env!("CARGO_PKG_VERSION")))
            (author: "TONLabs")
            (@arg METHOD: +required +takes_value "Name of the function being called.")
            (@arg PARAMS: +required +takes_value "Function arguments. Can be specified with a filename, which contains json data.")
            (@arg ABI: --abi +takes_value "Path to the contract ABI file.")
        )
        (@subcommand run =>
            (@setting AllowLeadingHyphen)
            (about: "Runs contract function locally.")
            (@arg ADDRESS: +required +takes_value "Contract address or path to the saved account state if --boc flag is specified.")
            (@arg METHOD: +required +takes_value "Name of the function being called.")
            (@arg PARAMS: +required +takes_value "Function arguments. Can be specified with a filename, which contains json data.")
            (@arg ABI: --abi +takes_value "Path to the contract ABI file.")
            (@arg BOC: --boc "Flag that changes behavior of the command to work with the saved account state.")
        )
        (subcommand: runget_sub_command)
        (@subcommand config =>
            (@setting AllowLeadingHyphen)
            (about: "Allows to tune certain default values for options in the config file.")
            (version: &*format!("{}", env!("CARGO_PKG_VERSION")))
            (author: "TONLabs")
            (@arg URL: --url +takes_value "Url to connect.")
            (@arg ABI: --abi +takes_value conflicts_with[DATA] "Path to the contract ABI file.")
            (@arg KEYS: --keys +takes_value "Path to the file with keypair.")
            (@arg ADDR: --addr +takes_value "Contract address.")
            (@arg WALLET: --wallet +takes_value "Multisig wallet address. Used in commands which send internal messages through multisig wallets.")
            (@arg PUBKEY: --pubkey +takes_value "User public key. Used by DeBot Browser.")
            (@arg WC: --wc +takes_value "Workchain id.")
            (@arg RETRIES: --retries +takes_value "Number of attempts to call smart contract function if previous attempt was unsuccessful.")
            (@arg TIMEOUT: --timeout +takes_value "Contract call timeout in ms.")
            (@arg LIST: --list conflicts_with[NO_ANSWER ASYNC_CALL LOCAL_RUN BALANCE_IN_TONS LIFETIME DEPOOL_FEE PUBKEY URL ABI KEYS ADDR RETRIES TIMEOUT WC WALLET] "Prints all config parameters.")
            (@arg DEPOOL_FEE: --depool_fee +takes_value "Value added to the message sent to depool to cover it's fees (change will be returned).")
            (@arg LIFETIME: --lifetime +takes_value "Period of time in seconds while message is valid.")
            (arg: no_answer_with_value)
            (@arg BALANCE_IN_TONS: --balance_in_tons +takes_value "Print balance for account command in tons. If false balance is printed in nanotons.")
            (@arg LOCAL_RUN: --local_run +takes_value "Enable preliminary local run before deploy and call commands.")
            (@arg ASYNC_CALL: --async_call +takes_value "Disables wait for transaction to appear in the network after call command.")
            (@subcommand clear =>
                (@setting AllowLeadingHyphen)
                (about: "Resets certain default values for options in the config file. Resets all values if used without options.")
                (@arg URL: --url "Url to connect.")
                (@arg ABI: --abi "Path to the contract ABI file.")
                (@arg KEYS: --keys "Path to the file with keypair.")
                (@arg ADDR: --addr "Contract address.")
                (@arg WALLET: --wallet "Multisig wallet address.")
                (@arg WC: --wc "Workchain id.")
                (@arg RETRIES: --retries "Number of attempts to call smart contract function if previous attempt was unsuccessful.")
                (@arg TIMEOUT: --timeout "Contract call timeout in ms.")
                (@arg DEPOOL_FEE: --depool_fee "Value added to the message sent to depool to cover it's fees (change will be returned).")
                (@arg LIFETIME: --lifetime "Period of time in seconds while message is valid.")
                (arg: no_answer)
                (@arg BALANCE_IN_TONS: --balance_in_tons "Print balance for account command in tons. If false balance is printed in nanotons.")
                (@arg LOCAL_RUN: --local_run "Enable preliminary local run before deploy and call commands.")
            )
            (@subcommand endpoint =>
                (about: "Commands to work with the map of endpoints.")
                (@subcommand add =>
                    (about: "Add endpoints list.")
                    (@arg URL: +required +takes_value "Url of the endpoints list.")
                    (@arg ENDPOINTS: +required +takes_value "List of endpoints.")
                )
                (@subcommand remove =>
                    (about: "Remove endpoints list.")
                    (@arg URL: +required +takes_value "Url of the endpoints list.")
                )
                (@subcommand reset =>
                    (about: "Reset the endpoints map.")
                )
                (@subcommand print =>
                    (about: "Print current endpoints map.")
                )
            )
        )
        (@subcommand account =>
            (@setting AllowLeadingHyphen)
            (about: "Obtains and prints account information.")
            (version: &*format!("{}", env!("CARGO_PKG_VERSION")))
            (author: "TONLabs")
            (@arg ADDRESS: +required +takes_value "Smart contract address.")
            (@arg DUMPTVC: -d --dumptvc +takes_value "Dumps account StateInit to specified tvc file.")
        )
        (@subcommand fee =>
            (about: "Calculates fees for executing message or account storage fee.")
            (@subcommand storage =>
                (@setting AllowLeadingHyphen)
                (about: "Gets account storage fee for specified period in nanotons.")
                (version: &*format!("{}", env!("CARGO_PKG_VERSION")))
                (author: "TONLabs")
                (@arg ADDRESS: +required +takes_value "Smart contract address.")
                (@arg PERIOD: -p --period +takes_value "Time period in seconds (default value is 1 day).")
            )
            (@subcommand deploy =>
                (@setting AllowNegativeNumbers)
                (@setting AllowLeadingHyphen)
                (about: "Executes deploy locally, calculates fees and prints table of fees in nanotons.")
                (version: &*format!("{}", env!("CARGO_PKG_VERSION")))
                (author: "TONLabs")
                (@arg TVC: +required +takes_value "Path to the compiled smart contract (tvc file).")
                (@arg PARAMS: +required +takes_value "Constructor arguments. Can be specified with a filename, which contains json data.")
                (@arg ABI: --abi +takes_value "Path to the contract ABI file.")
                (@arg SIGN: --sign +takes_value "Seed phrase or path to the file with keypair used to sign 'constructor message'.")
                (@arg WC: --wc +takes_value "Workchain id of the smart contract (default value is taken from the config).")
            )
            (@subcommand call =>
                (@setting AllowLeadingHyphen)
                (about: "Executes call locally, calculates fees and prints table of all fees in nanotons.")
                (version: &*format!("{}", env!("CARGO_PKG_VERSION")))
                (author: "TONLabs")
                (@arg ADDRESS: +required +takes_value "Contract address.")
                (@arg METHOD: +required +takes_value "Name of the function being called.")
                (@arg PARAMS: +required +takes_value "Function arguments. Can be specified with a filename, which contains json data.")
                (@arg ABI: --abi +takes_value "Path to the contract ABI file.")
                (@arg SIGN: --sign +takes_value "Seed phrase or path to the file with keypair used to sign the message.")
            )
        )
        (@subcommand proposal =>
            (about: "Submits a proposal transaction in the multisignature wallet with a text comment.")
            (@subcommand create =>
                (@arg ADDRESS: +required +takes_value "Address of the multisignature wallet.")
                (@arg DEST: +required +takes_value "Address of the proposal contract.")
                (@arg COMMENT: +required +takes_value "Proposal description (max symbols 382).")
                (@arg KEYS: +required +takes_value "Seed phrase or path to the keypair file.")
                (@arg OFFLINE: -f --offline "Prints signed message to terminal instead of sending it.")
                (@arg LIFETIME: -l --lifetime +takes_value "Period of time in seconds while message is valid.")
            )
            (@subcommand vote =>
                (about: "Confirms a proposal transaction in the multisignature wallet.")
                (@arg ADDRESS: +required +takes_value "Address of the multisignature wallet.")
                (@arg ID: +required +takes_value "Proposal transaction id.")
                (@arg KEYS: +required +takes_value "Seed phrase or path to the keypair file.")
                (@arg OFFLINE: -f --offline "Prints signed message to terminal instead of sending it.")
                (@arg LIFETIME: -l --lifetime +takes_value "Period of time in seconds while message is valid.")
            )
            (@subcommand decode =>
                (about: "Prints a comment string from the proposal transaction.")
                (@arg ADDRESS: +required +takes_value "Address of the multisignature wallet.")
                (@arg ID: +required +takes_value "Proposal transaction id.")
            )
        )
        (subcommand: create_multisig_command())
        (subcommand: create_depool_command())
        (subcommand: create_decode_command())
        (subcommand: create_debot_command())
        (@subcommand getconfig =>
            (about: "Reads the global configuration parameter with defined index.")
            (@arg INDEX: +required +takes_value "Parameter index.")
        )
        (@subcommand nodeid =>
            (about: "Calculates node ID from the validator public key")
            (@arg KEY: --pubkey +takes_value "Validator public key.")
            (@arg KEY_PAIR: --keypair +takes_value "Validator seed phrase or path to the file with keypair.")
        )
        (@subcommand sendfile =>
            (about: "Sends the boc file with an external inbound message to account.")
            (@arg BOC: +required +takes_value "Message boc file.")
        )
        (@subcommand fetch =>
            (about: "Fetches account's zerostate and transactions.")
            (@arg ADDRESS: +required +takes_value "Account address to fetch zerostate and txns for.")
            (@arg OUTPUT: +required +takes_value "Output file name")
        )
        (@subcommand replay =>
            (about: "Replays account's transactions starting from zerostate.")
            (@arg CONFIG_TXNS: +required +takes_value "File containing zerostate and txns of -1:555..5 account.")
            (@arg INPUT_TXNS: +required +takes_value "File containing zerostate and txns of the account to replay.")
            (@arg TXNID: +required +takes_value "Dump account state before this transaction ID and stop replaying.")
        )
        (@setting SubcommandRequired)
    ).get_matches();

    let is_json = matches.is_present("JSON");

    let config_file = matches.value_of("CONFIG").map(|v| v.to_string())
        .or(env::var("TONOSCLI_CONFIG").ok())
        .unwrap_or(default_config_name()?);

    let mut conf = match Config::from_file(&config_file) {
        Some(c) => {
            if !is_json { println!("Config: {}", config_file); }
            c
        },
        None => {
            if !is_json { println!("Config: default"); }
            Config::new()
        },
    };
    conf.is_json = is_json;

    if let Some(url) = matches.value_of("NETWORK") {
        conf.url = url.to_string();
        let empty : Vec<String> = Vec::new();
        conf.endpoints = FullConfig::get_map(&config_file).get(url).unwrap_or(&empty).clone();
    }

    if let Some(m) = matches.subcommand_matches("convert") {
        if let Some(m) = m.subcommand_matches("tokens") {
            return convert_tokens(m);
        }
    }
    if let Some(m) = matches.subcommand_matches("callex") {
        return callex_command(m, conf).await;
    }
    if let Some(m) = matches.subcommand_matches("call") {
        return call_command(m, conf, CallType::Call).await;
    }
    if let Some(m) = matches.subcommand_matches("run") {
        if m.is_present("BOC") {
            return run_account(m, conf).await;
        } else {
            return call_command(m, conf, CallType::Run).await;
        }
    }
    if let Some(m) = matches.subcommand_matches("runget") {
        return runget_command(m, conf).await;
    }
    if let Some(m) = matches.subcommand_matches("body") {
        return body_command(m, conf).await;
    }
    if let Some(m) = matches.subcommand_matches("message") {
        return call_command(m, conf, CallType::Msg).await;
    }
    if let Some(m) = matches.subcommand_matches("send") {
        return send_command(m, conf).await;
    }
    if let Some(m) = matches.subcommand_matches("deploy") {
        return deploy_command(m, conf, DeployType::Full).await;
    }
    if let Some(m) = matches.subcommand_matches("deploy_message") {
        return deploy_command(m, conf, DeployType::MsgOnly).await;
    }
    if let Some(m) = matches.subcommand_matches("config") {
        return config_command(m, conf, config_file);
    }
    if let Some(m) = matches.subcommand_matches("genaddr") {
        return genaddr_command(m, conf).await;
    }
    if let Some(m) = matches.subcommand_matches("getkeypair") {
        return getkeypair_command(m, conf);
    }
    if let Some(m) = matches.subcommand_matches("account") {
        return account_command(m, conf).await;
    }
    if let Some(m) = matches.subcommand_matches("fee") {
        if let Some(m) = m.subcommand_matches("storage") {
            return storage_command(m, conf).await;
        }
        if let Some(m) = m.subcommand_matches("deploy") {
            return deploy_command(m, conf, DeployType::Fee).await;
        }
        if let Some(m) = m.subcommand_matches("call") {
            return call_command(m, conf, CallType::Fee).await;
        }
    }
    if let Some(m) = matches.subcommand_matches("genphrase") {
        return genphrase_command(m, conf);
    }
    if let Some(m) = matches.subcommand_matches("genpubkey") {
        return genpubkey_command(m, conf);
    }
    if let Some(m) = matches.subcommand_matches("proposal") {
        if let Some(m) = m.subcommand_matches("create") {
            return proposal_create_command(m, conf).await;
        }
        if let Some(m) = m.subcommand_matches("vote") {
            return proposal_vote_command(m, conf).await;
        }
        if let Some(m) = m.subcommand_matches("decode") {
            return proposal_decode_command(m, conf).await;
        }
    }
    if let Some(m) = matches.subcommand_matches("multisig") {
        return multisig_command(m, conf).await;
    }
    if let Some(m) = matches.subcommand_matches("depool") {
        return depool_command(m, conf).await;
    }
    if let Some(m) = matches.subcommand_matches("getconfig") {
        return getconfig_command(m, conf).await;
    }
    if let Some(m) = matches.subcommand_matches("nodeid") {
        return nodeid_command(m);
    }
    if let Some(m) = matches.subcommand_matches("sendfile") {
        return sendfile_command(m, conf).await;
    }
    if let Some(m) = matches.subcommand_matches("decode") {
        return decode_command(m, conf).await;
    }
    if let Some(m) = matches.subcommand_matches("debot") {
        return debot_command(m, conf).await;
    }
    if let Some(m) = matches.subcommand_matches("fetch") {
        return fetch_command(m, conf).await;
    }
    if let Some(m) = matches.subcommand_matches("replay") {
        return replay_command(m).await;
    }
    if let Some(_) = matches.subcommand_matches("version") {
        println!(
            "tonos-cli {}\nCOMMIT_ID: {}\nBUILD_DATE: {}\nCOMMIT_DATE: {}\nGIT_BRANCH: {}",
            env!("CARGO_PKG_VERSION"),
            env!("BUILD_GIT_COMMIT"),
            env!("BUILD_TIME") ,
            env!("BUILD_GIT_DATE"),
            env!("BUILD_GIT_BRANCH")
        );
        return Ok(());
    }
    Err("invalid arguments".to_string())
}

fn convert_tokens(matches: &ArgMatches) -> Result<(), String> {
    let amount = matches.value_of("AMOUNT").unwrap();
    let result = convert::convert_token(amount)?;
    println!("{}", result);
    Ok(())
}

fn genphrase_command(_matches: &ArgMatches, _config: Config) -> Result<(), String> {
    generate_mnemonic()
}

fn genpubkey_command(matches: &ArgMatches, _config: Config) -> Result<(), String> {
    let mnemonic = matches.value_of("PHRASE").unwrap();
    extract_pubkey(mnemonic)
}

fn getkeypair_command(matches: &ArgMatches, _config: Config) -> Result<(), String> {
    let key_file = matches.value_of("KEY_FILE");
    let phrase = matches.value_of("PHRASE");
    print_args!(key_file, phrase);
    generate_keypair(key_file.unwrap(), phrase.unwrap())
}

async fn send_command(matches: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let message = matches.value_of("MESSAGE");
    let abi = Some(
        matches.value_of("ABI")
            .map(|s| s.to_string())
            .or(config.abi_path.clone())
            .ok_or("ABI file is not defined. Supply it in the config file or command line.".to_string())?
    );

    print_args!(message, abi);

    let abi = std::fs::read_to_string(abi.unwrap())
        .map_err(|e| format!("failed to read ABI file: {}", e.to_string()))?;

    call_contract_with_msg(config, message.unwrap().to_owned(), abi).await
}

fn load_params(params: &str) -> Result<String, String> {
    Ok(if params.find('{').is_none() {
        std::fs::read_to_string(params)
            .map_err(|e| format!("failed to load params from file: {}", e))?
    } else {
        params.to_string()
    })
}

async fn body_command(matches: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let method = matches.value_of("METHOD");
    let params = matches.value_of("PARAMS");
    let output = matches.value_of("OUTPUT");
    let abi = Some(
        matches.value_of("ABI")
            .map(|s| s.to_string())
            .or(config.abi_path.clone())
            .ok_or("ABI file is not defined. Supply it in the config file or command line.".to_string())?
    );
    let params = Some(load_params(params.unwrap())?);
    print_args!(method, params, abi, output);

    let params = serde_json::from_str(&params.unwrap())
        .map_err(|e| format!("arguments are not in json format: {}", e))?;

    let abi = std::fs::read_to_string(abi.unwrap())
        .map_err(|e| format!("failed to read ABI file: {}", e.to_string()))?;


    let client = create_client_local()?;
    let body = ton_client::abi::encode_message_body(
        client.clone(),
        ParamsOfEncodeMessageBody {
            abi: load_abi(&abi)?,
            call_set: CallSet::some_with_function_and_input(&method.unwrap(), params).unwrap(),
            is_internal: true,
            ..Default::default()
        },
    ).await
    .map_err(|e| format!("failed to encode body: {}", e))
    .map(|r| r.body)?;

    println!("Message body: {}", body);

    Ok(())
}

async fn run_account(matches: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let account = matches.value_of("ADDRESS");
    let method = matches.value_of("METHOD");
    let params = matches.value_of("PARAMS");

    let abi = Some(
        matches.value_of("ABI")
            .map(|s| s.to_string())
            .or(config.abi_path.clone())
            .ok_or("ABI file is not defined. Supply it in the config file or command line.".to_string())?
    );

    let params = Some(load_params(params.unwrap())?);
    if !config.is_json {
        print_args!(account, method, params, abi);
    }

    let abi = std::fs::read_to_string(abi.unwrap())
        .map_err(|e| format!("failed to read ABI file: {}", e.to_string()))?;

    run_local_for_account(config,
    account.unwrap(),
        abi,
        method.unwrap(),
        &params.unwrap(),
    ).await
}

async fn call_command(matches: &ArgMatches<'_>, config: Config, call: CallType) -> Result<(), String> {
    let address = matches.value_of("ADDRESS");
    let method = matches.value_of("METHOD");
    let params = matches.value_of("PARAMS");
    let lifetime = matches.value_of("LIFETIME");
    let raw = matches.is_present("RAW");
    let output = matches.value_of("OUTPUT");

    let abi = Some(
        matches.value_of("ABI")
            .map(|s| s.to_string())
            .or(config.abi_path.clone())
            .ok_or("ABI file is not defined. Supply it in the config file or command line.".to_string())?
    );

    let keys = match call {
        CallType::Call | CallType::Msg | CallType::Fee => {
            matches.value_of("SIGN")
                .map(|s| s.to_string())
                .or(config.keys_path.clone())
        },
        CallType::Run => {
            None
        }
    };

    let params = Some(load_params(params.unwrap())?);
    if !config.is_json {
        print_args!(address, method, params, abi, keys, lifetime, output);
    }

    let abi = std::fs::read_to_string(abi.unwrap())
        .map_err(|e| format!("failed to read ABI file: {}", e.to_string()))?;
    let address = load_ton_address(address.unwrap(), &config)?;

    match call {
        CallType::Call | CallType::Run | CallType::Fee => {
            let local = if let CallType::Run = call { true } else { false };
            let is_fee = if let CallType::Fee = call { true } else { false };
            call_contract(
                config,
                address.as_str(),
                abi,
                method.unwrap(),
                &params.unwrap(),
                keys,
                local,
                is_fee,
            ).await
        },
        CallType::Msg => {
            let lifetime = lifetime.map(|val| {
                    u32::from_str_radix(val, 10)
                        .map_err(|e| format!("failed to parse lifetime: {}", e))
                })
                .transpose()?
                .unwrap_or(DEF_MSG_LIFETIME);

            generate_message(
                config,
                address.as_str(),
                abi,
                method.unwrap(),
                &params.unwrap(),
                keys,
                lifetime,
                raw,
                output,
            ).await
        },
    }
}

async fn callex_command(matches: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let method_opt = matches.value_of("METHOD");
    let method = method_opt.ok_or("METHOD is not defined")?;
    let address = Some(
        matches.value_of("ADDRESS")
            .map(|s| s.to_string())
            .or(config.addr.clone())
            .ok_or("ADDRESS is not defined. Supply it in the config file or in command line.".to_string())?
    );
    let abi = Some(
        matches.value_of("ABI")
        .map(|s| s.to_string())
        .or(config.abi_path.clone())
        .ok_or("ABI is not defined. Supply it in the config file or in command line.".to_string())?
    );
    let loaded_abi = std::fs::read_to_string(abi.as_ref().unwrap())
        .map_err(|e| format!("failed to read ABI file: {}", e.to_string()))?;
    let params = matches.values_of("PARAMS").ok_or("PARAMS is not defined")?;
    let params = Some(parse_params(
        params.collect::<Vec<_>>(), &loaded_abi, method_opt.clone().unwrap()
    )?);
    let keys = matches.value_of("SIGN")
        .map(|s| s.to_string())
        .or(config.keys_path.clone());

    print_args!(address, method_opt, params, abi, keys);
    let address = load_ton_address(address.unwrap().as_str(), &config)?;

    call_contract(
        config,
        address.as_str(),
        loaded_abi,
        method,
        &params.unwrap(),
        keys,
        false,
        false,
    ).await
}

async fn runget_command(matches: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let address = matches.value_of("ADDRESS");
    let method = matches.value_of("METHOD");
    let params = matches.values_of("PARAMS");
    let params = params.map(|values| {
        json!(values.collect::<Vec<_>>()).to_string()
    });
    print_args!(address, method, params);
    let address = load_ton_address(address.unwrap(), &config)?;
    run_get_method(config, address.as_str(), method.unwrap(), params).await
}

async fn deploy_command(matches: &ArgMatches<'_>, config: Config, deploy_type: DeployType) -> Result<(), String> {
    let tvc = matches.value_of("TVC");
    let params = matches.value_of("PARAMS");
    let wc = matches.value_of("WC");
    let raw = matches.is_present("RAW");
    let output = matches.value_of("OUTPUT");
    let abi = Some(
        matches.value_of("ABI")
            .map(|s| s.to_string())
            .or(config.abi_path.clone())
            .ok_or("ABI file is not defined. Supply it in the config file or command line.".to_string())?
    );
    let keys = Some(
        matches.value_of("SIGN")
            .map(|s| s.to_string())
            .or(config.keys_path.clone())
            .ok_or("keypair file is not defined. Supply it in the config file or command line.".to_string())?
    );
    let params = Some(load_params(params.unwrap())?);
    if !config.is_json {
        print_args!(tvc, params, abi, keys, wc);
    }

    let wc = wc.map(|v| i32::from_str_radix(v, 10))
        .transpose()
        .map_err(|e| format!("failed to parse workchain id: {}", e))?
        .unwrap_or(config.wc);
    
    match deploy_type {
        DeployType::Full => deploy_contract(config, tvc.unwrap(), &abi.unwrap(), &params.unwrap(), &keys.unwrap(), wc, false).await,
        DeployType::MsgOnly => generate_deploy_message(tvc.unwrap(), &abi.unwrap(), &params.unwrap(), &keys.unwrap(), wc, raw, output).await,
        DeployType::Fee => deploy_contract(config, tvc.unwrap(), &abi.unwrap(), &params.unwrap(), &keys.unwrap(), wc, true).await,
    }
}

fn config_command(matches: &ArgMatches, config: Config, config_file: String) -> Result<(), String> {
    let mut result = Ok(());
    if !matches.is_present("LIST") {
        if let Some(clear_matches) = matches.subcommand_matches("clear") {
            let url = clear_matches.is_present("URL");
            let address = clear_matches.is_present("ADDR");
            let wallet = clear_matches.is_present("WALLET");
            let keys = clear_matches.is_present("KEYS");
            let abi = clear_matches.is_present("ABI");
            let wc = clear_matches.is_present("WC");
            let retries = clear_matches.is_present("RETRIES");
            let timeout = clear_matches.is_present("TIMEOUT");
            let depool_fee = clear_matches.is_present("DEPOOL_FEE");
            let lifetime = clear_matches.is_present("LIFETIME");
            let no_answer = clear_matches.is_present("NO_ANSWER");
            let balance_in_tons = clear_matches.is_present("BALANCE_IN_TONS");
            let local_run = clear_matches.is_present("LOCAL_RUN");
            result = clear_config(config, config_file.as_str(), url, address, wallet, abi, keys, wc, retries, timeout, depool_fee, lifetime, no_answer, balance_in_tons, local_run);
        } else if let Some(endpoint_matches) = matches.subcommand_matches("endpoint") {
            if let Some(endpoint_matches) = endpoint_matches.subcommand_matches("add") {
                let url = endpoint_matches.value_of("URL").unwrap();
                let endpoints = endpoint_matches.value_of("ENDPOINTS").unwrap();
                FullConfig::add_endpoint(config_file.as_str(), url, endpoints)?;
            } else if let Some(endpoint_matches) = endpoint_matches.subcommand_matches("remove") {
                let url = endpoint_matches.value_of("URL").unwrap();
                FullConfig::remove_endpoint(config_file.as_str(), url)?;
            } else if endpoint_matches.subcommand_matches("reset").is_some() {
                FullConfig::reset_endpoints(config_file.as_str())?;
            }
            FullConfig::print_endpoints(config_file.as_str());
            return Ok(());
        } else {
            let url = matches.value_of("URL");
            let address = matches.value_of("ADDR");
            let wallet = matches.value_of("WALLET");
            let pubkey = matches.value_of("PUBKEY");
            let keys = matches.value_of("KEYS");
            let abi = matches.value_of("ABI");
            let wc = matches.value_of("WC");
            let retries = matches.value_of("RETRIES");
            let timeout = matches.value_of("TIMEOUT");
            let depool_fee = matches.value_of("DEPOOL_FEE");
            let lifetime = matches.value_of("LIFETIME");
            let no_answer = matches.value_of("NO_ANSWER");
            let balance_in_tons = matches.value_of("BALANCE_IN_TONS");
            let local_run = matches.value_of("LOCAL_RUN");
            let async_call = matches.value_of("ASYNC_CALL");
            result = set_config(config, config_file.as_str(), url, address, wallet, pubkey, abi, keys, wc, retries, timeout, depool_fee, lifetime, no_answer, balance_in_tons, local_run, async_call);
        }
    }
    let config = match Config::from_file(config_file.as_str()) {
        Some(c) => {
            c
        },
        None => {
            Config::new()
        },
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&config)
            .map_err(|e| format!("failed to print config parameters: {}", e))?
    );
    result
}

async fn genaddr_command(matches: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let tvc = matches.value_of("TVC");
    let wc = matches.value_of("WC");
    let keys = matches.value_of("GENKEY").or(matches.value_of("SETKEY"));
    let new_keys = matches.is_present("GENKEY");
    let init_data = matches.value_of("DATA");
    let update_tvc = matches.is_present("SAVE");
    let abi = matches.value_of("ABI");
    let is_update_tvc = if update_tvc { Some("true") } else { None };
    print_args!(tvc, wc, keys, init_data, is_update_tvc);
    generate_address(config, tvc.unwrap(), abi.unwrap(), wc, keys, new_keys, init_data, update_tvc).await
}

async fn account_command(matches: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let address = matches.value_of("ADDRESS");
    let tvcname = matches.value_of("DUMPTVC");
    if !config.is_json {
        print_args!(address);
    }
    let address = load_ton_address(address.unwrap(), &config)?;
    get_account(config, address.as_str(), tvcname).await
}

async fn storage_command(matches: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let address = matches.value_of("ADDRESS");
    let period = matches.value_of("PERIOD");
    if !config.is_json {
        print_args!(address, period);
    }
    let address = load_ton_address(address.unwrap(), &config)?;
    let period = period.map(|val| {
        u32::from_str_radix(val, 10)
            .map_err(|e| format!("failed to parse period: {}", e))
    })
    .transpose()?
    .unwrap_or(DEF_STORAGE_PERIOD);
    calc_storage(config, address.as_str(), period).await
}

async fn proposal_create_command(matches: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let address = matches.value_of("ADDRESS");
    let dest = matches.value_of("DEST");
    let keys = matches.value_of("KEYS");
    let comment = matches.value_of("COMMENT");
    let lifetime = matches.value_of("LIFETIME");
    let offline = matches.is_present("OFFLINE");
    print_args!(address, comment, keys, lifetime);

    let address = load_ton_address(address.unwrap(), &config)?;
    let lifetime = lifetime.map(|val| {
        u32::from_str_radix(val, 10)
            .map_err(|e| format!("failed to parse lifetime: {}", e))
    })
    .transpose()?
    .unwrap_or(config.timeout);

    create_proposal(
        config,
        address.as_str(),
        keys,
        dest.unwrap(),
        comment.unwrap(),
        lifetime,
        offline
    ).await
}

async fn proposal_vote_command(matches: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let address = matches.value_of("ADDRESS");
    let keys = matches.value_of("KEYS");
    let id = matches.value_of("ID");
    let lifetime = matches.value_of("LIFETIME");
    let offline = matches.is_present("OFFLINE");
    print_args!(address, id, keys, lifetime);

    let address = load_ton_address(address.unwrap(), &config)?;
    let lifetime = lifetime.map(|val| {
        u32::from_str_radix(val, 10)
            .map_err(|e| format!("failed to parse lifetime: {}", e))
    })
    .transpose()?
    .unwrap_or(config.timeout);

    vote(config, address.as_str(), keys, id.unwrap(), lifetime, offline).await
}

async fn proposal_decode_command(matches: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let address = matches.value_of("ADDRESS");
    let id = matches.value_of("ID");
    print_args!(address, id);

    let address = load_ton_address(address.unwrap(), &config)?;
    decode_proposal(config, address.as_str(), id.unwrap()).await
}

async fn getconfig_command(matches: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let index = matches.value_of("INDEX");
    if !config.is_json {
        print_args!(index);
    }
    query_global_config(config, index.unwrap()).await
}

fn nodeid_command(matches: &ArgMatches) -> Result<(), String> {
    let key = matches.value_of("KEY");
    let keypair = matches.value_of("KEY_PAIR");
    print_args!(key, keypair);
    let nodeid = if let Some(key) = key {
        let vec = hex::decode(key)
            .map_err(|e| format!("failed to decode public key: {}", e))?;
        convert::nodeid_from_pubkey(&vec)?
    } else if let Some(pair) = keypair {
        let pair = crypto::load_keypair(pair)?;
        convert::nodeid_from_pubkey(&hex::decode(&pair.public).unwrap())?
    } else {
        return Err("Either public key or key pair parameter should be provided".to_owned());
    };
    println!("{}", nodeid);
    Ok(())
}

async fn sendfile_command(m: &ArgMatches<'_>, conf: Config) -> Result<(), String> {
    let boc = m.value_of("BOC");
    print_args!(boc);
    sendfile::sendfile(conf, boc.unwrap()).await
}
