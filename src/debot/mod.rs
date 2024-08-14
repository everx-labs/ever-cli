/*
* Copyright 2018-2023 EverX.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific EVERX DEV software governing permissions and
* limitations under the License.
*/
mod callbacks;
mod interfaces;
mod pipechain;
mod processor;
pub mod term_browser;
mod term_encryption_box;
mod term_signing_box;

use crate::config::Config;
use crate::helpers::load_ton_address;
use callbacks::Callbacks;
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
pub use interfaces::dinterface::SupportedInterfaces;
use pipechain::{ApproveKind, ChainLink, PipeChain};
use processor::{ChainProcessor, ProcessorError};
use simplelog::*;
use term_browser::{action_input, input, run_debot_browser, terminal_input};

pub fn create_debot_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("debot")
        .about("Debot commands.")
        .setting(AppSettings::AllowLeadingHyphen)
        .setting(AppSettings::TrailingVarArg)
        .setting(AppSettings::DontCollapseArgsInUsage)
        .arg(Arg::with_name("DEBUG").long("--debug").short("-d"))
        .subcommand(
            SubCommand::with_name("fetch")
                .setting(AppSettings::AllowLeadingHyphen)
                .arg(
                    Arg::with_name("ADDRESS")
                        .required(true)
                        .help("DeBot TON address."),
                ),
        )
        .subcommand(
            SubCommand::with_name("start")
                .setting(AppSettings::AllowLeadingHyphen)
                .arg(
                    Arg::with_name("ADDRESS")
                        .required(true)
                        .help("DeBot TON address."),
                )
                .arg(
                    Arg::with_name("PIPECHAIN")
                        .short("m")
                        .long("pipechain")
                        .takes_value(true)
                        .help("Path to the DeBot Manifest."),
                )
                .arg(
                    Arg::with_name("SIGNKEY")
                        .short("s")
                        .long("signkey")
                        .takes_value(true)
                        .help("Define keypair to auto sign transactions."),
                ),
        )
        .subcommand(
            SubCommand::with_name("invoke")
                .setting(AppSettings::AllowLeadingHyphen)
                .arg(
                    Arg::with_name("ADDRESS")
                        .required(true)
                        .help("Debot TON address."),
                )
                .arg(
                    Arg::with_name("MESSAGE")
                        .required(true)
                        .help("Message to DeBot encoded as base64/base64url."),
                ),
        )
}

pub async fn debot_command(m: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let debug = m.is_present("DEBUG");
    let log_conf = ConfigBuilder::new()
        .add_filter_ignore_str("executor")
        .add_filter_ignore_str("hyper")
        .add_filter_ignore_str("reqwest")
        .build();

    let mut loggers: Vec<Box<dyn SharedLogger>> = vec![];
    let file = std::fs::File::create("debot_err.log");
    if file.is_ok() {
        loggers.push(WriteLogger::new(
            LevelFilter::Error,
            log_conf.clone(),
            file.unwrap(),
        ));
    }

    if debug {
        loggers.push(TermLogger::new(
            LevelFilter::Debug,
            log_conf.clone(),
            TerminalMode::Mixed,
        ));
    }
    CombinedLogger::init(loggers).unwrap();

    if let Some(m) = m.subcommand_matches("fetch") {
        return fetch_command(m, config).await;
    }
    if let Some(m) = m.subcommand_matches("start") {
        return fetch_command(m, config).await;
    }
    if let Some(m) = m.subcommand_matches("invoke") {
        return invoke_command(m, config).await;
    }
    Err("unknown debot command".to_owned())
}

async fn fetch_command(m: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let addr = m.value_of("ADDRESS");
    let pipechain = m.value_of("PIPECHAIN");
    let signkey_path = m
        .value_of("SIGNKEY")
        .map(|x| x.to_owned())
        .or(config.keys_path.clone());
    let is_json = config.is_json;
    let pipechain = if let Some(filename) = pipechain {
        let manifest_raw = std::fs::read_to_string(filename)
            .map_err(|e| format!("failed to read pipechain: {}", e))?;
        serde_json::from_str(&manifest_raw)
            .map_err(|e| format!("failed to parse pipechain: {}", e))?
    } else {
        PipeChain::new()
    };
    let addr = load_ton_address(addr.unwrap(), &config)?;
    let result = run_debot_browser(addr.as_str(), config, pipechain, signkey_path).await;
    match result {
        Ok(Some(arg)) => {
            if !is_json {
                println!("Returned value:");
            }
            println!("{:#}", arg);
            Ok(())
        }
        Err(err) if err.contains("NoMoreChainlinks") => Ok(()),
        result => result.map(|_| ()),
    }
}

async fn invoke_command(m: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let addr = m.value_of("ADDRESS");
    load_ton_address(addr.unwrap(), &config)?;
    let _ = m.value_of("MESSAGE").unwrap().to_owned();
    Ok(())
}
