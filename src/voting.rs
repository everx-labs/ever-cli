use crate::crypto::{SdkClient};
use crate::config::Config;
use crate::call;
use serde_json;
use ton_client_rs::{
    TonClient
};

const MSIG_ABI: &str = r#"{
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

const TRANSFER_WITH_COMMENT: &str = r#"{
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

fn encode_transfer_body(text: &str) -> Result<String, String> {
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

pub fn create_proposal(
	conf: Config,
	addr: &str,
	keys: Option<&str>,
	dest: &str,
	text: &str,
	lifetime: u32,
	offline: bool,
) -> Result<(), String> {

	let msg_body: serde_json::Value = serde_json::from_str(
		&encode_transfer_body(text)?
	).map_err(|e| format!("failed to encode body for proposal: {}", e))?;
	
	let body_base64 = msg_body.get("bodyBase64")
		.ok_or(format!(r#"internal error: "bodyBase64" not found in sdk call result"#))?
		.as_str()
		.ok_or(format!(r#"internal error: "bodyBase64" field is not a string"#))?;

	let params = json!({
		"dest": dest,
		"value": 1000000,
		"bounce": false,
		"allBalance": false,
		"payload": body_base64,
	}).to_string();

	let keys = keys.map(|s| s.to_owned());

	if offline {
		call::generate_message(
			conf,
			addr,
			MSIG_ABI.to_string(),
			"submitTransaction",
			&params,
			keys,
			lifetime)
	} else {

		call::call_contract(
			conf,
			addr,
			MSIG_ABI.to_string(),
			"submitTransaction",
			&params,
			keys,
			false
		)
	}
}

pub fn vote(
	conf: Config,
	addr: &str,
	keys: Option<&str>,
	trid: &str,
	lifetime: u32,
	offline: bool,
) -> Result<(), String> {

	let params = json!({
		"transactionId": trid,
	}).to_string();

	let keys = keys.map(|s| s.to_owned());

	if offline {
		call::generate_message(
			conf,
			addr,
			MSIG_ABI.to_string(),
			"confirmTransaction",
			&params,
			keys,
			lifetime
		)
	} else {
		call::call_contract(
			conf,
			addr,
			MSIG_ABI.to_string(),
			"confirmTransaction",
			&params,
			keys,
			false
		)
	}
}

pub fn decode_proposal(
	conf: Config,
	addr: &str,
	proposal_id: &str,
) -> Result<(), String> {

	let result = call::call_contract_with_result(
		conf,
		addr,
		MSIG_ABI.to_string(),
		"getTransactions",
		"{}",
		None,
		true
	)?;

	let txns = result["transactions"].as_array()
		.ok_or(format!(r#"failed to decode result: "transactions" array not found"#))?;

	for txn in txns {
		let txn_id = txn["id"].as_str()
			.ok_or(format!(r#"failed to parse transaction in list: "id" not found"#))?;

		if txn_id == proposal_id {
			let body = txn["payload"].as_str()
				.ok_or(format!(r#"failed to parse transaction in list: "payload" not found"#))?;
			let ton = TonClient::default()
				.map_err(|e| format!("failed to create tonclient: {}", e.to_string()))?;

			let result = ton.contracts.decode_input_message_body(
				TRANSFER_WITH_COMMENT,
				&base64::decode(&body).unwrap(),
				true,
			).map_err(|e| format!("failed to decode proposal payload: {}", e))?;
				
			let comment = String::from_utf8(
				hex::decode(
					result.output["comment"].as_str().unwrap()
				).map_err(|e| format!("failed to parse comment from transaction payload: {}", e))?
			).map_err(|e| format!("failed to convert comment to string: {}", e))?;
	
			println!("Comment: {}", comment);
			return Ok(());
		}
	}
	println!("Proposal with id {} not found", proposal_id);
	Ok(())
}