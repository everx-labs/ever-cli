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

use std::path::Path;
use clap::{ArgMatches, SubCommand, Arg, App};
use crate::config::Config;
use sold_lib::{Args, build, solidity_version};
use crate::generate_address;

pub fn create_compile_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("compile")
        .about("Compile commands.")
        .subcommand(SubCommand::with_name("sol")
            .about("Compile TVM Solidity code to the ready to deploy tvc.")
            .arg(Arg::with_name("INPUT")
                .required_unless("VERSION")
                .help("Path to the Solidity source file."))
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
            .arg(Arg::with_name("VERSION")
                .long("--version")
                .short("-v")
                .help("Print version of the Solidity compiler."))
            .arg(Arg::with_name("WC")
                .takes_value(true)
                .long("--wc")
                .help("Workchain id of the smart contract (default value is taken from the config)."))
            .arg(Arg::with_name("CONTRACT")
                .takes_value(true)
                .long("--contract")
                .help("Contract to build if sources define more than one contract."))
            .arg(Arg::with_name("OUTPUT_DIR")
                .takes_value(true)
                .long("--output_dir")
                .short("-o")
                .help("Output directory (by default, current directory is used)."))
            .arg(Arg::with_name("OUTPUT_PREFIX")
                .takes_value(true)
                .long("--output_prefix")
                .short("-p")
                .help("Output prefix (by default, input file stem is used as prefix)."))
            .arg(Arg::with_name("INCLUDE_PATH")
                .takes_value(true)
                .long("--include_path")
                .short("-i")
                .help("Include additional path to search for imports."))
            .arg(Arg::with_name("LIB")
                .takes_value(true)
                .long("--lib")
                .short("-l")
                .help("Library to use instead of default."))
            .arg(Arg::with_name("REFRESH_REMOTE")
                .long("--refresh_remote")
                .help("Force download and rewrite remote import files."))
        )
}

pub async fn compile_command(matches: &ArgMatches<'_>, config: &Config) -> Result<(), String> {
    if let Some(matches) = matches.subcommand_matches("sol") {
        return compile_solidity(matches, config).await;
    }
    Err("unknown command".to_owned())
}

async fn compile_solidity(matches: &ArgMatches<'_>, config: &Config) -> Result<(), String> {
    if matches.is_present("VERSION") {
        println!("Solidity compiler version: {}", solidity_version());
        return Ok(());
    }
    let input= matches.value_of("INPUT").unwrap().to_owned();
    let include_path = matches.value_of("INCLUDE_PATH")
        .map(|input| {
            input
                .trim_end_matches('[')
                .trim_start_matches('[')
                .split(',')
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        })
        .unwrap_or(vec![]);

    let args = Args {
        input: input.clone(),
        contract: matches.value_of("CONTRACT").map(|s| s.to_owned()),
        output_dir: matches.value_of("OUTPUT_DIR").map(|s| s.to_owned()),
        output_prefix: matches.value_of("OUTPUT_PREFIX").map(|s| s.to_owned()),
        include_path,
        lib: matches.value_of("LIB").map(|s| s.to_owned()),
        function_ids: false,
        ast_json: false,
        ast_compact_json: false,
        abi_json: false,
        tvm_refresh_remote: matches.is_present("REFRESH_REMOTE"),
    };
    build(args, config.is_json).map_err(|e| format!("Failed to compile the contract: {}", e))?;
    let input_canonical = Path::new(&input).canonicalize()
        .map_err(|e| format!("Failed to format input path: {}", e))?;
    let stem = input_canonical.file_stem().ok_or("Failed to format input path".to_owned())?
        .to_str().unwrap().to_owned();
    let tvc_path = format!("{}.tvc", stem);
    let abi_path = format!("{}.abi.json", stem);
    if !config.is_json {
        println!("Path to the TVC file: {}", tvc_path);
    }
    let keys = matches.value_of("GENKEY").or(matches.value_of("SETKEY"));
    let new_keys = matches.is_present("GENKEY");
    let wc = matches.value_of("WC");
    generate_address(config, &tvc_path, &abi_path, wc, keys, new_keys, None, true).await
}