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

use clap::{App, ArgMatches, SubCommand, Arg, AppSettings};
use ton_types::cells_serialization::deserialize_cells_tree;
use ton_types::{Cell, SliceData, HashmapE, HashmapType};
use ton_types::Result as Result_;
use std::io::Cursor;
use std::ops::{Not, Range};
use num_traits::Zero;

pub fn create_disasm_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("disasm")
        .about("Decode commands.")
        .setting(AppSettings::AllowLeadingHyphen)
        .setting(AppSettings::TrailingVarArg)
        .setting(AppSettings::DontCollapseArgsInUsage)
        .subcommand(SubCommand::with_name("dump")
            .arg(Arg::with_name("TVC")
                    .required(true)
                    .help("Path to tvc file")))
        .subcommand(SubCommand::with_name("solidity")
            .arg(Arg::with_name("TVC")
                    .required(true)
                    .help("Path to tvc file")))
}

pub fn disasm_command(m: &ArgMatches) -> Result<(), String> {
    if let Some(m) = m.subcommand_matches("dump") {
        return disasm_dump_command(m);
    } else if let Some(m) = m.subcommand_matches("solidity") {
        return disasm_solidity_command(m);
    }
    Err("unknown command".to_owned())
}

fn disasm_dump_command(m: &ArgMatches) -> Result<(), String> {
    let filename = m.value_of("TVC");
    let tvc = filename.map(|f| std::fs::read(f))
        .transpose()
        .map_err(|e| format!(" failed to read tvc file: {}", e))?
        .unwrap();
    let mut csor = Cursor::new(tvc);
    let mut roots = deserialize_cells_tree(&mut csor).unwrap();
    let code = roots.remove(0).reference(0).unwrap();
    print_tree_of_cells(&code);
    Ok(())
}

fn disasm_solidity_command(m: &ArgMatches) -> Result<(), String> {
    let filename = m.value_of("TVC");
    let tvc = filename.map(|f| std::fs::read(f))
        .transpose()
        .map_err(|e| format!(" failed to read tvc file: {}", e))?
        .unwrap();
    let mut csor = Cursor::new(tvc);
    let roots = deserialize_cells_tree(&mut csor);
    if roots.is_err() {
        println!(";; failed to deserialize the tvc");
        return Ok(())
    }
    let code = roots.unwrap().remove(0).reference(0).unwrap();

    println!(";; selector");
    let mut data = SliceData::from(code);
    let asm = disasm(&mut data, false);
    print!("{}", asm);

    let dict1_cell = data.reference(0).unwrap();
    let dict1 = HashmapE::with_hashmap(32, Some(dict1_cell));
    if dict1.len().is_err() {
        println!(";; failed to recognize methods dictionary");
        return Ok(())
    }

    for entry in dict1.into_iter() {
        let (key, mut method) = entry.unwrap();
        let mut key_slice = SliceData::from(key.into_cell().unwrap());
        let id = key_slice.get_next_u32().unwrap();
        println!();
        println!(";; method {:x}", id);
        print!("{}", disasm(&mut method, true));
    }

    println!();
    println!(";; c3 continuation (current dictionary)");
    let c3 = data.reference(1).unwrap();
    let mut c3_slice = SliceData::from(c3);
    print!("{}", disasm(&mut c3_slice, false));

    let dict2_cell = c3_slice.reference(0).unwrap();
    if dict2_cell == Cell::default() {
        println!(";; current dictionary is empty");
        return Ok(())
    }

    let dict2 = HashmapE::with_hashmap(32, Some(dict2_cell));
    for entry in dict2.into_iter() {
        let (key, mut func) = entry.unwrap();
        let mut key_slice = SliceData::from(key.into_cell().unwrap());
        let id = key_slice.get_next_u32().unwrap();
        println!();
        println!(";; function {}", id);
        print!("{}", disasm(&mut func, true));
    }

    Ok(())
}

fn print_tree_of_cells(toc: &Cell) {
    fn print_tree_of_cells(cell: &Cell, prefix: String, last: bool) {
        let indent = if last { "└ " } else { "├ " };
        let mut hex = cell.to_hex_string(true);
        if hex.len() > 0 {
            let mut first = true;
            let indent_next = if !last { "│ " } else { "  " };
            while hex.len() > 64 {
                let tail = hex.split_off(64);
                println!("{}{}{}…", prefix, if first { indent } else { indent_next }, hex);
                hex = tail;
                first = false;
            }
            println!("{}{}{}", prefix, if first { indent } else { indent_next }, hex);
        } else {
            println!("{}{}{}", prefix, indent, "1_");
        }

        let prefix_child = if last { "  " } else { "│ " };
        let prefix = prefix + prefix_child;
        if cell.references_count() > 0 {
            let last_child = cell.references_count() - 1;
            for i in 0..cell.references_count() {
                let child = cell.reference(i).unwrap();
                print_tree_of_cells(&child, prefix.to_string(), i == last_child);
            }
        }
    }
    print_tree_of_cells(&toc, "".to_string(), true);
}

fn indent(text: String) -> String {
    let mut indented = "".to_string();
    for line in text.split("\n") {
        if line.is_empty() { break; }
        indented += "  ";
        indented += line;
        indented += "\n";
    }
    indented
}

fn disasm(slice: &mut SliceData, cont: bool) -> String {
    let mut disasm = String::new();
    let handlers = Handlers::new_code_page_0();
    let mut stop = false;
    if slice.is_empty() && cont {
        if slice.remaining_references() == 1 {
            *slice = SliceData::from(slice.reference(0).unwrap())
        } else if slice.remaining_references() > 1 {
            panic!();
        }
    }
    while !slice.is_empty() {
        while let Ok(handler) = handlers.get_handler(&mut slice.clone()) {
            if let Some(insn) = handler(slice) {
                disasm += &insn;
                disasm += "\n";
            } else {
                disasm += "> ";
                disasm += &slice.to_hex_string();
                disasm += "\n";
                stop = true;
                break;
            }
        }
        if stop || !cont {
            break;
        }
        assert!(slice.remaining_references() < 2);
        if slice.remaining_references() > 0 {
            *slice = SliceData::from(slice.reference(0).unwrap());
        }
    }
    disasm
}

macro_rules! create_handler_1 {
    ($func_name:ident, $opc:literal, $mnemonic:literal) => {
        fn $func_name(slice: &mut SliceData) -> Option<String> {
            let opc = slice.get_next_int(8).unwrap();
            assert!(opc == $opc);
            Some($mnemonic.to_string())
        }
    };
}

macro_rules! create_handler_1t {
    ($func_name:ident, $opc:literal, $mnemonic:literal) => {
        fn $func_name<T>(slice: &mut SliceData) -> Option<String>
        where T : OperationBehavior {
            let opc = slice.get_next_int(8).unwrap();
            assert!(opc == $opc);
            Some(format!("{}{}", $mnemonic, T::suffix()).to_string())
        }
    };
}

macro_rules! create_handler_2 {
    ($func_name:ident, $opc:literal, $mnemonic:literal) => {
        fn $func_name(slice: &mut SliceData) -> Option<String> {
            let opc = slice.get_next_int(16).unwrap();
            assert!(opc == $opc);
            Some($mnemonic.to_string())
        }
    };
}

macro_rules! create_handler_2t {
    ($func_name:ident, $opc:literal, $mnemonic:literal) => {
        fn $func_name<T>(slice: &mut SliceData) -> Option<String>
        where T : OperationBehavior {
            let opc = slice.get_next_int(16).unwrap();
            assert!(opc == $opc);
            Some(format!("{}{}", $mnemonic, T::suffix()).to_string())
        }
    };
}

// adapted from ton-labs-vm/src/stack/integer/conversion.rs
fn disasm_bigint(slice: &mut SliceData) -> Result_<num::BigInt>
{
    fn twos_complement(digits: &mut Vec<u32>)
    {
        let mut carry = true;
        for d in digits {
            *d = d.not();
            if carry {
                *d = d.wrapping_add(1);
                carry = d.is_zero();
            }
        }
    }
    let first_byte = slice.get_next_byte()?;
    let byte_len = ((first_byte & 0b11111000u8) as usize >> 3) + 3;
    let greatest3bits = (first_byte & 0b111) as u32;
    let digit_count = (byte_len + 3) >> 2;
    let mut digits: Vec<u32> = vec![0; digit_count];
    let (sign, mut value) = if greatest3bits & 0b100 == 0 {
        (num::bigint::Sign::Plus, greatest3bits)
    } else {
        (num::bigint::Sign::Minus, 0xFFFF_FFF8u32 | greatest3bits)
    };

    let mut upper = byte_len & 0b11;
    if upper == 0 {
        upper = 4;
    }
    for _ in 1..upper {
        value <<= 8;
        value |= slice.get_next_byte()? as u32;
    }
    let last_index = digit_count - 1;
    digits[last_index] = value;

    for i in (0..last_index).rev() {
        let mut value = (slice.get_next_byte()? as u32) << 24;
        value |= (slice.get_next_byte()? as u32) << 16;
        value |= (slice.get_next_byte()? as u32) << 8;
        value |= slice.get_next_byte()? as u32;

        digits[i] = value;
    }

    if sign == num::bigint::Sign::Minus {
        twos_complement(&mut digits);
    }
    Ok(num::BigInt::new(sign, digits))
}

trait OperationBehavior {
    fn suffix() -> String;
}
pub struct Signaling {}
pub struct Quiet {}
impl OperationBehavior for Signaling {
    fn suffix() -> String { "".to_string() }
}
impl OperationBehavior for Quiet {
    fn suffix() -> String { "Q".to_string() }
}

type ExecuteHandler = fn(&mut SliceData) -> Option<String>;

#[derive(Clone, Copy)]
enum Handler {
    Direct(ExecuteHandler),
    Subset(usize),
}

pub struct Handlers {
    directs: [Handler; 256],
    subsets: Vec<Handlers>,
}

// adapted from ton-labs-vm/src/executor/engine/handlers.rs
impl Handlers {
    fn new() -> Handlers {
        Handlers {
            directs: [Handler::Direct(disasm_unknown); 256],
            subsets: Vec::new(),
        }
    }

    pub(super) fn new_code_page_0() -> Handlers {
        let mut handlers = Handlers::new();
        handlers
            .add_code_page_0_part_stack()
            .add_code_page_0_tuple()
            .add_code_page_0_part_constant()
            .add_code_page_0_arithmetic()
            .add_code_page_0_comparsion()
            .add_code_page_0_cell()
            .add_code_page_0_control_flow()
            .add_code_page_0_exceptions()
            .add_code_page_0_dictionaries()
            .add_code_page_0_gas_rand_config()
            .add_code_page_0_blockchain()
            .add_code_page_0_crypto()
            .add_code_page_0_debug()
            .add_subset(0xFF, Handlers::new()
                .set_range(0x00..0xF0, disasm_setcp)
                .set(0xF0, disasm_setcpx)
                .set_range(0xF1..0xFF, disasm_setcp)
                .set(0xFF, disasm_setcp)
            );
        handlers
    }

    fn add_code_page_0_part_stack(&mut self) -> &mut Handlers {
        self
            .set(0x00, disasm_nop)
            .set_range(0x01..0x10, disasm_xchg_simple)
            .set(0x10, disasm_xchg_std)
            .set(0x11, disasm_xchg_long)
            .set_range(0x12..0x20, disasm_xchg_simple)
            .set_range(0x20..0x30, disasm_push_simple)
            .set_range(0x30..0x40, disasm_pop_simple)
            .set_range(0x40..0x50, disasm_xchg3)
            .set(0x50, disasm_xchg2)
            .set(0x51, disasm_xcpu)
            .set(0x52, disasm_puxc)
            .set(0x53, disasm_push2)
            .add_subset(0x54, Handlers::new() 
                .set_range(0x00..0x10, disasm_xchg3)
                .set_range(0x10..0x20, disasm_xc2pu)
                .set_range(0x20..0x30, disasm_xcpuxc)
                .set_range(0x30..0x40, disasm_xcpu2)
                .set_range(0x40..0x50, disasm_puxc2)
                .set_range(0x50..0x60, disasm_puxcpu)
                .set_range(0x60..0x70, disasm_pu2xc)
                .set_range(0x70..0x80, disasm_push3)
            )
            .set(0x55, disasm_blkswap)
            .set(0x56, disasm_push)
            .set(0x57, disasm_pop)
            .set(0x58, disasm_rot)
            .set(0x59, disasm_rotrev)
            .set(0x5A, disasm_swap2)
            .set(0x5B, disasm_drop2)
            .set(0x5C, disasm_dup2)
            .set(0x5D, disasm_over2)
            .set(0x5E, disasm_reverse)
            .add_subset(0x5F, Handlers::new()
                .set_range(0x00..0x10, disasm_blkdrop)
                .set_range(0x10..0xFF, disasm_blkpush)
                .set(0xFF, disasm_blkpush)
            )
            .set(0x60, disasm_pick)
            .set(0x61, disasm_roll)
            .set(0x62, disasm_rollrev)
            .set(0x63, disasm_blkswx)
            .set(0x64, disasm_revx)
            .set(0x65, disasm_dropx)
            .set(0x66, disasm_tuck)
            .set(0x67, disasm_xchgx)
            .set(0x68, disasm_depth)
            .set(0x69, disasm_chkdepth)
            .set(0x6A, disasm_onlytopx)
            .set(0x6B, disasm_onlyx)
            .add_subset(0x6C, Handlers::new()
                .set_range(0x10..0xFF, disasm_blkdrop2)
                .set(0xFF, disasm_blkdrop2)
            )
    }

    fn add_code_page_0_tuple(&mut self) -> &mut Handlers {
        self
            .set(0x6D, disasm_null)
            .set(0x6E, disasm_isnull)
            .add_subset(0x6F, Handlers::new()
                .set_range(0x00..0x10, disasm_tuple_create)
                .set_range(0x10..0x20, disasm_tuple_index)
                .set_range(0x20..0x30, disasm_tuple_un)
                .set_range(0x30..0x40, disasm_tuple_unpackfirst)
                .set_range(0x40..0x50, disasm_tuple_explode)
                .set_range(0x50..0x60, disasm_tuple_setindex)
                .set_range(0x60..0x70, disasm_tuple_index_quiet)
                .set_range(0x70..0x80, disasm_tuple_setindex_quiet)
                .set(0x80, disasm_tuple_createvar)
                .set(0x81, disasm_tuple_indexvar)
                .set(0x82, disasm_tuple_untuplevar)
                .set(0x83, disasm_tuple_unpackfirstvar)
                .set(0x84, disasm_tuple_explodevar)
                .set(0x85, disasm_tuple_setindexvar)
                .set(0x86, disasm_tuple_indexvar_quiet)
                .set(0x87, disasm_tuple_setindexvar_quiet)
                .set(0x88, disasm_tuple_len)
                .set(0x89, disasm_tuple_len_quiet)
                .set(0x8A, disasm_istuple)
                .set(0x8B, disasm_tuple_last)
                .set(0x8C, disasm_tuple_push)
                .set(0x8D, disasm_tuple_pop)
                .set(0xA0, disasm_nullswapif)
                .set(0xA1, disasm_nullswapifnot)
                .set(0xA2, disasm_nullrotrif)
                .set(0xA3, disasm_nullrotrifnot)
                .set(0xA4, disasm_nullswapif2)
                .set(0xA5, disasm_nullswapifnot2)
                .set(0xA6, disasm_nullrotrif2)
                .set(0xA7, disasm_nullrotrifnot2)
                .set_range(0xB0..0xC0, disasm_tuple_index2)
                .set_range(0xC0..0xFF, disasm_tuple_index3)
                .set(0xFF, disasm_tuple_index3)
            )
    }

    fn add_code_page_0_part_constant(&mut self) -> &mut Handlers {
        self
            .set_range(0x70..0x82, disasm_pushint)
            .set(0x82, disasm_pushint_big)
            .add_subset(0x83, Handlers::new()
                .set_range(0x00..0xFF, disasm_pushpow2)
                .set(0xFF, disasm_pushnan)
            )
            .set(0x84, disasm_pushpow2dec)
            .set(0x85, disasm_pushnegpow2)
            .set(0x88, disasm_pushref)
            .set(0x89, disasm_pushrefslice)
            .set(0x8A, disasm_pushrefcont)
            .set(0x8B, disasm_pushslice_short)
            .set(0x8C, disasm_pushslice_mid)
            .set(0x8D, disasm_pushslice_long)
            .set_range(0x8E..0x90, disasm_pushcont_long)
            .set_range(0x90..0xA0, disasm_pushcont_short)
    }

    fn add_code_page_0_arithmetic(&mut self) -> &mut Handlers {
        self
            .set(0xA0, disasm_add::<Signaling>)
            .set(0xA1, disasm_sub::<Signaling>)
            .set(0xA2, disasm_subr::<Signaling>)
            .set(0xA3, disasm_negate::<Signaling>)
            .set(0xA4, disasm_inc::<Signaling>)
            .set(0xA5, disasm_dec::<Signaling>)
            .set(0xA6, disasm_addconst::<Signaling>)
            .set(0xA7, disasm_mulconst::<Signaling>)
            .set(0xA8, disasm_mul::<Signaling>)
            .set(0xA9, disasm_divmod::<Signaling>)
            .set(0xAA, disasm_lshift::<Signaling>)
            .set(0xAB, disasm_rshift::<Signaling>)
            .set(0xAC, disasm_lshift::<Signaling>)
            .set(0xAD, disasm_rshift::<Signaling>)
            .set(0xAE, disasm_pow2::<Signaling>)
            .set(0xB0, disasm_and::<Signaling>)
            .set(0xB1, disasm_or::<Signaling>)
            .set(0xB2, disasm_xor::<Signaling>)
            .set(0xB3, disasm_not::<Signaling>)
            .set(0xB4, disasm_fits::<Signaling>)
            .set(0xB5, disasm_ufits::<Signaling>)
            .add_subset(0xB6, Handlers::new()
                .set(0x00, disasm_fitsx::<Signaling>)
                .set(0x01, disasm_ufitsx::<Signaling>)
                .set(0x02, disasm_bitsize::<Signaling>)
                .set(0x03, disasm_ubitsize::<Signaling>)
                .set(0x08, disasm_min::<Signaling>)
                .set(0x09, disasm_max::<Signaling>)
                .set(0x0A, disasm_minmax::<Signaling>)
                .set(0x0B, disasm_abs::<Signaling>)
            )
            .add_subset(0xB7, Handlers::new()
                .set(0xA0, disasm_add::<Quiet>)
                .set(0xA1, disasm_sub::<Quiet>)
                .set(0xA2, disasm_subr::<Quiet>)
                .set(0xA3, disasm_negate::<Quiet>)
                .set(0xA4, disasm_inc::<Quiet>)
                .set(0xA5, disasm_dec::<Quiet>)
                .set(0xA6, disasm_addconst::<Quiet>)
                .set(0xA7, disasm_mulconst::<Quiet>)
                .set(0xA8, disasm_mul::<Quiet>)
                .set(0xA9, disasm_divmod::<Quiet>)
                .set(0xAA, disasm_lshift::<Quiet>)
                .set(0xAB, disasm_rshift::<Quiet>)
                .set(0xAC, disasm_lshift::<Quiet>)
                .set(0xAD, disasm_rshift::<Quiet>)
                .set(0xAE, disasm_pow2::<Quiet>)
                .set(0xB0, disasm_and::<Quiet>)
                .set(0xB1, disasm_or::<Quiet>)
                .set(0xB2, disasm_xor::<Quiet>)
                .set(0xB3, disasm_not::<Quiet>)
                .set(0xB4, disasm_fits::<Quiet>)
                .set(0xB5, disasm_ufits::<Quiet>)
                .add_subset(0xB6, Handlers::new()
                    .set(0x00, disasm_fitsx::<Quiet>)
                    .set(0x01, disasm_ufitsx::<Quiet>)
                    .set(0x02, disasm_bitsize::<Quiet>)
                    .set(0x03, disasm_ubitsize::<Quiet>)
                    .set(0x08, disasm_min::<Quiet>)
                    .set(0x09, disasm_max::<Quiet>)
                    .set(0x0A, disasm_minmax::<Quiet>)
                    .set(0x0B, disasm_abs::<Quiet>)
                )
                .set(0xB8, disasm_sgn::<Quiet>)
                .set(0xB9, disasm_less::<Quiet>)
                .set(0xBA, disasm_equal::<Quiet>)
                .set(0xBB, disasm_leq::<Quiet>)
                .set(0xBC, disasm_greater::<Quiet>)
                .set(0xBD, disasm_neq::<Quiet>)
                .set(0xBE, disasm_geq::<Quiet>)
                .set(0xBF, disasm_cmp::<Quiet>)
                .set(0xC0, disasm_eqint::<Quiet>)
                .set(0xC1, disasm_lessint::<Quiet>)
                .set(0xC2, disasm_gtint::<Quiet>)
                .set(0xC3, disasm_neqint::<Quiet>)
            )
    }

    fn add_code_page_0_comparsion(&mut self) -> &mut Handlers {
        self
            .set(0xB8, disasm_sgn::<Signaling>)
            .set(0xB9, disasm_less::<Signaling>)
            .set(0xBA, disasm_equal::<Signaling>)
            .set(0xBB, disasm_leq::<Signaling>)
            .set(0xBC, disasm_greater::<Signaling>)
            .set(0xBD, disasm_neq::<Signaling>)
            .set(0xBE, disasm_geq::<Signaling>)
            .set(0xBF, disasm_cmp::<Signaling>)
            .set(0xC0, disasm_eqint::<Signaling>)
            .set(0xC1, disasm_lessint::<Signaling>)
            .set(0xC2, disasm_gtint::<Signaling>)
            .set(0xC3, disasm_neqint::<Signaling>)
            .set(0xC4, disasm_isnan)
            .set(0xC5, disasm_chknan)
            .add_subset(0xC7, Handlers::new()
                .set(0x00, disasm_sempty)
                .set(0x01, disasm_sdempty)
                .set(0x02, disasm_srempty)
                .set(0x03, disasm_sdfirst)
                .set(0x04, disasm_sdlexcmp)
                .set(0x05, disasm_sdeq)
                .set(0x08, disasm_sdpfx)
                .set(0x09, disasm_sdpfxrev)
                .set(0x0A, disasm_sdppfx)
                .set(0x0B, disasm_sdppfxrev)
                .set(0x0C, disasm_sdsfx)
                .set(0x0D, disasm_sdsfxrev)
                .set(0x0E, disasm_sdpsfx)
                .set(0x0F, disasm_sdpsfxrev)
                .set(0x10, disasm_sdcntlead0)
                .set(0x11, disasm_sdcntlead1)
                .set(0x12, disasm_sdcnttrail0)
                .set(0x13, disasm_sdcnttrail1)
            )
    }

    fn add_code_page_0_cell(&mut self) -> &mut Handlers {
        self
            .set(0xC8, disasm_newc)
            .set(0xC9, disasm_endc)
            .set(0xCA, disasm_sti)
            .set(0xCB, disasm_stu)
            .set(0xCC, disasm_stref)
            .set(0xCD, disasm_endcst)
            .set(0xCE, disasm_stslice)
            .add_subset(0xCF, Handlers::new()
                .set(0x00, disasm_stix)
                .set(0x01, disasm_stux)
                .set(0x02, disasm_stixr)
                .set(0x03, disasm_stuxr)
                .set(0x04, disasm_stixq)
                .set(0x05, disasm_stuxq)
                .set(0x06, disasm_stixrq)
                .set(0x07, disasm_stuxrq)
                .set(0x08, disasm_sti)
                .set(0x09, disasm_stu)
                .set(0x0A, disasm_stir)
                .set(0x0B, disasm_stur)
                .set(0x0C, disasm_stiq)
                .set(0x0D, disasm_stuq)
                .set(0x0E, disasm_stirq)
                .set(0x0F, disasm_sturq)
                .set(0x10, disasm_stref)
                .set(0x11, disasm_stbref)
                .set(0x12, disasm_stslice)
                .set(0x13, disasm_stb)
                .set(0x14, disasm_strefr)
                .set(0x15, disasm_endcst)
                .set(0x16, disasm_stslicer)
                .set(0x17, disasm_stbr)
                .set(0x18, disasm_strefq)
                .set(0x19, disasm_stbrefq)
                .set(0x1A, disasm_stsliceq)
                .set(0x1B, disasm_stbq)
                .set(0x1C, disasm_strefrq)
                .set(0x1D, disasm_stbrefrq)
                .set(0x1E, disasm_stslicerq)
                .set(0x1F, disasm_stbrq)
                .set(0x20, disasm_strefconst)
                .set(0x21, disasm_stref2const)
                .set(0x23, disasm_endxc)
                .set(0x28, disasm_stile4)
                .set(0x29, disasm_stule4)
                .set(0x2A, disasm_stile8)
                .set(0x2B, disasm_stule8)
                .set(0x30, disasm_bdepth)
                .set(0x31, disasm_bbits)
                .set(0x32, disasm_brefs)
                .set(0x33, disasm_bbitrefs)
                .set(0x35, disasm_brembits)
                .set(0x36, disasm_bremrefs)
                .set(0x37, disasm_brembitrefs)
                .set(0x38, disasm_bchkbits_short)
                .set(0x39, disasm_bchkbits_long)
                .set(0x3A, disasm_bchkrefs)
                .set(0x3B, disasm_bchkbitrefs)
                .set(0x3C, disasm_bchkbitsq_short)
                .set(0x3D, disasm_bchkbitsq_long)
                .set(0x3E, disasm_bchkrefsq)
                .set(0x3F, disasm_bchkbitrefsq)
                .set(0x40, disasm_stzeroes)
                .set(0x41, disasm_stones)
                .set(0x42, disasm_stsame)
                .set_range(0x80..0xFF, disasm_stsliceconst)
                .set(0xFF, disasm_stsliceconst)
            )
            .set(0xD0, disasm_ctos)
            .set(0xD1, disasm_ends)
            .set(0xD2, disasm_ldi)
            .set(0xD3, disasm_ldu)
            .set(0xD4, disasm_ldref)
            .set(0xD5, disasm_ldrefrtos)
            .set(0xD6, disasm_ldslice)
            .add_subset(0xD7, Handlers::new()
                .set(0x00, disasm_ldix)
                .set(0x01, disasm_ldux)
                .set(0x02, disasm_pldix)
                .set(0x03, disasm_pldux)
                .set(0x04, disasm_ldixq)
                .set(0x05, disasm_lduxq)
                .set(0x06, disasm_pldixq)
                .set(0x07, disasm_plduxq)
                .set(0x08, disasm_ldi)
                .set(0x09, disasm_ldu)
                .set(0x0A, disasm_pldi)
                .set(0x0B, disasm_pldu)
                .set(0x0C, disasm_ldiq)
                .set(0x0D, disasm_lduq)
                .set(0x0E, disasm_pldiq)
                .set(0x0F, disasm_plduq)
                .set_range(0x10..0x18, disasm_plduz)
                .set(0x18, disasm_ldslicex)
                .set(0x19, disasm_pldslicex)
                .set(0x1A, disasm_ldslicexq)
                .set(0x1B, disasm_pldslicexq)
                .set(0x1C, disasm_ldslice)
                .set(0x1D, disasm_pldslice)
                .set(0x1E, disasm_ldsliceq)
                .set(0x1F, disasm_pldsliceq)
                .set(0x20, disasm_pldslicex)
                .set(0x21, disasm_sdskipfirst)
                .set(0x22, disasm_sdcutlast)
                .set(0x23, disasm_sdskiplast)
                .set(0x24, disasm_sdsubstr)
                .set(0x26, disasm_sdbeginsx)
                .set(0x27, disasm_sdbeginsxq)
                .set_range(0x28..0x2C, disasm_sdbegins)
                .set_range(0x2C..0x30, disasm_sdbeginsq)
                .set(0x30, disasm_scutfirst)
                .set(0x31, disasm_sskipfirst)
                .set(0x32, disasm_scutlast)
                .set(0x33, disasm_sskiplast)
                .set(0x34, disasm_subslice)
                .set(0x36, disasm_split)
                .set(0x37, disasm_splitq)
                .set(0x39, disasm_xctos)
                .set(0x3A, disasm_xload)
                .set(0x3B, disasm_xloadq)
                .set(0x41, disasm_schkbits)
                .set(0x42, disasm_schkrefs)
                .set(0x43, disasm_schkbitrefs)
                .set(0x45, disasm_schkbitsq)
                .set(0x46, disasm_schkrefsq)
                .set(0x47, disasm_schkbitrefsq)
                .set(0x48, disasm_pldrefvar)
                .set(0x49, disasm_sbits)
                .set(0x4A, disasm_srefs)
                .set(0x4B, disasm_sbitrefs)
                .set(0x4C, disasm_pldref)
                .set_range(0x4D..0x50, disasm_pldrefidx)
                .set(0x50, disasm_ldile4) 
                .set(0x51, disasm_ldule4) 
                .set(0x52, disasm_ldile8) 
                .set(0x53, disasm_ldule8) 
                .set(0x54, disasm_pldile4)
                .set(0x55, disasm_pldule4)
                .set(0x56, disasm_pldile8)
                .set(0x57, disasm_pldule8)
                .set(0x58, disasm_ldile4q) 
                .set(0x59, disasm_ldule4q) 
                .set(0x5A, disasm_ldile8q) 
                .set(0x5B, disasm_ldule8q) 
                .set(0x5C, disasm_pldile4q)
                .set(0x5D, disasm_pldule4q)
                .set(0x5E, disasm_pldile8q)
                .set(0x5F, disasm_pldule8q)
                .set(0x60, disasm_ldzeroes)
                .set(0x61, disasm_ldones)
                .set(0x62, disasm_ldsame)
                .set(0x64, disasm_sdepth)
                .set(0x65, disasm_cdepth)
            )
    }

    fn add_code_page_0_control_flow(&mut self) -> &mut Handlers {
        self
            .set(0xD8, disasm_callx)
            .set(0xD9, disasm_jmpx)
            .set(0xDA, disasm_callxargs)
            .add_subset(0xDB, Handlers::new()
                .set_range(0x00..0x10, disasm_callxargs)
                .set_range(0x10..0x20, disasm_jmpxargs)
                .set_range(0x20..0x30, disasm_retargs)
                .set(0x30, disasm_ret)
                .set(0x31, disasm_retalt)
                .set(0x32, disasm_retbool)
                .set(0x34, disasm_callcc)
                .set(0x35, disasm_jmpxdata)
                .set(0x36, disasm_callccargs)
                .set(0x38, disasm_callxva)
                .set(0x39, disasm_retva)
                .set(0x3A, disasm_jmpxva)
                .set(0x3B, disasm_callccva)
                .set(0x3C, disasm_callref)
                .set(0x3D, disasm_jmpref)
                .set(0x3E, disasm_jmprefdata)
                .set(0x3F, disasm_retdata)
            )
            .set(0xDE, disasm_if)
            .set(0xDC, disasm_ifret)
            .set(0xDD, disasm_ifnotret)
            .set(0xDF, disasm_ifnot)
            .set(0xE0, disasm_ifjmp)
            .set(0xE1, disasm_ifnotjmp)
            .set(0xE2, disasm_ifelse)
            .add_subset(0xE3, Handlers::new()
                .set(0x00, disasm_ifref)
                .set(0x01, disasm_ifnotref)
                .set(0x02, disasm_ifjmpref)
                .set(0x03, disasm_ifnotjmpref)
                .set(0x04, disasm_condsel)
                .set(0x05, disasm_condselchk)
                .set(0x08, disasm_ifretalt)
                .set(0x09, disasm_ifnotretalt)
                .set(0x0D, disasm_ifrefelse)
                .set(0x0E, disasm_ifelseref)
                .set(0x0F, disasm_ifrefelseref)
                .set(0x14, disasm_repeat_break)
                .set(0x15, disasm_repeatend_break)
                .set(0x16, disasm_until_break)
                .set(0x17, disasm_untilend_break)
                .set(0x18, disasm_while_break)
                .set(0x19, disasm_whileend_break)
                .set(0x1A, disasm_again_break)
                .set(0x1B, disasm_againend_break)
                .set_range(0x80..0xA0, disasm_ifbitjmp)
                .set_range(0xA0..0xC0, disasm_ifnbitjmp)
                .set_range(0xC0..0xE0, disasm_ifbitjmpref)
                .set_range(0xE0..0xFF, disasm_ifnbitjmpref)
                .set(0xFF, disasm_ifnbitjmpref)
             )
            .set(0xE4, disasm_repeat)
            .set(0xE5, disasm_repeatend)
            .set(0xE6, disasm_until)
            .set(0xE7, disasm_untilend)
            .set(0xE8, disasm_while)
            .set(0xE9, disasm_whileend)
            .set(0xEA, disasm_again)
            .set(0xEB, disasm_againend)
            .set(0xEC, disasm_setcontargs)
            .add_subset(0xED, Handlers::new()
                .set_range(0x00..0x10, disasm_returnargs)
                .set(0x10, disasm_returnva)
                .set(0x11, disasm_setcontva)
                .set(0x12, disasm_setnumva)
                .set(0x1E, disasm_bless)
                .set(0x1F, disasm_blessva)
                .set_range(0x40..0x50, disasm_pushctr)
                .set_range(0x50..0x60, disasm_popctr)
                .set_range(0x60..0x70, disasm_setcontctr)
                .set_range(0x70..0x80, disasm_setretctr)
                .set_range(0x80..0x90, disasm_setaltctr)
                .set_range(0x90..0xA0, disasm_popsave)
                .set_range(0xA0..0xB0, disasm_save)
                .set_range(0xB0..0xC0, disasm_savealt)
                .set_range(0xC0..0xD0, disasm_saveboth)
                .set(0xE0, disasm_pushctrx)
                .set(0xE1, disasm_popctrx)
                .set(0xE2, disasm_setcontctrx)
                .set(0xF0, disasm_compos)
                .set(0xF1, disasm_composalt)
                .set(0xF2, disasm_composboth)
                .set(0xF3, disasm_atexit)
                .set(0xF4, disasm_atexitalt)
                .set(0xF5, disasm_setexitalt)
                .set(0xF6, disasm_thenret)
                .set(0xF7, disasm_thenretalt)
                .set(0xF8, disasm_invert)
                .set(0xF9, disasm_booleval)
                .set(0xFA, disasm_samealt)
                .set(0xFB, disasm_samealt_save)
            )
            .set(0xEE, disasm_blessargs)
            .set(0xF0, disasm_call_short)
            .add_subset(0xF1, Handlers::new()
                .set_range(0x00..0x40, disasm_call_long)
                .set_range(0x40..0x80, disasm_jmp)
                .set_range(0x80..0xC0, disasm_prepare)
            )
    }

    fn add_code_page_0_exceptions(&mut self) -> &mut Handlers {
        self
            .add_subset(0xF2, Handlers::new()
                .set_range(0x00..0x40, disasm_throw_short)
                .set_range(0x40..0x80, disasm_throwif_short)
                .set_range(0x80..0xC0, disasm_throwifnot_short)
                .set_range(0xC0..0xC8, disasm_throw_long)
                .set_range(0xC8..0xD0, disasm_throwarg)
                .set_range(0xD0..0xD8, disasm_throwif_long)
                .set_range(0xD8..0xE0, disasm_throwargif)
                .set_range(0xE0..0xE8, disasm_throwifnot_long)
                .set_range(0xE8..0xF0, disasm_throwargifnot)
                .set(0xF0, disasm_throwany)
                .set(0xF1, disasm_throwargany)
                .set(0xF2, disasm_throwanyif)
                .set(0xF3, disasm_throwarganyif)
                .set(0xF4, disasm_throwanyifnot)
                .set(0xF5, disasm_throwarganyifnot)
                .set(0xFF, disasm_try)
            )
            .set(0xF3, disasm_tryargs)
    }

    fn add_code_page_0_blockchain(&mut self) -> &mut Handlers {
        self
            .add_subset(0xFA, Handlers::new()
                .set(0x00, disasm_ldgrams)
                .set(0x01, disasm_ldvarint16)
                .set(0x02, disasm_stgrams)
                .set(0x03, disasm_stvarint16)
                .set(0x04, disasm_ldvaruint32)
                .set(0x05, disasm_ldvarint32)
                .set(0x06, disasm_stvaruint32)
                .set(0x07, disasm_stvarint32)
                .set(0x40, disasm_ldmsgaddr::<Signaling>)
                .set(0x41, disasm_ldmsgaddr::<Quiet>)
                .set(0x42, disasm_parsemsgaddr::<Signaling>)
                .set(0x43, disasm_parsemsgaddr::<Quiet>)
                .set(0x44, disasm_rewrite_std_addr::<Signaling>)
                .set(0x45, disasm_rewrite_std_addr::<Quiet>)
                .set(0x46, disasm_rewrite_var_addr::<Signaling>)
                .set(0x47, disasm_rewrite_var_addr::<Quiet>)
            )
            .add_subset(0xFB, Handlers::new()
                .set(0x00, disasm_sendrawmsg)
                .set(0x02, disasm_rawreserve)
                .set(0x03, disasm_rawreservex)
                .set(0x04, disasm_setcode)
                .set(0x06, disasm_setlibcode)
                .set(0x07, disasm_changelib)
            )
    }

    fn add_code_page_0_dictionaries(&mut self) -> &mut Handlers {
        self
            .add_subset(0xF4, Handlers::new()
                .set(0x00, disasm_stdict)
                .set(0x01, disasm_skipdict)
                .set(0x02, disasm_lddicts)
                .set(0x03, disasm_plddicts)
                .set(0x04, disasm_lddict)
                .set(0x05, disasm_plddict)
                .set(0x06, disasm_lddictq)
                .set(0x07, disasm_plddictq)
                .set(0x0A, disasm_dictget)
                .set(0x0B, disasm_dictgetref)
                .set(0x0C, disasm_dictiget)
                .set(0x0D, disasm_dictigetref)
                .set(0x0E, disasm_dictuget)
                .set(0x0F, disasm_dictugetref)
                .set(0x12, disasm_dictset)
                .set(0x13, disasm_dictsetref)
                .set(0x14, disasm_dictiset)
                .set(0x15, disasm_dictisetref)
                .set(0x16, disasm_dictuset)
                .set(0x17, disasm_dictusetref)
                .set(0x1A, disasm_dictsetget)
                .set(0x1B, disasm_dictsetgetref)
                .set(0x1C, disasm_dictisetget)
                .set(0x1D, disasm_dictisetgetref)
                .set(0x1E, disasm_dictusetget)
                .set(0x1F, disasm_dictusetgetref)
                .set(0x22, disasm_dictreplace)
                .set(0x23, disasm_dictreplaceref)
                .set(0x24, disasm_dictireplace)
                .set(0x25, disasm_dictireplaceref)
                .set(0x26, disasm_dictureplace)
                .set(0x27, disasm_dictureplaceref)
                .set(0x2A, disasm_dictreplaceget)
                .set(0x2B, disasm_dictreplacegetref)
                .set(0x2C, disasm_dictireplaceget)
                .set(0x2D, disasm_dictireplacegetref)
                .set(0x2E, disasm_dictureplaceget)
                .set(0x2F, disasm_dictureplacegetref)
                .set(0x32, disasm_dictadd)
                .set(0x33, disasm_dictaddref)
                .set(0x34, disasm_dictiadd)
                .set(0x35, disasm_dictiaddref)
                .set(0x36, disasm_dictuadd)
                .set(0x37, disasm_dictuaddref)
                .set(0x3A, disasm_dictaddget)
                .set(0x3B, disasm_dictaddgetref)
                .set(0x3C, disasm_dictiaddget)
                .set(0x3D, disasm_dictiaddgetref)
                .set(0x3E, disasm_dictuaddget)
                .set(0x3F, disasm_dictuaddgetref)
                .set(0x41, disasm_dictsetb)
                .set(0x42, disasm_dictisetb)
                .set(0x43, disasm_dictusetb)
                .set(0x45, disasm_dictsetgetb)
                .set(0x46, disasm_dictisetgetb)
                .set(0x47, disasm_dictusetgetb)
                .set(0x49, disasm_dictreplaceb)
                .set(0x4A, disasm_dictireplaceb)
                .set(0x4B, disasm_dictureplaceb)
                .set(0x4D, disasm_dictreplacegetb)
                .set(0x4E, disasm_dictireplacegetb)
                .set(0x4F, disasm_dictureplacegetb)
                .set(0x51, disasm_dictaddb)
                .set(0x52, disasm_dictiaddb)
                .set(0x53, disasm_dictuaddb)
                .set(0x55, disasm_dictaddgetb)
                .set(0x56, disasm_dictiaddgetb)
                .set(0x57, disasm_dictuaddgetb)
                .set(0x59, disasm_dictdel)
                .set(0x5A, disasm_dictidel)
                .set(0x5B, disasm_dictudel)
                .set(0x62, disasm_dictdelget)
                .set(0x63, disasm_dictdelgetref)
                .set(0x64, disasm_dictidelget)
                .set(0x65, disasm_dictidelgetref)
                .set(0x66, disasm_dictudelget)
                .set(0x67, disasm_dictudelgetref)
                .set(0x69, disasm_dictgetoptref)
                .set(0x6A, disasm_dictigetoptref)
                .set(0x6B, disasm_dictugetoptref)
                .set(0x6D, disasm_dictsetgetoptref)
                .set(0x6E, disasm_dictisetgetoptref)
                .set(0x6F, disasm_dictusetgetoptref)
                .set(0x70, disasm_pfxdictset)
                .set(0x71, disasm_pfxdictreplace)
                .set(0x72, disasm_pfxdictadd)
                .set(0x73, disasm_pfxdictdel)
                .set(0x74, disasm_dictgetnext)
                .set(0x75, disasm_dictgetnexteq)
                .set(0x76, disasm_dictgetprev)
                .set(0x77, disasm_dictgetpreveq)
                .set(0x78, disasm_dictigetnext)
                .set(0x79, disasm_dictigetnexteq)
                .set(0x7A, disasm_dictigetprev)
                .set(0x7B, disasm_dictigetpreveq)
                .set(0x7C, disasm_dictugetnext)
                .set(0x7D, disasm_dictugetnexteq)
                .set(0x7E, disasm_dictugetprev)
                .set(0x7F, disasm_dictugetpreveq)
                .set(0x82, disasm_dictmin)
                .set(0x83, disasm_dictminref)
                .set(0x84, disasm_dictimin)
                .set(0x85, disasm_dictiminref)
                .set(0x86, disasm_dictumin)
                .set(0x87, disasm_dictuminref)
                .set(0x8A, disasm_dictmax)
                .set(0x8B, disasm_dictmaxref)
                .set(0x8C, disasm_dictimax)
                .set(0x8D, disasm_dictimaxref)
                .set(0x8E, disasm_dictumax)
                .set(0x8F, disasm_dictumaxref)
                .set(0x92, disasm_dictremmin)
                .set(0x93, disasm_dictremminref)
                .set(0x94, disasm_dictiremmin)
                .set(0x95, disasm_dictiremminref)
                .set(0x96, disasm_dicturemmin)
                .set(0x97, disasm_dicturemminref)
                .set(0x9A, disasm_dictremmax)
                .set(0x9B, disasm_dictremmaxref)
                .set(0x9C, disasm_dictiremmax)
                .set(0x9D, disasm_dictiremmaxref)
                .set(0x9E, disasm_dicturemmax)
                .set(0x9F, disasm_dicturemmaxref)
                .set(0xA0, disasm_dictigetjmp)
                .set(0xA1, disasm_dictugetjmp)
                .set(0xA2, disasm_dictigetexec)
                .set(0xA3, disasm_dictugetexec)
                .set_range(0xA4..0xA8, disasm_dictpushconst)
                .set(0xA8, disasm_pfxdictgetq)
                .set(0xA9, disasm_pfxdictget)
                .set(0xAA, disasm_pfxdictgetjmp)
                .set(0xAB, disasm_pfxdictgetexec)
                .set_range(0xAC..0xAF, disasm_pfxdictswitch)
                .set(0xAF, disasm_pfxdictswitch)
                .set(0xB1, disasm_subdictget)
                .set(0xB2, disasm_subdictiget)
                .set(0xB3, disasm_subdictuget)
                .set(0xB5, disasm_subdictrpget)
                .set(0xB6, disasm_subdictirpget)
                .set(0xB7, disasm_subdicturpget)
                .set(0xBC, disasm_dictigetjmpz)
                .set(0xBD, disasm_dictugetjmpz)
                .set(0xBE, disasm_dictigetexecz)
                .set(0xBF, disasm_dictugetexecz)
            )
    }

    fn add_code_page_0_gas_rand_config(&mut self) -> &mut Handlers {
        self
            .add_subset(0xF8, Handlers::new()
                .set(0x00, disasm_accept)
                .set(0x01, disasm_setgaslimit)
                .set(0x02, disasm_buygas)
                .set(0x04, disasm_gramtogas)
                .set(0x05, disasm_gastogram)
                .set(0x0F, disasm_commit)
                .set(0x10, disasm_randu256)
                .set(0x11, disasm_rand)
                .set(0x14, disasm_setrand)
                .set(0x15, disasm_addrand)
                .set(0x20, disasm_getparam)
                .set(0x21, disasm_getparam)
                .set(0x22, disasm_getparam)
                .set(0x23, disasm_now)
                .set(0x24, disasm_blocklt)
                .set(0x25, disasm_ltime)
                .set(0x26, disasm_randseed)
                .set(0x27, disasm_balance)
                .set(0x28, disasm_my_addr)
                .set(0x29, disasm_config_root)
                .set(0x30, disasm_config_dict)
                .set(0x32, disasm_config_ref_param)
                .set(0x33, disasm_config_opt_param)
                .set(0x40, disasm_getglobvar)
                .set_range(0x41..0x5F, disasm_getglob)
                .set(0x5F, disasm_getglob)
                .set(0x60, disasm_setglobvar)
                .set_range(0x61..0x7F, disasm_setglob)
                .set(0x7F, disasm_setglob)
            )
    }

    fn add_code_page_0_crypto(&mut self) -> &mut Handlers {
        self
        .add_subset(0xF9, Handlers::new()
            .set(0x00, disasm_hashcu)
            .set(0x01, disasm_hashsu)
            .set(0x02, disasm_sha256u)
            .set(0x10, disasm_chksignu)
            .set(0x11, disasm_chksigns)
            .set(0x40, disasm_cdatasizeq)
            .set(0x41, disasm_cdatasize)
            .set(0x42, disasm_sdatasizeq)
            .set(0x43, disasm_sdatasize)
        )
    }

    fn add_code_page_0_debug(&mut self) -> &mut Handlers {
        self.add_subset(0xFE, Handlers::new()
            .set(0x00, disasm_dump_stack)
            .set_range(0x01..0x0F, disasm_dump_stack_top)
            .set(0x10, disasm_dump_hex)
            .set(0x11, disasm_print_hex)
            .set(0x12, disasm_dump_bin)
            .set(0x13, disasm_print_bin)
            .set(0x14, disasm_dump_str)
            .set(0x15, disasm_print_str)
            .set(0x1E, disasm_debug_off)
            .set(0x1F, disasm_debug_on)
            .set_range(0x20..0x2F, disasm_dump_var)
            .set_range(0x30..0x3F, disasm_print_var)
            .set_range(0xF0..0xFF, disasm_dump_string)
            .set(0xFF, disasm_dump_string)
        )
    }

    fn get_handler(&self, slice: &mut SliceData) -> ton_types::Result<ExecuteHandler> {
        let cmd = slice.get_next_byte()?;
        match self.directs[cmd as usize] {
            Handler::Direct(handler) => Ok(handler),
            Handler::Subset(i) => self.subsets[i].get_handler(slice),
        }
    }

    fn add_subset(&mut self, code: u8, subset: &mut Handlers) -> &mut Handlers {
        match self.directs[code as usize] {
            Handler::Direct(x) => if x as usize == disasm_unknown as usize {
                self.directs[code as usize] = Handler::Subset(self.subsets.len());
                self.subsets.push(std::mem::replace(subset, Handlers::new()))
            } else {
                panic!("Slot for subset {:02x} is already occupied", code)
            },
            _ => panic!("Subset {:02x} is already registered", code),
        }
        self
    }

    fn register_handler(&mut self, code: u8, handler: ExecuteHandler) {
        match self.directs[code as usize] {
            Handler::Direct(x) => if x as usize == disasm_unknown as usize {
                self.directs[code as usize] = Handler::Direct(handler)
            } else {
                panic!("Code {:02x} is already registered", code)
            },
            _ => panic!("Slot for code {:02x} is already occupied", code),
        }
    }

    fn set(&mut self, code: u8, handler: ExecuteHandler) -> &mut Handlers {
        self.register_handler(code, handler);
        self
    }

    fn set_range(&mut self, codes: Range<u8>, handler: ExecuteHandler) -> &mut Handlers {
        for code in codes {
            self.register_handler(code, handler);
        }
        self
    }
}

fn disasm_unknown(slice: &mut SliceData) -> Option<String> {
    println!("XXX: {}", slice.to_hex_string());
    None
}
fn disasm_setcp(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0xff);
    match slice.get_next_byte() {
        Ok(0) => Some("SETCP0".to_string()),
        _ => None
    }
}
create_handler_2!(disasm_setcpx, 0xfff0, "SETCPX");
create_handler_1!(disasm_nop, 0x00, "NOP");
fn disasm_xchg_simple(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(4).unwrap();
    assert!(opc == 0 || opc == 1);
    let i = slice.get_next_int(4).unwrap();
    match opc {
        0 => Some(format!("XCHG s{}", i).to_string()),
        1 => Some(format!("XCHG s1, s{}", i).to_string()),
        _ => None
    }
}
fn disasm_xchg_std(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0x10);
    let i = slice.get_next_int(4).unwrap();
    let j = slice.get_next_int(4).unwrap();
    Some(format!("XCHG s{}, s{}", i, j).to_string())
}
fn disasm_xchg_long(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0x11);
    let ii = slice.get_next_int(8).unwrap();
    Some(format!("XCHG s0, s{}", ii).to_string())
}
fn disasm_push_simple(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(4).unwrap();
    assert!(opc == 0x2);
    let i = slice.get_next_int(4).unwrap();
    if i == 0 {
        Some("DUP".to_string())
    } else {
        Some(format!("PUSH s{}", i).to_string())
    }
}
fn disasm_pop_simple(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(4).unwrap();
    assert!(opc == 0x3);
    let i = slice.get_next_int(4).unwrap();
    if i == 0 {
        Some("DROP".to_string())
    } else {
        Some(format!("POP s{}", i).to_string())
    }
}
fn disasm_xchg3(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(4).unwrap();
    assert!(opc == 0x4);
    let i = slice.get_next_int(4).unwrap();
    let j = slice.get_next_int(4).unwrap();
    let k = slice.get_next_int(4).unwrap();
    Some(format!("XCHG3 s{}, s{}, s{}", i, j, k).to_string())
}
fn disasm_xchg2(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0x50);
    let i = slice.get_next_int(4).unwrap();
    let j = slice.get_next_int(4).unwrap();
    Some(format!("XCHG2 s{}, s{}", i, j).to_string())
}
fn disasm_xcpu(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0x51);
    let i = slice.get_next_int(4).unwrap();
    let j = slice.get_next_int(4).unwrap();
    Some(format!("XCPU s{}, s{}", i, j).to_string())
}
fn disasm_puxc(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0x52);
    let i = slice.get_next_int(4).unwrap();
    let j = slice.get_next_int(4).unwrap();
    Some(format!("PUXC s{}, s{}", i, j - 1).to_string())
}
fn disasm_push2(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0x53);
    let i = slice.get_next_int(4).unwrap();
    let j = slice.get_next_int(4).unwrap();
    Some(format!("PUSH2 s{}, s{}", i, j).to_string())
}
fn disasm_xc2pu(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0x541);
    let i = slice.get_next_int(4).unwrap();
    let j = slice.get_next_int(4).unwrap();
    let k = slice.get_next_int(4).unwrap();
    Some(format!("XC2PU s{}, s{}, s{}", i, j, k).to_string())
}
fn disasm_xcpuxc(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0x542);
    let i = slice.get_next_int(4).unwrap();
    let j = slice.get_next_int(4).unwrap();
    let k = slice.get_next_int(4).unwrap();
    Some(format!("XCPUXC s{}, s{}, s{}", i, j, k - 1).to_string())
}
fn disasm_xcpu2(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0x543);
    let i = slice.get_next_int(4).unwrap();
    let j = slice.get_next_int(4).unwrap();
    let k = slice.get_next_int(4).unwrap();
    Some(format!("XCPU2 s{}, s{}, s{}", i, j, k).to_string())
}
fn disasm_puxc2(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0x544);
    let i = slice.get_next_int(4).unwrap();
    let j = slice.get_next_int(4).unwrap();
    let k = slice.get_next_int(4).unwrap();
    Some(format!("PUXC2 s{}, s{}, s{}", i, j - 1, k - 1).to_string())
}
fn disasm_puxcpu(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0x545);
    let i = slice.get_next_int(4).unwrap();
    let j = slice.get_next_int(4).unwrap();
    let k = slice.get_next_int(4).unwrap();
    Some(format!("PUXCPU s{}, s{}, s{}", i, j - 1, k - 1).to_string())
}
fn disasm_pu2xc(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0x546);
    let i = slice.get_next_int(4).unwrap();
    let j = slice.get_next_int(4).unwrap();
    let k = slice.get_next_int(4).unwrap();
    Some(format!("PU2XC s{}, s{}, s{}", i, j - 1, k - 2).to_string())
}
fn disasm_push3(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0x547);
    let i = slice.get_next_int(4).unwrap();
    let j = slice.get_next_int(4).unwrap();
    let k = slice.get_next_int(4).unwrap();
    Some(format!("PUSH3 s{}, s{}, s{}", i, j, k).to_string())
}
fn disasm_blkswap(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0x55);
    let i = slice.get_next_int(4).unwrap();
    let j = slice.get_next_int(4).unwrap();
    Some(format!("BLKSWAP {}, {}", i + 1, j + 1).to_string())
}
fn disasm_push(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0x56);
    let ii = slice.get_next_int(8).unwrap();
    Some(format!("PUSH s{}", ii).to_string())
}
fn disasm_pop(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0x57);
    let ii = slice.get_next_int(8).unwrap();
    Some(format!("POP s{}", ii).to_string())
}
create_handler_1!(disasm_rot,    0x58, "ROT");
create_handler_1!(disasm_rotrev, 0x59, "ROTREV");
create_handler_1!(disasm_swap2,  0x5a, "SWAP2");
create_handler_1!(disasm_drop2,  0x5b, "DROP2");
create_handler_1!(disasm_dup2,   0x5c, "DUP2");
create_handler_1!(disasm_over2,  0x5d, "OVER2");
fn disasm_reverse(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0x5e);
    let i = slice.get_next_int(4).unwrap();
    let j = slice.get_next_int(4).unwrap();
    Some(format!("REVERSE {}, {}", i + 2, j).to_string())
}
fn disasm_blkdrop(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0x5f0);
    let i = slice.get_next_int(4).unwrap();
    Some(format!("BLKDROP {}", i).to_string())
}
fn disasm_blkpush(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0x5f);
    let i = slice.get_next_int(4).unwrap();
    let j = slice.get_next_int(4).unwrap();
    Some(format!("BLKPUSH {}, {}", i, j).to_string())
}
create_handler_1!(disasm_pick,     0x60, "PICK");
create_handler_1!(disasm_roll,     0x61, "ROLL");
create_handler_1!(disasm_rollrev,  0x62, "ROLLREV");
create_handler_1!(disasm_blkswx,   0x63, "BLKSWX");
create_handler_1!(disasm_revx,     0x64, "REVX");
create_handler_1!(disasm_dropx,    0x65, "DROPX");
create_handler_1!(disasm_tuck,     0x66, "TUCK");
create_handler_1!(disasm_xchgx,    0x67, "XCHGX");
create_handler_1!(disasm_depth,    0x68, "DEPTH");
create_handler_1!(disasm_chkdepth, 0x69, "CHKDEPTH");
create_handler_1!(disasm_onlytopx, 0x6a, "ONLYTOPX");
create_handler_1!(disasm_onlyx,    0x6b, "ONLYX");
fn disasm_blkdrop2(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0x6c);
    let i = slice.get_next_int(4).unwrap();
    assert!(i > 0);
    let j = slice.get_next_int(4).unwrap();
    Some(format!("BLKDROP2 {}, {}", i, j).to_string())
}
create_handler_1!(disasm_null,   0x6d, "NULL");
create_handler_1!(disasm_isnull, 0x6e, "ISNULL");
fn disasm_tuple_create(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0x6f0);
    let k = slice.get_next_int(4).unwrap();
    Some(format!("TUPLE {}", k).to_string())
}
fn disasm_tuple_index(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0x6f1);
    let k = slice.get_next_int(4).unwrap();
    Some(format!("INDEX {}", k).to_string())
}
fn disasm_tuple_un(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0x6f2);
    let k = slice.get_next_int(4).unwrap();
    Some(format!("UNTUPLE {}", k).to_string())
}
fn disasm_tuple_unpackfirst(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0x6f3);
    let k = slice.get_next_int(4).unwrap();
    Some(format!("UNPACKFIRST {}", k).to_string())
}
fn disasm_tuple_explode(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0x6f4);
    let n = slice.get_next_int(4).unwrap();
    Some(format!("EXPLODE {}", n).to_string())
}
fn disasm_tuple_setindex(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0x6f5);
    let k = slice.get_next_int(4).unwrap();
    Some(format!("SETINDEX {}", k).to_string())
}
fn disasm_tuple_index_quiet(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0x6f6);
    let k = slice.get_next_int(4).unwrap();
    Some(format!("INDEXQ {}", k).to_string())
}
fn disasm_tuple_setindex_quiet(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0x6f7);
    let k = slice.get_next_int(4).unwrap();
    Some(format!("SETINDEXQ {}", k).to_string())
}
create_handler_2!(disasm_tuple_createvar,         0x6f80, "TUPLEVAR");
create_handler_2!(disasm_tuple_indexvar,          0x6f81, "INDEXVAR");
create_handler_2!(disasm_tuple_untuplevar,        0x6f82, "UNTUPLEVAR");
create_handler_2!(disasm_tuple_unpackfirstvar,    0x6f83, "UNPACKFIRSTVAR");
create_handler_2!(disasm_tuple_explodevar,        0x6f84, "EXPLODEVAR");
create_handler_2!(disasm_tuple_setindexvar,       0x6f85, "SETINDEXVAR");
create_handler_2!(disasm_tuple_indexvar_quiet,    0x6f86, "INDEXVARQ");
create_handler_2!(disasm_tuple_setindexvar_quiet, 0x6f87, "SETINDEXVARQ");
create_handler_2!(disasm_tuple_len,               0x6f88, "TLEN");
create_handler_2!(disasm_tuple_len_quiet,         0x6f89, "QTLEN");
create_handler_2!(disasm_istuple,                 0x6f8a, "ISTUPLE");
create_handler_2!(disasm_tuple_last,              0x6f8b, "LAST");
create_handler_2!(disasm_tuple_push,              0x6f8c, "TPUSH");
create_handler_2!(disasm_tuple_pop,               0x6f8d, "TPOP");
create_handler_2!(disasm_nullswapif,              0x6fa0, "NULLSWAPIF");
create_handler_2!(disasm_nullswapifnot,           0x6fa1, "NULLSWAPIFNOT");
create_handler_2!(disasm_nullrotrif,              0x6fa2, "NULLROTRIF");
create_handler_2!(disasm_nullrotrifnot,           0x6fa3, "NULLROTRIFNOT");
create_handler_2!(disasm_nullswapif2,             0x6fa4, "NULLSWAPIF2");
create_handler_2!(disasm_nullswapifnot2,          0x6fa5, "NULLSWAPIFNOT2");
create_handler_2!(disasm_nullrotrif2,             0x6fa6, "NULLROTRIF2");
create_handler_2!(disasm_nullrotrifnot2,          0x6fa7, "NULLROTRIFNOT2");
fn disasm_tuple_index2(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0x6fb);
    let i = slice.get_next_int(2).unwrap();
    let j = slice.get_next_int(2).unwrap();
    Some(format!("INDEX2 {}, {}", i, j).to_string())
}
fn disasm_tuple_index3(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(10).unwrap();
    assert!(opc << 2 == 0x6fe);
    let i = slice.get_next_int(2).unwrap();
    let j = slice.get_next_int(2).unwrap();
    let k = slice.get_next_int(2).unwrap();
    Some(format!("INDEX3 {}, {}, {}", i, j, k).to_string())
}
fn disasm_pushint(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(0x70 <= opc && opc < 0x82);
    let mut x: i16 = 0;
    if opc <= 0x7a {
        x = opc as i16 - 0x70;
    } else if opc < 0x80 {
        x = -(opc as i16 - 0x7f + 1);
    } else if opc == 0x80 {
        x = slice.get_next_int(8).unwrap() as i16;
    } else if opc == 0x81 {
        x = slice.get_next_int(16).unwrap() as i16;
    }
    Some(format!("PUSHINT {}", x).to_string())
}
fn disasm_pushint_big(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0x82);
    let int = disasm_bigint(slice).unwrap();
    Some(format!("PUSHINT {}", int).to_string())
}
fn disasm_pushpow2(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0x83);
    let xx = slice.get_next_int(8).unwrap();
    Some(format!("PUSHPOW2 {}", xx + 1).to_string())
}
create_handler_2!(disasm_pushnan, 0x83ff, "PUSHNAN");
fn disasm_pushpow2dec(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0x84);
    let xx = slice.get_next_int(8).unwrap();
    Some(format!("PUSHPOW2DEC {}", xx + 1).to_string())
}
fn disasm_pushnegpow2(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0x85);
    let xx = slice.get_next_int(8).unwrap();
    Some(format!("PUSHNEGPOW2 {}", xx + 1).to_string())
}
fn disasm_pushref(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0x88);
    // TODO shrink?
    Some("PUSHREF".to_string())
}
fn disasm_pushrefslice(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0x89);
    // TODO shrink?
    Some("PUSHREFSLICE".to_string())
}
fn disasm_pushrefcont(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0x8a);
    // TODO shrink?
    Some("PUSHREFCONT".to_string())
}
fn disasm_pushslice_short(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0x8b);
    let x = slice.get_next_int(4).unwrap() as usize;
    let mut bitstring = slice.get_next_slice(x * 8 + 4).unwrap();
    bitstring.trim_right();
    Some(format!("PUSHSLICE x{}", bitstring.to_hex_string()).to_string())
}
fn disasm_pushslice_mid(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0x8c);
    let r = slice.get_next_int(2).unwrap();
    assert!(r == 0); // TODO
    let xx = slice.get_next_int(5).unwrap() as usize;
    let mut bitstring = slice.get_next_slice(xx * 8 + 1).unwrap();
    bitstring.trim_right();
    Some(format!("PUSHSLICE x{}", bitstring.to_hex_string()).to_string())
}
fn disasm_pushslice_long(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0x8d);
    let r = slice.get_next_int(3).unwrap();
    assert!(r == 0); // TODO
    let xx = slice.get_next_int(7).unwrap() as usize;
    let mut bitstring = slice.get_next_slice(xx * 8 + 6).unwrap();
    bitstring.trim_right();
    Some(format!("PUSHSLICE x{}", bitstring.to_hex_string()).to_string())
}
fn disasm_pushcont_long(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(7).unwrap();
    assert!(opc << 1 == 0x8e);
    let r = slice.get_next_int(2).unwrap() as usize;
    let xx = slice.get_next_int(7).unwrap();
    let mut d = "".to_string();
    for i in 0..r {
        let c = slice.reference(i as usize).unwrap();
        let mut s = SliceData::from(c);
        d += &indent(disasm(&mut s, true));
    }
    if r > 0 {
        slice.shrink_references(r..);
    }
    let mut body = slice.get_next_slice(xx as usize * 8).unwrap();
    d += &indent(disasm(&mut body, false));
    Some(format!("PUSHCONT {{\n{}}}", d).to_string())
}
fn disasm_pushcont_short(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(4).unwrap();
    assert!(opc == 0x9);
    let x = slice.get_next_int(4).unwrap();
    let mut body = slice.get_next_slice(x as usize * 8).unwrap();
    let d = indent(disasm(&mut body, false));
    Some(format!("PUSHCONT {{\n{}}}", d).to_string())
}
create_handler_1t!(disasm_add,    0xa0, "ADD");
create_handler_1t!(disasm_sub,    0xa1, "SUB");
create_handler_1t!(disasm_subr,   0xa2, "SUBR");
create_handler_1t!(disasm_negate, 0xa3, "NEGATE");
create_handler_1t!(disasm_inc,    0xa4, "INC");
create_handler_1t!(disasm_dec,    0xa5, "DEC");
fn disasm_addconst<T>(slice: &mut SliceData) -> Option<String>
where T : OperationBehavior {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0xa6);
    let cc = slice.get_next_int(8).unwrap() as i8;
    Some(format!("ADDCONST{} {}", T::suffix(), cc).to_string())
}
fn disasm_mulconst<T>(slice: &mut SliceData) -> Option<String>
where T : OperationBehavior {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0xa7);
    let cc = slice.get_next_int(8).unwrap() as i8;
    Some(format!("MULCONST{} {}", T::suffix(), cc).to_string())
}
create_handler_1t!(disasm_mul, 0xa8, "MUL");
fn disasm_divmod<T>(slice: &mut SliceData) -> Option<String>
where T : OperationBehavior {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0xa9);
    let opc2 = slice.get_next_int(8).unwrap();
    match opc2 {
        0x04 => Some(format!("DIV{}", T::suffix()).to_string()),
        0x05 => Some(format!("DIVR{}", T::suffix()).to_string()),
        0x06 => Some(format!("DIVC{}", T::suffix()).to_string()),
        0x08 => Some(format!("MOD{}", T::suffix()).to_string()),
        0x0c => Some(format!("DIVMOD{}", T::suffix()).to_string()),
        0x0d => Some(format!("DIVMODR{}", T::suffix()).to_string()),
        0x0e => Some(format!("DIVMODC{}", T::suffix()).to_string()),
        0x24 => Some(format!("RSHIFT{}", T::suffix()).to_string()),
        0x34 => {
            let tt = slice.get_next_int(8).unwrap();
            Some(format!("RSHIFT{} {}", T::suffix(), tt + 1).to_string())
        },
        0x38 => {
            let tt = slice.get_next_int(8).unwrap();
            Some(format!("MODPOW2{} {}", T::suffix(), tt + 1).to_string())
        },
        0x84 => Some(format!("MULDIV{}", T::suffix()).to_string()),
        0x85 => Some(format!("MULDIVR{}", T::suffix()).to_string()),
        0x8c => Some(format!("MULDIVMOD{}", T::suffix()).to_string()),
        0xa4 => Some(format!("MULRSHIFT{}", T::suffix()).to_string()),
        0xa5 => Some(format!("MULRSHIFTR{}", T::suffix()).to_string()),
        0xb4 => {
            let tt = slice.get_next_int(8).unwrap();
            Some(format!("MULRSHIFT{} {}", T::suffix(), tt + 1).to_string())
        },
        0xb5 => {
            let tt = slice.get_next_int(8).unwrap();
            Some(format!("MULRSHIFTR{} {}", T::suffix(), tt + 1).to_string())
        },
        0xc4 => Some(format!("LSHIFTDIV{}", T::suffix()).to_string()),
        0xc5 => Some(format!("LSHIFTDIVR{}", T::suffix()).to_string()),
        0xd4 => {
            let tt = slice.get_next_int(8).unwrap();
            Some(format!("LSHIFTDIV{} {}", T::suffix(), tt + 1).to_string())
        },
        0xd5 => {
            let tt = slice.get_next_int(8).unwrap();
            Some(format!("LSHIFTDIVR{} {}", T::suffix(), tt + 1).to_string())
        },
        _ => {
            println!("divmod? {:x}", opc2);
            None
        }
    }
}
fn disasm_lshift<T>(slice: &mut SliceData) -> Option<String>
where T : OperationBehavior {
    let opc = slice.get_next_int(8).unwrap();
    match opc {
        0xaa => {
            let cc = slice.get_next_int(8).unwrap();
            Some(format!("LSHIFT{} {}", T::suffix(), cc + 1).to_string())
        }
        0xac => {
            Some(format!("LSHIFT{}", T::suffix()).to_string())
        }
        _ => None
    }
}
fn disasm_rshift<T>(slice: &mut SliceData) -> Option<String>
where T : OperationBehavior {
    let opc = slice.get_next_int(8).unwrap();
    match opc {
        0xab => {
            let cc = slice.get_next_int(8).unwrap();
            Some(format!("RSHIFT{} {}", T::suffix(), cc + 1).to_string())
        }
        0xad => {
            Some(format!("RSHIFT{}", T::suffix()).to_string())
        }
        _ => None
    }
}
create_handler_1t!(disasm_pow2,   0xae, "POW2");
create_handler_1t!(disasm_and,    0xb0, "AND");
create_handler_1t!(disasm_or,     0xb1, "OR");
create_handler_1t!(disasm_xor,    0xb2, "XOR");
create_handler_1t!(disasm_not,    0xb3, "NOT");
fn disasm_fits<T>(slice: &mut SliceData) -> Option<String>
where T : OperationBehavior {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0xb4);
    let cc = slice.get_next_int(8).unwrap();
    Some(format!("FITS{} {}", T::suffix(), cc + 1).to_string())
}
fn disasm_ufits<T>(slice: &mut SliceData) -> Option<String>
where T : OperationBehavior {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0xb5);
    let cc = slice.get_next_int(8).unwrap();
    Some(format!("UFITS{} {}", T::suffix(), cc + 1).to_string())
}
create_handler_2t!(disasm_fitsx,    0xb600, "FITSX");
create_handler_2t!(disasm_ufitsx,   0xb601, "UFITSX");
create_handler_2t!(disasm_bitsize,  0xb602, "BITSIZE");
create_handler_2t!(disasm_ubitsize, 0xb603, "UBITSIZE");
create_handler_2t!(disasm_min,      0xb608, "MIN");
create_handler_2t!(disasm_max,      0xb609, "MAX");
create_handler_2t!(disasm_minmax,   0xb60a, "MINMAX");
create_handler_2t!(disasm_abs,      0xb60b, "ABS");
create_handler_1t!(disasm_sgn,     0xb8, "SGN");
create_handler_1t!(disasm_less,    0xb9, "LESS");
create_handler_1t!(disasm_equal,   0xba, "EQUAL");
create_handler_1t!(disasm_leq,     0xbb, "LEQ");
create_handler_1t!(disasm_greater, 0xbc, "GREATER");
create_handler_1t!(disasm_neq,     0xbd, "NEQ");
create_handler_1t!(disasm_geq,     0xbe, "GEQ");
create_handler_1t!(disasm_cmp,     0xbf, "CMP");
fn disasm_eqint<T>(slice: &mut SliceData) -> Option<String>
where T : OperationBehavior {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0xc0);
    let yy = slice.get_next_int(8).unwrap();
    Some(format!("EQINT{} {}", T::suffix(), yy).to_string())
}
fn disasm_lessint<T>(slice: &mut SliceData) -> Option<String>
where T : OperationBehavior {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0xc1);
    let yy = slice.get_next_int(8).unwrap();
    Some(format!("LESSINT{} {}", T::suffix(), yy).to_string())
}
fn disasm_gtint<T>(slice: &mut SliceData) -> Option<String>
where T : OperationBehavior {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0xc2);
    let yy = slice.get_next_int(8).unwrap();
    Some(format!("GTINT{} {}", T::suffix(), yy).to_string())
}
fn disasm_neqint<T>(slice: &mut SliceData) -> Option<String>
where T : OperationBehavior {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0xc3);
    let yy = slice.get_next_int(8).unwrap();
    Some(format!("NEQINT{} {}", T::suffix(), yy).to_string())
}
create_handler_1!(disasm_isnan,  0xc4, "ISNAN");
create_handler_1!(disasm_chknan, 0xc5, "CHKNAN");
create_handler_2!(disasm_sempty,      0xc700, "SEMPTY");
create_handler_2!(disasm_sdempty,     0xc701, "SDEMPTY");
create_handler_2!(disasm_srempty,     0xc702, "SREMPTY");
create_handler_2!(disasm_sdfirst,     0xc703, "SDFIRST");
create_handler_2!(disasm_sdlexcmp,    0xc704, "SDLEXCMP");
create_handler_2!(disasm_sdeq,        0xc705, "SDEQ");
create_handler_2!(disasm_sdpfx,       0xc708, "SDPFX");
create_handler_2!(disasm_sdpfxrev,    0xc709, "SDPFXREV");
create_handler_2!(disasm_sdppfx,      0xc70a, "SDPPFX");
create_handler_2!(disasm_sdppfxrev,   0xc70b, "SDPPFXREV");
create_handler_2!(disasm_sdsfx,       0xc70c, "SDSFX");
create_handler_2!(disasm_sdsfxrev,    0xc70d, "SDSFXREV");
create_handler_2!(disasm_sdpsfx,      0xc70e, "SDPSFX");
create_handler_2!(disasm_sdpsfxrev,   0xc70f, "SDPSFXREV");
create_handler_2!(disasm_sdcntlead0,  0xc710, "SDCNTLEAD0");
create_handler_2!(disasm_sdcntlead1,  0xc711, "SDCNTLEAD1");
create_handler_2!(disasm_sdcnttrail0, 0xc712, "SDCNTTRAIL0");
create_handler_2!(disasm_sdcnttrail1, 0xc713, "SDCNTTRAIL1");
create_handler_1!(disasm_newc, 0xc8, "NEWC");
create_handler_1!(disasm_endc, 0xc9, "ENDC");
fn disasm_sti(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0xca);
    let cc = slice.get_next_int(8).unwrap();
    Some(format!("STI {}", cc + 1).to_string())
}
fn disasm_stu(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0xcb);
    let cc = slice.get_next_int(8).unwrap();
    Some(format!("STU {}", cc + 1).to_string())
}
create_handler_1!(disasm_stref,   0xcc, "STREF");
create_handler_1!(disasm_endcst,  0xcd, "STBREFR");
create_handler_1!(disasm_stslice, 0xce, "STSLICE");
create_handler_2!(disasm_stix,   0xcf00, "STIX");
create_handler_2!(disasm_stux,   0xcf01, "STUX");
create_handler_2!(disasm_stixr,  0xcf02, "STIXR");
create_handler_2!(disasm_stuxr,  0xcf03, "STUXR");
create_handler_2!(disasm_stixq,  0xcf04, "STIXQ");
create_handler_2!(disasm_stuxq,  0xcf05, "STUXQ");
create_handler_2!(disasm_stixrq, 0xcf06, "STIXRQ");
create_handler_2!(disasm_stuxrq, 0xcf07, "STUXRQ");
fn disasm_stir(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(16).unwrap();
    assert!(opc == 0xcf0a);
    let cc = slice.get_next_int(8).unwrap();
    Some(format!("STIR {}", cc + 1).to_string())
}
fn disasm_stur(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(16).unwrap();
    assert!(opc == 0xcf0b);
    let cc = slice.get_next_int(8).unwrap();
    Some(format!("STUR {}", cc + 1).to_string())
}

fn disasm_stiq(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(16).unwrap();
    assert!(opc == 0xcf0c);
    let cc = slice.get_next_int(8).unwrap();
    Some(format!("STIQ {}", cc + 1).to_string())
}
fn disasm_stuq(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(16).unwrap();
    assert!(opc == 0xcf0d);
    let cc = slice.get_next_int(8).unwrap();
    Some(format!("STUQ {}", cc + 1).to_string())
}
fn disasm_stirq(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(16).unwrap();
    assert!(opc == 0xcf0e);
    let cc = slice.get_next_int(8).unwrap();
    Some(format!("STIRQ {}", cc + 1).to_string())
}
fn disasm_sturq(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(16).unwrap();
    assert!(opc == 0xcf0f);
    let cc = slice.get_next_int(8).unwrap();
    Some(format!("STURQ {}", cc + 1).to_string())
}
create_handler_2!(disasm_stbref,      0xcf11, "STBREF");
create_handler_2!(disasm_stb,         0xcf13, "STB");
create_handler_2!(disasm_strefr,      0xcf14, "STREFR");
create_handler_2!(disasm_stslicer,    0xcf16, "STSLICER");
create_handler_2!(disasm_stbr,        0xcf17, "STBR");
create_handler_2!(disasm_strefq,      0xcf18, "STREFQ");
create_handler_2!(disasm_stbrefq,     0xcf19, "STBREFQ");
create_handler_2!(disasm_stsliceq,    0xcf1a, "STSLICEQ");
create_handler_2!(disasm_stbq,        0xcf1b, "STBQ");
create_handler_2!(disasm_strefrq,     0xcf1c, "STREFRQ");
create_handler_2!(disasm_stbrefrq,    0xcf1d, "STBREFRQ");
create_handler_2!(disasm_stslicerq,   0xcf1e, "STSLICERQ");
create_handler_2!(disasm_stbrq,       0xcf1f, "STBRQ");
create_handler_2!(disasm_strefconst,  0xcf20, "STREFCONST");
create_handler_2!(disasm_stref2const, 0xcf21, "STREF2CONST");
create_handler_2!(disasm_endxc,       0xcf23, "ENDXC");
create_handler_2!(disasm_stile4,      0xcf28, "STILE4");
create_handler_2!(disasm_stule4,      0xcf29, "STULE4");
create_handler_2!(disasm_stile8,      0xcf2a, "STILE8");
create_handler_2!(disasm_stule8,      0xcf2b, "STULE8");
create_handler_2!(disasm_bdepth,      0xcf30, "BDEPTH");
create_handler_2!(disasm_bbits,       0xcf31, "BBITS");
create_handler_2!(disasm_brefs,       0xcf32, "BREFS");
create_handler_2!(disasm_bbitrefs,    0xcf33, "BBITREFS");
create_handler_2!(disasm_brembits,    0xcf35, "BREMBITS");
create_handler_2!(disasm_bremrefs,    0xcf36, "BREMREFS");
create_handler_2!(disasm_brembitrefs, 0xcf37, "BREMBITREFS");
fn disasm_bchkbits_short(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(16).unwrap();
    assert!(opc == 0xcf38);
    let cc = slice.get_next_int(8).unwrap();
    Some(format!("BCHKBITS {}", cc + 1).to_string())
}
create_handler_2!(disasm_bchkbits_long, 0xcf39, "BCHKBITS");
create_handler_2!(disasm_bchkrefs,      0xcf3a, "BCHKREFS");
create_handler_2!(disasm_bchkbitrefs,   0xcf3b, "BCHKBITREFS");
fn disasm_bchkbitsq_short(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(16).unwrap();
    assert!(opc == 0xcf3c);
    let cc = slice.get_next_int(8).unwrap();
    Some(format!("BCHKBITSQ {}", cc + 1).to_string())
}
create_handler_2!(disasm_bchkbitsq_long, 0xcf3d, "BCHKBITSQ");
create_handler_2!(disasm_bchkrefsq,      0xcf3e, "BCHKREFSQ");
create_handler_2!(disasm_bchkbitrefsq,   0xcf3f, "BCHKBITREFSQ");
create_handler_2!(disasm_stzeroes,       0xcf40, "STZEROES");
create_handler_2!(disasm_stones,         0xcf41, "STONES");
create_handler_2!(disasm_stsame,         0xcf42, "STSAME");
fn disasm_stsliceconst(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(9).unwrap();
    assert!(opc << 3 == 0xcf8);
    let x = slice.get_next_int(2).unwrap();
    assert!(x == 0);
    let y = slice.get_next_int(3).unwrap();
    let sss = slice.get_next_slice(y as usize * 8 + 2).unwrap();
    Some(format!("STSLICECONST x{}", sss.into_cell().to_hex_string(true)).to_string())
}
create_handler_1!(disasm_ctos, 0xd0, "CTOS");
create_handler_1!(disasm_ends, 0xd1, "ENDS");
fn disasm_ldi(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0xd2);
    let cc = slice.get_next_int(8).unwrap();
    Some(format!("LDI {}", cc + 1).to_string())
}
fn disasm_ldu(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0xd3);
    let cc = slice.get_next_int(8).unwrap();
    Some(format!("LDU {}", cc + 1).to_string())
}
create_handler_1!(disasm_ldref,     0xd4, "LDREF");
create_handler_1!(disasm_ldrefrtos, 0xd5, "LDREFRTOS");
fn disasm_ldslice(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0xd6);
    let cc = slice.get_next_int(8).unwrap();
    Some(format!("LDSLICE {}", cc + 1).to_string())
}
create_handler_2!(disasm_ldix,   0xd700, "LDIX");
create_handler_2!(disasm_ldux,   0xd701, "LDUX");
create_handler_2!(disasm_pldix,  0xd702, "PLDIX");
create_handler_2!(disasm_pldux,  0xd703, "PLDUX");
create_handler_2!(disasm_ldixq,  0xd704, "LDIXQ");
create_handler_2!(disasm_lduxq,  0xd705, "LDUXQ");
create_handler_2!(disasm_pldixq, 0xd706, "PLDIXQ");
create_handler_2!(disasm_plduxq, 0xd707, "PLDUXQ");
fn disasm_pldi(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(16).unwrap();
    assert!(opc == 0xd70a);
    let cc = slice.get_next_int(8).unwrap();
    Some(format!("PLDI {}", cc + 1).to_string())
}
fn disasm_pldu(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(16).unwrap();
    assert!(opc == 0xd70b);
    let cc = slice.get_next_int(8).unwrap();
    Some(format!("PLDU {}", cc + 1).to_string())
}
fn disasm_ldiq(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(16).unwrap();
    assert!(opc == 0xd70c);
    let cc = slice.get_next_int(8).unwrap();
    Some(format!("LDIQ {}", cc + 1).to_string())
}
fn disasm_lduq(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(16).unwrap();
    assert!(opc == 0xd70d);
    let cc = slice.get_next_int(8).unwrap();
    Some(format!("LDUQ {}", cc + 1).to_string())
}
fn disasm_pldiq(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(16).unwrap();
    assert!(opc == 0xd70e);
    let cc = slice.get_next_int(8).unwrap();
    Some(format!("PLDIQ {}", cc + 1).to_string())
}
fn disasm_plduq(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(16).unwrap();
    assert!(opc == 0xd70f);
    let cc = slice.get_next_int(8).unwrap();
    Some(format!("PLDUQ {}", cc + 1).to_string())
}
fn disasm_plduz(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(13).unwrap();
    assert!(opc << 3 == 0xd710);
    let c = slice.get_next_int(3).unwrap();
    Some(format!("PLDUZ {}", 32 * (c + 1)).to_string())
}
create_handler_2!(disasm_ldslicex,   0xd718, "LDSLICEX");
create_handler_2!(disasm_pldslicex,  0xd719, "PLDSLICEX");
create_handler_2!(disasm_ldslicexq,  0xd71a, "LDSLICEXQ");
create_handler_2!(disasm_pldslicexq, 0xd71b, "PLDSLICEXQ");
fn disasm_pldslice(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(16).unwrap();
    assert!(opc == 0xd71d);
    let cc = slice.get_next_int(8).unwrap();
    Some(format!("PLDSLICE {}", cc + 1).to_string())
}
fn disasm_ldsliceq(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(16).unwrap();
    assert!(opc == 0xd71e);
    let cc = slice.get_next_int(8).unwrap();
    Some(format!("LDSLICEQ {}", cc + 1).to_string())
}
fn disasm_pldsliceq(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(16).unwrap();
    assert!(opc == 0xd71f);
    let cc = slice.get_next_int(8).unwrap();
    Some(format!("PLDSLICEQ {}", cc + 1).to_string())
}
create_handler_2!(disasm_sdskipfirst,  0xd721, "SDSKIPFIRST");
create_handler_2!(disasm_sdcutlast,    0xd722, "SDCUTLAST");
create_handler_2!(disasm_sdskiplast,   0xd723, "SDSKIPLAST");
create_handler_2!(disasm_sdsubstr,     0xd724, "SDSUBSTR");
create_handler_2!(disasm_sdbeginsx,    0xd726, "SDBEGINSX");
create_handler_2!(disasm_sdbeginsxq,   0xd727, "SDBEGINSXQ");
fn disasm_sdbegins(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(14).unwrap();
    assert!(opc << 2 == 0xd728);
    let x = slice.get_next_int(7).unwrap() as usize;
    let mut bitstring = slice.get_next_slice(8 * x + 3).unwrap();
    bitstring.trim_right();
    Some(format!("SDBEGINS {}", bitstring.to_hex_string()).to_string())
}
fn disasm_sdbeginsq(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(14).unwrap();
    assert!(opc << 2 == 0xd72c);
    let x = slice.get_next_int(7).unwrap() as usize;
    let mut bitstring = slice.get_next_slice(8 * x + 3).unwrap();
    bitstring.trim_right();
    Some(format!("SDBEGINSQ {}", bitstring.to_hex_string()).to_string())
}
create_handler_2!(disasm_scutfirst,    0xd730, "SCUTFIRST");
create_handler_2!(disasm_sskipfirst,   0xd731, "SSKIPFIRST");
create_handler_2!(disasm_scutlast,     0xd732, "SCUTLAST");
create_handler_2!(disasm_sskiplast,    0xd733, "SSKIPLAST");
create_handler_2!(disasm_subslice,     0xd734, "SUBSLICE");
create_handler_2!(disasm_split,        0xd736, "SPLIT");
create_handler_2!(disasm_splitq,       0xd737, "SPLITQ");
create_handler_2!(disasm_xctos,        0xd739, "XCTOS");
create_handler_2!(disasm_xload,        0xd73a, "XLOAD");
create_handler_2!(disasm_xloadq,       0xd73b, "XLOADQ");
create_handler_2!(disasm_schkbits,     0xd741, "SCHKBITS");
create_handler_2!(disasm_schkrefs,     0xd742, "SCHKREFS");
create_handler_2!(disasm_schkbitrefs,  0xd743, "XCHKBITREFS");
create_handler_2!(disasm_schkbitsq,    0xd745, "SCHKBITSQ");
create_handler_2!(disasm_schkrefsq,    0xd746, "SCHKREFSQ");
create_handler_2!(disasm_schkbitrefsq, 0xd747, "SCHKBITREFSQ");
create_handler_2!(disasm_pldrefvar,    0xd748, "PLDREFVAR");
create_handler_2!(disasm_sbits,        0xd749, "SBITS");
create_handler_2!(disasm_srefs,        0xd74a, "SREFS");
create_handler_2!(disasm_sbitrefs,     0xd74b, "SBITREFS");
create_handler_2!(disasm_pldref,       0xd74c, "PLDREF");
fn disasm_pldrefidx(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(14).unwrap();
    assert!(opc << 2 == 0xd74c);
    let n = slice.get_next_int(2).unwrap();
    Some(format!("PLDREFIDX {}", n).to_string())
}
create_handler_2!(disasm_ldile4,       0xd750, "LDILE4"); 
create_handler_2!(disasm_ldule4,       0xd751, "LDULE4"); 
create_handler_2!(disasm_ldile8,       0xd752, "LDILE8"); 
create_handler_2!(disasm_ldule8,       0xd753, "LDULE8"); 
create_handler_2!(disasm_pldile4,      0xd754, "PLDILE4");
create_handler_2!(disasm_pldule4,      0xd755, "PLDULE4");
create_handler_2!(disasm_pldile8,      0xd756, "PLDILE8");
create_handler_2!(disasm_pldule8,      0xd757, "PLDULE8");
create_handler_2!(disasm_ldile4q,      0xd758, "LDILE4Q"); 
create_handler_2!(disasm_ldule4q,      0xd759, "LDULE4Q"); 
create_handler_2!(disasm_ldile8q,      0xd75a, "LDILE8Q"); 
create_handler_2!(disasm_ldule8q,      0xd75b, "LDULE8Q"); 
create_handler_2!(disasm_pldile4q,     0xd75c, "PLDILE4Q");
create_handler_2!(disasm_pldule4q,     0xd75d, "PLDULE4Q");
create_handler_2!(disasm_pldile8q,     0xd75e, "PLDILE8Q");
create_handler_2!(disasm_pldule8q,     0xd75f, "PLDULE8Q");
create_handler_2!(disasm_ldzeroes,     0xd760, "LDZEROES");
create_handler_2!(disasm_ldones,       0xd761, "LDONES");
create_handler_2!(disasm_ldsame,       0xd762, "LDSAME");
create_handler_2!(disasm_sdepth,       0xd764, "SDEPTH");
create_handler_2!(disasm_cdepth,       0xd765, "CDEPTH");
create_handler_1!(disasm_callx, 0xd8, "CALLX");
create_handler_1!(disasm_jmpx,  0xd9, "JMPX");
fn disasm_callxargs(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    match opc {
        0xda => {
            let p = slice.get_next_int(4).unwrap();
            let r = slice.get_next_int(4).unwrap();
            Some(format!("CALLXARGS {}, {}", p, r).to_string())
        }
        0xdb => {
            let z = slice.get_next_int(4).unwrap();
            assert!(z == 0);
            let p = slice.get_next_int(4).unwrap();
            Some(format!("CALLXARGS {}, -1", p).to_string())
        }
        _ => None
    }
}
fn disasm_jmpxargs(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0xdb1);
    let p = slice.get_next_int(4).unwrap();
    Some(format!("JMPXARGS {}", p).to_string())
}
fn disasm_retargs(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0xdb2);
    let r = slice.get_next_int(4).unwrap();
    Some(format!("RETARGS {}", r).to_string())
}
create_handler_2!(disasm_ret,      0xdb30, "RET");
create_handler_2!(disasm_retalt,   0xdb31, "RETALT");
create_handler_2!(disasm_retbool,  0xdb32, "RETBOOL");
create_handler_2!(disasm_callcc,   0xdb34, "CALLCC");
create_handler_2!(disasm_jmpxdata, 0xdb35, "JMPXDATA");
fn disasm_callccargs(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(16).unwrap();
    assert!(opc == 0xdb36);
    let p = slice.get_next_int(4).unwrap();
    let r = slice.get_next_int(4).unwrap();
    Some(format!("CALLCCARGS {}, {}", p, r).to_string())
}
create_handler_2!(disasm_callxva,    0xdb38, "CALLXVARARGS");
create_handler_2!(disasm_retva,      0xdb39, "RETVARARGS");
create_handler_2!(disasm_jmpxva,     0xdb3a, "JMPXVARARGS");
create_handler_2!(disasm_callccva,   0xdb3b, "CALLCCVARARGS");
create_handler_2!(disasm_callref,    0xdb3c, "CALLREF");
create_handler_2!(disasm_jmpref,     0xdb3d, "JMPREF");
create_handler_2!(disasm_jmprefdata, 0xdb3e, "JMPREFDATA");
create_handler_2!(disasm_retdata,    0xdb3f, "RETDATA");
create_handler_1!(disasm_ifret,    0xdc, "IFRET");
create_handler_1!(disasm_ifnotret, 0xdd, "IFNOTRET");
create_handler_1!(disasm_if,       0xde, "IF");
create_handler_1!(disasm_ifnot,    0xdf, "IFNOT");
create_handler_1!(disasm_ifjmp,    0xe0, "IFJMP");
create_handler_1!(disasm_ifnotjmp, 0xe1, "IFNOTJMP");
create_handler_1!(disasm_ifelse,   0xe2, "IFELSE");
create_handler_2!(disasm_ifref,        0xe300, "IFREF");
create_handler_2!(disasm_ifnotref,     0xe301, "IFNOTREF");
create_handler_2!(disasm_ifjmpref,     0xe302, "IFJMPREF");
create_handler_2!(disasm_ifnotjmpref,  0xe303, "IFNOTJMPREF");
create_handler_2!(disasm_condsel,      0xe304, "CONDSEL");
create_handler_2!(disasm_condselchk,   0xe305, "CONDSELCHK");
create_handler_2!(disasm_ifretalt,     0xe308, "IFRETALT");
create_handler_2!(disasm_ifnotretalt,  0xe309, "IFNOTRETALT");
create_handler_2!(disasm_ifrefelse,    0xe30d, "IFREFELSE");
create_handler_2!(disasm_ifelseref,    0xe30e, "IFELSEREF");
create_handler_2!(disasm_ifrefelseref, 0xe30f, "IFREFELSEREF");
create_handler_2!(disasm_repeat_break,    0xe314, "REPEATBRK");
create_handler_2!(disasm_repeatend_break, 0xe315, "REPEATENDBRK");
create_handler_2!(disasm_until_break,     0xe316, "UNTILBRK");
create_handler_2!(disasm_untilend_break,  0xe317, "UNTILENDBRK");
create_handler_2!(disasm_while_break,     0xe318, "WHILEBRK");
create_handler_2!(disasm_whileend_break,  0xe319, "WHILEENDBRK");
create_handler_2!(disasm_again_break,     0xe31a, "AGAINBRK");
create_handler_2!(disasm_againend_break,  0xe31b, "AGAINENDBRK");
fn disasm_ifbitjmp(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(15).unwrap();
    assert!(opc << 1 == 0xe38);
    let n = slice.get_next_int(5).unwrap();
    Some(format!("IFBITJMP {}", n).to_string())
}
fn disasm_ifnbitjmp(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(15).unwrap();
    assert!(opc << 1 == 0xe3a);
    let n = slice.get_next_int(5).unwrap();
    Some(format!("IFNBITJMP {}", n).to_string())
}
fn disasm_ifbitjmpref(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(15).unwrap();
    assert!(opc << 1 == 0xe3c);
    let n = slice.get_next_int(5).unwrap();
    Some(format!("IFBITJMPREF {}", n).to_string())
}
fn disasm_ifnbitjmpref(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(15).unwrap();
    assert!(opc << 1 == 0xe3e);
    let n = slice.get_next_int(5).unwrap();
    Some(format!("IFNBITJMPREF {}", n).to_string())
}
create_handler_1!(disasm_repeat,    0xe4, "REPEAT");
create_handler_1!(disasm_repeatend, 0xe5, "REPEATEND");
create_handler_1!(disasm_until,     0xe6, "UNTIL");
create_handler_1!(disasm_untilend,  0xe7, "UNTILEND");
create_handler_1!(disasm_while,     0xe8, "WHILE");
create_handler_1!(disasm_whileend,  0xe9, "WHILEEND");
create_handler_1!(disasm_again,     0xea, "AGAIN");
create_handler_1!(disasm_againend,  0xeb, "AGAINEND");
fn disasm_setcontargs(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0xec);
    let r = slice.get_next_int(4).unwrap();
    let n = slice.get_next_int(4).unwrap();
    Some(format!("SETCONTARGS {}, {}", r, n).to_string())
}
fn disasm_returnargs(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0xed0);
    let p = slice.get_next_int(4).unwrap();
    Some(format!("RETURNARGS {}", p).to_string())
}
create_handler_2!(disasm_returnva,  0xed10, "RETURNVARARGS");
create_handler_2!(disasm_setcontva, 0xed11, "SETCONTVARARGS");
create_handler_2!(disasm_setnumva,  0xed12, "SETNUMVARARGS");
create_handler_2!(disasm_bless,     0xed1e, "BLESS");
create_handler_2!(disasm_blessva,   0xed1f, "BLESSVARARGS");
fn disasm_pushctr(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0xed4);
    let i = slice.get_next_int(4).unwrap();
    Some(format!("PUSHCTR c{}", i).to_string())
}
fn disasm_popctr(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0xed5);
    let i = slice.get_next_int(4).unwrap();
    Some(format!("POPCTR c{}", i).to_string())
}
fn disasm_setcontctr(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0xed6);
    let i = slice.get_next_int(4).unwrap();
    Some(format!("SETCONTCTR c{}", i).to_string())
}
fn disasm_setretctr(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0xed7);
    let i = slice.get_next_int(4).unwrap();
    Some(format!("SETRETCTR c{}", i).to_string())
}
fn disasm_setaltctr(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0xed8);
    let i = slice.get_next_int(4).unwrap();
    Some(format!("SETALTCTR c{}", i).to_string())
}
fn disasm_popsave(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0xed9);
    let i = slice.get_next_int(4).unwrap();
    Some(format!("POPSAVE c{}", i).to_string())
}
fn disasm_save(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0xeda);
    let i = slice.get_next_int(4).unwrap();
    Some(format!("SAVE c{}", i).to_string())
}
fn disasm_savealt(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0xedb);
    let i = slice.get_next_int(4).unwrap();
    Some(format!("SAVEALT c{}", i).to_string())
}
fn disasm_saveboth(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0xedc);
    let i = slice.get_next_int(4).unwrap();
    Some(format!("SAVEBOTH c{}", i).to_string())
}
create_handler_2!(disasm_pushctrx,     0xede0, "PUSHCTRX");
create_handler_2!(disasm_popctrx,      0xede1, "POPCTRX");
create_handler_2!(disasm_setcontctrx,  0xede2, "SETCONTCTRX");
create_handler_2!(disasm_compos,       0xedf0, "COMPOS");
create_handler_2!(disasm_composalt,    0xedf1, "COMPOSALT");
create_handler_2!(disasm_composboth,   0xedf2, "COMPOSBOTH");
create_handler_2!(disasm_atexit,       0xedf3, "ATEXIT");
create_handler_2!(disasm_atexitalt,    0xedf4, "ATEXITALT");
create_handler_2!(disasm_setexitalt,   0xedf5, "SETEXITALT");
create_handler_2!(disasm_thenret,      0xedf6, "THENRET");
create_handler_2!(disasm_thenretalt,   0xedf7, "THENRETALT");
create_handler_2!(disasm_invert,       0xedf8, "INVERT");
create_handler_2!(disasm_booleval,     0xedf9, "BOOLEVAL");
create_handler_2!(disasm_samealt,      0xedfa, "SAMEALT");
create_handler_2!(disasm_samealt_save, 0xedfb, "SAMEALTSAVE");
fn disasm_blessargs(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0xee);
    let r = slice.get_next_int(4).unwrap();
    let n = slice.get_next_int(4).unwrap();
    Some(format!("BLESSARGS {}, {}", r, n).to_string())
}
fn disasm_call_short(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0xf0);
    let n = slice.get_next_int(8).unwrap();
    Some(format!("CALL {}", n).to_string())
}
fn disasm_call_long(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(10).unwrap();
    assert!(opc << 2 == 0xf10);
    let n = slice.get_next_int(14).unwrap();
    Some(format!("CALL {}", n).to_string())
}
fn disasm_jmp(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(10).unwrap();
    assert!(opc << 2 == 0xf14);
    let n = slice.get_next_int(14).unwrap();
    Some(format!("JMPDICT {}", n).to_string())
}
fn disasm_prepare(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(10).unwrap();
    assert!(opc << 2 == 0xf18);
    let n = slice.get_next_int(14).unwrap();
    Some(format!("PREPARE {}", n).to_string())
}
fn disasm_throw_short(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(10).unwrap();
    assert!(opc << 2 == 0xf20);
    let nn = slice.get_next_int(6).unwrap();
    Some(format!("THROW {}", nn).to_string())
}
fn disasm_throwif_short(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(10).unwrap();
    assert!(opc << 2 == 0xf24);
    let nn = slice.get_next_int(6).unwrap();
    Some(format!("THROWIF {}", nn).to_string())
}
fn disasm_throwifnot_short(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(10).unwrap();
    assert!(opc << 2 == 0xf28);
    let nn = slice.get_next_int(6).unwrap();
    Some(format!("THROWIFNOT {}", nn).to_string())
}
fn disasm_throw_long(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(13).unwrap();
    assert!(opc << 3 == 0xf2c0);
    let nn = slice.get_next_int(11).unwrap();
    Some(format!("THROW {}", nn).to_string())
}
fn disasm_throwarg(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(13).unwrap();
    assert!(opc << 3 == 0xf2c8);
    let nn = slice.get_next_int(11).unwrap();
    Some(format!("THROWARG {}", nn).to_string())
}
fn disasm_throwif_long(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(13).unwrap();
    assert!(opc << 3 == 0xf2d0);
    let nn = slice.get_next_int(11).unwrap();
    Some(format!("THROWIF {}", nn).to_string())
}
fn disasm_throwargif(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(13).unwrap();
    assert!(opc << 3 == 0xf2d8);
    let nn = slice.get_next_int(11).unwrap();
    Some(format!("THROWARGIF {}", nn).to_string())
}
fn disasm_throwifnot_long(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(13).unwrap();
    assert!(opc << 3 == 0xf2e0);
    let nn = slice.get_next_int(11).unwrap();
    Some(format!("THROWIFNOT {}", nn).to_string())
}
fn disasm_throwargifnot(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(13).unwrap();
    assert!(opc << 3 == 0xf2e8);
    let nn = slice.get_next_int(11).unwrap();
    Some(format!("THROWARGIFNOT {}", nn).to_string())
}
create_handler_2!(disasm_throwany,         0xf2f0, "THROWANY");
create_handler_2!(disasm_throwargany,      0xf2f1, "THROWARGANY");
create_handler_2!(disasm_throwanyif,       0xf2f2, "THROWANYIF");
create_handler_2!(disasm_throwarganyif,    0xf2f3, "THROWARGANYIF");
create_handler_2!(disasm_throwanyifnot,    0xf2f4, "THROWANYIFNOT");
create_handler_2!(disasm_throwarganyifnot, 0xf2f5, "THROWARGANYIFNOT");
create_handler_2!(disasm_try,              0xf2ff, "TRY");
fn disasm_tryargs(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(8).unwrap();
    assert!(opc == 0xf3);
    let p = slice.get_next_int(4).unwrap();
    let r = slice.get_next_int(4).unwrap();
    Some(format!("TRYARGS {}, {}", p, r).to_string())
}
create_handler_2!(disasm_ldgrams,     0xfa00, "LDGRAMS");
create_handler_2!(disasm_ldvarint16,  0xfa01, "LDVARINT16");
create_handler_2!(disasm_stgrams,     0xfa02, "STGRAMS");
create_handler_2!(disasm_stvarint16,  0xfa03, "STVARINT16");
create_handler_2!(disasm_ldvaruint32, 0xfa04, "LDVARUINT32");
create_handler_2!(disasm_ldvarint32,  0xfa05, "LDVARINT32");
create_handler_2!(disasm_stvaruint32, 0xfa06, "STVARUINT32");
create_handler_2!(disasm_stvarint32,  0xfa07, "STVARINT32");
fn disasm_ldmsgaddr<T>(slice: &mut SliceData) -> Option<String>
where T : OperationBehavior {
    let opc = slice.get_next_int(16).unwrap();
    assert!(opc & 0xfffe == 0xfa40);
    Some(format!("LDMSGADDR{}", T::suffix()).to_string())
}
fn disasm_parsemsgaddr<T>(slice: &mut SliceData) -> Option<String>
where T : OperationBehavior {
    let opc = slice.get_next_int(16).unwrap();
    assert!(opc & 0xfffe == 0xfa42);
    Some(format!("PARSEMSGADDR{}", T::suffix()).to_string())
}
fn disasm_rewrite_std_addr<T>(slice: &mut SliceData) -> Option<String>
where T : OperationBehavior {
    let opc = slice.get_next_int(16).unwrap();
    assert!(opc & 0xfffe == 0xfa44);
    Some(format!("REWRITESTDADDR{}", T::suffix()).to_string())
}
fn disasm_rewrite_var_addr<T>(slice: &mut SliceData) -> Option<String>
where T : OperationBehavior {
    let opc = slice.get_next_int(16).unwrap();
    assert!(opc & 0xfffe == 0xfa46);
    Some(format!("REWRITEVARADDR{}", T::suffix()).to_string())
}
create_handler_2!(disasm_sendrawmsg,         0xfb00, "SENDRAWMSG");
create_handler_2!(disasm_rawreserve,         0xfb02, "RAWRESERVE");
create_handler_2!(disasm_rawreservex,        0xfb03, "RAWRESERVEX");
create_handler_2!(disasm_setcode,            0xfb04, "SETCODE");
create_handler_2!(disasm_setlibcode,         0xfb06, "SETLIBCODE");
create_handler_2!(disasm_changelib,          0xfb07, "CHANGELIB");
create_handler_2!(disasm_stdict,             0xf400, "STDICT");
create_handler_2!(disasm_skipdict,           0xf401, "SKIPDICT");
create_handler_2!(disasm_lddicts,            0xf402, "LDDICTS");
create_handler_2!(disasm_plddicts,           0xf403, "PLDDICTS");
create_handler_2!(disasm_lddict,             0xf404, "LDDICT");
create_handler_2!(disasm_plddict,            0xf405, "PLDDICT");
create_handler_2!(disasm_lddictq,            0xf406, "LDDICT");
create_handler_2!(disasm_plddictq,           0xf407, "PLDDICT");
create_handler_2!(disasm_dictget,            0xf40a, "DICTGET");
create_handler_2!(disasm_dictgetref,         0xf40b, "DICTGETREF");
create_handler_2!(disasm_dictiget,           0xf40c, "DICTIGET");
create_handler_2!(disasm_dictigetref,        0xf40d, "DICTIGETREF");
create_handler_2!(disasm_dictuget,           0xf40e, "DICTUGET");
create_handler_2!(disasm_dictugetref,        0xf40f, "DICTUGETREF");
create_handler_2!(disasm_dictset,            0xf412, "DICTSET");
create_handler_2!(disasm_dictsetref,         0xf413, "DICTSETREF");
create_handler_2!(disasm_dictiset,           0xf414, "DICTISET");
create_handler_2!(disasm_dictisetref,        0xf415, "DICTISETREF");
create_handler_2!(disasm_dictuset,           0xf416, "DICTUSET");
create_handler_2!(disasm_dictusetref,        0xf417, "DICTUSETREF");
create_handler_2!(disasm_dictsetget,         0xf41a, "DICTSETGET");
create_handler_2!(disasm_dictsetgetref,      0xf41b, "DICTSETGETREF");
create_handler_2!(disasm_dictisetget,        0xf41c, "DICTISETGET");
create_handler_2!(disasm_dictisetgetref,     0xf41d, "DICTISETGETREF");
create_handler_2!(disasm_dictusetget,        0xf41e, "DICTUSETGET");
create_handler_2!(disasm_dictusetgetref,     0xf41f, "DICTUSETGETREF");
create_handler_2!(disasm_dictreplace,        0xf422, "DICTREPLACE");
create_handler_2!(disasm_dictreplaceref,     0xf423, "DICTREPLACEREF");
create_handler_2!(disasm_dictireplace,       0xf424, "DICTIREPLACE");
create_handler_2!(disasm_dictireplaceref,    0xf425, "DICTIREPLACEREF");
create_handler_2!(disasm_dictureplace,       0xf426, "DICTUREPLACE");
create_handler_2!(disasm_dictureplaceref,    0xf427, "DICTUREPLACEREF");
create_handler_2!(disasm_dictreplaceget,     0xf42a, "DICTREPLACEGET");
create_handler_2!(disasm_dictreplacegetref,  0xf42b, "DICTREPLACEGETREF");
create_handler_2!(disasm_dictireplaceget,    0xf42c, "DICTIREPLACEGET");
create_handler_2!(disasm_dictireplacegetref, 0xf42d, "DICTIREPLACEGETREF");
create_handler_2!(disasm_dictureplaceget,    0xf42e, "DICTUREPLACEGET");
create_handler_2!(disasm_dictureplacegetref, 0xf42f, "DICTUREPLACEGETREF");
create_handler_2!(disasm_dictadd,            0xf432, "DICTADD");
create_handler_2!(disasm_dictaddref,         0xf433, "DICTADDREF");
create_handler_2!(disasm_dictiadd,           0xf434, "DICTIADD");
create_handler_2!(disasm_dictiaddref,        0xf435, "DICTIADDREF");
create_handler_2!(disasm_dictuadd,           0xf436, "DICTUADD");
create_handler_2!(disasm_dictuaddref,        0xf437, "DICTUADDREF");
create_handler_2!(disasm_dictaddget,         0xf43a, "DICTADDGET");
create_handler_2!(disasm_dictaddgetref,      0xf43b, "DICTADDGETREF");
create_handler_2!(disasm_dictiaddget,        0xf43c, "DICTIADDGET");
create_handler_2!(disasm_dictiaddgetref,     0xf43d, "DICTIADDGETREF");
create_handler_2!(disasm_dictuaddget,        0xf43e, "DICTUADDGET");
create_handler_2!(disasm_dictuaddgetref,     0xf43f, "DICTUADDGETREF");
create_handler_2!(disasm_dictsetb,           0xf441, "DICTSETB");
create_handler_2!(disasm_dictisetb,          0xf442, "DICTISETB");
create_handler_2!(disasm_dictusetb,          0xf443, "DICTUSETB");
create_handler_2!(disasm_dictsetgetb,        0xf445, "DICTSETGETB");
create_handler_2!(disasm_dictisetgetb,       0xf446, "DICTISETGETB");
create_handler_2!(disasm_dictusetgetb,       0xf447, "DICTUSETGETB");
create_handler_2!(disasm_dictreplaceb,       0xf449, "DICTREPLACEB");
create_handler_2!(disasm_dictireplaceb,      0xf44a, "DICTIREPLACEB");
create_handler_2!(disasm_dictureplaceb,      0xf44b, "DICTUREPLACEB");
create_handler_2!(disasm_dictreplacegetb,    0xf44d, "DICTREPLACEGETB");
create_handler_2!(disasm_dictireplacegetb,   0xf44e, "DICTIREPLACEGETB");
create_handler_2!(disasm_dictureplacegetb,   0xf44f, "DICTUREPLACEGETB");
create_handler_2!(disasm_dictaddb,           0xf451, "DICTADDB");
create_handler_2!(disasm_dictiaddb,          0xf452, "DICTIADDB");
create_handler_2!(disasm_dictuaddb,          0xf453, "DICTUADDB");
create_handler_2!(disasm_dictaddgetb,        0xf455, "DICTADDGETB");
create_handler_2!(disasm_dictiaddgetb,       0xf456, "DICTIADDGETB");
create_handler_2!(disasm_dictuaddgetb,       0xf457, "DICTUADDGETB");
create_handler_2!(disasm_dictdel,            0xf459, "DICTDEL");
create_handler_2!(disasm_dictidel,           0xf45a, "DICTIDEL");
create_handler_2!(disasm_dictudel,           0xf45b, "DICTUDEL");
create_handler_2!(disasm_dictdelget,         0xf462, "DICTDELGET");
create_handler_2!(disasm_dictdelgetref,      0xf443, "DICTDELGETREF");
create_handler_2!(disasm_dictidelget,        0xf444, "DICTIDELGET");
create_handler_2!(disasm_dictidelgetref,     0xf445, "DICTIDELGETREF");
create_handler_2!(disasm_dictudelget,        0xf446, "DICTUDELGET");
create_handler_2!(disasm_dictudelgetref,     0xf467, "DICTUDELGETREF");
create_handler_2!(disasm_dictgetoptref,      0xf469, "DICTGETOPTREF");
create_handler_2!(disasm_dictigetoptref,     0xf46a, "DICTIGETOPTREF");
create_handler_2!(disasm_dictugetoptref,     0xf46b, "DICTUGETOPTREF");
create_handler_2!(disasm_dictsetgetoptref,   0xf46d, "DICTSETGETOPTREF");
create_handler_2!(disasm_dictisetgetoptref,  0xf46e, "DICTISETGETOPTREF");
create_handler_2!(disasm_dictusetgetoptref,  0xf46f, "DICTUSETGETOPTREF");
create_handler_2!(disasm_pfxdictset,         0xf470, "PFXDICTSET");
create_handler_2!(disasm_pfxdictreplace,     0xf471, "PFXDICTREPLACE");
create_handler_2!(disasm_pfxdictadd,         0xf472, "PFXDICTADD");
create_handler_2!(disasm_pfxdictdel,         0xf473, "PFXDICTDEL");
create_handler_2!(disasm_dictgetnext,        0xf474, "DICTGETNEXT");
create_handler_2!(disasm_dictgetnexteq,      0xf475, "DICTGETNEXTEQ");
create_handler_2!(disasm_dictgetprev,        0xf476, "DICTGETPREV");
create_handler_2!(disasm_dictgetpreveq,      0xf477, "DICTGETPREVEQ");
create_handler_2!(disasm_dictigetnext,       0xf478, "DICTIGETNEXT");
create_handler_2!(disasm_dictigetnexteq,     0xf479, "DICTIGETNEXTEQ");
create_handler_2!(disasm_dictigetprev,       0xf47a, "DICTIGETPREV");
create_handler_2!(disasm_dictigetpreveq,     0xf47b, "DICTIGETPREVEQ");
create_handler_2!(disasm_dictugetnext,       0xf47c, "DICTUGETNEXT");
create_handler_2!(disasm_dictugetnexteq,     0xf47d, "DICTUGETNEXTEQ");
create_handler_2!(disasm_dictugetprev,       0xf47e, "DICTUGETPREV");
create_handler_2!(disasm_dictugetpreveq,     0xf47f, "DICTUGETPREVEQ");
create_handler_2!(disasm_dictmin,            0xf482, "DICTMIN");
create_handler_2!(disasm_dictminref,         0xf483, "DICTMINREF");
create_handler_2!(disasm_dictimin,           0xf484, "DICTIMIN");
create_handler_2!(disasm_dictiminref,        0xf485, "DICTIMINREF");
create_handler_2!(disasm_dictumin,           0xf486, "DICTUMIN");
create_handler_2!(disasm_dictuminref,        0xf487, "DICTUMINREF");
create_handler_2!(disasm_dictmax,            0xf48a, "DICTMAX");
create_handler_2!(disasm_dictmaxref,         0xf48b, "DICTMAXREF");
create_handler_2!(disasm_dictimax,           0xf48c, "DICTIMAX");
create_handler_2!(disasm_dictimaxref,        0xf48d, "DICTIMAXREF");
create_handler_2!(disasm_dictumax,           0xf48e, "DICTUMAX");
create_handler_2!(disasm_dictumaxref,        0xf48f, "DICTUMAXREF");
create_handler_2!(disasm_dictremmin,         0xf492, "DICTREMMIN");
create_handler_2!(disasm_dictremminref,      0xf493, "DICTREMMINREF");
create_handler_2!(disasm_dictiremmin,        0xf494, "DICTIREMMIN");
create_handler_2!(disasm_dictiremminref,     0xf495, "DICTIREMMINREF");
create_handler_2!(disasm_dicturemmin,        0xf496, "DICTUREMMIN");
create_handler_2!(disasm_dicturemminref,     0xf497, "DICTUREMMINREF");
create_handler_2!(disasm_dictremmax,         0xf49a, "DICTREMMAX");
create_handler_2!(disasm_dictremmaxref,      0xf49b, "DICTREMMAXREF");
create_handler_2!(disasm_dictiremmax,        0xf49c, "DICTIREMMAX");
create_handler_2!(disasm_dictiremmaxref,     0xf49d, "DICTIREMMAXREF");
create_handler_2!(disasm_dicturemmax,        0xf49e, "DICTUREMMAX");
create_handler_2!(disasm_dicturemmaxref,     0xf49f, "DICTUREMMAXREF");
create_handler_2!(disasm_dictigetjmp,        0xf4a0, "DICTIGETJMP");
create_handler_2!(disasm_dictugetjmp,        0xf4a1, "DICTUGETJMP");
create_handler_2!(disasm_dictigetexec,       0xf4a2, "DICTIGETEXEC");
create_handler_2!(disasm_dictugetexec,       0xf4a3, "DICTUGETEXEC");
fn disasm_dictpushconst(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(14).unwrap();
    assert!(opc << 2 == 0xf4a4);
    let n = slice.get_next_int(10).unwrap();
    // TODO shrink?
    Some(format!("DICTPUSHCONST {}", n).to_string())
}
create_handler_2!(disasm_pfxdictgetq,    0xf4a8, "PFXDICTGETQ");
create_handler_2!(disasm_pfxdictget,     0xf4a9, "PFXDICTGET");
create_handler_2!(disasm_pfxdictgetjmp,  0xf4aa, "PFXDICTGETJMP");
create_handler_2!(disasm_pfxdictgetexec, 0xf4ab, "PFXDICTGETEXEC");
fn disasm_pfxdictswitch(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(14).unwrap();
    assert!(opc << 2 == 0xf4ac);
    let n = slice.get_next_int(10).unwrap();
    // TODO shrink?
    Some(format!("PFXDICTSWITCH {}", n).to_string())
}
create_handler_2!(disasm_subdictget,    0xf4b1, "SUBDICTGET");
create_handler_2!(disasm_subdictiget,   0xf4b2, "SUBDICTIGET");
create_handler_2!(disasm_subdictuget,   0xf4b3, "SUBDICTUGET");
create_handler_2!(disasm_subdictrpget,  0xf4b5, "SUBDICTRPGET");
create_handler_2!(disasm_subdictirpget, 0xf4b6, "SUBDICTIRPGET");
create_handler_2!(disasm_subdicturpget, 0xf4b7, "SUBDICTURPGET");
create_handler_2!(disasm_dictigetjmpz,  0xf4bc, "DICTIGETJMPZ");
create_handler_2!(disasm_dictugetjmpz,  0xf4bd, "DICTUGETJMPZ");
create_handler_2!(disasm_dictigetexecz, 0xf4be, "DICTIGETEXECZ");
create_handler_2!(disasm_dictugetexecz, 0xf4bf, "DICTUGETEXECZ");
create_handler_2!(disasm_accept,        0xf800, "ACCEPT");
create_handler_2!(disasm_setgaslimit,   0xf801, "SETGASLIMIT");
create_handler_2!(disasm_buygas,        0xf802, "BUYGAS");
create_handler_2!(disasm_gramtogas,     0xf804, "GRAMTOGAS");
create_handler_2!(disasm_gastogram,     0xf805, "GASTOGRAM");
create_handler_2!(disasm_commit,        0xf80f, "COMMIT");
create_handler_2!(disasm_randu256,      0xf810, "RANDU256");
create_handler_2!(disasm_rand,          0xf811, "RAND");
create_handler_2!(disasm_setrand,       0xf814, "SETRAND");
create_handler_2!(disasm_addrand,       0xf815, "ADDRAND");
fn disasm_getparam(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0xf82);
    let i = slice.get_next_int(4).unwrap();
    Some(format!("GETPARAM {}", i).to_string())
}
create_handler_2!(disasm_now,              0xf823, "NOW");
create_handler_2!(disasm_blocklt,          0xf824, "BLOCKLT");
create_handler_2!(disasm_ltime,            0xf825, "LTIME");
create_handler_2!(disasm_randseed,         0xf826, "RANDSEED");
create_handler_2!(disasm_balance,          0xf827, "BALANCE");
create_handler_2!(disasm_my_addr,          0xf828, "MYADDR");
create_handler_2!(disasm_config_root,      0xf829, "CONFIGROOT");
create_handler_2!(disasm_config_dict,      0xf830, "CONFIGDICT");
create_handler_2!(disasm_config_ref_param, 0xf832, "CONFIGPARAM");
create_handler_2!(disasm_config_opt_param, 0xf833, "CONFIGOPTPARAM");
create_handler_2!(disasm_getglobvar,       0xf840, "GETGLOBVAR");
fn disasm_getglob(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(11).unwrap();
    assert!(opc << 1 == 0xf84);
    let k = slice.get_next_int(5).unwrap();
    assert!(k != 0);
    Some(format!("GETGLOB {}", k).to_string())
}
create_handler_2!(disasm_setglobvar, 0xf860, "SETGLOBVAR");
fn disasm_setglob(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(11).unwrap();
    assert!(opc << 1 == 0xf86);
    let k = slice.get_next_int(5).unwrap();
    assert!(k != 0);
    Some(format!("SETGLOB {}", k).to_string())
}
create_handler_2!(disasm_hashcu,     0xf900, "HASHCU");
create_handler_2!(disasm_hashsu,     0xf901, "HASHSU");
create_handler_2!(disasm_sha256u,    0xf902, "SHA256U");
create_handler_2!(disasm_chksignu,   0xf910, "CHKSIGNU");
create_handler_2!(disasm_chksigns,   0xf911, "CHKSIGNS");
create_handler_2!(disasm_cdatasizeq, 0xf940, "CDATASIZEQ");
create_handler_2!(disasm_cdatasize,  0xf941, "CDATASIZE");
create_handler_2!(disasm_sdatasizeq, 0xf942, "SDATASIZEQ");
create_handler_2!(disasm_sdatasize,  0xf943, "SDATASIZE");
create_handler_2!(disasm_dump_stack, 0xfe00, "DUMPSTK");
fn disasm_dump_stack_top(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0xfe0);
    let n = slice.get_next_int(4).unwrap();
    assert!(n > 0);
    Some(format!("DUMPSTKTOP {}", n).to_string())
}
create_handler_2!(disasm_dump_hex,  0xfe10, "HEXDUMP");
create_handler_2!(disasm_print_hex, 0xfe11, "HEXPRINT");
create_handler_2!(disasm_dump_bin,  0xfe12, "BINDUMP");
create_handler_2!(disasm_print_bin, 0xfe13, "BINPRINT");
create_handler_2!(disasm_dump_str,  0xfe14, "STRDUMP");
create_handler_2!(disasm_print_str, 0xfe15, "STRPRINT");
create_handler_2!(disasm_debug_off, 0xfe1e, "DEBUGOFF");
create_handler_2!(disasm_debug_on,  0xfe1f, "DEBUGON");
fn disasm_dump_var(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0xfe2);
    let n = slice.get_next_int(4).unwrap();
    assert!(n < 15);
    Some(format!("DUMP s{}", n).to_string())
}
fn disasm_print_var(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0xfe3);
    let n = slice.get_next_int(4).unwrap();
    assert!(n < 15);
    Some(format!("PRINT s{}", n).to_string())
}
fn disasm_dump_string(slice: &mut SliceData) -> Option<String> {
    let opc = slice.get_next_int(12).unwrap();
    assert!(opc == 0xfef);
    let n = slice.get_next_int(4).unwrap();
    let mode = slice.get_next_int(8).unwrap();
    match n {
        0 => {
            assert!(mode == 0x00);
            Some("LOGFLUSH".to_string())
        }
        _ => {
            if mode == 0x00 {
                let s = slice.get_next_slice(n as usize * 8).unwrap();
                Some(format!("LOGSTR {}", s.to_hex_string()).to_string())
            } else if mode == 0x01 {
                let s = slice.get_next_slice(n as usize * 8).unwrap();
                Some(format!("PRINTSTR {}", s.to_hex_string()).to_string())
            } else {
                println!("dump_string?");
                None
            }
        }
    }
}
