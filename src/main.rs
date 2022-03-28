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
extern crate clap;
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
mod debug;
mod debug_executor;

use account::{get_account, calc_storage};
use call::{call_contract, call_contract_with_msg, generate_message, parse_params, run_get_method, run_local_for_account};
use clap::{ArgMatches, SubCommand, Arg, AppSettings, App};
use config::{Config, set_config, clear_config};
use crypto::{generate_mnemonic, extract_pubkey, generate_keypair};
use debot::{create_debot_command, debot_command};
use decode::{create_decode_command, decode_command};
use debug::{create_debug_command, debug_command};
use deploy::{deploy_contract, generate_deploy_message};
use depool::{create_depool_command, depool_command};
use helpers::{load_ton_address, load_abi, create_client_local};
use genaddr::generate_address;
use getconfig::{query_global_config, dump_blockchain_config};
use multisig::{create_multisig_command, multisig_command};
use std::{env, path::PathBuf};
use voting::{create_proposal, decode_proposal, vote};
use replay::{fetch_block_command, fetch_command, replay_command};
use ton_client::abi::{ParamsOfEncodeMessageBody, CallSet};
use crate::account::dump_accounts;
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
            dir.join(PathBuf::from(CONFIG_BASE_NAME)).to_str().unwrap().to_string()
        })
}

pub fn abi_from_matches_or_config(matches: &ArgMatches<'_>, config: &Config) -> Result<String, String> {
    Ok(matches.value_of("ABI")
        .map(|s| s.to_string())
        .or(config.abi_path.clone())
        .ok_or("ABI file is not defined. Supply it in the config file or command line.".to_string())?
    )
}

fn address_from_matches_or_config(matches: &ArgMatches<'_>, config: Config) -> Result<String, String> {
    Ok(matches.value_of("ADDRESS")
        .map(|s| s.to_string())
        .or(config.addr.clone())
        .ok_or("ADDRESS is not defined. Supply it in the config file or command line.".to_string())?
    )
}

fn parse_lifetime(lifetime: Option<&str>, config: Config) -> Result<u32, String> {
    Ok(lifetime.map(|val| {
        u32::from_str_radix(val, 10)
            .map_err(|e| format!("failed to parse lifetime: {}", e))
    })
        .transpose()?
        .unwrap_or(config.lifetime))
}

#[tokio::main]
async fn main() -> Result<(), i32> {
    main_internal().await.map_err(|err_str| {
        println!("{}", err_str);
        1
    })
}

async fn main_internal() -> Result <(), String> {
    let version_string = format!("{}", env!("CARGO_PKG_VERSION"));
    let callex_cmd = SubCommand::with_name("callex")
        .about("Sends an external message with encoded function call to the contract (alternative syntax). Deprecated use `callx` instead.")
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

    let abi_arg = Arg::with_name("ABI")
        .long("--abi")
        .takes_value(true)
        .help("Path to the contract ABI file. Can be specified in the config file.");

    let keys_arg = Arg::with_name("KEYS")
        .long("--keys")
        .takes_value(true)
        .help("Seed phrase or path to the file with keypair used to sign the message. Can be specified in the config file.");

    let callx_cmd = SubCommand::with_name("callx")
        .about("Sends an external message with encoded function call to the contract (alternative syntax).")
        .version(&*version_string)
        .author("TONLabs")
        .setting(AppSettings::AllowLeadingHyphen)
        .setting(AppSettings::TrailingVarArg)
        .setting(AppSettings::DontCollapseArgsInUsage)
        .arg(Arg::with_name("ADDRESS")
            .long("--addr")
            .takes_value(true)
            .help("Contract address. Can be specified in the config file."))
        .arg(abi_arg.clone())
        .arg(keys_arg.clone())
        .arg(Arg::with_name("METHOD")
            .help("Name of the function being called.")
            .required(true)
            .takes_value(true))
        .arg(Arg::with_name("PARAMS")
            .help("Function arguments. Must be a list of `--name value` pairs or a json string with all arguments.")
            .multiple(true));

    let tvc_arg = Arg::with_name("TVC")
        .takes_value(true)
        .required(true)
        .help("Path to the compiled smart contract (tvc file).");

    let wc_arg = Arg::with_name("WC")
        .takes_value(true)
        .long("--wc")
        .help("Workchain id of the smart contract (default value is taken from the config).");

    let deployx_cmd = SubCommand::with_name("deployx")
        .about("Deploys a smart contract to the blockchain (alternative syntax).")
        .version(&*version_string)
        .author("TONLabs")
        .setting(AppSettings::AllowLeadingHyphen)
        .setting(AppSettings::TrailingVarArg)
        .setting(AppSettings::DontCollapseArgsInUsage)
        .arg(abi_arg.clone())
        .arg(keys_arg.clone())
        .arg(wc_arg.clone())
        .arg(tvc_arg.clone())
        .arg(Arg::with_name("PARAMS")
            .help("Function arguments. Must be a list of `--name value` pairs or a json string with all arguments.")
            .multiple(true));

    let runx_cmd = SubCommand::with_name("runx")
        .about("Runs contract function locally (alternative syntax).")
        .version(&*version_string)
        .author("TONLabs")
        .setting(AppSettings::AllowLeadingHyphen)
        .setting(AppSettings::TrailingVarArg)
        .setting(AppSettings::DontCollapseArgsInUsage)
        .arg(Arg::with_name("ADDRESS")
            .long("--addr")
            .takes_value(true)
            .help("Contract address or path to the saved account state if --boc or --tvc flag is specified."))
        .arg(abi_arg.clone())
        .arg(Arg::with_name("METHOD")
            .help("Name of the function being called.")
            .required(true)
            .takes_value(true))
        .arg(Arg::with_name("PARAMS")
            .help("Function arguments. Must be a list of `--name value` pairs or a json string with all arguments.")
            .multiple(true))
        .arg(Arg::with_name("BOC")
            .long("--boc")
            .conflicts_with("TVC")
            .help("Flag that changes behavior of the command to work with the saved account state (account BOC)."))
        .arg(Arg::with_name("TVC")
            .long("--tvc")
            .conflicts_with("BOC")
            .help("Flag that changes behavior of the command to work with the saved contract state (stateInit TVC)."))
        .arg(Arg::with_name("BCCONFIG")
            .long("--bc_config")
            .requires("BOC")
            .takes_value(true)
            .help("Path to the file with blockchain config."));

    let runget_cmd = SubCommand::with_name("runget")
        .about("Runs get-method of a FIFT contract.")
        .setting(AppSettings::AllowLeadingHyphen)
        .setting(AppSettings::TrailingVarArg)
        .setting(AppSettings::DontCollapseArgsInUsage)
        .arg(Arg::with_name("ADDRESS")
            .required(true)
            .help("Contract address or path to the saved account state if --boc or --tvc flag is specified."))
        .arg(Arg::with_name("METHOD")
            .required(true)
            .help("Name of the function being called."))
        .arg(Arg::with_name("PARAMS")
            .help("Function arguments.")
            .multiple(true))
        .arg(Arg::with_name("BOC")
            .long("--boc")
            .conflicts_with("TVC")
            .help("Flag that changes behavior of the command to work with the saved account state(account BOC)."))
        .arg(Arg::with_name("TVC")
            .long("--tvc")
            .conflicts_with("BOC")
            .help("Flag that changes behavior of the command to work with the saved contract state (stateInit TVC)."))
        .arg(Arg::with_name("BCCONFIG")
            .long("--bc_config")
            .requires("BOC")
            .takes_value(true)
            .help("Path to the file with blockchain config."));

    let version_cmd = SubCommand::with_name("version")
        .about("Prints build and version info.");

    let convert_cmd = SubCommand::with_name("convert")
        .about("Converts tokens to nanotokens.")
        .subcommand(SubCommand::with_name("tokens")
            .about("Converts tokens to nanotokens.")
            .arg(Arg::with_name("AMOUNT")
                .takes_value(true)
                .required(true)
                .help("Token amount value")));

    let genphrase_cmd = SubCommand::with_name("genphrase")
        .about("Generates a seed phrase for keypair.")
        .author("TONLabs")
        .arg(Arg::with_name("DUMP_KEYPAIR")
            .long("--dump")
            .takes_value(true)
            .help("Path where to dump keypair generated from the phrase"));

    let genpubkey_cmd = SubCommand::with_name("genpubkey")
        .about("Generates a public key from the seed phrase.")
        .author("TONLabs")
        .arg(Arg::with_name("PHRASE")
            .takes_value(true)
            .required(true)
            .help("Seed phrase (12 words). Should be specified in quotes."));

    let getkeypair_cmd = SubCommand::with_name("getkeypair")
        .about("Generates a keypair from the seed phrase or private key and saves it to the file.")
        .author("TONLabs")
        .arg(Arg::with_name("KEY_FILE")
            .takes_value(true)
            .required(true)
            .help("Path to the file where to store the keypair."))
        .arg(Arg::with_name("PHRASE")
            .takes_value(true)
            .required(true)
            .help("Seed phrase (12 words) or secret (private) key. Seed phrase should be specified in quotes, secret key as 64 chars of hex."));

    let genaddr_cmd = SubCommand::with_name("genaddr")
        .setting(AppSettings::AllowNegativeNumbers)
        .about("Calculates smart contract address in different formats. By default, input tvc file isn't modified.")
        .version(&*version_string)
        .author("TONLabs")
        .arg(tvc_arg.clone())
        .arg(Arg::with_name("ABI")
            .takes_value(true)
            .required(true)
            .help("Path to the contract ABI file."))
        .arg(wc_arg.clone())
        .arg(Arg::with_name("GENKEY")
            .takes_value(true)
            .long("--genkey")
            .conflicts_with("SETKEY")
            .help("Path to the file, where a new generated keypair for the contract will be saved."))
        .arg(Arg::with_name("SETKEY")
            .takes_value(true)
            .long("--setkey")
            .conflicts_with("GENKEY")
            .help("Seed phrase or path to the file with keypair."))
        .arg(Arg::with_name("DATA")
            .takes_value(true)
            .long("--data")
            .help("Initial data to insert into the contract. Should be specified in json format."))
        .arg(Arg::with_name("SAVE")
            .long("--save")
            .help("If this flag is specified, modifies the tvc file with the keypair and initial data"));

    let sign_arg = Arg::with_name("SIGN")
        .long("--sign")
        .takes_value(true)
        .help("Seed phrase or path to the file with keypair used to sign the message. Can be specified in the config.");

    let deploy_cmd = SubCommand::with_name("deploy")
        .setting(AppSettings::AllowNegativeNumbers)
        .setting(AppSettings::AllowLeadingHyphen)
        .about("Deploys a smart contract to the blockchain.")
        .version(&*version_string)
        .author("TONLabs")
        .arg(tvc_arg.clone())
        .arg(Arg::with_name("PARAMS")
            .required(true)
            .takes_value(true)
            .help("Constructor arguments. Can be specified with a filename, which contains json data."))
        .arg(abi_arg.clone())
        .arg(sign_arg.clone())
        .arg(wc_arg.clone());

    let output_arg = Arg::with_name("OUTPUT")
        .short("-o")
        .long("--output")
        .takes_value(true)
        .help("Path to the file where to store the message.");

    let raw_arg = Arg::with_name("RAW")
        .long("--raw")
        .help("Creates raw message boc.");

    let deploy_message_cmd = deploy_cmd.clone()
        .name("deploy_message")
        .about("Generates a signed message to deploy a smart contract to the blockchain.")
        .arg(output_arg.clone())
        .arg(raw_arg.clone());

    let address_arg = Arg::with_name("ADDRESS")
        .required(true)
        .takes_value(true)
        .help("Contract address.");

    let method_arg = Arg::with_name("METHOD")
        .required(true)
        .takes_value(true)
        .help("Name of the function being called.");

    let params_arg = Arg::with_name("PARAMS")
        .required(true)
        .takes_value(true)
        .help("Function arguments. Can be specified with a filename, which contains json data.");

    let call_cmd = SubCommand::with_name("call")
        .setting(AppSettings::AllowLeadingHyphen)
        .about("Sends an external message with encoded function call to the contract.")
        .version(&*version_string)
        .author("TONLabs")
        .arg(address_arg.clone())
        .arg(method_arg.clone())
        .arg(params_arg.clone())
        .arg(abi_arg.clone())
        .arg(sign_arg.clone());

    let send_cmd = SubCommand::with_name("send")
        .about("Sends a prepared message to the contract.")
        .version(&*version_string)
        .author("TONLabs")
        .arg(Arg::with_name("MESSAGE")
            .required(true)
            .takes_value(true)
            .help("Message to send. Message data should be specified in quotes."))
        .arg(abi_arg.clone());

    let message_cmd = SubCommand::with_name("message")
        .setting(AppSettings::AllowLeadingHyphen)
        .about("Generates a signed message with encoded function call.")
        .author("TONLabs")
        .arg(address_arg.clone())
        .arg(method_arg.clone())
        .arg(params_arg.clone())
        .arg(abi_arg.clone())
        .arg(sign_arg.clone())
        .arg(Arg::with_name("LIFETIME")
            .long("--lifetime")
            .takes_value(true)
            .help("Period of time in seconds while message is valid."))
        .arg(output_arg.clone())
        .arg(raw_arg.clone());

    let body_cmd = SubCommand::with_name("body")
        .setting(AppSettings::AllowLeadingHyphen)
        .about("Generates a payload for internal function call.")
        .version(&*version_string)
        .author("TONLabs")
        .arg(method_arg.clone())
        .arg(params_arg.clone())
        .arg(abi_arg.clone());

    let run_cmd = SubCommand::with_name("run")
        .setting(AppSettings::AllowLeadingHyphen)
        .about("Runs contract function locally.")
        .version(&*version_string)
        .author("TONLabs")
        .arg(Arg::with_name("ADDRESS")
            .required(true)
            .takes_value(true)
            .help("Contract address or path to the saved account state if --boc or --tvc flag is specified."))
        .arg(method_arg.clone())
        .arg(params_arg.clone())
        .arg(abi_arg.clone())
        .arg(Arg::with_name("BOC")
            .long("--boc")
            .conflicts_with("TVC")
            .help("Flag that changes behavior of the command to work with the saved account state (account BOC)."))
        .arg(Arg::with_name("TVC")
            .long("--tvc")
            .conflicts_with("BOC")
            .help("Flag that changes behavior of the command to work with the saved contract state (stateInit TVC)."))
        .arg(Arg::with_name("BCCONFIG")
            .long("--bc_config")
            .requires("BOC")
            .takes_value(true)
            .help("Path to the file with blockchain config."));

    let config_clear_cmd = SubCommand::with_name("clear")
        .setting(AppSettings::AllowLeadingHyphen)
        .about("Resets certain default values for options in the config file. Resets all values if used without options.")
        .arg(Arg::with_name("URL")
            .long("--url")
            .help("Url to connect."))
        .arg(Arg::with_name("ABI")
            .long("--abi")
            .help("Path to the contract ABI file."))
        .arg(Arg::with_name("KEYS")
            .long("--keys")
            .help("Path to the file with keypair."))
        .arg(Arg::with_name("ADDR")
            .long("--addr")
            .help("Contract address."))
        .arg(Arg::with_name("WALLET")
            .long("--wallet")
            .help("Multisig wallet address."))
        .arg(Arg::with_name("WC")
            .long("--wc")
            .help("Workchain id."))
        .arg(Arg::with_name("RETRIES")
            .long("--retries")
            .help("Number of attempts to call smart contract function if previous attempt was unsuccessful."))
        .arg(Arg::with_name("TIMEOUT")
            .long("--timeout")
            .help("Network `wait_for` timeout in ms."))
        .arg(Arg::with_name("DEPOOL_FEE")
            .long("--depool_fee")
            .help("Value added to the message sent to depool to cover it's fees (change will be returned)."))
        .arg(Arg::with_name("LIFETIME")
            .long("--lifetime")
            .help("Period of time in seconds while message is valid."))
        .arg(Arg::with_name("NO_ANSWER")
            .long("--no-answer")
            .help("Flag whether to wait for depool answer when calling a depool function."))
        .arg(Arg::with_name("BALANCE_IN_TONS")
            .long("--balance_in_tons")
            .help("Print balance for account command in tons. If false balance is printed in nanotons."))
        .arg(Arg::with_name("LOCAL_RUN")
            .long("--local_run")
            .help("Enable preliminary local run before deploy and call commands."));

    let config_endpoint_cmd = SubCommand::with_name("endpoint")
        .about("Commands to work with the map of endpoints.")
        .subcommand(SubCommand::with_name("add")
            .about("Add endpoints list.")
            .arg(Arg::with_name("URL")
                .required(true)
                .takes_value(true)
                .help("Url of the endpoints list."))
            .arg(Arg::with_name("ENDPOINTS")
                .required(true)
                .takes_value(true)
                .help("List of endpoints.")))
        .subcommand(SubCommand::with_name("remove")
            .about("Remove endpoints list.")
            .arg(Arg::with_name("URL")
                .required(true)
                .takes_value(true)
                .help("Url of the endpoints list.")))
        .subcommand(SubCommand::with_name("reset")
            .about("Reset the endpoints map."))
        .subcommand(SubCommand::with_name("print")
            .about("Print current endpoints map."));

    let config_cmd = SubCommand::with_name("config")
        .setting(AppSettings::AllowLeadingHyphen)
        .about("Allows to tune certain default values for options in the config file.")
        .version(&*version_string)
        .author("TONLabs")
        .arg(Arg::with_name("URL")
            .long("--url")
            .takes_value(true)
            .help("Url to connect."))
        .arg(Arg::with_name("ABI")
            .long("--abi")
            .takes_value(true)
            .help("Path to the contract ABI file."))
        .arg(Arg::with_name("KEYS")
            .long("--keys")
            .takes_value(true)
            .help("Path to the file with keypair."))
        .arg(Arg::with_name("ADDR")
            .long("--addr")
            .takes_value(true)
            .help("Contract address."))
        .arg(Arg::with_name("WALLET")
            .long("--wallet")
            .takes_value(true)
            .help("Multisig wallet address."))
        .arg(Arg::with_name("PUBKEY")
            .long("--pubkey")
            .takes_value(true)
            .help("User public key. Used by DeBot Browser."))
        .arg(Arg::with_name("WC")
            .long("--wc")
            .takes_value(true)
            .help("Workchain id."))
        .arg(Arg::with_name("RETRIES")
            .long("--retries")
            .takes_value(true)
            .help("Number of attempts to call smart contract function if previous attempt was unsuccessful."))
        .arg(Arg::with_name("TIMEOUT")
            .long("--timeout")
            .takes_value(true)
            .help("Network `wait_for` timeout in ms."))
        .arg(Arg::with_name("MSG_TIMEOUT")
            .long("--message_processing_timeout")
            .takes_value(true)
            .help("Network message processing timeout in ms."))
        .arg(Arg::with_name("LIST")
            .long("--list")
            .conflicts_with_all(&["OUT_OF_SYNC", "NO_ANSWER", "ASYNC_CALL", "LOCAL_RUN", "BALANCE_IN_TONS", "LIFETIME", "DEPOOL_FEE", "PUBKEY", "URL", "ABI", "KEYS", "ADDR", "RETRIES", "TIMEOUT", "WC", "WALLET"])
            .help("Prints all config parameters."))
        .arg(Arg::with_name("DEPOOL_FEE")
            .long("--depool_fee")
            .takes_value(true)
            .help("Value added to the message sent to depool to cover it's fees (change will be returned)."))
        .arg(Arg::with_name("LIFETIME")
            .long("--lifetime")
            .takes_value(true)
            .help("Period of time in seconds while message is valid. Change of this parameter may affect \"out_of_sync\" parameter, because \"lifetime\" should be at least 2 times greater than \"out_of_sync\"."))
        .arg(Arg::with_name("NO_ANSWER")
            .long("--no-answer")
            .takes_value(true)
            .help("Flag whether to wait for depool answer when calling a depool function."))
        .arg(Arg::with_name("BALANCE_IN_TONS")
            .long("--balance_in_tons")
            .takes_value(true)
            .help("Print balance for account command in tons. If false balance is printed in nanotons."))
        .arg(Arg::with_name("LOCAL_RUN")
            .long("--local_run")
            .takes_value(true)
            .help("Enable preliminary local run before deploy and call commands."))
        .arg(Arg::with_name("ASYNC_CALL")
            .long("--async_call")
            .takes_value(true)
            .help("Disables wait for transaction to appear in the network after call command."))
        .arg(Arg::with_name("OUT_OF_SYNC")
            .long("--out_of_sync")
            .takes_value(true)
            .help("Network connection \"out_of_sync_threshold\" parameter in seconds. Mind that it cant exceed half of the \"lifetime\" parameter."))
        .subcommand(config_clear_cmd)
        .subcommand(config_endpoint_cmd);

    let account_cmd = SubCommand::with_name("account")
        .setting(AppSettings::AllowLeadingHyphen)
        .about("Obtains and prints account information.")
        .version(&*version_string)
        .author("TONLabs")
        .arg(Arg::with_name("ADDRESS")
            .required(true)
            .takes_value(true)
            .help("List of addresses.")
            .multiple(true))
        .arg(Arg::with_name("DUMPTVC")
            .long("--dumptvc")
            .short("-d")
            .takes_value(true)
            .conflicts_with("DUMPBOC")
            .help("Dumps account StateInit to the specified tvc file. Works only if one address was given."))
        .arg(Arg::with_name("DUMPBOC")
            .long("--dumpboc")
            .short("-b")
            .takes_value(true)
            .conflicts_with("DUMPTVC")
            .help("Dumps the whole account state boc to the specified file. Works only if one address was given. Use 'tonos-cli dump account` to dump several accounts."));

    let fee_cmd = SubCommand::with_name("fee")
        .about("Calculates fees for executing message or account storage fee.")
        .subcommand(SubCommand::with_name("storage")
            .setting(AppSettings::AllowLeadingHyphen)
            .about("Gets account storage fee for specified period in nanotons.")
            .version(&*version_string)
            .author("TONLabs")
            .arg(address_arg.clone())
            .arg(Arg::with_name("PERIOD")
                .long("--period")
                .short("-p")
                .takes_value(true)
                .help("Time period in seconds (default value is 1 year).")))
        .subcommand(deploy_cmd.clone()
            .about("Executes deploy locally, calculates fees and prints table of fees in nanotons."))
        .subcommand(call_cmd.clone()
            .about("Executes call locally, calculates fees and prints table of all fees in nanotons."));

    let proposal_cmd = SubCommand::with_name("proposal")
        .subcommand(
            SubCommand::with_name("create")
                .about("Submits a proposal transaction in the multisignature wallet with a text comment.")
                .arg(address_arg.clone().help("Address of the multisignature wallet."))
                .arg(Arg::with_name("DEST")
                    .required(true)
                    .takes_value(true)
                    .help("Address of the proposal contract."))
                .arg(Arg::with_name("COMMENT")
                    .required(true)
                    .takes_value(true)
                    .help("Proposal description (max symbols 382)."))
                .arg(Arg::with_name("KEYS")
                    .required(true)
                    .takes_value(true)
                    .help("Seed phrase or path to the keypair file."))
                .arg(Arg::with_name("OFFLINE")
                    .short("-f")
                    .long("--offline")
                    .help("Prints signed message to terminal instead of sending it."))
                .arg(Arg::with_name("LIFETIME")
                    .short("-l")
                    .long("--lifetime")
                    .takes_value(true)
                    .help("Period of time in seconds while message is valid.")))
        .subcommand(
            SubCommand::with_name("vote")
                .about("Confirms a proposal transaction in the multisignature wallet.")
                .arg(address_arg.clone().help("Address of the multisignature wallet."))
                .arg(Arg::with_name("ID")
                    .required(true)
                    .takes_value(true)
                    .help("Proposal transaction id."))
                .arg(Arg::with_name("KEYS")
                    .required(true)
                    .takes_value(true)
                    .help("Seed phrase or path to the keypair file."))
                .arg(Arg::with_name("OFFLINE")
                    .short("-f")
                    .long("--offline")
                    .help("Prints signed message to terminal instead of sending it."))
                .arg(Arg::with_name("LIFETIME")
                    .short("-l")
                    .long("--lifetime")
                    .takes_value(true)
                    .help("Period of time in seconds while message is valid.")))
        .subcommand(
            SubCommand::with_name("decode")
                .about("Prints a comment string from the proposal transaction.")
                .arg(address_arg.clone().help("Address of the multisignature wallet."))
                .arg(Arg::with_name("ID")
                    .required(true)
                    .takes_value(true)
                    .help("Proposal transaction id.")));

    let getconfig_cmd = SubCommand::with_name("getconfig")
        .about("Reads the global configuration parameter with defined index.")
        .arg(Arg::with_name("INDEX")
            .takes_value(true)
            .help("Parameter index. If not specified, command will print all config parameters."));

    let bcconfig_cmd = SubCommand::with_name("dump")
        .about("Commands to dump network entities.")
        .version(&*version_string)
        .author("TONLabs")
        .subcommand(SubCommand::with_name("config")
            .about("Dumps the blockchain config for the last key block.")
            .arg(Arg::with_name("PATH")
                .required(true)
                .takes_value(true)
                .help("Path to the file where to save the blockchain config.")))
        .subcommand(SubCommand::with_name("account")
            .about("Dumps state of given accounts.")
            .setting(AppSettings::AllowLeadingHyphen)
            .arg(Arg::with_name("ADDRESS")
                .required(true)
                .takes_value(true)
                .help("List of addresses.")
                .multiple(true))
            .arg(Arg::with_name("PATH")
                .takes_value(true)
                .long("--path")
                .short("-p")
                .help("Path to folder where to store the dumped accounts. Default value is \".\".")));

    let nodeid_cmd = SubCommand::with_name("nodeid")
        .about("Calculates node ID from the validator public key")
        .arg(Arg::with_name("KEY")
            .long("--pubkey")
            .takes_value(true)
            .help("Validator public key."))
        .arg(Arg::with_name("KEY_PAIR")
            .long("--keypair")
            .takes_value(true)
            .help("Validator seed phrase or path to the file with keypair."));

    let sendfile_cmd = SubCommand::with_name("sendfile")
        .about("Sends the boc file with an external inbound message to account.")
        .arg(Arg::with_name("BOC")
            .required(true)
            .takes_value(true)
            .help("Message boc file."));

    let fetch_block_cmd = SubCommand::with_name("fetch-block")
        .about("Fetches a block.")
        .arg(Arg::with_name("BLOCKID")
            .required(true)
            .takes_value(true)
            .help("Block ID."))
        .arg(Arg::with_name("OUTPUT")
            .required(true)
            .takes_value(true)
            .help("Output file name"));

    let fetch_cmd = SubCommand::with_name("fetch")
        .about("Fetches account's zerostate and transactions.")
        .setting(AppSettings::AllowLeadingHyphen)
        .arg(address_arg.clone().help("Account address to fetch zerostate and txns for."))
        .arg(Arg::with_name("OUTPUT")
            .required(true)
            .takes_value(true)
            .help("Output file name"));

    let replay_cmd = SubCommand::with_name("replay")
        .about("Replays account's transactions starting from zerostate.")
        .arg(Arg::with_name("CONFIG_TXNS")
            .required(true)
            .takes_value(true)
            .help("File containing zerostate and txns of -1:555..5 account."))
        .arg(Arg::with_name("INPUT_TXNS")
            .required(true)
            .takes_value(true)
            .help("File containing zerostate and txns of the account to replay."))
        .arg(Arg::with_name("TXNID")
            .required(true)
            .takes_value(true)
            .help("Dump account state before this transaction ID and stop replaying."));

    let matches = App::new("tonos_cli")
        .version(&*format!("{}\nCOMMIT_ID: {}\nBUILD_DATE: {}\nCOMMIT_DATE: {}\nGIT_BRANCH: {}",
                           env!("CARGO_PKG_VERSION"),
                           env!("BUILD_GIT_COMMIT"),
                           env!("BUILD_TIME"),
                           env!("BUILD_GIT_DATE"),
                           env!("BUILD_GIT_BRANCH"))
        )
        .author("TONLabs")
        .about("TONLabs console tool for TON")
        .arg(Arg::with_name("NETWORK")
            .help("Network to connect.")
            .short("-u")
            .long("--url")
            .takes_value(true))
        .arg(Arg::with_name("CONFIG")
            .help("Path to the tonos-cli configuration file.")
            .short("-c")
            .long("--config")
            .takes_value(true))
        .arg(Arg::with_name("JSON")
            .help("Cli prints output in json format.")
            .short("-j")
            .long("--json"))
        .subcommand(version_cmd)
        .subcommand(convert_cmd)
        .subcommand(genphrase_cmd)
        .subcommand(genpubkey_cmd)
        .subcommand(getkeypair_cmd)
        .subcommand(genaddr_cmd)
        .subcommand(deploy_cmd)
        .subcommand(deploy_message_cmd)
        .subcommand(callex_cmd)
        .subcommand(call_cmd)
        .subcommand(send_cmd)
        .subcommand(message_cmd)
        .subcommand(body_cmd)
        .subcommand(run_cmd)
        .subcommand(runget_cmd)
        .subcommand(config_cmd)
        .subcommand(account_cmd)
        .subcommand(fee_cmd)
        .subcommand(proposal_cmd)
        .subcommand(create_multisig_command())
        .subcommand(create_depool_command())
        .subcommand(create_decode_command())
        .subcommand(create_debot_command())
        .subcommand(create_debug_command())
        .subcommand(getconfig_cmd)
        .subcommand(bcconfig_cmd)
        .subcommand(nodeid_cmd)
        .subcommand(sendfile_cmd)
        .subcommand(fetch_block_cmd)
        .subcommand(fetch_cmd)
        .subcommand(replay_cmd)
        .subcommand(callx_cmd)
        .subcommand(deployx_cmd)
        .subcommand(runx_cmd)
        .setting(AppSettings::SubcommandRequired)
        .get_matches();

    let is_json = matches.is_present("JSON");

    command_parser(&matches, is_json).await
        .map_err(|e| {
            if !is_json {
                format!("Error: {}", e)
            } else {
                let err: serde_json::Value = json!(e);
                let res = json!({"Error": err});
                serde_json::to_string_pretty(&res)
                    .unwrap_or("{{ \"JSON serialization error\" }}".to_string())
            }
        })
}

async fn command_parser(matches: &ArgMatches<'_>, is_json: bool) -> Result <(), String> {
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
            return convert_tokens(m, conf);
        }
    }
    if let Some(m) = matches.subcommand_matches("callex") {
        return callex_command(m, conf).await;
    }
    if let Some(m) = matches.subcommand_matches("callx") {
        return callx_command(m, conf, CallType::Call).await;
    }
    if let Some(m) = matches.subcommand_matches("runx") {
        return if m.is_present("BOC") {
            runx_account(m, conf).await
        } else {
            callx_command(m, conf, CallType::Run).await
        };
    }
    if let Some(m) = matches.subcommand_matches("deployx") {
        return deployx_command(m, conf).await;
    }
    if let Some(m) = matches.subcommand_matches("call") {
        return call_command(m, conf, CallType::Call).await;
    }
    if let Some(m) = matches.subcommand_matches("run") {
        return if m.is_present("BOC") || m.is_present("TVC") {
            run_account(m, conf).await
        } else {
            call_command(m, conf, CallType::Run).await
        };
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
    if let Some(matches) = matches.subcommand_matches("dump") {
        if let Some(m) = matches.subcommand_matches("config") {
            return dump_bc_config_command(m, conf).await;
        }
        if let Some(m) = matches.subcommand_matches("account") {
            return dump_accounts_command(m, conf).await;
        }
    }
    if let Some(m) = matches.subcommand_matches("nodeid") {
        return nodeid_command(m, conf);
    }
    if let Some(m) = matches.subcommand_matches("sendfile") {
        return sendfile_command(m, conf).await;
    }
    if let Some(m) = matches.subcommand_matches("decode") {
        return decode_command(m, conf).await;
    }
    if let Some(m) = matches.subcommand_matches("debug") {
        return debug_command(m, conf).await;
    }
    if let Some(m) = matches.subcommand_matches("debot") {
        return debot_command(m, conf).await;
    }
    if let Some(m) = matches.subcommand_matches("fetch-block") {
        return fetch_block_command(m, conf).await;
    }
    if let Some(m) = matches.subcommand_matches("fetch") {
        return fetch_command(m, conf).await;
    }
    if let Some(m) = matches.subcommand_matches("replay") {
        return replay_command(m).await;
    }
    if let Some(_) = matches.subcommand_matches("version") {
        if conf.is_json {
            println!("{{");
            println!(r#"  "tonos-cli": "{}","#, env!("CARGO_PKG_VERSION"));
            println!(r#"  "COMMIT_ID": "{}","#, env!("BUILD_GIT_COMMIT"));
            println!(r#"  "BUILD_DATE": "{}","#, env!("BUILD_TIME"));
            println!(r#"  "COMMIT_DATE": "{}","#, env!("BUILD_GIT_DATE"));
            println!(r#"  "GIT_BRANCH": "{}""#, env!("BUILD_GIT_BRANCH"));
            println!("}}");
        } else {
            println!(
                "tonos-cli {}\nCOMMIT_ID: {}\nBUILD_DATE: {}\nCOMMIT_DATE: {}\nGIT_BRANCH: {}",
                env!("CARGO_PKG_VERSION"),
                env!("BUILD_GIT_COMMIT"),
                env!("BUILD_TIME"),
                env!("BUILD_GIT_DATE"),
                env!("BUILD_GIT_BRANCH")
            );
        }
        return Ok(());
    }
    Err("invalid arguments".to_string())
}

fn convert_tokens(matches: &ArgMatches, config: Config) -> Result<(), String> {
    let amount = matches.value_of("AMOUNT").unwrap();
    let result = convert::convert_token(amount)?;
    if config.is_json {
        let result = json!({"value": result});
        println!("{}", serde_json::to_string_pretty(&result)
            .unwrap_or("Failed to serialize the result".to_string()));
    } else {
        println!("{}", result);
    }
    Ok(())
}

fn genphrase_command(matches: &ArgMatches, config: Config) -> Result<(), String> {
    generate_mnemonic(matches.value_of("DUMP_KEYPAIR"), config)
}

fn genpubkey_command(matches: &ArgMatches, _config: Config) -> Result<(), String> {
    let mnemonic = matches.value_of("PHRASE").unwrap();
    extract_pubkey(mnemonic, _config.is_json)
}

fn getkeypair_command(matches: &ArgMatches, config: Config) -> Result<(), String> {
    let key_file = matches.value_of("KEY_FILE");
    let phrase = matches.value_of("PHRASE");
    if !config.is_json {
        print_args!(key_file, phrase);
    }
    generate_keypair(key_file.unwrap(), phrase.unwrap(), config)
}

async fn send_command(matches: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let message = matches.value_of("MESSAGE");
    let abi = Some(abi_from_matches_or_config(matches, &config)?);

    if !config.is_json {
        print_args!(message, abi);
    }

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
    let abi = Some(abi_from_matches_or_config(matches, &config)?);
    let params = Some(load_params(params.unwrap())?);
    if !config.is_json {
        print_args!(method, params, abi, output);
    }

    let params = serde_json::from_str(&params.unwrap())
        .map_err(|e| format!("arguments are not in json format: {}", e))?;

    let abi = std::fs::read_to_string(abi.unwrap())
        .map_err(|e| format!("failed to read ABI file: {}", e.to_string()))?;


    let client = create_client_local()?;
    let body = ton_client::abi::encode_message_body(
        client.clone(),
        ParamsOfEncodeMessageBody {
            abi: load_abi(&abi)?,
            call_set: CallSet::some_with_function_and_input(&method.unwrap(), params)
                .ok_or("failed to create CallSet with specified parameters.")?,
            is_internal: true,
            ..Default::default()
        },
    ).await
    .map_err(|e| format!("failed to encode body: {}", e))
    .map(|r| r.body)?;

    if !config.is_json {
        println!("Message body: {}", body);
    } else {
        println!("{{");
        println!("  \"Message\": \"{}\"", body);
        println!("}}");
    }

    Ok(())
}

fn unpack_alternative_params(matches: &ArgMatches<'_>, abi: &str, method: &str) -> Result<Option<String>, String> {
    let params = if matches.is_present("PARAMS") {
        matches.values_of("PARAMS").unwrap().collect::<Vec<_>>()
    } else {
        vec!("{}")
    };
    Ok(Some(parse_params(params, abi, method)?))
}

async fn runx_account(matches: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let account = matches.value_of("ADDRESS");
    let method = matches.value_of("METHOD");
    let abi = Some(abi_from_matches_or_config(matches, &config)?);

    let loaded_abi = std::fs::read_to_string(abi.as_ref().unwrap())
        .map_err(|e| format!("failed to read ABI file: {}", e.to_string()))?;

    let params = unpack_alternative_params(
        matches,
        &loaded_abi,
        method.clone().unwrap()
    )?;

    if !config.is_json {
        print_args!(account, method, params, abi);
    }
    let bc_config = matches.value_of("BCCONFIG");
    run_local_for_account(config,
                          account.unwrap(),
                          loaded_abi,
                          method.unwrap(),
                          &params.unwrap(),
                          bc_config,
                          matches.is_present("TVC"),
    ).await
}

async fn run_account(matches: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let account = matches.value_of("ADDRESS");
    let method = matches.value_of("METHOD");
    let params = matches.value_of("PARAMS");

    let abi = Some(abi_from_matches_or_config(matches, &config)?);

    let params = Some(load_params(params.unwrap())?);
    if !config.is_json {
        print_args!(account, method, params, abi);
    }

    let abi = std::fs::read_to_string(abi.unwrap())
        .map_err(|e| format!("failed to read ABI file: {}", e.to_string()))?;

    let bc_config = matches.value_of("BCCONFIG");
    run_local_for_account(config,
    account.unwrap(),
        abi,
        method.unwrap(),
        &params.unwrap(),
        bc_config,
        matches.is_present("TVC"),
    ).await
}

async fn call_command(matches: &ArgMatches<'_>, config: Config, call: CallType) -> Result<(), String> {
    let address = matches.value_of("ADDRESS");
    let method = matches.value_of("METHOD");
    let params = matches.value_of("PARAMS");
    let lifetime = matches.value_of("LIFETIME");
    let raw = matches.is_present("RAW");
    let output = matches.value_of("OUTPUT");

    let abi = Some(abi_from_matches_or_config(matches, &config)?);

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

async fn callx_command(matches: &ArgMatches<'_>, config: Config, call_type: CallType) -> Result<(), String> {
    let method = matches.value_of("METHOD");
    let address = Some(address_from_matches_or_config(matches, config.clone())?);
    let abi = Some(abi_from_matches_or_config(matches, &config)?);
    let loaded_abi = std::fs::read_to_string(abi.as_ref().unwrap())
        .map_err(|e| format!("failed to read ABI file: {}", e.to_string()))?;

    let params = unpack_alternative_params(
        matches,
        &loaded_abi,
        method.clone().unwrap()
    )?;
    let keys = if let CallType::Call = call_type {
        matches.value_of("KEYS")
            .map(|s| s.to_string())
            .or(config.keys_path.clone())
    } else {
        None
    };

    if !config.is_json {
        print_args!(address, method, params, abi, keys);
    }

    let address = load_ton_address(address.unwrap().as_str(), &config)?;

    call_contract(
        config,
        address.as_str(),
        loaded_abi,
        method.unwrap(),
        &params.unwrap(),
        keys,
        if let CallType::Call = call_type { false } else { true },
        false,
    ).await
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
    let abi = Some(abi_from_matches_or_config(matches, &config)?);
    let loaded_abi = std::fs::read_to_string(abi.as_ref().unwrap())
        .map_err(|e| format!("failed to read ABI file: {}", e.to_string()))?;
    let params = matches.values_of("PARAMS").ok_or("PARAMS is not defined")?;
    let params = Some(parse_params(
        params.collect::<Vec<_>>(), &loaded_abi, method_opt.clone().unwrap()
    )?);
    let keys = matches.value_of("SIGN")
        .map(|s| s.to_string())
        .or(config.keys_path.clone());

    if !config.is_json {
        print_args!(address, method_opt, params, abi, keys);
    }

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
    if !config.is_json {
        print_args!(address, method, params);
    }
    let is_local = matches.is_present("BOC") || matches.is_present("TVC");
    let is_tvc = matches.is_present("TVC");
    let address =  if is_local {
        address.unwrap().to_string()
    } else {
        load_ton_address(address.unwrap(), &config)?
    };
    let bc_config = matches.value_of("BCCONFIG");
    run_get_method(config, &address, method.unwrap(), params, is_local, is_tvc, bc_config).await
}

fn wc_from_matches_or_config(matches: &ArgMatches<'_>, config: Config) -> Result<i32 ,String> {
    Ok(matches.value_of("WC")
        .map(|v| i32::from_str_radix(v, 10))
        .transpose()
        .map_err(|e| format!("failed to parse workchain id: {}", e))?
        .unwrap_or(config.wc))
}

async fn deploy_command(matches: &ArgMatches<'_>, config: Config, deploy_type: DeployType) -> Result<(), String> {
    let tvc = matches.value_of("TVC");
    let params = matches.value_of("PARAMS");
    let wc = wc_from_matches_or_config(matches, config.clone())?;
    let raw = matches.is_present("RAW");
    let output = matches.value_of("OUTPUT");
    let abi = Some(abi_from_matches_or_config(matches, &config)?);
    let keys = matches.value_of("SIGN")
            .map(|s| s.to_string())
            .or(config.keys_path.clone());

    let params = Some(load_params(params.unwrap())?);
    if !config.is_json {
        let opt_wc = Some(format!("{}", wc));
        print_args!(tvc, params, abi, keys, opt_wc);
    }
    match deploy_type {
        DeployType::Full => deploy_contract(config, tvc.unwrap(), &abi.unwrap(), &params.unwrap(), keys, wc, false).await,
        DeployType::MsgOnly => generate_deploy_message(tvc.unwrap(), &abi.unwrap(), &params.unwrap(), keys, wc, raw, output, config.is_json).await,
        DeployType::Fee => deploy_contract(config, tvc.unwrap(), &abi.unwrap(), &params.unwrap(), keys, wc, true).await,
    }
}

async fn deployx_command(matches: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let tvc = matches.value_of("TVC");
    let wc = wc_from_matches_or_config(matches, config.clone())?;
    let abi = Some(abi_from_matches_or_config(matches, &config)?);
    let loaded_abi = std::fs::read_to_string(abi.as_ref().unwrap())
        .map_err(|e| format!("failed to read ABI file: {}", e.to_string()))?;
    let params = unpack_alternative_params(
        matches,
        &loaded_abi,
        "constructor"
    )?;
    let keys = matches.value_of("KEYS")
        .map(|s| s.to_string())
        .or(config.keys_path.clone());

    if !config.is_json {
        let opt_wc = Some(format!("{}", wc));
        print_args!(tvc, params, abi, keys, opt_wc);
    }
    deploy_contract(config, tvc.unwrap(), &abi.unwrap(), &params.unwrap(), keys, wc, false).await
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
            let msg_timeout = matches.value_of("MSG_TIMEOUT");
            let depool_fee = matches.value_of("DEPOOL_FEE");
            let lifetime = matches.value_of("LIFETIME");
            let no_answer = matches.value_of("NO_ANSWER");
            let balance_in_tons = matches.value_of("BALANCE_IN_TONS");
            let local_run = matches.value_of("LOCAL_RUN");
            let async_call = matches.value_of("ASYNC_CALL");
            let out_of_sync = matches.value_of("OUT_OF_SYNC");
            result = set_config(config, config_file.as_str(), url, address, wallet, pubkey, abi, keys, wc, retries, timeout, msg_timeout, depool_fee, lifetime, no_answer, balance_in_tons, local_run, async_call, out_of_sync);
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
    let new_keys = matches.is_present("GENKEY") ;
    let init_data = matches.value_of("DATA");
    let update_tvc = matches.is_present("SAVE");
    let abi = matches.value_of("ABI");
    let is_update_tvc = if update_tvc { Some("true") } else { None };
    if !config.is_json {
        print_args!(tvc, wc, keys, init_data, is_update_tvc);
    }
    generate_address(config, tvc.unwrap(), abi.unwrap(), wc, keys, new_keys, init_data, update_tvc).await
}

async fn account_command(matches: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let addresses_list = matches.values_of("ADDRESS").unwrap().collect::<Vec<_>>();
    let mut formatted_list = vec![];
    for address in addresses_list.iter() {
        let formatted = load_ton_address(address, &config)?;
        formatted_list.push(formatted);
    }
    let tvcname = matches.value_of("DUMPTVC");
    let bocname = matches.value_of("DUMPBOC");
    let addresses = Some(formatted_list.join(", "));
    if !config.is_json {
        print_args!(addresses);
    }
    get_account(config, formatted_list, tvcname, bocname).await
}

async fn dump_accounts_command(matches: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let addresses_list = matches.values_of("ADDRESS").unwrap().collect::<Vec<_>>();
    let mut formatted_list = vec![];
    for address in addresses_list.iter() {
        let formatted = load_ton_address(address, &config)?;
        formatted_list.push(formatted);
    }
    let path = matches.value_of("PATH");
    let addresses = Some(formatted_list.join(", "));
    if !config.is_json {
        print_args!(addresses, path);
    }
    dump_accounts(config, formatted_list, path).await
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
    if !config.is_json {
        print_args!(address, comment, keys, lifetime);
    }
    let address = load_ton_address(address.unwrap(), &config)?;
    let lifetime = parse_lifetime(lifetime, config.clone())?;

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
    if !config.is_json {
        print_args!(address, id, keys, lifetime);
    }
    let address = load_ton_address(address.unwrap(), &config)?;
    let lifetime = parse_lifetime(lifetime, config.clone())?;

    vote(config, address.as_str(), keys, id.unwrap(), lifetime, offline).await?;
    println!("{{}}");
    Ok(())
}

async fn proposal_decode_command(matches: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let address = matches.value_of("ADDRESS");
    let id = matches.value_of("ID");
    if !config.is_json {
        print_args!(address, id);
    }
    let address = load_ton_address(address.unwrap(), &config)?;
    decode_proposal(config, address.as_str(), id.unwrap()).await
}

async fn getconfig_command(matches: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let index = matches.value_of("INDEX");
    if !config.is_json {
        print_args!(index);
    }
    query_global_config(config, index).await
}

async fn dump_bc_config_command(matches: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let path = matches.value_of("PATH");
    if !config.is_json {
        print_args!(path);
    }
    dump_blockchain_config(config, path.unwrap()).await
}

fn nodeid_command(matches: &ArgMatches, config: Config) -> Result<(), String> {
    let key = matches.value_of("KEY");
    let keypair = matches.value_of("KEY_PAIR");
    if !config.is_json {
        print_args!(key, keypair);
    }
    let nodeid = if let Some(key) = key {
        let vec = hex::decode(key)
            .map_err(|e| format!("failed to decode public key: {}", e))?;
        convert::nodeid_from_pubkey(&vec)?
    } else if let Some(pair) = keypair {
        let pair = crypto::load_keypair(pair)?;
        convert::nodeid_from_pubkey(&hex::decode(&pair.public)
            .map_err(|e| format!("failed to decode public key: {}", e))?)?
    } else {
        return Err("Either public key or key pair parameter should be provided".to_owned());
    };
    if !config.is_json {
        println!("{}", nodeid);
    } else {
        println!("{{");
        println!("  \"nodeid\": \"{}\"", nodeid);
        println!("}}");
    }
    Ok(())
}

async fn sendfile_command(m: &ArgMatches<'_>, conf: Config) -> Result<(), String> {
    let boc = m.value_of("BOC");
    if !conf.is_json {
        print_args!(boc);
    }
    sendfile::sendfile(conf, boc.unwrap()).await
}
