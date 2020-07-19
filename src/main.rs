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
#[macro_use]
extern crate clap;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod account;
mod call;
mod config;
mod convert;
mod crypto;
mod deploy;
mod genaddr;
mod getconfig;
mod helpers;
mod image;
mod multisig;
mod voting;

use account::get_account;
use call::{call_contract, call_contract_with_msg, generate_message, parse_params, run_get_method};
use clap::{ArgMatches, SubCommand, Arg, AppSettings};
use config::{Config, set_config};
use crypto::{generate_mnemonic, extract_pubkey, generate_keypair};
use deploy::deploy_contract;
use genaddr::generate_address;
use getconfig::query_global_config;
use multisig::{create_multisig_command, multisig_command};
use std::{env, path::PathBuf};
use voting::{create_proposal, decode_proposal, vote};

const VERBOSE_MODE: bool = true;
const DEF_MSG_LIFETIME: u32 = 30;
const CONFIG_BASE_NAME: &'static str = "tonlabs-cli.conf.json";

enum CallType {
    Run,
    Call,
    Msg,
}

macro_rules! print_args {
    ($m:ident, $( $arg:ident ),* ) => {
        if ($m.is_present("VERBOSE") || VERBOSE_MODE) {
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

fn value_of_address(m: &ArgMatches, conf: &Config) -> Result<String, String> {
    m.value_of("ADDRESS")
        .map(|s| s.to_string())
        .or(conf.addr.clone())
        .ok_or("ADDRESS is not defined".to_string())
}

fn main() -> Result<(), i32> {    
    main_internal().map_err(|err_str| {
        println!("Error: {}", err_str);
        1
    })
}

fn main_internal() -> Result <(), String> {
    let build_info = match option_env!("BUILD_INFO") {
        Some(s) => s,
        None => "none",
    };

    let callex_sub_command = SubCommand::with_name("callex")
        .about("Sends external message to contract with encoded function call.")
        .setting(AppSettings::AllowMissingPositional)
        .setting(AppSettings::AllowLeadingHyphen)  
        .setting(AppSettings::TrailingVarArg)
        .setting(AppSettings::DontCollapseArgsInUsage)
        .arg(Arg::with_name("METHOD")
            .help("Name of the calling method."))
        .arg(Arg::with_name("ADDRESS")
            .help("Contract address."))
        .arg(Arg::with_name("ABI")
            .help("Path to contract ABI file."))
        .arg(Arg::with_name("SIGN")
            .help("Path to keypair file used to sign message."))
        .arg(Arg::with_name("PARAMS")
            .help("Method arguments. Must be a list of --name value ... pairs or a json string with all arguments.")
            .multiple(true));

    let runget_sub_command = SubCommand::with_name("runget")
        .about("Runs contract get-method.")
        .setting(AppSettings::AllowLeadingHyphen)  
        .setting(AppSettings::TrailingVarArg)
        .setting(AppSettings::DontCollapseArgsInUsage)
        .arg(Arg::with_name("ADDRESS")
            .required(true)
            .help("Contract address."))
        .arg(Arg::with_name("METHOD")
            .required(true)
            .help("Name of the calling method."))
        .arg(Arg::with_name("PARAMS")
            .help("Arguments for the contract method.")
            .multiple(true));

    let matches = clap_app! (tonlabs_cli =>
        (version: &*format!("0.1 ({})", build_info))
        (author: "TONLabs")
        (about: "TONLabs console tool for TON")
        (@arg NETWORK: -u --url +takes_value "Network to connect.")
        (@arg CONFIG: -c --config +takes_value "Path to tonos-cli configuration file.") 
        (@subcommand version =>
            (about: "Prints build and version info.")
        )
        (@subcommand convert =>
            (@subcommand tokens =>
                (about: "Converts tokens to nanotokens.")
                (@arg AMOUNT: +required +takes_value "Token amount value")
            )    
        )
        (@subcommand genphrase =>
            (about: "Generates seed phrase.")
            (author: "TONLabs")
        )
        (@subcommand genpubkey =>
            (about: "Generates seed phrase.")
            (author: "TONLabs")
            (@arg PHRASE: +required +takes_value "Seed phrase (12 words).")
        )
        (@subcommand getkeypair =>
            (about: "Generates keypair from seed phrase and saves it to file.")
            (author: "TONLabs")
            (@arg KEY_FILE: +required +takes_value "Path to file where to store keypair.")
            (@arg PHRASE: +required +takes_value "Seed phrase (12 words)")
            (@arg VERBOSE: -v --verbose "Prints additional information about command execution.")
        )
        (@subcommand genaddr =>
            (@setting AllowNegativeNumbers)
            (about: "Calculates smart contract address in different formats. By default, input tvc file isn't modified.")
            (version: "0.1")
            (author: "TONLabs")
            (@arg TVC: +required +takes_value "Compiled smart contract (tvc file).")
            (@arg ABI: +required +takes_value "Json file with contract ABI.")
            (@arg WC: --wc +takes_value "Workchain id used to generate user-friendly addresses (default 0).")
            (@arg GENKEY: --genkey +takes_value conflicts_with[SETKEY] "Generates new keypair for the contract and saves it to the file.")
            (@arg SETKEY: --setkey +takes_value conflicts_with[GENKEY] "Loads existing keypair from the file.")
            (@arg DATA: --data +takes_value "Supplies initial data to insert into contract.")
            (@arg SAVE: --save "Rewrite tvc file with supplied kepair and initial data.")
            (@arg VERBOSE: -v --verbose "Prints additional information about command execution.")
        )
        (@subcommand deploy =>
            (@setting AllowNegativeNumbers)
            (@setting AllowLeadingHyphen)
            (about: "Deploy smart contract to blockchain.")
            (version: "0.1")
            (author: "TONLabs")
            (@arg TVC: +required +takes_value "Compiled smart contract (tvc file)")
            (@arg PARAMS: +required +takes_value "Constructor arguments.")
            (@arg ABI: --abi +takes_value "Json file with contract ABI.")
            (@arg SIGN: --sign +takes_value "Keypair used to sign 'constructor message'.")
            (@arg WC: --wc +takes_value "Workchain id of the smart contract (default 0).")
            (@arg VERBOSE: -v --verbose "Prints additional information about command execution.")
        )
        (subcommand: callex_sub_command)
        (@subcommand call =>
            (@setting AllowLeadingHyphen)
            (about: "Sends external message to contract with encoded function call.")
            (version: "0.1")
            (author: "TONLabs")
            (@arg ADDRESS: +required +takes_value "Contract address.")
            (@arg METHOD: +required +takes_value "Name of calling contract method.")
            (@arg PARAMS: +required +takes_value "Arguments for the contract method.")
            (@arg ABI: --abi +takes_value "Json file with contract ABI.")
            (@arg SIGN: --sign +takes_value "Keypair used to sign message.")
            (@arg STATE: --state + takes_value "Path to tvc file. Allows to include account code and data into message.")
            (@arg VERBOSE: -v --verbose "Prints additional information about command execution.")
        )
        (@subcommand send =>
            (about: "Sends prepared message to contract.")
            (version: "0.1")
            (author: "TONLabs")
            (@arg MESSAGE: +required +takes_value "Message to send.")
            (@arg ABI: --abi +takes_value "Json file with contract ABI.")
            (@arg VERBOSE: -v --verbose "Prints additional information about command execution.")
        )
        (@subcommand message =>
            (@setting AllowLeadingHyphen)
            (about: "Generates a signed message with encoded function call.")
            (author: "TONLabs")
            (@arg ADDRESS: +required +takes_value "Contract address.")
            (@arg METHOD: +required +takes_value "Name of calling contract method.")
            (@arg PARAMS: +required +takes_value "Arguments for the contract method.")
            (@arg ABI: --abi +takes_value "Json file with contract ABI.")
            (@arg SIGN: --sign +takes_value "Keypair used to sign message.")
            (@arg LIFETIME: --lifetime +takes_value "Period of time in seconds while message is valid.")
            (@arg STATE: --state + takes_value "Path to tvc file. Allows to include account code and data into message.")
            (@arg VERBOSE: -v --verbose "Prints additional information about command execution.")
        )
        (@subcommand run =>
            (@setting AllowLeadingHyphen)
            (about: "Runs contract function locally.")
            (@arg ADDRESS: +required +takes_value "Contract address.")
            (@arg METHOD: +required +takes_value "Name of calling contract method.")
            (@arg PARAMS: +required +takes_value "Arguments for the contract method.")
            (@arg ABI: --abi +takes_value "Json file with contract ABI.")
            (@arg VERBOSE: -v --verbose "Prints additional information about command execution.")
        )
        (subcommand: runget_sub_command)
        (@subcommand config =>
            (@setting AllowLeadingHyphen)
            (about: "Saves certain default values for options into config file.")
            (version: "0.1")
            (author: "TONLabs")
            (@arg URL: --url +takes_value "Url to connect.")
            (@arg ABI: --abi +takes_value conflicts_with[DATA] "File with contract ABI.")
            (@arg KEYS: --keys +takes_value "File with keypair.")
            (@arg ADDR: --addr +takes_value "Contract address.")
            (@arg WC: --wc +takes_value "Workchain id.")
            (@arg RETRIES: --retries +takes_value "Number of attempts to call smart contract function if previous attempt was unsuccessful.")
            (@arg TIMEOUT: --timeout +takes_value "Contract call timeout in ms.")
            (@arg LIST: --list conflicts_with[URL ABI KEYS ADDR RETRIES TIMEOUT WC] "Prints all config parameters.")
        )
        (@subcommand account =>
            (@setting AllowLeadingHyphen)
            (about: "Gets account information.")
            (version: "0.1")
            (author: "TONLabs")
            (@arg ADDRESS: +required +takes_value "Smart contract address.")
            (@arg VERBOSE: -v --verbose "Prints additional information about command execution.")
            (@subcommand replaytxn =>
                (about: "Replays account transaction locally using smart contract image file.")
                (@arg TVC: --tvc +takes_value "Path to smart contract image file (tvc).")
                (@arg TXN: --txnid +takes_value "ID of transaction which should be replayed.")
            )
            (@subcommand replaystate =>
                (about: "Replays all transactions for account until defined transaction and saves new contract state to tvc file.")
                (@arg TVC: --tvc +takes_value "Path to original smart contract image (tvc).")
                (@arg TXN: --txnid +takes_value "ID of threshold transaction.")
                (@arg ABI: --abi +takes_value "Path to file with contract ABI.")
            )
        )
        (@subcommand proposal =>
            (@subcommand create =>
                (about: "Submits proposal transaction in multisignature wallet with text comment.")
                (@arg ADDRESS: +required +takes_value "Address of multisignature wallet.")
                (@arg DEST: +required +takes_value "Address of proposal contract.")
                (@arg COMMENT: +required +takes_value "Proposal description (max symbols 382).")
                (@arg KEYS: +required +takes_value "Seed phrase or path to keypair file.")
                (@arg OFFLINE: -f --offline "Prints signed message to terminal instead of sending it.")
                (@arg LIFETIME: -l --lifetime +takes_value "Period of time in seconds while message is valid.")
            )
            (@subcommand vote =>
                (about: "Confirms proposal transaction in multisignature wallet.")
                (@arg ADDRESS: +required +takes_value "Address of multisignature wallet.")
                (@arg ID: +required +takes_value "Proposal transaction id.")
                (@arg KEYS: +required +takes_value "Seed phrase or path to keypair file.")
                (@arg OFFLINE: -f --offline "Prints signed message to terminal instead of sending it.")
                (@arg LIFETIME: -l --lifetime +takes_value "Period of time in seconds while message is valid.")
            )
            (@subcommand decode =>
                (about: "Prints comment string from proposal transaction.")
                (@arg ADDRESS: +required +takes_value "Address of multisignature wallet.")
                (@arg ID: +required +takes_value "Proposal transaction id.")
            )
        )
        (subcommand: create_multisig_command())
        (@subcommand getconfig =>
            (about: "Reads global configuration parameter with defined index.")
            (@arg INDEX: +required +takes_value "Parameter index.")
        )
        (@subcommand nodeid =>
            (about: "Calculates node ID from validator public key")
            (@arg KEY: --pubkey +takes_value "Validator public key.")
            (@arg KEY_PAIR: --keypair +takes_value "Validator key pair as 12 words mnemonic or file path.")
        )
        (@setting SubcommandRequired)
    ).get_matches();

    let config_file = matches.value_of("CONFIG").map(|v| v.to_string())
        .or(env::var("TONOSCLI_CONFIG").ok())
        .unwrap_or(default_config_name()?);

    let mut conf = match Config::from_file(&config_file) {
        Some(c) => {
            println!("Config: {}", config_file);
            c
        },
        None => {
            println!("Config: default");
            Config::new()
        },
    };

    if let Some(url) = matches.value_of("NETWORK") {
        conf.url = url.to_string();
    }

    if let Some(m) = matches.subcommand_matches("convert") {
        if let Some(m) = m.subcommand_matches("tokens") {
            return convert_tokens(m);
        }
    }
    if let Some(m) = matches.subcommand_matches("callex") {
        return callex_command(m, conf);
    }
    if let Some(m) = matches.subcommand_matches("call") {
        return call_command(m, conf, CallType::Call);
    }
    if let Some(m) = matches.subcommand_matches("run") {
        return call_command(m, conf, CallType::Run);
    }
    if let Some(m) = matches.subcommand_matches("runget") {
        return runget_command(m, conf);
    }
    if let Some(m) = matches.subcommand_matches("message") {
        return call_command(m, conf, CallType::Msg);
    }
    if let Some(m) = matches.subcommand_matches("send") {
        return send_command(m, conf);
    }
    if let Some(m) = matches.subcommand_matches("deploy") {        
        return deploy_command(m, conf);
    } 
    if let Some(m) = matches.subcommand_matches("config") {
        return config_command(m, conf);
    }
    if let Some(m) = matches.subcommand_matches("genaddr") {
        return genaddr_command(m, conf);
    }
    if let Some(m) = matches.subcommand_matches("getkeypair") {
        return getkeypair_command(m, conf);
    }
    if let Some(m) = matches.subcommand_matches("account") {
        let address = value_of_address(m, &conf)?;
        if let Some(m) = m.subcommand_matches("replaytxn") {
            return replaytxn_command(m, conf, &address);
        }
        if let Some(m) = m.subcommand_matches("replaystate") {
            return replaystate_command(m, conf, &address);
        }
        return account_command(m, conf, &address);
    }
    if let Some(m) = matches.subcommand_matches("genphrase") {
        return genphrase_command(m, conf);
    }
    if let Some(m) = matches.subcommand_matches("genpubkey") {
        return genpubkey_command(m, conf);
    }
    if let Some(m) = matches.subcommand_matches("proposal") {
        if let Some(m) = m.subcommand_matches("create") {
            return proposal_create_command(m, conf);
        }
        if let Some(m) = m.subcommand_matches("vote") {
            return proposal_vote_command(m, conf);
        }
        if let Some(m) = m.subcommand_matches("decode") {
            return proposal_decode_command(m, conf);
        }
    }
    if let Some(m) = matches.subcommand_matches("multisig") {
        return multisig_command(m, conf);
    }
    if let Some(m) = matches.subcommand_matches("getconfig") {
        return getconfig_command(m, conf);
    }
    if let Some(m) = matches.subcommand_matches("nodeid") {
        return nodeid_command(m);
    }
    if let Some(_) = matches.subcommand_matches("version") {
        println!(
            "tonlabs-cli {}\nCOMMIT_ID: {}\nBUILD_DATE: {}\nCOMMIT_DATE: {}\nGIT_BRANCH: {}",
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
    print_args!(matches, key_file, phrase);
    generate_keypair(key_file.unwrap(), phrase.unwrap())
}

fn send_command(matches: &ArgMatches, config: Config) -> Result<(), String> {
    let message = matches.value_of("MESSAGE");
    let abi = Some(
        matches.value_of("ABI")
            .map(|s| s.to_string())
            .or(config.abi_path.clone())
            .ok_or("ABI file not defined. Supply it in config file or command line.".to_string())?
    );
    
    print_args!(matches, message, abi);

    let abi = std::fs::read_to_string(abi.unwrap())
        .map_err(|e| format!("failed to read ABI file: {}", e.to_string()))?;

    call_contract_with_msg(config, message.unwrap().to_owned(), abi)
}

fn call_command(matches: &ArgMatches, config: Config, call: CallType) -> Result<(), String> {
    let address = matches.value_of("ADDRESS");
    let method = matches.value_of("METHOD");
    let params = matches.value_of("PARAMS");
    let state = matches.value_of("STATE");
    let lifetime = matches.value_of("LIFETIME");
    let abi = Some(
        matches.value_of("ABI")
            .map(|s| s.to_string())
            .or(config.abi_path.clone())
            .ok_or("ABI file not defined. Supply it in config file or command line.".to_string())?
    );
    
    let keys = match call {
        CallType::Call | CallType::Msg => {
            matches.value_of("SIGN")
                .map(|s| s.to_string())
                .or(config.keys_path.clone())
        },
        CallType::Run => {
            None
        }
    };

    print_args!(matches, address, method, params, abi, keys, lifetime, state);

    let abi = std::fs::read_to_string(abi.unwrap())
        .map_err(|e| format!("failed to read ABI file: {}", e.to_string()))?;
    
    match call {
        CallType::Call | CallType::Run => {
            let local = if let CallType::Call = call { false } else { true };
            call_contract(
                config,
                address.unwrap(),
                abi,
                method.unwrap(),
                params.unwrap(),
                keys,
                local,
                state,
            )
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
                address.unwrap(),
                abi,
                method.unwrap(),
                params.unwrap(),
                keys,
                lifetime,
                state,
            )
        },
    }
}

fn callex_command(matches: &ArgMatches, config: Config) -> Result<(), String> {
    let method = matches.value_of("METHOD");
    let address = Some(
        matches.value_of("ADDRESS")
            .map(|s| s.to_string())
            .or(config.addr.clone())
            .ok_or("ADDRESS is not defined. Supply it in config file or in command line.".to_string())?
    );
    let abi = Some(
        matches.value_of("ABI")
        .map(|s| s.to_string())
        .or(config.abi_path.clone())
        .ok_or("ABI is not defined. Supply it in config file or in command line.".to_string())?
    );
    let loaded_abi = std::fs::read_to_string(abi.as_ref().unwrap())
        .map_err(|e| format!("failed to read ABI file: {}", e.to_string()))?;
    let params = Some(parse_params(
        matches.values_of("PARAMS").unwrap().collect::<Vec<_>>(), &loaded_abi, method.clone().unwrap()
    )?);
    let keys = matches.value_of("SIGN")
        .map(|s| s.to_string())
        .or(config.keys_path.clone());
    
    print_args!(matches, address, method, params, abi, keys);
    call_contract(
        config,
        &address.unwrap(),
        loaded_abi,
        method.unwrap(),
        &params.unwrap(),
        keys,
        false,
        None,
    )
}

fn runget_command(matches: &ArgMatches, config: Config) -> Result<(), String> {
    let address = matches.value_of("ADDRESS");
    let method = matches.value_of("METHOD");
    let params = matches.values_of("PARAMS");
    let params = params.map(|values| {
        json!(values.collect::<Vec<_>>()).to_string()
    });
    print_args!(matches, address, method, params);
    run_get_method(config, address.unwrap(), method.unwrap(), params)
}

fn deploy_command(matches: &ArgMatches, config: Config) -> Result<(), String> {
    let tvc = matches.value_of("TVC");
    let params = matches.value_of("PARAMS");
    let wc = matches.value_of("WC");
    let abi = Some(
        matches.value_of("ABI")
            .map(|s| s.to_string())
            .or(config.abi_path.clone())
            .ok_or("ABI file not defined. Supply it in config file or command line.".to_string())?
    );
    let keys = Some(
        matches.value_of("SIGN")
            .map(|s| s.to_string())
            .or(config.keys_path.clone())
            .ok_or("keypair file not defined. Supply it in config file or command line.".to_string())?
    );
    print_args!(matches, tvc, params, abi, keys, wc);

    let wc = wc.map(|v| i32::from_str_radix(v, 10))
        .transpose()
        .map_err(|e| format!("failed to parse workchain id: {}", e))?
        .unwrap_or(config.wc);
    deploy_contract(config, tvc.unwrap(), &abi.unwrap(), params.unwrap(), &keys.unwrap(), wc)
}

fn config_command(matches: &ArgMatches, config: Config) -> Result<(), String> {    
    if matches.is_present("LIST") {
        println!(
            "{}",
            serde_json::to_string_pretty(&config)
                .map_err(|e| format!("failed to print config parameters: {}", e))?
        );
        Ok(())
    } else {
        let url = matches.value_of("URL");
        let address = matches.value_of("ADDR");
        let keys = matches.value_of("KEYS");
        let abi = matches.value_of("ABI");
        let wc = matches.value_of("WC");
        let retries = matches.value_of("RETRIES");
        let timeout = matches.value_of("TIMEOUT");
        print_args!(matches, url, address, keys, abi, wc, retries, timeout);
        set_config(config, "tonlabs-cli.conf.json", url, address, abi, keys, wc, retries, timeout)
    }
}

fn genaddr_command(matches: &ArgMatches, config: Config) -> Result<(), String> {
    let tvc = matches.value_of("TVC");
    let wc = matches.value_of("WC");
    let keys = matches.value_of("GENKEY").or(matches.value_of("SETKEY"));
    let new_keys = matches.is_present("GENKEY");
    let init_data = matches.value_of("DATA");
    let update_tvc = matches.is_present("SAVE");
    let abi = matches.value_of("ABI");
    let is_update_tvc = if update_tvc { Some("true") } else { None };
    print_args!(matches, tvc, wc, keys, init_data, is_update_tvc);
    generate_address(config, tvc.unwrap(), abi.unwrap(), wc, keys, new_keys, init_data, update_tvc)
}

fn account_command(matches: &ArgMatches, config: Config, address: &str) -> Result<(), String> {
    let address = Some(address);
    print_args!(matches, address);
    get_account(config, address.unwrap())
}

fn proposal_create_command(matches: &ArgMatches, config: Config) -> Result<(), String> {
    let address = matches.value_of("ADDRESS");
    let dest = matches.value_of("DEST");
    let keys = matches.value_of("KEYS");
    let comment = matches.value_of("COMMENT");
    let lifetime = matches.value_of("LIFETIME");
    let offline = matches.is_present("OFFLINE");
    print_args!(matches, address, comment, keys, lifetime);

    let lifetime = lifetime.map(|val| {
        u32::from_str_radix(val, 10)
            .map_err(|e| format!("failed to parse lifetime: {}", e))
    })
    .transpose()?
    .unwrap_or(config.timeout);

    create_proposal(config, address.unwrap(), keys, dest.unwrap(), comment.unwrap(), lifetime, offline)
}

fn proposal_vote_command(matches: &ArgMatches, config: Config) -> Result<(), String> {
    let address = matches.value_of("ADDRESS");
    let keys = matches.value_of("KEYS");
    let id = matches.value_of("ID");
    let lifetime = matches.value_of("LIFETIME");
    let offline = matches.is_present("OFFLINE");
    print_args!(matches, address, id, keys, lifetime);

    let lifetime = lifetime.map(|val| {
        u32::from_str_radix(val, 10)
            .map_err(|e| format!("failed to parse lifetime: {}", e))
    })
    .transpose()?
    .unwrap_or(config.timeout);

    vote(config, address.unwrap(), keys, id.unwrap(), lifetime, offline)
}

fn proposal_decode_command(matches: &ArgMatches, config: Config) -> Result<(), String> {
    let address = matches.value_of("ADDRESS");
    let id = matches.value_of("ID");
    print_args!(matches, address, id);
    decode_proposal(config, address.unwrap(), id.unwrap())
}

fn getconfig_command(matches: &ArgMatches, config: Config) -> Result<(), String> {
    let index = matches.value_of("INDEX");
    print_args!(matches, index);
    query_global_config(config, index.unwrap())
}

fn nodeid_command(matches: &ArgMatches) -> Result<(), String> {
    let key = matches.value_of("KEY");
    let keypair = matches.value_of("KEY_PAIR");
    print_args!(matches, key, keypair);
    let nodeid = if let Some(key) = key {
        let vec = hex::decode(key)
            .map_err(|e| format!("failed to decode public key: {}", e))?;
        convert::nodeid_from_pubkey(&vec)?
    } else if let Some(pair) = keypair {
        let pair = crypto::load_keypair(pair)?;
        convert::nodeid_from_pubkey(&pair.public.0)?
    } else {
        return Err("Either public key or key pair parameter should be provided".to_owned());
    };
    println!("{}", nodeid);
    Ok(())
}

fn replaytxn_command(matches: &ArgMatches, config: Config, address: &str) -> Result<(), String> {
    let address = Some(address);
    let tvc = matches.value_of("TVC");
    let transaction_id = matches.value_of("TXN");    
    print_args!(matches, address, tvc, transaction_id);
    image::replay_transaction(config, address.unwrap(), tvc.unwrap(), transaction_id.unwrap())
}

fn replaystate_command(matches: &ArgMatches, config: Config, address: &str) -> Result<(), String> {
    let address = Some(address);
    let tvc = matches.value_of("TVC");
    let last_txn_id = matches.value_of("TXN");
    let abi = matches.value_of("ABI")
        .map(|s| s.to_string())
        .or(config.abi_path.clone());
    
    print_args!(matches, address, tvc, last_txn_id, abi);
    let abi = abi.map(|v| std::fs::read_to_string(v)
        .map_err(|e| format!("failed to read ABI file: {}", e.to_string()))
    ).transpose()?;
    image::replay_state(config, address.unwrap(), tvc.unwrap(), last_txn_id, abi)
}