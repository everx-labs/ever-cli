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
use crate::helpers::create_client_verbose;
use crate::config::Config;
use crate::crypto::load_keypair;

pub fn deploy_contract(conf: Config, tvc: &str, abi: &str, params: &str, keys_file: &str, wc: i32) -> Result<(), String> {
    let ton = create_client_verbose(&conf)?;
    
    let abi = std::fs::read_to_string(abi)
        .map_err(|e| format!("failed to read ABI file: {}", e.to_string()))?;
    
    let keys = load_keypair(keys_file)?;
    
    let contract = std::fs::read(tvc)
        .map_err(|e| format!("failed to read smart contract file: {}", e.to_string()))?;
    
    println!("Deploying...");
    let result = ton.contracts.deploy(abi.into(), &contract, None, params.into(), None, &keys, wc)
        .map_err(|e| format!("deploy failed: {}", e.to_string()))?;

    println!("Transaction succeeded.");
    println!("Contract deployed at address: {}", result.address);
    Ok(())
}