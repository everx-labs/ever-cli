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
use crate::helpers::{create_client_verbose, query};
use crate::config::Config;
use serde_json::json;
use ton_client::net::{OrderBy, SortDirection};

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

pub async fn query_global_config(conf: Config, index: &str) -> Result<(), String> {
    let ton = create_client_verbose(&conf)?;

    let _i = i32::from_str_radix(index, 10)
        .map_err(|e| format!(r#"failed to parse "index": {}"#, e))?;
    
    let config_name = format!("p{}", index);

    let last_key_block_query = query(
        ton.clone(),
        "blocks",
        json!({ "workchain_id": { "eq":-1 } }),
        "id prev_key_block_seqno",
        Some(vec![OrderBy{ path: "seq_no".to_owned(), direction: SortDirection::DESC }]),
    ).await.map_err(|e| format!("failed to query last key block: {}", e))?;

    if last_key_block_query.len() == 0 {
        Err("Key block not found".to_string())?;
    }

    let config_query = query(
        ton.clone(),
        "blocks",
        json!({
            "seq_no": {
                "eq": last_key_block_query[0]["prev_key_block_seqno"].as_u64().unwrap() 
            },
            "workchain_id": {
                "eq": -1 
            }
        }),
        QUERY_FIELDS,
        None,
    ).await.map_err(|e| format!("failed to query master block config: {}", e))?;

    if config_query.len() == 0 {
        Err("Config was not set".to_string())?;
    }

    let config = &config_query[0]["master"]["config"][&config_name];
    let config_str = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("failed to parse config body from sdk: {}", e))?;

    if conf.is_json {
        println!("{}", config_str);
    } else {
        println!("Config {}: {}", config_name, config_str);
    }
    Ok(())
}