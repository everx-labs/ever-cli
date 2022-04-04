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
use crate::config::Config;
use crate::{call};
use crate::helpers::{create_client_local, decode_msg_body};
use crate::multisig::{encode_transfer_body, MSIG_ABI, TRANSFER_WITH_COMMENT};

pub async fn create_proposal(
	conf: Config,
	addr: &str,
	keys: Option<&str>,
	dest: &str,
	text: &str,
	lifetime: u32,
	offline: bool,
) -> Result<(), String> {

	let payload = encode_transfer_body(text).await?;

	let params = json!({
		"dest": dest,
		"value": 1000000,
		"bounce": false,
		"allBalance": false,
		"payload": payload,
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
			false,
			None).await
	} else {

		call::call_contract(
			conf,
			addr,
			MSIG_ABI.to_string(),
			"submitTransaction",
			&params,
			keys,
			false,
			false,
		).await
	}
}

pub async fn vote(
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
			false,
			None,
		).await
	} else {
		call::call_contract(
			conf,
			addr,
			MSIG_ABI.to_string(),
			"confirmTransaction",
			&params,
			keys,
			false,
			false,
		).await
	}
}

pub async fn decode_proposal(
	conf: Config,
	addr: &str,
	proposal_id: &str,
) -> Result<(), String> {

	let result = call::call_contract_with_result(
		conf.clone(),
		addr,
		MSIG_ABI.to_string(),
		"getTransactions",
		"{}",
		None,
		true,
		false,
	).await?;

	let txns = result["transactions"].as_array()
		.ok_or(r#"failed to decode result: "transactions" array not found"#.to_string())?;

	for txn in txns {
		let txn_id = txn["id"].as_str()
			.ok_or(r#"failed to parse transaction in list: "id" not found"#.to_string())?;

		if txn_id == proposal_id {
			let body = txn["payload"].as_str()
				.ok_or(r#"failed to parse transaction in list: "payload" not found"#.to_string())?;
			let ton = create_client_local()?;
			let result = decode_msg_body(
				ton.clone(),
				TRANSFER_WITH_COMMENT,
				body,
				true,
			)
            .await
            .map_err(|e| format!("failed to decode proposal payload: {}", e))?;

			let comment = String::from_utf8(
				hex::decode(
					result.value.ok_or("failed to get result value")?["comment"].as_str()
						.ok_or("failed to obtain result comment")?
				).map_err(|e| format!("failed to parse comment from transaction payload: {}", e))?
			).map_err(|e| format!("failed to convert comment to string: {}", e))?;

			if !conf.is_json {
				println!("Comment: {}", comment);
			} else {
				println!("{{");
				println!("  \"Comment\": \"{}\"", comment);
				println!("}}");
			}
			return Ok(());
		}
	}
	if !conf.is_json {
		println!("Proposal with id {} not found", proposal_id);
	} else {
		println!("{{");
		println!("  \"Error\": \"Proposal with id {} not found\"", proposal_id);
		println!("}}");
	}
	Ok(())
}