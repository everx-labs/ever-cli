/*
 * Copyright 2023 TON DEV SOLUTIONS LTD.
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

use std::{collections::HashSet, sync::Arc};

use ton_labs_assembler::DbgInfo;
use ton_types::{UInt256, SliceData};
use ton_utils::disasm::loader::{Loader, print_code_ex};
use ton_vm::{stack::savelist::SaveList, executor::{Engine, EngineTraceInfo, EngineTraceInfoType}};

fn help() {
    println!("h or ?  for help");
    println!("q       quit the debugger");
    println!("s       step one instruction");
    println!("b loc   set a breakpoint at loc = hash:offset");
    println!("c       continue execution until a breakpoint");
    println!("p gas   print gas");
    println!("p sN    print stack slot N");
    println!("p sN..  print stack slots from s0 to sN");
    println!("p cN    print control register");
    println!("p cc    print current continuation register");
    println!("p       print all stack and registers");
    println!("<enter> repeat last command");
}

#[derive(Eq, PartialEq, Hash)]
struct Breakpoint {
    cell_hash: UInt256,
    offset: usize,
}

impl Breakpoint {
    fn new(cell_hash: UInt256, offset: usize) -> Self {
        Self { cell_hash, offset }
    }
}

#[derive(PartialEq)]
enum Next {
    Start,
    Step,
    Continue,
    Quit,
}

struct DebugState {
    editor: rustyline::Editor<()>,
    next: Next,
    last: Option<Command>,
    breakpoints: HashSet<Breakpoint>,
}

lazy_static::lazy_static!(
    static ref DEBUG_STATE: Arc<std::sync::Mutex<Option<DebugState>>> =
        Arc::new(std::sync::Mutex::new(None));
);

#[derive(Clone)]
enum Command {
    Help,
    Quit,
    Step,
    Continue,
    Breakpoint(UInt256, usize),
    PrintGas,
    PrintStack(std::ops::Range<usize>),
    PrintRegister(usize),
    PrintCC,
    PrintAll,
}

fn parse_line(state: &mut DebugState, line: String) -> Result<Command, String> {
    let lower = line.to_lowercase();
    let split = lower.split_whitespace().collect::<Vec<&str>>();
    if split.len() == 0 {
        if let Some(cmd) = &state.last {
            return Ok(cmd.clone())
        } else {
            return Err("there's no last command".to_string())
        }
    } else if split.len() == 1 {
        let word = split[0];
        if word == "h" || word == "?" {
            return Ok(Command::Help)
        } else if word == "q" {
            return Ok(Command::Quit)
        } else if word == "s" {
            return Ok(Command::Step)
        } else if word == "c" {
            return Ok(Command::Continue)
        } else if word == "p" {
            return Ok(Command::PrintAll)
        }
    } else if split.len() == 2 {
        let word1 = split[0];
        let word2 = split[1];
        if word1 == "p" {
            if word2 == "gas" {
                return Ok(Command::PrintGas)
            } else if word2 == "cc" {
                return Ok(Command::PrintCC);
            } else if word2.starts_with("c") {
                let num = word2[1..].parse::<u8>()
                    .map_err(|e| format!("{}", e))?;
                return Ok(Command::PrintRegister(num as usize))
            } else if word2.starts_with("s") {
                if word2.ends_with("..") {
                    let num = word2[1..word2.len() - 2].parse::<u32>()
                        .map_err(|e| format!("{}", e))?;
                    return Ok(Command::PrintStack(0..num as usize + 1))
                } else {
                    let num = word2[1..].parse::<u32>()
                        .map_err(|e| format!("{}", e))?;
                    return Ok(Command::PrintStack(num as usize..num as usize + 1))
                }
            }
        } else if word1 == "b" {
            let (s1, s2) = word2.split_once(":")
                .ok_or(format!("bad parameter {}", word2))?;
            let cell_hash = hex::decode(s1)
                .map_err(|e| format!("{}", e))?;
            let cell_hash = UInt256::from_raw(cell_hash, 256);
            let offset = hex::decode(s2)
                .map_err(|e| format!("{}", e))?;
            if offset.len() != 1 {
                return Err(format!("failed to parse offset {}", s2))
            }
            return Ok(Command::Breakpoint(cell_hash, offset[0] as usize))
        }
    }
    Err("unknown command".to_string())
}

fn print_cc(info: &EngineTraceInfo) {
    let cc = info.cmd_code.cell().clone();
    let mut loader = Loader::new(false);
    match loader.load(&mut SliceData::load_cell(cc).unwrap(), false) {
        Err(_) => println!("failed to disasm"),
        Ok(code) => {
            let disasm = print_code_ex(&code, "", false);
            print!("{}", disasm);
        }
    }
}

fn execute_line(state: &mut DebugState, engine: &Engine, info: &EngineTraceInfo) {
    loop {
        match state.editor.readline("(dbg) ") {
            Ok(line) => {
                let cmd = parse_line(state, line.clone());
                if let Ok(cmd) = &cmd {
                    state.editor.add_history_entry(line.as_str());
                    state.last = Some(cmd.clone());
                }
                match cmd {
                    Ok(Command::Help) => help(),
                    Ok(Command::Quit) => {
                        state.next = Next::Quit;
                        break
                    }
                    Ok(Command::Step) => {
                        state.next = Next::Step;
                        break
                    }
                    Ok(Command::Continue) => {
                        state.next = Next::Continue;
                        break
                    }
                    Ok(Command::PrintGas) => {
                        let gas = engine.get_gas();
                        println!("    used remaining   credit    limit");
                        println!("{: >8}  {: >8} {: >8} {: >8}",
                            gas.get_gas_used(),
                            gas.get_gas_remaining(),
                            gas.get_gas_credit(),
                            gas.get_gas_limit(),
                        )
                    }
                    Ok(Command::PrintStack(range)) => {
                        let depth = engine.stack().depth();
                        if depth < range.end {
                            println!("invalid stack range (stack depth is {})", depth)
                        } else {
                            for i in range {
                                println!("s{}: {}", i, engine.stack().get(i));
                            }
                        }
                    }
                    Ok(Command::PrintRegister(reg)) => {
                        match engine.ctrl(reg) {
                            Ok(item) => println!("{}", item),
                            Err(e) => println!("{}", e)
                        }
                    }
                    Ok(Command::PrintCC) => {
                        print_cc(info)
                    }
                    Ok(Command::PrintAll) => {
                        let depth = engine.stack().depth();
                        for i in 0..depth {
                            println!("s{}: {}", i, engine.stack().get(i))
                        }
                        for reg in SaveList::REGS {
                            match engine.ctrl(reg) {
                                Ok(item) => println!("c{}: {}", reg, item),
                                Err(e) => println!("c{}: {}", reg, e)
                            }
                        }
                        println!("cc:");
                        print_cc(info)
                    }
                    Ok(Command::Breakpoint(cell_hash, offset)) => {
                        state.breakpoints.insert(Breakpoint::new(cell_hash, offset));
                        println!("Breakpoint added");
                    }
                    Err(e) => println!("{}", e),
                }
            }
            Err(e) => {
                println!("{}", e);
                state.next = Next::Quit;
                break;
            }
        }
    }
}

const INST_WIDTH: usize = 32;
const SOURCE_POS_WIDTH: usize = 24;

pub fn debug_callback(engine: &Engine, info: &EngineTraceInfo, debug_info: &Option<DbgInfo>) {
    let mut state = DEBUG_STATE.lock().unwrap();
    if info.info_type == EngineTraceInfoType::Start {
        println!("Interactive debugger");
        println!("For help, type h or ?");
        *state = Some(DebugState {
            editor: rustyline::Editor::<()>::new().unwrap(),
            next: Next::Start,
            last: None,
            breakpoints: HashSet::new(),
        });
        execute_line(state.as_mut().unwrap(), engine, info);
        return
    }
    let state = state.as_mut().unwrap();
    if state.next == Next::Quit {
        return
    }
    if info.info_type == EngineTraceInfoType::Implicit {
        println!("{}", info.cmd_str);
        return
    }
    if info.info_type == EngineTraceInfoType::Exception {
        execute_line(state, engine, info);
        return
    }
    let cell_hash = info.cmd_code.cell().repr_hash();
    let offset = info.cmd_code.pos();
    println!("{: <w1$} at {: <w2$}{}:{:02x}",
        trim(info.cmd_str.clone(), INST_WIDTH),
        if debug_info.is_some() {
            let pos = get_position(info, debug_info);
            trim(pos, SOURCE_POS_WIDTH)
        } else {
            String::new()
        },
        cell_hash.to_hex_string(),
        offset as u8,
        w1 = INST_WIDTH,
        w2 = if debug_info.is_none() { 0 } else { SOURCE_POS_WIDTH }
    );
    if state.next == Next::Quit {
        return
    } else if state.next == Next::Continue {
        let p = Breakpoint::new(cell_hash.clone(), offset);
        if !state.breakpoints.contains(&p) {
            return
        }
    }
    execute_line(state, engine, info);
}

fn trim(s: String, len: usize) -> String {
    assert!(len > 0);
    if s.len() > len {
        format!("{}…", &s[..len - 1])
    } else {
        s
    }
}

fn get_position(info: &EngineTraceInfo, debug_info: &Option<DbgInfo>) -> String {
    if let Some(debug_info) = debug_info {
        let cell_hash = info.cmd_code.cell().repr_hash();
        let offset = info.cmd_code.pos();
        match debug_info.get(&cell_hash) {
            Some(offset_map) => match offset_map.get(&offset) {
                Some(pos) => format!("{}:{} ", pos.filename, pos.line),
                None => String::new()
            },
            None => String::new()
        }
    } else {
        String::new()
    }
}
