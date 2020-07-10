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
use crate::crypto::{SdkClient};
use crate::call;
use crate::config::Config;
use crate::convert;
use clap::{App, ArgMatches, SubCommand, Arg, AppSettings};
use serde_json;

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

pub fn create_multisig_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("multisig")
        .about("Multisignature wallet commands.")        
        .setting(AppSettings::AllowNegativeNumbers)
        .setting(AppSettings::DontCollapseArgsInUsage)
        .subcommand(SubCommand::with_name("send")
            .setting(AppSettings::AllowLeadingHyphen)
            .about("Transfer funds from multisignature wallet to recepient.")
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
                .help("Amount of funds to transfer."))
            .arg(Arg::with_name("PURPOSE")
                .long("--purpose")
                .takes_value(true)
                .help("Purpose of payment."))
            .arg(Arg::with_name("SIGN")
                .long("--sign")
                .takes_value(true)
                .help("Path to keys or seed phrase.")))
}

pub fn multisig_command(m: &ArgMatches, config: Config) -> Result<(), String> {
    if let Some(m) = m.subcommand_matches("send") {
        return multisig_send_command(m, config);
    }
    Err("unknown multisig command".to_owned())
}

fn multisig_send_command(matches: &ArgMatches, config: Config) -> Result<(), String> {
    let address = matches.value_of("ADDRESS")
        .ok_or(format!("--addr parameter is not defined"))?;
    let dest = matches.value_of("DEST")
        .ok_or(format!("--dst parameter is not defined"))?;
    let keys = matches.value_of("SIGN")
        .ok_or(format!("--sign parameter is not defined"))?;
    let value = matches.value_of("VALUE")
        .ok_or(format!("--value parameter is not defined"))?;
    let comment = matches.value_of("PURPOSE");

    send(config, address, dest, value, keys, comment)
}

pub fn encode_transfer_body(text: &str) -> Result<String, String> {
	let text = hex::encode(text.as_bytes());
	let client = SdkClient::new();
	let abi: serde_json::Value = serde_json::from_str(TRANSFER_WITH_COMMENT).unwrap();
    client.request(
        "contracts.run.body",
        json!({
            "abi": abi,
            "function": "transfer",
            "params": json!({
				"comment": text
			}),
			"internal": true,
        })
    )
}

fn send(
    conf: Config,
    addr: &str,
    dest: &str,
    value: &str,
    keys: &str,
    comment: Option<&str>
) -> Result<(), String> {
    let body = if let Some(text) = comment {
        let msg_body: serde_json::Value = 
            serde_json::from_str(&encode_transfer_body(text)?)
                .map_err(|e| format!("failed to encode comment: {}", e))?;

        msg_body.get("bodyBase64")
            .ok_or(format!(r#"internal error: "bodyBase64" not found in sdk call result"#))?
            .as_str()
            .ok_or(format!(r#"internal error: "bodyBase64" field is not a string"#))?
            .to_owned()
    } else {
        "".to_owned()
    };
    
    let params = json!({
        "dest": dest,
        "value": convert::convert_token(value)?,
        "bounce": true,
        "allBalance": false,
        "payload": body,
    }).to_string();

    call::call_contract(
        conf,
        addr,
        MSIG_ABI.to_string(),
        "submitTransaction",
        &params,
        Some(keys.to_owned()),
        false
    )
}