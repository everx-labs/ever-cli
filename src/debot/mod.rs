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
use crate::config::Config;
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use simplelog::*;
use term_browser::run_debot_browser;
use crate::helpers::load_ton_address;

mod interfaces;
mod manifest;
mod processor;
mod term_signing_box;
pub mod term_browser;

use processor::{ManifestProcessor, ProcessorError};
use manifest::{ApproveKind, DebotManifest, ChainLink};
pub use interfaces::dinterface::SupportedInterfaces;

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
                )
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
                    Arg::with_name("MANIFEST")
                        .short("m")
                        .long("manifest")
                        .takes_value(true)
                        .help("Path to DeBot Manifest."),
                )
                .arg(
                    Arg::with_name("SIGNKEY")
                        .short("s")
                        .long("signkey")
                        .takes_value(true)
                        .help("Define keypair to auto sign transactions."),
                )
        )
        .subcommand(
            SubCommand::with_name("invoke")
                .setting(AppSettings::AllowLeadingHyphen)
                .arg(
                    Arg::with_name("ADDRESS")
                        .required(true)
                        .help("Debot TON address.")
                )
                .arg(
                    Arg::with_name("MESSAGE")
                        .required(true)
                        .help("Message to DeBot encoded as base64/base64url.")
                )
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
    let manifest = m.value_of("MANIFEST");
    let signkey_path = m.value_of("SIGNKEY")
        .map(|x| x.to_owned())
        .or(config.keys_path.clone());
    let manifest = if let Some(filename) = manifest {
        let manifest_raw = std::fs::read_to_string(filename)
            .map_err(|e| format!("failed to read manifest: {}", e))?;
        serde_json::from_str(&manifest_raw)
            .map_err(|e| format!("failed to parse manifest: {}", e))?
    } else {
        DebotManifest::default()
    };
    let addr = load_ton_address(addr.unwrap(), &config)?;
    println!("DeBot Browser started");
    let mut result = run_debot_browser(addr.as_str(), config, manifest, signkey_path).await;
    if let Err(ref msg) = result {
        if msg.contains("NoMoreChainlinks") {
            result = Ok(());
        }
    }
    println!("Debot Browser finished");
    result
}

async fn invoke_command(m: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let addr = m.value_of("ADDRESS");
    let _addr = load_ton_address(addr.unwrap(), &config)?;
    let message = m.value_of("MESSAGE").unwrap().to_owned();
    let mut manifest = DebotManifest::default();
    manifest.init_msg = Some(message);
    //run_debot_browser(addr.as_str(), config, false, Some(message)).await
    Ok(())
}
