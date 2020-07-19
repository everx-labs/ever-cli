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
use crate::config::Config;
use crate::call;
use crate::multisig::{encode_transfer_body, MSIG_ABI, TRANSFER_WITH_COMMENT};
use serde_json;
use ton_client_rs::TonClient;

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
			lifetime,
			None,
		)
	} else {

		call::call_contract(
			conf,
			addr,
			MSIG_ABI.to_string(),
			"submitTransaction",
			&params,
			keys,
			false,
			None
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
			lifetime,
			None,
		)
	} else {
		call::call_contract(
			conf,
			addr,
			MSIG_ABI.to_string(),
			"confirmTransaction",
			&params,
			keys,
			false,
			None
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
		true,
		None
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
				TRANSFER_WITH_COMMENT.into(),
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