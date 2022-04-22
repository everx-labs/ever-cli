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
use crate::helpers::{create_client_verbose, query_with_limit};
use crate::config::Config;
use serde_json::{json};
use ton_block::{Serializable};
use ton_client::net::{OrderBy, SortDirection};
use ton_client::boc::{get_blockchain_config, ParamsOfGetBlockchainConfig};
use crate::call::{prepare_message_new_config_param, serialize_config_param};

const QUERY_FIELDS: &str = r#"
master { 
    config {
      p0
      p1
      p2
      p3
      p4
      p6 {
        mint_new_price
        mint_add_price
      }
      p7 {
        currency
        value
      }
      p8 {
        version
        capabilities
      }
      p9 
      p10
      p12 {
        workchain_id
        enabled_since
        actual_min_split
        min_split
        max_split
        active
        accept_msgs flags
        zerostate_root_hash
        zerostate_file_hash
        version
        basic
        vm_version
        vm_mode
        min_addr_len
        max_addr_len
        addr_len_step
        workchain_type_id
      }
      p14 {
        masterchain_block_fee
        basechain_block_fee
      }
      p15 {
        validators_elected_for
        elections_start_before
        elections_end_before
        stake_held_for
      }
       p16 {
        max_validators
        max_main_validators
        min_validators
      }
      p17 {
        min_stake(format:DEC)
        max_stake(format:DEC)
        min_total_stake(format:DEC)
        max_stake_factor
      }
      p20 {
        gas_price
        gas_limit
        special_gas_limit
        gas_credit
        block_gas_limit
        freeze_due_limit
        delete_due_limit
        flat_gas_limit
        flat_gas_price
      }
      p21 {
        gas_price
        gas_limit
        special_gas_limit
        gas_credit
        block_gas_limit
        freeze_due_limit
        delete_due_limit
        flat_gas_limit
        flat_gas_price
      }
      p22 {
        bytes {
          underload soft_limit hard_limit
        }
        gas {
          underload soft_limit hard_limit
        }
        lt_delta {
          underload soft_limit hard_limit
        }
      }
      p23 {
        bytes {
          underload soft_limit hard_limit
        }
        gas {
          underload soft_limit hard_limit
        }
        lt_delta {
          underload soft_limit hard_limit
        }
      }
      p24 {
        lump_price bit_price cell_price ihr_price_factor first_frac next_frac
      }
      p25 {
        lump_price bit_price cell_price ihr_price_factor first_frac next_frac
      }
      p28 {
        shuffle_mc_validators
        mc_catchain_lifetime
        shard_catchain_lifetime
        shard_validators_lifetime
        shard_validators_num
      }
      p29 {
        new_catchain_ids
        round_candidates
        next_candidate_delay_ms
        consensus_timeout_ms
        fast_attempts
        attempt_duration
        catchain_max_deps
        max_block_bytes
        max_collated_bytes
      }
      p31
      p32 {
        utime_since
        utime_until
        total
        total_weight(format:DEC)
        list {
          public_key
          adnl_addr
          weight(format:DEC)
        }
      }
      p33 {
        utime_since
        utime_until
        total
        total_weight(format:DEC)
        list {
          public_key
          adnl_addr
          weight(format:DEC)
        }
      }
      p34 {
        utime_since
        utime_until
        total
        total_weight(format:DEC)
        list {
          public_key
          adnl_addr
          weight(format:DEC)
        }
      }
      p35 {
        utime_since
        utime_until
        total
        total_weight(format:DEC)
        list {
          public_key
          adnl_addr
          weight(format:DEC)
        }
      }
      p36 {
        utime_since
        utime_until
        total
        total_weight(format:DEC)
        list {
          public_key
          adnl_addr
          weight(format:DEC)
        }
      }
      p37 {
        utime_since
        utime_until
        total
        total_weight(format:DEC)
        list {
          public_key
          adnl_addr
          weight(format:DEC)
        }
      }
      p39 {
        adnl_addr
        temp_public_key
        seqno
        valid_until
        signature_r
        signature_s
      }
    }
  }
"#;

pub async fn query_global_config(config: &Config, index: Option<&str>) -> Result<(), String> {
    let ton = create_client_verbose(&config)?;

    let last_key_block_query = query_with_limit(
        ton.clone(),
        "blocks",
        json!({ "workchain_id": { "eq":-1 } }),
        "id prev_key_block_seqno",
        Some(vec![OrderBy{ path: "seq_no".to_owned(), direction: SortDirection::DESC }]),
        Some(1),
    ).await.map_err(|e| format!("failed to query last key block: {}", e))?;

    if last_key_block_query.is_empty()  ||
        last_key_block_query[0]["prev_key_block_seqno"].as_u64() == Some(0) {
        return Err("Key block not found".to_string());
    }

    let config_query = query_with_limit(
        ton.clone(),
        "blocks",
        json!({
            "seq_no": {
                "eq": last_key_block_query[0]["prev_key_block_seqno"].as_u64()
                    .ok_or("failed to decode prev_key_block_seqno")?
            },
            "workchain_id": {
                "eq": -1 
            }
        }),
        QUERY_FIELDS,
        None,
        Some(1),
    ).await.map_err(|e| format!("failed to query master block config: {}", e))?;

    if config_query.is_empty() {
        return Err("Config was not set".to_string());
    }

    match index {
        None => {
            let config_value = &config_query[0]["master"]["config"];
            println!("{}{}", if !config.is_json {
                "Config: "
            } else {
                ""
            }, serde_json::to_string_pretty(&config_value)
                .map_err(|e| format!("failed to parse config body from sdk: {}", e))?);
        },
        Some(index) => {
            let _i = i32::from_str_radix(index, 10)
                .map_err(|e| format!(r#"failed to parse "index": {}"#, e))?;
            let config_name = format!("p{}", index);
            let config_value = &config_query[0]["master"]["config"][&config_name];
            println!("{}{}", if !config.is_json {
                format!("Config {}: ", config_name)
            } else {
                "".to_string()
            }, serde_json::to_string_pretty(&config_value)
                         .map_err(|e| format!("failed to parse config body from sdk: {}", e))?);
        }
    }
    Ok(())
}

pub async fn gen_update_config_message(
    seqno: &str,
    config_master_file: &str,
    new_param_file: &str,
    is_json: bool
) -> Result<(), String> {
    let seqno = u32::from_str_radix(seqno, 10)
        .map_err(|e| format!(r#"failed to parse "seqno": {}"#, e))?;

    let config_master_address = std::fs::read(&*(config_master_file.to_string() + ".addr"))
        .map_err(|e| format!(r#"failed to read "config_master": {}"#, e))?;
    let config_account = ton_types::AccountId::from_raw(config_master_address, 32*8);

    let private_key = std::fs::read(&*(config_master_file.to_string() + ".pk"))
        .map_err(|e| format!(r#"failed to read "config_master": {}"#, e))?;

    let config_str = std::fs::read_to_string(new_param_file)
        .map_err(|e| format!(r#"failed to read "new_param_file": {}"#, e))?;

    let (config_cell, key_number) = serialize_config_param(config_str)?;
    let message = prepare_message_new_config_param(config_cell, seqno, key_number, config_account, private_key)?;

    let msg_bytes = message.write_to_bytes()
        .map_err(|e| format!(r#"failed to serialize message": {}"#, e))?;
    let msg_hex = hex::encode(&msg_bytes);

    if is_json {
        println!("{{\"Message\": \"{}\"}}", msg_hex);
    } else {
        println!("Message: {}", msg_hex);
    }

    Ok(())
}

pub async fn dump_blockchain_config(config: &Config, path: &str) -> Result<(), String> {
    let ton = create_client_verbose(&config)?;

    let last_key_block_query = query_with_limit(
        ton.clone(),
        "blocks",
        json!({ "workchain_id": { "eq":-1 }, "key_block": { "eq":true }}),
        "boc",
        Some(vec![OrderBy{ path: "seq_no".to_owned(), direction: SortDirection::DESC }]),
        Some(1),
    ).await.map_err(|e| format!("failed to query last key block: {}", e))?;

    if last_key_block_query.is_empty() {
        return Err("Key block not found".to_string());
    }

    let block = last_key_block_query[0]["boc"].as_str()
        .ok_or("Failed to query last block BOC.")?.to_owned();

    let bc_config = get_blockchain_config(
        ton.clone(),
        ParamsOfGetBlockchainConfig {
            block_boc: block,
        },
    ).await
        .map_err(|e| format!("Failed to get blockchain config: {}", e))?;

    let bc_config = base64::decode(&bc_config.config_boc)
        .map_err(|e| format!("Failed to decode BOC: {}", e))?;
    std::fs::write(path, bc_config)
        .map_err(|e| format!("Failed to write data to the file {}: {}", path, e))?;
    if !config.is_json {
        println!("Config successfully saved to {}", path);
    } else {
        println!("{{}}");
    }
    Ok(())
}
