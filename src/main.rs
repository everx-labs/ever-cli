/*
 * Copyright 2018-2019 TON DEV SOLUTIONS LTD.
 *
 * Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
 * this file except in compliance with the License.  You may obtain a copy of the
 * License at: https://ton.dev/licenses
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
mod config;
mod crypto;
mod deploy;
mod genaddr;
mod helpers;
mod call;

use account::get_account;
use call::{call_contract, call_contract_with_msg, generate_message};
use clap::ArgMatches;
use config::{Config, set_config};
use crypto::{generate_mnemonic, extract_pubkey};
use deploy::deploy_contract;
use genaddr::generate_address;

const VERBOSE_MODE: bool = true;
const DEF_MSG_LIFETIME: u32 = 30;
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

    let matches = clap_app! (tonlabs_cli =>        
        (version: &*format!("0.1 ({})", build_info))
        (author: "TONLabs")
        (about: "TONLabs console tool for TON")
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
            (@arg PHRASE: +required +takes_value "Seed phrase (12 words)")
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
            (about: "Deploy smart contract to blockchain.")
            (version: "0.1")
            (author: "TONLabs")
            (@arg TVC: +required +takes_value "Compiled smart contract (tvc file)")
            (@arg PARAMS: +required +takes_value "Constructor arguments.")
            (@arg ABI: --abi +takes_value "Json file with contract ABI.")
            (@arg SIGN: --sign +takes_value "Keypair used to sign 'constructor message'.")
            (@arg VERBOSE: -v --verbose "Prints additional information about command execution.")
        )
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
            (about: "Generates a signed message with encoded function call.")
            (author: "TONLabs")
            (@arg ADDRESS: +required +takes_value "Contract address.")
            (@arg METHOD: +required +takes_value "Name of calling contract method.")
            (@arg PARAMS: +required +takes_value "Arguments for the contract method.")
            (@arg ABI: --abi +takes_value "Json file with contract ABI.")
            (@arg SIGN: --sign +takes_value "Keypair used to sign message.")
            (@arg LIFETIME: --lifetime +takes_value "Period of time in seconds while message is valid.")
            (@arg VERBOSE: -v --verbose "Prints additional information about command execution.")
        )
        (@subcommand run =>
            (@setting AllowLeadingHyphen)
            (about: "Runs contract's get-method.")
            (version: "0.1")
            (author: "TONLabs")
            (@arg ADDRESS: +required +takes_value "Contract address.")
            (@arg METHOD: +required +takes_value conflicts_with[BODY] "Name of calling contract method.")
            (@arg PARAMS: +required +takes_value conflicts_with[BODY] "Arguments for the contract method.")
            (@arg ABI: --abi +takes_value conflicts_with[BODY] "Json file with contract ABI.")
            (@arg VERBOSE: -v --verbose "Prints additional information about command execution.")
        )
        (@subcommand config =>
            (about: "Writes parameters to config file that can be used later in subcommands.")
            (version: "0.1")
            (author: "TONLabs")
            (@arg URL: --url +takes_value "Url to connect.")
            (@arg ABI: --abi +takes_value conflicts_with[DATA] "File with contract ABI.")
            (@arg KEYS: --keys +takes_value "File with keypair.")
            (@arg ADDR: --addr +takes_value "Contract address.")            
        )
        (@subcommand account =>
            (@setting AllowLeadingHyphen)
            (about: "Gets account information.")
            (version: "0.1")
            (author: "TONLabs")
            (@arg ADDRESS: +required +takes_value "Smart contract address.")
            (@arg VERBOSE: -v --verbose "Prints additional information about command execution.")
        )
        (@setting SubcommandRequired)
    ).get_matches();

    let conf = Config::from_file("tonlabs-cli.conf.json").unwrap_or(Config::new());

    if let Some(m) = matches.subcommand_matches("convert") {
        if let Some(m) = m.subcommand_matches("tokens") {
            return convert_tokens(m);
        }
    }
    if let Some(m) = matches.subcommand_matches("call") {
        return call_command(m, conf, CallType::Call);
    }
    if let Some(m) = matches.subcommand_matches("run") {
        return call_command(m, conf, CallType::Run);
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
    if let Some(m) = matches.subcommand_matches("account") {
        return account_command(m, conf);
    }
    if let Some(m) = matches.subcommand_matches("genphrase") {
        return genphrase_command(m, conf);
    }
    if let Some(m) = matches.subcommand_matches("genpubkey") {
        return genpubkey_command(m, conf);
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
    let parts: Vec<&str> = amount.split(".").collect();
    if parts.len() >= 1 && parts.len() <= 2 {
        let mut result = String::new();
        result += parts[0];
        if parts.len() == 2 {
            let fraction = format!("{:0<9}", parts[1]);
            if fraction.len() != 9 {
                return Err("invalid fractional part".to_string());
            }
            result += &fraction;
        } else {
            result += "000000000";
        }
        u64::from_str_radix(&result, 10)
            .map_err(|e| format!("failed to parse amount: {}", e))?;
        println!("{}", result);
        return Ok(());
    }
    return Err("Invalid amout value".to_string());
}

fn genphrase_command(_matches: &ArgMatches, _config: Config) -> Result<(), String> {
    generate_mnemonic()
}

fn genpubkey_command(matches: &ArgMatches, _config: Config) -> Result<(), String> {
    let mnemonic = matches.value_of("PHRASE").unwrap();
    extract_pubkey(mnemonic)
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

    print_args!(matches, address, method, params, abi, keys, lifetime);

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
                local
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
                lifetime)
        },
    }
}

fn deploy_command(matches: &ArgMatches, config: Config) -> Result<(), String> {
    let tvc = matches.value_of("TVC");
    let params = matches.value_of("PARAMS");
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
    print_args!(matches, tvc, params, abi, keys);
    deploy_contract(config, tvc.unwrap(), &abi.unwrap(), params.unwrap(), &keys.unwrap())
}

fn config_command(matches: &ArgMatches, config: Config) -> Result<(), String> {
    let url = matches.value_of("URL");
    let address = matches.value_of("ADDR");
    let keys = matches.value_of("KEYS");
    let abi = matches.value_of("ABI");
    print_args!(matches, url, address, keys, abi);
    set_config(config, "tonlabs-cli.conf.json", url, address, abi, keys)
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

fn account_command(matches: &ArgMatches, config: Config) -> Result<(), String> {
    let address = matches.value_of("ADDRESS");
    print_args!(matches, address);
    get_account(config, address.unwrap())
}