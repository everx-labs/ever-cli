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
use std::fmt::Write;

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
    if !config.is_json {
        print_args!(m, body, abi);
    }
    println!("{}", decode_body(body.unwrap(), &abi.unwrap(), config.is_json)?);
    Ok(())
}

fn decode_message_command(m: &ArgMatches, config: Config) -> Result<(), String> {
    let msg = m.value_of("MSG");
    let abi = Some(
        match_abi_path(m, &config)
            .ok_or("ABI file not defined. Supply it in config file or command line.".to_string())?
    );
    if !config.is_json {
        print_args!(m, msg, abi);
    }
    let msg = msg.map(|f| std::fs::read(f))
        .transpose()
        .map_err(|e| format!(" failed to read msg boc file: {}", e))?
        .unwrap();
    println!("{}", decode_message(msg, abi, config.is_json)?);
    Ok(())
}

fn print_decoded_body(body_vec: Vec<u8>, abi: &str, is_json: bool) -> Result<String, String> {
    let ton = create_client_local()?;
    let mut empty_boc = vec![];
    serialize_tree_of_cells(&Cell::default(), &mut empty_boc).unwrap();
    if body_vec.cmp(&empty_boc) == std::cmp::Ordering::Equal {
        return Err(format!("body is empty"));
    }
    let res = ton.contracts.decode_input_message_body(
        abi.into(),
        &body_vec,
        false,
    ).or_else(|_| ton.contracts.decode_input_message_body(
        abi.into(),
        &body_vec,
        true,
    )).map_err(|e| format!("failed to decode msg body: {}", e))?;

    Ok(if is_json {
        format!(" \"BodyCall\": {{\n  \"{}\": {}\n }}", res.function, res.output)
    } else {
        format!("{}: {}", res.function, serde_json::to_string_pretty(&res.output).unwrap())
    })
}

fn decode_body(body: &str, abi: &str, is_json: bool) -> Result<String, String> {
    let abi = std::fs::read_to_string(abi)
        .map_err(|e| format!("failed to read ABI file: {}", e))?;

    let body_vec  = base64::decode(body)
        .map_err(|e| format!("body is not a valid base64 string: {}", e))?;

    let mut result = String::new();
    let s = &mut result;
    if is_json { writeln!(s, "{{").unwrap(); }
    writeln!(s, "{}", print_decoded_body(body_vec, &abi, is_json)?).unwrap();
    if is_json { writeln!(s, "}}").unwrap(); }
    Ok(result)
}

fn decode_message(msg_boc: Vec<u8>, abi: Option<String>, is_json: bool) -> Result<String, String> {
    let abi = abi.map(|f| std::fs::read_to_string(f))
        .transpose()
        .map_err(|e| format!("failed to read ABI file: {}", e))?;
    
    let tvm_msg = ton_sdk::Contract::deserialize_message(&msg_boc[..])
        .map_err(|e| format!("failed to deserialize message boc: {}", e))?;
    
    let mut printer = msg_printer::MsgPrinter::new(&tvm_msg, is_json);
    let mut result = String::new();
    let s = &mut result;
    write!(s, "{}", printer.print(false)).unwrap();

    if abi.is_some() && tvm_msg.body().is_some() {
        let abi = abi.unwrap();
        let mut body_vec = Vec::new();
        serialize_tree_of_cells(&tvm_msg.body().unwrap().into_cell(), &mut body_vec)
            .map_err(|e| format!("failed to serialize body: {}", e))?;
        
        writeln!(s, "{}", print_decoded_body(body_vec, &abi, is_json)?).unwrap();
    }
    if is_json { writeln!(s, "}}").unwrap(); }
    Ok(result)
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

        pub fn print(&mut self, close: bool) -> String {
            let mut result = String::new();
            let s = &mut result;
            if self.is_json {
                write!(s, "{{\n").unwrap();
            }
            self.json(s, "Type", &self.print_msg_type());
            let hdr = self.print_msg_header();
            self.start = "{\n";
            self.end = " },";
            self.off = " ";
            self.json(s, "Header", &hdr);
            self.state_init_printer(s);
            self.start = "\"";
            if close { self.end = "\""; } else { self.end = "\","; }
            self.off = " ";
            self.json(s, "Body", &tree_of_cells_into_base64(
                self.msg.body().map(|slice| slice.into_cell()).as_ref(),
            ));
            if self.is_json && close {
                write!(s, "}}\n").unwrap();
            }
            result
        }
        
        fn print_msg_type(&self) -> String {
            match self.msg.header() {
                CommonMsgInfo::IntMsgInfo(_) => "internal",
                CommonMsgInfo::ExtInMsgInfo(_) => "external inbound",
                CommonMsgInfo::ExtOutMsgInfo(_) => "external outbound",
            }.to_owned() + " message"
        }
        
        
        fn json<T: std::fmt::Display>(&self, s: &mut String, name: &str, value: &T) {
            write!(s, "{}\"{}\": {}{}{}\n", self.off, name, self.start, value, self.end).unwrap();
        }
        
        fn print_msg_header(&mut self) -> String {
            let mut result = String::new();
            let s = &mut result;
            self.start = "\"";
            self.end = "\",";
            self.off = "   ";
            match self.msg.header() {
                CommonMsgInfo::IntMsgInfo(header) => {
                    self.json(s, "ihr_disabled", &header.ihr_disabled);
                    self.json(s, "bounce", &header.bounce);
                    self.json(s, "bounced", &header.bounced);
                    self.json(s, "source", &header.src);
                    self.json(s, "destination", &header.dst);
                    self.json(s, "value", &print_cc(&header.value));
                    self.json(s, "ihr_fee", &print_grams(&header.ihr_fee));
                    self.json(s, "fwd_fee", &print_grams(&header.fwd_fee));
                    self.json(s, "created_lt", &header.created_lt);
                    self.end = "\"";
                    self.json(s, "created_at", &header.created_at);
                },
                CommonMsgInfo::ExtInMsgInfo(header) => {
                    self.json(s, "source", &header.src);
                    self.json(s, "destination", &header.dst);
                    self.end = "\"";
                    self.json(s, "import_fee", &print_grams(&header.import_fee));
                },
                CommonMsgInfo::ExtOutMsgInfo(header) => {
                    self.json(s, "source", &header.src);
                    self.json(s, "destination", &header.dst);
                    self.json(s, "created_lt", &header.created_lt);
                    self.end = "\"";
                    self.json(s, "created_at", &header.created_at);
                }
            };
            result
        }
        
        fn state_init_printer(&self, s: &mut String) {
            match self.msg.state_init().as_ref() {
                Some(x) => {
                    let init = format!(
                        "StateInit\n split_depth: {}\n special: {}\n data: {}\n code: {}\n lib:  {}\n",
                        x.split_depth.as_ref().map(|x| format!("{:?}", x)).unwrap_or("None".to_string()),
                        x.special.as_ref().map(|x| format!("{:?}", x)).unwrap_or("None".to_string()),
                        tree_of_cells_into_base64(x.data.as_ref()),
                        tree_of_cells_into_base64(x.code.as_ref()),
                        tree_of_cells_into_base64(x.library.root())
                    );
                    self.json(s, "Init", &init);
                },
                None => (),
            };
        }
    }
    
    pub fn tree_of_cells_into_base64(root_cell: Option<&Cell>) -> String {
        match root_cell {
            Some(cell) => {
                let mut bytes = Vec::new();
                serialize_tree_of_cells(cell, &mut bytes).unwrap();
                base64::encode(&bytes)
            }
            None => "".to_string()
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

    #[test]
    fn test_decode_msg_json() {
        let msg_boc = std::fs::read("tests/samples/wallet.boc").unwrap();
        let out = decode_message(msg_boc, Some("tests/samples/wallet.abi.json".to_owned()), true).unwrap();
        let _ : serde_json::Value = serde_json::from_str(&out).unwrap();
    }

    #[test]
    fn test_decode_body_json() {
        let body = "te6ccgEBAQEARAAAgwAAALqUCTqWL8OX7JivfJrAAzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMQAAAAAAAAAAAAAAAEeGjADA==";
        let out = decode_body(body, "tests/samples/wallet.abi.json", true).unwrap();
        let _ : serde_json::Value = serde_json::from_str(&out).unwrap();
    }
}