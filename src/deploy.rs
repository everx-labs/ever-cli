/*
 * Copyright 2018-2019 TON DEV SOLUTIONS LTD.
 *
 * Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
 * this file except in compliance with the License.  You may obtain a copy of the
 * License at: https://ton.dev/licenses
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific TON DEV software governing permissions and
 * limitations under the License.
 */
use crate::config::Config;
use crate::helpers::read_keys;
use ton_client_rs::TonClient;

pub fn deploy_contract(conf: Config, tvc: &str, abi: &str, params: &str, keys_file: &str) -> Result<(), String> {
    let ton = TonClient::new_with_base_url(&conf.url)
        .map_err(|e| format!("failed to create tonclient: {}", e.to_string()))?;
    
    let abi = std::fs::read_to_string(abi)
        .map_err(|e| format!("failed to read ABI file: {}", e.to_string()))?;
    
    let keys = read_keys(keys_file)?;
    
    let contract = std::fs::read(tvc)
        .map_err(|e| format!("failed to read smart contract file: {}", e.to_string()))?;
    
    println!("Deploying...");
    let result = ton.contracts.deploy(&abi, &contract, None, params.into(), None, &keys, 0)
        .map_err(|e| format!("deploy failed: {}", e.to_string()))?;

    println!("Transaction succeded.");
    println!("Contract deployed at address: {}", result.address);
    Ok(())
}