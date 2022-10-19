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
extern crate reqwest;
use crate::call;
use crate::config::Config;
use crate::convert;
use crate::deploy::prepare_deploy_message_params;
use crate::helpers::{create_client_local, load_abi, load_ton_address, create_client_verbose, load_file_with_url};
use clap::{App, ArgMatches, SubCommand, Arg, AppSettings};
use ton_client::abi::{encode_message_body, ParamsOfEncodeMessageBody, CallSet};
use crate::crypto::load_keypair;

const SAFEMULTISIG_LINK: &str = "https://github.com/tonlabs/ton-labs-contracts/blob/master/solidity/safemultisig/SafeMultisigWallet.tvc?raw=true";
const SETCODEMULTISIG_LINK: &str = "https://github.com/tonlabs/ton-labs-contracts/blob/master/solidity/setcodemultisig/SetcodeMultisigWallet.tvc?raw=true";

pub const MSIG_ABI: &str = r#"{
	"ABI version": 2,
	"header": ["pubkey", "time", "expire"],
	"functions": [
		{
			"name": "constructor",
			"inputs": [
				{"name":"owners","type":"uint256[]"},
				{"name":"reqConfirms","type":"uint8"}
			],
			"outputs": [
			]
		},
		{
			"name": "acceptTransfer",
			"inputs": [
				{"name":"payload","type":"bytes"}
			],
			"outputs": [
			]
		},
		{
			"name": "sendTransaction",
			"inputs": [
				{"name":"dest","type":"address"},
				{"name":"value","type":"uint128"},
				{"name":"bounce","type":"bool"},
				{"name":"flags","type":"uint8"},
				{"name":"payload","type":"cell"}
			],
			"outputs": [
			]
		},
		{
			"name": "submitTransaction",
			"inputs": [
				{"name":"dest","type":"address"},
				{"name":"value","type":"uint128"},
				{"name":"bounce","type":"bool"},
				{"name":"allBalance","type":"bool"},
				{"name":"payload","type":"cell"}
			],
			"outputs": [
				{"name":"transId","type":"uint64"}
			]
		},
		{
			"name": "confirmTransaction",
			"inputs": [
				{"name":"transactionId","type":"uint64"}
			],
			"outputs": [
			]
		},
		{
			"name": "isConfirmed",
			"inputs": [
				{"name":"mask","type":"uint32"},
				{"name":"index","type":"uint8"}
			],
			"outputs": [
				{"name":"confirmed","type":"bool"}
			]
		},
		{
			"name": "getParameters",
			"inputs": [
			],
			"outputs": [
				{"name":"maxQueuedTransactions","type":"uint8"},
				{"name":"maxCustodianCount","type":"uint8"},
				{"name":"expirationTime","type":"uint64"},
				{"name":"minValue","type":"uint128"},
				{"name":"requiredTxnConfirms","type":"uint8"}
			]
		},
		{
			"name": "getTransaction",
			"inputs": [
				{"name":"transactionId","type":"uint64"}
			],
			"outputs": [
				{"components":[{"name":"id","type":"uint64"},{"name":"confirmationsMask","type":"uint32"},{"name":"signsRequired","type":"uint8"},{"name":"signsReceived","type":"uint8"},{"name":"creator","type":"uint256"},{"name":"index","type":"uint8"},{"name":"dest","type":"address"},{"name":"value","type":"uint128"},{"name":"sendFlags","type":"uint16"},{"name":"payload","type":"cell"},{"name":"bounce","type":"bool"}],"name":"trans","type":"tuple"}
			]
		},
		{
			"name": "getTransactions",
			"inputs": [
			],
			"outputs": [
				{"components":[{"name":"id","type":"uint64"},{"name":"confirmationsMask","type":"uint32"},{"name":"signsRequired","type":"uint8"},{"name":"signsReceived","type":"uint8"},{"name":"creator","type":"uint256"},{"name":"index","type":"uint8"},{"name":"dest","type":"address"},{"name":"value","type":"uint128"},{"name":"sendFlags","type":"uint16"},{"name":"payload","type":"cell"},{"name":"bounce","type":"bool"}],"name":"transactions","type":"tuple[]"}
			]
		},
		{
			"name": "getTransactionIds",
			"inputs": [
			],
			"outputs": [
				{"name":"ids","type":"uint64[]"}
			]
		},
		{
			"name": "getCustodians",
			"inputs": [
			],
			"outputs": [
				{"components":[{"name":"index","type":"uint8"},{"name":"pubkey","type":"uint256"}],"name":"custodians","type":"tuple[]"}
			]
		}
	],
	"data": [
	],
	"events": [
		{
			"name": "TransferAccepted",
			"inputs": [
				{"name":"payload","type":"bytes"}
			],
			"outputs": [
			]
		}
	]
}"#;

pub const TRANSFER_WITH_COMMENT: &str = r#"{
	"ABI version": 1,
	"functions": [
		{
			"name": "transfer",
			"id": "0x00000000",
			"inputs": [{"name":"comment","type":"bytes"}],
			"outputs": []
		}
	],
	"events": [],
	"data": []
}"#;

const LOCAL_GIVER_TRANSFER: &str = r#"{
	"ABI version": 1,
	"functions": [
		{
			"name": "sendGrams",
			"inputs": [
				{"name": "dest", "type": "address"},
				{"name": "amount", "type": "uint64"}
			],
			"outputs": []
		}
	],
	"events": [],
	"data": []
}"#;

const LOCAL_GIVER_ADDR: &str = "0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94";

pub fn create_multisig_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("multisig")
        .about("Multisignature wallet commands.")        
        .setting(AppSettings::AllowNegativeNumbers)
        .setting(AppSettings::DontCollapseArgsInUsage)
        .subcommand(SubCommand::with_name("send")
            .setting(AppSettings::AllowLeadingHyphen)
            .about("Transfers funds from the multisignature wallet to the recipient.")
            .arg(Arg::with_name("ADDRESS")
                .long("--addr")
                .takes_value(true)
                .help("Wallet address."))
            .arg(Arg::with_name("DEST")
                .long("--dest")
                .takes_value(true)
                .help("Recepient address."))
            .arg(Arg::with_name("VALUE")
                .long("--value")
                .takes_value(true)
                .help("Amount of funds to transfer (in tons)."))
            .arg(Arg::with_name("PURPOSE")
                .long("--purpose")
                .takes_value(true)
                .help("Purpose of payment."))
            .arg(Arg::with_name("SIGN")
                .long("--sign")
                .takes_value(true)
                .help("Seed phrase or path to the file with keypair.")))
        .subcommand(SubCommand::with_name("deploy")
            .setting(AppSettings::AllowLeadingHyphen)
            .about("Deploys a multisignature wallet with a given public key. By default deploys a SafeMultisigWallet with one custodian, which can be tuned with flags.")
            .arg(Arg::with_name("KEYS")
                .long("--keys")
                .short("-k")
                .takes_value(true)
                .help("Path to the file with a keypair."))
            .arg(Arg::with_name("SETCODE")
                .long("--setcode")
                .help("Deploy a SetcodeMultisigWallet."))
            .arg(Arg::with_name("VALUE")
                .long("--local")
                .takes_value(true)
                .short("-l")
                .help("Perform a preliminary call of local giver to initialize contract with given value."))
            .arg(Arg::with_name("OWNERS")
                .long("--owners")
                .takes_value(true)
                .short("-o")
                .help("Array of Multisignature wallet owners public keys. Note that deployer could be not included in this case. If not specified the only owner is contract deployer."))
            .arg(Arg::with_name("CONFIRMS")
                .long("--confirms")
                .takes_value(true)
                .short("-c")
                .help("Number of confirmations required for executing transaction. Default value is 1.")))
}

pub async fn multisig_command(m: &ArgMatches<'_>, config: &Config) -> Result<(), String> {
    if let Some(m) = m.subcommand_matches("send") {
        return multisig_send_command(m, config).await;
    }
    if let Some(m) = m.subcommand_matches("deploy") {
        return multisig_deploy_command(m, config).await;
    }
    Err("unknown multisig command".to_owned())
}

async fn multisig_send_command(matches: &ArgMatches<'_>, config: &Config) -> Result<(), String> {
    let address = matches.value_of("ADDRESS")
        .ok_or("--addr parameter is not defined".to_string())?;
    let dest = matches.value_of("DEST")
        .ok_or("--dst parameter is not defined".to_string())?;
    let keys = matches.value_of("SIGN")
        .ok_or("--sign parameter is not defined".to_string())?;
    let value = matches.value_of("VALUE")
        .ok_or("--value parameter is not defined".to_string())?;
    let comment = matches.value_of("PURPOSE");

    let address = load_ton_address(address, &config)?;
    send(config, address.as_str(), dest, value, keys, comment).await
}

pub async fn encode_transfer_body(text: &str, config: &Config) -> Result<String, String> {
    let text = hex::encode(text.as_bytes());
    let client = create_client_local()?;
    let abi = load_abi(TRANSFER_WITH_COMMENT, config).await?;
    encode_message_body(
        client.clone(),
        ParamsOfEncodeMessageBody {
            abi,
            call_set: CallSet::some_with_function_and_input(
                "transfer",
                json!({ "comment": text    }),
            ).ok_or("failed to create CallSet with specified parameters")?,
            is_internal: true,
            ..Default::default()
        },
    ).await
    .map_err(|e| format!("failed to encode transfer body: {}", e))
    .map(|r| r.body)
}

async fn send(
    config: &Config,
    addr: &str,
    dest: &str,
    value: &str,
    keys: &str,
    comment: Option<&str>
) -> Result<(), String> {
    let body = if let Some(text) = comment {
        encode_transfer_body(text, config).await?
    } else {
        "".to_owned()
    };

    send_with_body(config, addr, dest, value, keys, &body).await
}

pub async fn send_with_body(
    config: &Config,
    addr: &str,
    dest: &str,
    value: &str,
    keys: &str,
    body: &str,
) -> Result<(), String> {
    let params = json!({
        "dest": dest,
        "value": convert::convert_token(value)?,
        "bounce": true,
        "allBalance": false,
        "payload": body,
    }).to_string();

    call::call_contract(
        config,
        addr,
        MSIG_ABI,
        "submitTransaction",
        &params,
        Some(keys.to_owned()),
        false,
        None,
    ).await
}

async fn multisig_deploy_command(matches: &ArgMatches<'_>, config: &Config) -> Result<(), String> {
    let keys = matches.value_of("KEYS")
        .map(|s| s.to_string())
        .or(config.keys_path.clone())
        .ok_or("keypair file is not defined. Supply it in the config file or command line.".to_string())?;

    let is_setcode = matches.is_present("SETCODE");

    let target = if is_setcode {
        SETCODEMULTISIG_LINK
    } else {
        SAFEMULTISIG_LINK
    };

    let tvc_bytes = load_file_with_url(target, config.timeout as u64).await?;
    let abi = load_abi(MSIG_ABI, config).await?;

    let keys = load_keypair(&keys)?;

    let owners_string = if let Some(owners) = matches.value_of("OWNERS") {
        owners.replace('[', "")
            .replace(']', "")
            .replace('\"', "")
            .replace('\'', "")
            .replace("0x", "")
            .split(',')
            .map(|o|
                format!("\"0x{}\"", o)
            )
            .collect::<Vec<String>>()
            .join(",")
    } else {
        format!(r#""0x{}""#, keys.public.clone())
    };
    let param_str = format!(r#"{{"owners":[{}],"reqConfirms":{}}}"#,
                            owners_string,
                            matches.value_of("CONFIRMS").unwrap_or("1")
    );

    let (msg, address) = prepare_deploy_message_params(&tvc_bytes, abi, &param_str, Some(keys), config.wc).await?;

    if !config.is_json {
        println!("Wallet address: {}", address);
    }

    let ton = create_client_verbose(&config)?;

    if let Some(value) = matches.value_of("VALUE") {
        let params = format!(r#"{{"dest":"{}","amount":"{}"}}"#, address, value);
        call::call_contract_with_client(
            ton.clone(),
            config,
            LOCAL_GIVER_ADDR,
            LOCAL_GIVER_TRANSFER,
            "sendGrams",
            &params,
            None,
            false,
            None,
        ).await?;
    }

    let res = call::process_message(ton.clone(), msg, config).await
        .map_err(|e| format!("{:#}", e));

    if res.is_err() {
        if res.clone().err().unwrap().contains("Account does not exist.") {
            if !config.is_json {
                println!("Your account should have initial balance for deployment. Please transfer some value to your wallet address before deploy.");
            } else {
                println!("{{");
                println!("  \"Error\": \"Your account should have initial balance for deployment. Please transfer some value to your wallet address before deploy.\",");
                println!("  \"Address\": \"{}\"", address);
                println!("}}");
            }
            return Ok(());
        }
        return Err(res.err().unwrap());
    }

    if !config.is_json {
        println!("Wallet successfully deployed");
        println!("Wallet address: {}", address);
    } else {
        println!("{{");
        println!("  \"Address\": \"{}\"", address);
        println!("}}");
    }

    Ok(())
}
