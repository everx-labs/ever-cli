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
use crate::helpers::{decode_msg_body, print_account, create_client_local, create_client_verbose, query, TonClient};
use clap::{ArgMatches, SubCommand, Arg, App, AppSettings};
use ton_types::cells_serialization::serialize_tree_of_cells;
use ton_types::Cell;
use std::fmt::Write;
use ton_block::{Account, Deserializable, Serializable, AccountStatus, StateInit};
use ton_client::abi::{decode_account_data, ParamsOfDecodeAccountData, Abi};
use ton_client::boc::{get_compiler_version, ParamsOfGetCompilerVersion};
use crate::decode::msg_printer::tree_of_cells_into_base64;

fn match_abi_path(matches: &ArgMatches, config: &Config) -> Option<String> {
    matches.value_of("ABI")
        .map(|s| s.to_string())
        .or(config.abi_path.clone())
}

pub fn create_decode_command<'a, 'b>() -> App<'a, 'b> {
    let version_cmd = SubCommand::with_name("compiler_version")
        .about("Decodes compiler version from the contract's code.")
        .arg(Arg::with_name("TVC")
            .long("--tvc")
            .conflicts_with("BOC")
            .help("Contract is passed via path to the TVC file."))
        .arg(Arg::with_name("BOC")
            .long("--boc")
            .conflicts_with("TVC")
            .help("Contract is passed via path to the BOC file."))
        .arg(Arg::with_name("INPUT")
            .required(true)
            .help("Contract address or path to the file with contract data."));
    SubCommand::with_name("decode")
        .about("Decode commands.")
        .setting(AppSettings::AllowLeadingHyphen)
        .setting(AppSettings::TrailingVarArg)
        .setting(AppSettings::DontCollapseArgsInUsage)
        .subcommand(SubCommand::with_name("body")
            .about("Decodes body base64 string.")
            .arg(Arg::with_name("BODY")
                .required(true)
                .help("Message body encoded as base64."))
            .arg(Arg::with_name("ABI")
                .long("--abi")
                .takes_value(true)
                .help("Path to the contract ABI file.")))
        .subcommand(SubCommand::with_name("msg")
            .about("Decodes message file.")
            .arg(Arg::with_name("MSG")
                    .required(true)
                    .help("Path to the message boc file."))
            .arg(Arg::with_name("ABI")
                    .long("--abi")
                    .takes_value(true)
                    .help("Path to the contract ABI file.")))
        .subcommand(version_cmd.clone())
        .subcommand(version_cmd.clone().name("tvc")
            .about("Decodes tvc from different sources"))
        .subcommand(SubCommand::with_name("account")
            .about("Top level command of account decode commands.")
            .subcommand(SubCommand::with_name("data")
                .about("Decodes data fields from the contract state.")
                .arg(Arg::with_name("TVC")
                    .long("--tvc")
                    .short("-t")
                    .takes_value(true)
                    .help("Path to the tvc file with contract state.")
                    .conflicts_with("ADDRESS"))
                .arg(Arg::with_name("ADDRESS")
                    .long("--addr")
                    .short("-a")
                    .takes_value(true)
                    .help("Contract address.")
                    .conflicts_with("TVC"))
                .arg(Arg::with_name("ABI")
                    .long("--abi")
                    .takes_value(true)
                    .help("Path to the contract ABI file.")))
            .subcommand(SubCommand::with_name("boc")
                .about("Decodes data from the file with boc of the account and saves contract tvc file if needed.")
                .arg(Arg::with_name("BOCFILE")
                    .required(true)
                    .help("Path to the account boc file."))
                .arg(Arg::with_name("DUMPTVC")
                    .long("--dumptvc")
                    .short("-d")
                    .takes_value(true)
                    .help("Path to the TVC file where to save the dump."))))
}

pub async fn decode_command(m: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    if let Some(m) = m.subcommand_matches("body") {
        return decode_body_command(m, config).await;
    }
    if let Some(m) = m.subcommand_matches("msg") {
        return decode_message_command(m, config).await;
    }
    if let Some(m) = m.subcommand_matches("compiler_version") {
        return decode_version_command(m, config).await;
    }
    if let Some(m) = m.subcommand_matches("tvc") {
        return decode_tvc_command(m, config).await;
    }
    if let Some(m) = m.subcommand_matches("account") {
        if let Some(m) = m.subcommand_matches("boc") {
            return decode_account_from_boc(m, config).await;
        }
        if let Some(m) = m.subcommand_matches("data") {
            return decode_data_command(m, config).await;
        }
    }
    Err("unknown command".to_owned())
}

async fn decode_data_command(m: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    if m.is_present("TVC") {
        return decode_tvc_fields(m, config).await;
    }
    if m.is_present("ADDRESS") {
        return decode_account_fields(m, config).await;
    }
    Err("unknown command".to_owned())
}

async fn decode_body_command(m: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let body = m.value_of("BODY");
    let abi = Some(
        match_abi_path(m, &config)
            .ok_or("ABI file is not defined. Supply it in the config file or command line.".to_string())?
    );
    if !config.is_json {
        print_args!(body, abi);
    }
    println!("{}", decode_body(body.unwrap(), &abi.unwrap(), config.is_json).await?);
    Ok(())
}

async fn decode_account_from_boc(m: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let boc = m.value_of("BOCFILE");
    let tvc_path = m.value_of("DUMPTVC");

    if !config.is_json {
        print_args!(boc, tvc_path);
    }

    let account = Account::construct_from_file(boc.unwrap())
        .map_err(|e| format!(" failed to load account from the boc file: {}", e))?;

    print_account_data(&account, tvc_path, config)
}

pub fn print_account_data(account: &Account, tvc_path: Option<&str>, config: Config) -> Result<(), String> {
    if account.is_none() {
        println!("\nAccount is None");
        return Ok(());
    }
    let state_init = account.state_init();

    let address = match account.get_addr() {
        Some(address) => format!("{}", address),
        _ => "Undefined".to_owned(),
    };

    let state = match account.status() {
        AccountStatus::AccStateUninit => "Uninit".to_owned(),
        AccountStatus::AccStateFrozen => "Frozen".to_owned(),
        AccountStatus::AccStateActive => "Active".to_owned(),
        AccountStatus::AccStateNonexist => "NonExist".to_owned(),
    };

    let balance = match account.balance() {
        Some(balance) => format!("{}", balance.grams.clone()),
        _ => "Undefined".to_owned(),
    };

    let trans_lt = account.last_tr_time()
        .map_or("Undefined".to_owned(), |v| format!("{}", v));
    let paid = format!("{}", account.last_paid());

    let (si, code_hash) = match state_init {
        Some(state_init) => {
            let code = state_init.code.clone()
                .ok_or("failed to obtain code from the StateInit")?;
            (msg_printer::state_init_to_str(state_init, config.is_json)?,
             Some(code.repr_hash().to_hex_string()))
        },
        _ => ("Undefined".to_owned(), None)
    };

    print_account(
        &config,
        Some(state),
        Some(address),
        Some(balance),
        Some(paid),
        Some(trans_lt),
        None,
        code_hash,
        Some(si),
    );

    if tvc_path.is_some() && state_init.is_some() {
        state_init.unwrap().write_to_file(tvc_path.unwrap())
            .map_err(|e| format!("{}", e))?;
    }

    Ok(())
}

async fn decode_message_command(m: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let msg = m.value_of("MSG");
    let abi = Some(
        match_abi_path(m, &config)
            .ok_or("ABI file is not defined. Supply it in the config file or command line.".to_string())?
    );
    if !config.is_json {
        print_args!(msg, abi);
    }
    let msg = msg.map(|f| std::fs::read(f))
        .transpose()
        .map_err(|e| format!(" failed to read msg boc file: {}", e))?
        .unwrap();
    println!("{}", decode_message(msg, abi, config.is_json).await?);
    Ok(())
}

async fn decode_tvc_fields(m: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let tvc = m.value_of("TVC");
    let abi = Some(
        match_abi_path(m, &config)
            .ok_or("ABI file is not defined. Supply it in the config file or command line.".to_string())?
    );
    if !config.is_json {
        print_args!(tvc, abi);
    }
    let abi = std::fs::read_to_string(abi.unwrap())
        .map_err(|e| format!("failed to read ABI file: {}", e))?;
    let state = StateInit::construct_from_file(tvc.unwrap())
        .map_err(|e| format!("failed to load StateInit from the tvc file: {}", e))?;
    let b64 = tree_of_cells_into_base64(state.data.as_ref())?;
    let ton = create_client_local()?;
    let res = decode_account_data(
        ton,
        ParamsOfDecodeAccountData {
                abi: Abi::Json(abi),
                data: b64,
            }
        )
        .await
        .map_err(|e| format!("failed to decode data: {}", e))?;
    println!("TVC fields:\n{}", serde_json::to_string_pretty(&res.data)
        .map_err(|e| format!("failed to serialize the result: {}", e))?);
    Ok(())
}

async fn decode_account_fields(m: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let address = m.value_of("ADDRESS");
    let abi = Some(
        match_abi_path(m, &config)
            .ok_or("ABI file is not defined. Supply it in the config file or command line.".to_string())?
    );
    if !config.is_json {
        print_args!(address, abi);
    }
    let abi = std::fs::read_to_string(abi.unwrap())
        .map_err(|e| format!("failed to read ABI file: {}", e))?;

    let ton = create_client_verbose(&config)?;

    let data = query_field(ton.clone(), &address.unwrap(), "data").await?;

    let res = decode_account_data(
        ton,
        ParamsOfDecodeAccountData {
                abi: Abi::Json(abi),
                data,
            }
        )
        .await
        .map_err(|e| format!("failed to decode data: {}", e))?;
    println!("Account fields:\n{}", serde_json::to_string_pretty(&res.data)
        .map_err(|e| format!("failed to serialize the result: {}", e))?);
    Ok(())
}

async fn print_decoded_body(body_vec: Vec<u8>, abi: &str, is_json: bool) -> Result<String, String> {
    let ton = create_client_local()?;
    let mut empty_boc = vec![];
    serialize_tree_of_cells(&Cell::default(), &mut empty_boc)
        .map_err(|e| format!("failed to serialize tree of cells: {}", e))?;
    if body_vec.cmp(&empty_boc) == std::cmp::Ordering::Equal {
        return Err(format!("body is empty"));
    }

    let body_base64 = base64::encode(&body_vec);
    let mut res = {
        match decode_msg_body(ton.clone(), abi, &body_base64, false).await {
            Ok(res) => res,
            Err(_) => decode_msg_body(ton.clone(), abi, &body_base64, true).await?,
        }
    };
    let output = res.value.take().ok_or("failed to obtain the result")?;
    Ok(if is_json {
        format!(" \"BodyCall\": {{\n  \"{}\": {}\n }}", res.name, output)
    } else {
        format!("{}: {}", res.name, serde_json::to_string_pretty(&output)
            .map_err(|e| format!("failed to serialize the result: {}", e))?)
    })
}

async fn decode_body(body: &str, abi: &str, is_json: bool) -> Result<String, String> {
    let abi = std::fs::read_to_string(abi)
        .map_err(|e| format!("failed to read ABI file: {}", e))?;

    let body_vec  = base64::decode(body)
        .map_err(|e| format!("body is not a valid base64 string: {}", e))?;

    let mut result = String::new();
    let s = &mut result;
    if is_json { writeln!(s, "{{").map_err(|e| format!("failed to serialize the result: {}", e))?; }
    writeln!(s, "{}", print_decoded_body(body_vec, &abi, is_json).await?)
        .map_err(|e| format!("failed to serialize the result: {}", e))?;
    if is_json { writeln!(s, "}}").map_err(|e| format!("failed to serialize the result: {}", e))?; }
    Ok(result)
}

async fn decode_message(msg_boc: Vec<u8>, abi: Option<String>, is_json: bool) -> Result<String, String> {
    let abi = abi.map(|f| std::fs::read_to_string(f))
        .transpose()
        .map_err(|e| format!("failed to read ABI file: {}", e))?;

    let tvm_msg = ton_sdk::Contract::deserialize_message(&msg_boc[..])
        .map_err(|e| format!("failed to deserialize message boc: {}", e))?;

    let mut printer = msg_printer::MsgPrinter::new(&tvm_msg, is_json);
    let mut result = String::new();
    let s = &mut result;
    write!(s, "{}", printer.print(false)?).map_err(|e| format!("failed to serialize the result: {}", e))?;

    if abi.is_some() && tvm_msg.body().is_some() {
        let abi = abi.unwrap();
        let mut body_vec = Vec::new();
        serialize_tree_of_cells(&tvm_msg.body().unwrap().into_cell(), &mut body_vec)
            .map_err(|e| format!("failed to serialize body: {}", e))?;

        writeln!(s, "{}", print_decoded_body(body_vec, &abi, is_json).await?)
            .map_err(|e| format!("failed to serialize the result: {}", e))?;
    }
    if is_json { writeln!(s, "}}").map_err(|e| format!("failed to serialize the result: {}", e))?; }
    Ok(result)
}

fn load_state_init(m: &ArgMatches<'_>) -> Result<StateInit, String> {
    let input = m.value_of("INPUT").unwrap();
    let stat_init = if m.is_present("BOC") {
        let account = Account::construct_from_file(input)
            .map_err(|e| format!(" failed to load account from the boc file {}: {}", input, e))?;
        account.state_init().ok_or("Failed to load stateInit from the BOC.")?.to_owned()
    } else {
        StateInit::construct_from_file(input)
            .map_err(|e| format!("failed to load StateInit from the tvc file: {}", e))?
    };
    Ok(stat_init)
}

async fn query_field(ton: TonClient, address: &str, field: &str) -> Result<String, String> {
    let accounts = query(
        ton.clone(),
        "accounts",
        json!({ "id": { "eq": address } }),
        field,
        None,
    ).await
        .map_err(|e| format!("failed to query account data: {}", e))?;

    if accounts.len() == 0 {
        return Err(format!("account not found"));
    }
    let data = accounts[0][field].as_str();
    if data.is_none() {
        return Err(format!("account doesn't contain {}", field));
    }
    Ok(data.unwrap().to_string())
}

fn parse_arg_and_create_client(m: &ArgMatches<'_>, config: Config) -> Result<(String, TonClient), String>{
    let input = m.value_of("INPUT");
    if !config.is_json {
        print_args!(input);
    }
    let ton = if m.is_present("BOC") || m.is_present("TVC") {
        create_client_local()?
    } else {
        create_client_verbose(&config)?
    };

    Ok((input.unwrap().to_owned(), ton))
}

async fn get_version(ton: TonClient, code: String) -> Result<String, String>{
    let result = get_compiler_version(
        ton,
        ParamsOfGetCompilerVersion {
            code
        }
    ).await
        .map_err(|e| format!("Failed to get compiler version: {}", e))?;

    let version = if result.version.is_some() {
        result.version.unwrap()
    } else {
        "Undefined".to_owned()
    };
    Ok(version)
}


async fn decode_tvc_command(m: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let (input, ton) = parse_arg_and_create_client(m, config.clone())?;
    let state = if m.is_present("BOC") || m.is_present("TVC") {
        load_state_init(m)?
    } else {
        let input = if input.contains(":") {
            input
        } else {
            format!("{}:{}", config.wc, input)
        };
        let boc = query_field(ton.clone(), &input, "boc").await?;
        let account = Account::construct_from_base64(&boc)
            .map_err(|e| format!("Failed to query account BOC: {}", e))?;
        account.state_init().ok_or("Failed to load stateInit from the BOC.")?.to_owned()
    };

    let code = tree_of_cells_into_base64(state.code.as_ref())?;
    println!("StateInit\n split_depth: {}\n special: {}\n data: {}\n code: {}\n code_hash: {}\n data_hash: {}\n code_depth: {}\n data_depth: {}\n version: {}\n lib:  {}\n",
        state.split_depth.as_ref().map(|x| format!("{:?}", (x.0 as u8))).unwrap_or("None".to_string()),
        state.special.as_ref().map(|x| format!("{:?}", x)).unwrap_or("None".to_string()),
        tree_of_cells_into_base64(state.data.as_ref())?,
        code.clone(),
        state.code.clone().unwrap().repr_hash().to_hex_string(),
        state.data.clone().unwrap().repr_hash().to_hex_string(),
        state.code.clone().unwrap().depth(0),
        state.data.clone().unwrap().depth(0),
        get_version(ton, code).await?,
        tree_of_cells_into_base64(state.library.root())?,
    );
    Ok(())
}

async fn decode_version_command(m: &ArgMatches<'_>, config: Config) -> Result<(), String> {
    let (input, ton) = parse_arg_and_create_client(m, config.clone())?;
    let code = if m.is_present("BOC") || m.is_present("TVC") {
        let state_init = load_state_init(m)?;
        let code = state_init.code.ok_or("StateInit doesn't contain code.")?;
        let mut bytes = vec![];
        serialize_tree_of_cells(&code, &mut bytes)
            .map_err(|e| format!("failed to serialize tree of cells: {}", e))?;
        base64::encode(&bytes)
    } else {
        let input = if input.contains(":") {
            input
        } else {
            format!("{}:{}", config.wc, input)
        };
        query_field(ton.clone(), &input, "code").await?
    };

    let result = get_version(ton, code).await?;
    if !config.is_json {
        println!("Version: {}", result);
    } else {
        println!("{{");
        println!("  \"version\": \"{}\"", result);
        println!("}}");
    }

    Ok(())
}


mod msg_printer {
    use ton_block::*;
    use ton_types::cells_serialization::serialize_tree_of_cells;
    use ton_types::Cell;
    use std::fmt::Write as FmtWrite;

    pub struct MsgPrinter<'a> {
        start: &'static str,
        off: &'static str,
        end: &'static str,
        msg: &'a Message,
        is_json: bool,
    }

    impl<'a> MsgPrinter<'a> {
        pub fn new(msg: &'a Message, is_json: bool) -> Self {
            MsgPrinter {off: " ", start: "\"", end: "\",", msg, is_json }
        }

        pub fn print(&mut self, close: bool) -> Result<String, String> {
            let mut result = String::new();
            let s = &mut result;
            if self.is_json {
                write!(s, "{{\n").map_err(|e| format!("failed to serialize the result: {}", e))?;
            }
            self.json(s, "Type", &self.print_msg_type())?;
            let hdr = self.print_msg_header()?;
            self.start = "{\n";
            self.end = " },";
            self.off = " ";
            self.json(s, "Header", &hdr)?;
            self.state_init_printer(s)?;
            self.start = "\"";
            if close { self.end = "\""; } else { self.end = "\","; }
            self.off = " ";
            self.json(s, "Body", &tree_of_cells_into_base64(
                self.msg.body().map(|slice| slice.into_cell()).as_ref(),
            )?)?;
            if self.is_json && close {
                write!(s, "}}\n").map_err(|e| format!("failed to serialize the result: {}", e))?;
            }
            Ok(result)
        }

        fn print_msg_type(&self) -> String {
            match self.msg.header() {
                CommonMsgInfo::IntMsgInfo(_) => "internal",
                CommonMsgInfo::ExtInMsgInfo(_) => "external inbound",
                CommonMsgInfo::ExtOutMsgInfo(_) => "external outbound",
            }.to_owned() + " message"
        }


        fn json<T: std::fmt::Display>(&self, s: &mut String, name: &str, value: &T) -> Result<(), String>{
            write!(s, "{}\"{}\": {}{}{}\n", self.off, name, self.start, value, self.end)
                .map_err(|e| format!("failed to serialize message: {}", e))?;
            Ok(())
        }

        fn print_msg_header(&mut self) -> Result<String, String> {
            let mut result = String::new();
            let s = &mut result;
            self.start = "\"";
            self.end = "\",";
            self.off = "   ";
            match self.msg.header() {
                CommonMsgInfo::IntMsgInfo(header) => {
                    self.json(s, "ihr_disabled", &header.ihr_disabled)?;
                    self.json(s, "bounce", &header.bounce)?;
                    self.json(s, "bounced", &header.bounced)?;
                    self.json(s, "source", &header.src)?;
                    self.json(s, "destination", &header.dst)?;
                    self.json(s, "value", &print_cc(&header.value))?;
                    self.json(s, "ihr_fee", &print_grams(&header.ihr_fee))?;
                    self.json(s, "fwd_fee", &print_grams(&header.fwd_fee))?;
                    self.json(s, "created_lt", &header.created_lt)?;
                    self.end = "\"";
                    self.json(s, "created_at", &header.created_at)?;
                },
                CommonMsgInfo::ExtInMsgInfo(header) => {
                    self.json(s, "source", &header.src)?;
                    self.json(s, "destination", &header.dst)?;
                    self.end = "\"";
                    self.json(s, "import_fee", &print_grams(&header.import_fee))?;
                },
                CommonMsgInfo::ExtOutMsgInfo(header) => {
                    self.json(s, "source", &header.src)?;
                    self.json(s, "destination", &header.dst)?;
                    self.json(s, "created_lt", &header.created_lt)?;
                    self.end = "\"";
                    self.json(s, "created_at", &header.created_at)?;
                }
            };
            Ok(result)
        }

        fn state_init_printer(&self, s: &mut String) -> Result<(), String>{
            match self.msg.state_init().as_ref() {
                Some(x) => {
                    let init = format!(
                        "StateInit{}",
                        state_init_to_str(x, false)?
                    );
                    self.json(s, "Init", &init)?;
                },
                None => (),
            };
            Ok(())
        }
    }

    pub fn state_init_to_str(state_init: &StateInit, is_json: bool) -> Result<String, String> {
        if !is_json {
            Ok(format!("\n split_depth: {}\n special: {}\n data: {}\n code: {}\n lib:  {}\n",
                state_init.split_depth.as_ref().map(|x| format!("{:?}", x)).unwrap_or("None".to_string()),
                state_init.special.as_ref().map(|x| format!("{:?}", x)).unwrap_or("None".to_string()),
                tree_of_cells_into_base64(state_init.data.as_ref())?,
                tree_of_cells_into_base64(state_init.code.as_ref())?,
                tree_of_cells_into_base64(state_init.library.root())?
            ))
        } else {
            Ok(format!("{{\n    \"split_depth\": \"{}\"\n    \"special\": \"{}\"\n    \"data\": \"{}\"\n    \"code\": \"{}\"\n    \"lib\":  \"{}\"\n  }}",
                state_init.split_depth.as_ref().map(|x| format!("{:?}", x)).unwrap_or("None".to_string()),
                state_init.special.as_ref().map(|x| format!("{:?}", x)).unwrap_or("None".to_string()),
                tree_of_cells_into_base64(state_init.data.as_ref())?,
                tree_of_cells_into_base64(state_init.code.as_ref())?,
                tree_of_cells_into_base64(state_init.library.root())?
            ))
        }
    }

    pub fn tree_of_cells_into_base64(root_cell: Option<&Cell>) -> Result<String, String> {
        match root_cell {
            Some(cell) => {
                let mut bytes = Vec::new();
                serialize_tree_of_cells(cell, &mut bytes)
                    .map_err(|e| format!("failed to serialize tree of cells: {}", e))?;
                Ok(base64::encode(&bytes))
            }
            None => Ok("".to_string())
        }
    }

    fn print_grams(grams: &Grams) -> String {
        grams.0.to_string()
    }

    fn print_cc(cc: &CurrencyCollection) -> String {
        let mut result = print_grams(&cc.grams);
        if !cc.other.is_empty() {
            result += " other: {";
            cc.other.iterate_with_keys(|key: u32, value| {
                result += &format!(" \"{}\": \"{}\",", key, value.0);
                Ok(true)
            }).ok();
            result.pop(); // remove extra comma
            result += " }";
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_decode_msg_json() {
        let msg_boc = std::fs::read("tests/samples/wallet.boc").unwrap();
        let out = decode_message(msg_boc, Some("tests/samples/wallet.abi.json".to_owned()), true).await.unwrap();
        let _ : serde_json::Value = serde_json::from_str(&out).unwrap();
    }

    #[tokio::test]
    async fn test_decode_body_json() {
        let body = "te6ccgEBAQEARAAAgwAAALqUCTqWL8OX7JivfJrAAzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMQAAAAAAAAAAAAAAAEeGjADA==";
        let out = decode_body(body, "tests/samples/wallet.abi.json", true).await.unwrap();
        let _ : serde_json::Value = serde_json::from_str(&out).unwrap();
    }
}
