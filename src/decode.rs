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
use crate::helpers::create_client_local;
use clap::{ArgMatches, SubCommand, Arg, App, AppSettings};
use ton_types::cells_serialization::serialize_tree_of_cells;
use ton_types::Cell;

fn match_abi_path(matches: &ArgMatches, config: &Config) -> Option<String> {
    matches.value_of("ABI")
        .map(|s| s.to_string())
        .or(config.abi_path.clone())        
}

pub fn create_decode_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("decode")
        .about("Decode commands.")
        .setting(AppSettings::AllowLeadingHyphen)  
        .setting(AppSettings::TrailingVarArg)
        .setting(AppSettings::DontCollapseArgsInUsage)
        .subcommand(SubCommand::with_name("body")
            .arg(Arg::with_name("BODY")
                .required(true)
                .help("Message body encoded as base64."))
            .arg(Arg::with_name("ABI")
                .long("--abi")
                .takes_value(true)
                .help("Path to ABI file.")))
        .subcommand(SubCommand::with_name("msg")
           .arg(Arg::with_name("MSG")
                    .required(true)
                    .help("Path to message boc file."))
            .arg(Arg::with_name("ABI")
                    .long("--abi")
                    .takes_value(true)
                    .help("Path to ABI file.")))
}

pub fn decode_command(m: &ArgMatches, config: Config) -> Result<(), String> {
    if let Some(m) = m.subcommand_matches("body") {
        return decode_body_command(m, config);
    }
    if let Some(m) = m.subcommand_matches("msg") {
        return decode_message_command(m, config);
    }
    Err("unknown command".to_owned())
}

fn decode_body_command(m: &ArgMatches, config: Config) -> Result<(), String> {
    let body = m.value_of("BODY");
    let abi = Some(
        match_abi_path(m, &config)
            .ok_or("ABI file not defined. Supply it in config file or command line.".to_string())?
    );
    print_args!(m, body, abi);
    decode_body(body.unwrap(), &abi.unwrap())
}

fn decode_message_command(m: &ArgMatches, config: Config) -> Result<(), String> {
    let msg = m.value_of("MSG");
    let abi = Some(
        match_abi_path(m, &config)
            .ok_or("ABI file not defined. Supply it in config file or command line.".to_string())?
    );
    print_args!(m, msg, abi);
    let msg = msg.map(|f| std::fs::read(f))
        .transpose()
        .map_err(|e| format!(" failed to read msg boc file: {}", e))?
        .unwrap();
    decode_message(msg, abi)
}

fn print_decoded_body(body_vec: Vec<u8>, abi: &str) -> Result<(), String> {
    let ton = create_client_local()?;
    let res = ton.contracts.decode_input_message_body(
        abi.into(),
        &body_vec,
        false,
    ).or_else(|_| ton.contracts.decode_input_message_body(
        abi.into(),
        &body_vec,
        true,
    )).or_else(|e| {
        let mut boc = vec![];
        serialize_tree_of_cells(&Cell::default(), &mut boc).unwrap();
        if hex::encode(&boc) == hex::encode(&body_vec) {
            Err(format!("body is empty"))
        } else {
            Err(format!("failed to decode msg body: {}", e))
        }
    })?;

    println!("DecodedBody: {}({})", res.function, serde_json::to_string_pretty(&res.output).unwrap());
    Ok(())
}

fn decode_body(body: &str, abi: &str) -> Result<(), String> {
    let abi = std::fs::read_to_string(abi)
        .map_err(|e| format!("failed to read ABI file: {}", e))?;

    let body_vec  = base64::decode(body)
        .map_err(|e| format!("body is not a valid base64 string: {}", e))?;
    print_decoded_body(body_vec, &abi)
}

fn decode_message(msg_boc: Vec<u8>, abi: Option<String>) -> Result<(), String> {
    let abi = abi.map(|f| std::fs::read_to_string(f))
        .transpose()
        .map_err(|e| format!("failed to read ABI file: {}", e))?;
    
    let tvm_msg = ton_sdk::Contract::deserialize_message(&msg_boc[..])
        .map_err(|e| format!("failed to deserialize message boc: {}", e))?;
    
    println!("{}", msg_printer::print(&tvm_msg));

    if abi.is_some() && tvm_msg.body().is_some() {
        let abi = abi.unwrap();
        let mut body_vec = Vec::new();
        serialize_tree_of_cells(&tvm_msg.body().unwrap().into_cell(), &mut body_vec)
            .map_err(|e| format!("failed to serialize body: {}", e))?;
        
        print_decoded_body(body_vec, &abi)?;
    }
    Ok(())
}


mod msg_printer {
    use ton_block::*;
    use ton_types::cells_serialization::serialize_tree_of_cells;
    use ton_types::Cell;
    
    pub fn print(msg: &Message) -> String {
        let none = "None".to_string();
        format!("Type: {}\nHeader:\n{}Init: {}\nBody: {}",
            print_msg_type(&msg.header()),
            print_msg_header(&msg.header()),
            msg.state_init().as_ref().map(|x| {
                format!("{}", state_init_printer(x))
            }).unwrap_or(none.clone()),
            tree_of_cells_into_base64(
                msg.body()
                    .map(|slice| slice.into_cell())
                    .as_ref(),
            ),
        )
    }

    fn print_msg_type(header: &CommonMsgInfo) -> String {
        match header {
            CommonMsgInfo::IntMsgInfo(_) => "internal",
            CommonMsgInfo::ExtInMsgInfo(_) => "external inbound",
            CommonMsgInfo::ExtOutMsgInfo(_) => "external outbound",
        }.to_owned() + " message"
    }
    
    fn print_msg_header(header: &CommonMsgInfo) -> String {
        match header {
            CommonMsgInfo::IntMsgInfo(header) => {
                format!(" ihr_disabled: {}\n", header.ihr_disabled) +
                &format!(" bounce      : {}\n", header.bounce) +
                &format!(" bounced     : {}\n", header.bounced) +
                &format!(" source      : {}\n", &header.src) +
                &format!(" destination : {}\n", &header.dst) +
                &format!(" value       : {}\n", print_cc(&header.value)) +
                &format!(" ihr_fee     : {}\n", print_grams(&header.ihr_fee)) +
                &format!(" fwd_fee     : {}\n", print_grams(&header.fwd_fee)) +
                &format!(" created_lt  : {}\n", header.created_lt) +
                &format!(" created_at  : {}\n", header.created_at)
            },
            CommonMsgInfo::ExtInMsgInfo(header) => {
                format!( " source      : {}\n", &header.src) +
                &format!(" destination : {}\n", &header.dst) +
                &format!(" import_fee  : {}\n", print_grams(&header.import_fee))
            },
            CommonMsgInfo::ExtOutMsgInfo(header) => {
                format!( " source      : {}\n", &header.src) +
                &format!(" destination : {}\n", &header.dst) +
                &format!(" created_lt  : {}\n", header.created_lt) +
                &format!(" created_at  : {}\n", header.created_at)
            }
        }
    }

    fn state_init_printer(state: &StateInit) -> String {
        format!("StateInit\n split_depth: {}\n special: {}\n data: {}\n code: {}\n lib:  {}\n",
            state.split_depth.as_ref().map(|x| format!("{:?}", x)).unwrap_or("None".to_string()),
            state.special.as_ref().map(|x| format!("{:?}", x)).unwrap_or("None".to_string()),
            tree_of_cells_into_base64(state.data.as_ref()),
            tree_of_cells_into_base64(state.code.as_ref()),
            tree_of_cells_into_base64(state.library.root()),
        )
    }
    
    pub fn tree_of_cells_into_base64(root_cell: Option<&Cell>) -> String {
        match root_cell {
            Some(cell) => {
                let mut bytes = Vec::new();
                serialize_tree_of_cells(cell, &mut bytes).unwrap();
                base64::encode(&bytes)
            }
            None => "None".to_string()
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