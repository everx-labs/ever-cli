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

use sha2::{Sha256, Digest};

pub fn convert_token(amount: &str) -> Result<String, String> {
    convert_amount(amount, 9)
}

pub fn convert_amount(amount: &str, decimals: usize) -> Result<String, String> {
    let parts: Vec<&str> = amount.split(".").collect();
    if parts.len() >= 1 && parts.len() <= 2 {
        let mut result = String::new();
        result += parts[0];
        if parts.len() == 2 {
            let fraction = format!("{:0<width$}", parts[1], width = decimals);
            if fraction.len() != decimals {
                return Err("invalid fractional part".to_string());
            }
            result += &fraction;
        } else {
            result += &"0".repeat(decimals);
        }
        u64::from_str_radix(&result, 10)
            .map_err(|e| format!("failed to parse amount: {}", e))?;

        return Ok(result);
    }
    Err("Invalid amout value".to_string())
}

pub fn nodeid_from_pubkey(key: &[u8]) -> Result<String, String> {
    if key.len() != 32 {
        return Err("Public key must be 32 byte long".to_owned());
    }
    let mut hasher = Sha256::new();
    // node id magic
    hasher.input(&[0xc6, 0xb4, 0x13, 0x48]);
    //key
    hasher.input(key);

    Ok(hex::encode(&hasher.result()))
}
