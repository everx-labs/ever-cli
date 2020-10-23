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

pub mod term_browser;

pub fn create_debot_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("debot")
        .about("Debot commands.")
        .setting(AppSettings::AllowLeadingHyphen)
        .setting(AppSettings::TrailingVarArg)
        .setting(AppSettings::DontCollapseArgsInUsage)
        .arg(Arg::with_name("DEBUG").long("--debug").short("-d"))
        .subcommand(
            SubCommand::with_name("fetch")
                .arg(
                    Arg::with_name("ADDRESS")
                        .required(true)
                        .help("Debot address."),
                )
                .arg(
                    Arg::with_name("ABI")
                        .long("--abi")
                        .takes_value(true)
                        .help("Debot ABI."),
                ),
        )
}

pub fn debot_command(m: &ArgMatches, config: Config) -> Result<(), String> {
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
        return fetch_command(m, config);
    }
    Err("unknown debot command".to_owned())
}

fn fetch_command(m: &ArgMatches, config: Config) -> Result<(), String> {
    let addr = m.value_of("ADDRESS");
    let abi = m
        .value_of("ABI")
        .map(|s| s.to_string())
        .or(config.abi_path.clone());

    let abi = abi
        .map(|s| {
            std::fs::read_to_string(s)
                .map_err(|e| format!("failed to read ABI file: {}", e.to_string()))
        })
        .transpose()?;
    return run_debot_browser(addr.unwrap(), abi, config);
}
