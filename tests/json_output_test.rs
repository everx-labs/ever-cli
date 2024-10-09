use assert_cmd::Command;
use serde_json::Value;
use std::fs;

mod common;
use common::create::{
    generate_key_and_address, generate_phrase_and_key, giver_v2, BIN_NAME, GIVER_ABI, GIVER_ADDR,
    GIVER_V2_ADDR, GIVER_V2_KEY, NETWORK,
};

const DEPOOL_ABI: &str = "tests/samples/fakeDepool.abi.json";
const DEPOOL_TVC: &str = "tests/samples/fakeDepool.tvc";

fn run_command_and_decode_json(command: &str) -> Result<(), Box<dyn std::error::Error>> {
    _run_command_and_decode_json(command, true)
}

fn _run_command_and_decode_json(
    command: &str,
    local_node: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    println!("Command: {}", command);
    cmd.arg("-j");
    if local_node {
        cmd.arg("--url").arg(&*NETWORK);
    }
    let out = cmd
        .args(command.split(' ').collect::<Vec<&str>>())
        .output()
        .expect("Failed to execute command.");

    println!("OUT: {:?}", out);

    let out = core::str::from_utf8(&out.stdout)
        .map_err(|e| format!("Failed to decode command output: {}", e))?;

    println!("OUT: {}", out);
    let _: Value =
        serde_json::from_str(out).map_err(|e| format!("Failed to decode output as json: {}", e))?;
    Ok(())
}

#[test]
fn test_json_output_1() -> Result<(), Box<dyn std::error::Error>> {
    run_command_and_decode_json(&format!("account {}", GIVER_V2_ADDR))?;
    run_command_and_decode_json(&format!(
        "body --abi {} addOrdinaryStake {{\"stake\":65535}}",
        DEPOOL_ABI
    ))?;
    run_command_and_decode_json(&format!(
        "call {} sendGrams {{\"dest\":\"{}\",\"amount\":1111111}} --abi {}",
        GIVER_ADDR, GIVER_ADDR, GIVER_ABI
    ))?;
    run_command_and_decode_json(&format!(
        "callx --addr {} sendGrams --dest {} --amount 1111111 --abi {}",
        GIVER_ADDR, GIVER_ADDR, GIVER_ABI
    ))?;
    run_command_and_decode_json(r#"config endpoint add randomurl randomendpoint"#)?;
    run_command_and_decode_json(r#"config endpoint print"#)?;
    run_command_and_decode_json(r#"config endpoint remove randomurl"#)?;
    run_command_and_decode_json(r#"config endpoint reset"#)?;
    Ok(())
}

#[test]
fn test_json_output_2() -> Result<(), Box<dyn std::error::Error>> {
    let key_path = "json_test.key";
    let depool_addr = generate_key_and_address(key_path, DEPOOL_TVC, DEPOOL_ABI)?;
    giver_v2(&depool_addr);

    run_command_and_decode_json(
        r#"decode msg tests/samples/wallet.boc --abi tests/samples/wallet.abi.json"#,
    )?;
    run_command_and_decode_json(
        r#"decode body te6ccgEBAQEARAAAgwAAALqUCTqWL8OX7JivfJrAAzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMQAAAAAAAAAAAAAAAEeGjADA== --abi tests/samples/wallet.abi.json"#,
    )?;
    run_command_and_decode_json(&format!("decode stateinit {}", GIVER_ADDR))?;
    run_command_and_decode_json(
        r#"decode account data --abi tests/test_abi_v2.1.abi.json --tvc tests/decode_fields.tvc"#,
    )?;
    run_command_and_decode_json(r#"decode account boc tests/account.boc"#)?;
    run_command_and_decode_json(&format!(
        "fee deploy --abi {} --sign {} {} {{}}",
        DEPOOL_ABI, key_path, DEPOOL_TVC
    ))?;
    run_command_and_decode_json(&format!(
        "deploy --abi {} --sign {} {} {{}}",
        DEPOOL_ABI, key_path, DEPOOL_TVC
    ))?;

    let depool_addr = generate_key_and_address(key_path, DEPOOL_TVC, DEPOOL_ABI)?;
    giver_v2(&depool_addr);
    run_command_and_decode_json(&format!(
        "deployx --abi {} --keys {} {}",
        DEPOOL_ABI, key_path, DEPOOL_TVC
    ))?;
    run_command_and_decode_json(&format!(
        "deploy_message --raw --abi {} -o fakeDepool.msg --sign {} {} {{}}",
        DEPOOL_ABI, key_path, DEPOOL_TVC
    ))?;
    run_command_and_decode_json(
        r#"dump account 841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94"#,
    )?;
    let command = format!("run --abi {} {} getData {{}}", DEPOOL_ABI, depool_addr);
    run_command_and_decode_json(&command)?;
    let command = format!("runx --abi {} --addr {} getData ", DEPOOL_ABI, depool_addr);
    run_command_and_decode_json(&command)?;

    fs::remove_file(key_path)?;
    fs::remove_file("fakeDepool.msg")?;
    Ok(())
}

#[test]
fn test_json_output_3() -> Result<(), Box<dyn std::error::Error>> {
    run_command_and_decode_json("--config 123.conf config clear")?;
    _run_command_and_decode_json(
        r#"-c 123.conf config --project_id b2ad82504ee54fccb5bc6db8cbb3df1e --access_key 27377cc9027d4de792f100eb869e18e8"#,
        false,
    )?;
    _run_command_and_decode_json(
        r#"-c 123.conf --url net.ton.dev dump config conf.boc"#,
        false,
    )?;
    fs::remove_file("conf.boc")?;
    _run_command_and_decode_json(r#"-c 123.conf --url net.ton.dev getconfig 1"#, false)?;
    _run_command_and_decode_json(r#"-c 123.conf --url net.ton.dev getconfig"#, false)?;
    fs::remove_file("123.conf")?;
    run_command_and_decode_json(&format!("fee storage {}", GIVER_ADDR))?;
    run_command_and_decode_json(&format!(
        "fee call {} sendGrams {{\"dest\":\"{}\",\"amount\":1111111}} --abi {}",
        GIVER_ADDR, GIVER_ADDR, GIVER_ABI
    ))?;
    run_command_and_decode_json(
        r#"genaddr tests/samples/wallet.tvc --genkey tests/json_test1.key"#,
    )?;
    fs::remove_file("tests/json_test1.key")?;
    run_command_and_decode_json(r#"genphrase"#)?;
    // run_command_and_decode_json(r#"genpubkey "jar denial ozone coil heart tattoo science stay wire about act equip""#)?;
    let key_path = "json3_test.key";
    let _ = generate_phrase_and_key(key_path)?;
    run_command_and_decode_json(
        r#"message 0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94 sendTransaction {"dest":"0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94","value":1000000000,"bounce":true} --abi tests/samples/wallet.abi.json --raw --output fakeDepool1.msg"#,
    )?;
    run_command_and_decode_json(
        r#"message 0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94 sendTransaction {"dest":"0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94","value":1000000000,"bounce":true} --abi tests/samples/wallet.abi.json --raw"#,
    )?;
    run_command_and_decode_json(&format!("multisig deploy -k {} -l 1000000000", key_path))?;
    run_command_and_decode_json(
        r#"nodeid --pubkey cde8fbf86c44e4ed2095f83b6f3c97b7aec55a77e06e843f8b9ffeab66ad4b32"#,
    )?;
    run_command_and_decode_json(r#"nodeid --keypair tests/samples/exp.json"#)?;
    run_command_and_decode_json(&format!(
        "proposal vote 0:28a3738f08f5b3410e92aab20f702d64160e2891aaaed881f27d59ff518078d1 12313 {}",
        key_path
    ))?;
    run_command_and_decode_json(r#"runget --boc tests/account_fift.boc past_election_ids"#)?;
    run_command_and_decode_json(r#"sendfile fakeDepool1.msg"#)?;
    run_command_and_decode_json(&format!("debug call {} sendGrams {{\"dest\":\"{}\",\"amount\":1111111}} --abi {} -c tests/config.boc", GIVER_ADDR, GIVER_ADDR, GIVER_ABI))?;
    run_command_and_decode_json("convert tokens 0.123456789")?;
    run_command_and_decode_json("version")?;
    fs::remove_file(key_path)?;
    fs::remove_file("fakeDepool1.msg")?;
    Ok(())
}

#[test]
fn test_json_output_4() -> Result<(), Box<dyn std::error::Error>> {
    run_command_and_decode_json(
        "account 841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a23",
    )?;
    run_command_and_decode_json(&format!(
        "body --abi {} addOrdinaryStake {{\"stake1\":65535}}",
        DEPOOL_ABI
    ))?;
    run_command_and_decode_json("convert tokens 0.12345678a")?;
    run_command_and_decode_json(&format!("call 0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a95 sendGrams {{\"dest\":\"0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94\",\"amount\":1111111}} --abi {}", GIVER_ABI))?;
    run_command_and_decode_json(&format!("callx --addr 0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a95 sendGrams --dest 0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94 --amount 1111111 --abi {}", GIVER_ABI))?;
    run_command_and_decode_json(r#"config endpoint remove random"#)?;
    run_command_and_decode_json(&format!(
        "decode msg tests/samples/wallet.boc --abi {}",
        DEPOOL_ABI
    ))?;
    run_command_and_decode_json(&format!(
        "decode msg tests/account_fift.tvc --abi {}",
        DEPOOL_ABI
    ))?;
    run_command_and_decode_json(&format!("decode body te6ccgEBAQEARAAAgwAAALqUCTqWL8OX7JivfJrAAzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMQAAAAAAAAAAAAAAAEeGjADA== --abi {}", DEPOOL_ABI))?;
    run_command_and_decode_json(
        r#"decode stateinit 841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a65"#,
    )?;
    run_command_and_decode_json(
        r#"decode account data --abi tests/test_abi_v2.1.abi.json --tvc tests/account_fift.tvc"#,
    )?;
    run_command_and_decode_json(r#"decode account boc tests/account_fift.tvc"#)?;
    run_command_and_decode_json(r#"sendfile tests/account.boc"#)?;

    Ok(())
}

#[test]
fn test_json_output_5() -> Result<(), Box<dyn std::error::Error>> {
    run_command_and_decode_json(&format!("fee deploy --abi tests/samples/fakeDepool.abi.json --sign {} tests/samples/fakeDepool.tvc {{}}", GIVER_V2_KEY))?;
    run_command_and_decode_json(&format!("deploy --abi tests/samples/fakeDepool.abi.json --sign {} tests/samples/fakeDepool.tvc {{}}", GIVER_V2_KEY))?;
    run_command_and_decode_json(&format!(
        "deployx --abi tests/samples/fakeDepool.abi.json --keys {} tests/samples/fakeDepool.tvc ",
        GIVER_V2_KEY
    ))?;
    run_command_and_decode_json(&format!("deploy_message --raw --abi tests/samples/fakeDepool.abi.json --sign {} tests/account.boc {{}}", GIVER_V2_KEY))?;
    run_command_and_decode_json(
        r#"dump config 841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94.boc"#,
    )?;
    run_command_and_decode_json(
        r#"fee storage 0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a11"#,
    )?;
    run_command_and_decode_json(
        r#"fee call 0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a95 sendGrams {"dest":"0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94","amount":1111111} --abi tests/samples/giver.abi.json"#,
    )?;
    run_command_and_decode_json(&format!(
        "genaddr tests/account.boc --abi tests/samples/wallet.abi.json --setkey {}",
        GIVER_V2_KEY
    ))?;
    // run_command_and_decode_json(r#"genpubkey "jar denial ozone coil heart tattoo science stay wire about act""#)?;
    run_command_and_decode_json(r#"getconfig 1"#)?;
    run_command_and_decode_json(
        r#"message 0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94 sendansaction {"dest":"0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94","value":1000000000,"bounce":true} --abi tests/samples/wallet.abi.json --raw"#,
    )?;
    run_command_and_decode_json(&format!("multisig deploy -k {}", GIVER_V2_KEY))?;
    run_command_and_decode_json(&format!(
        "multisig deploy -k {} -l 1000000000",
        GIVER_V2_KEY
    ))?;
    run_command_and_decode_json(r#"nodeid --pubkey cde8fbf86c"#)?;
    run_command_and_decode_json(r#"nodeid --keypair tests/account.boc"#)?;
    // run_command_and_decode_json(&format!("proposal vote 0:28a3738f08f5b3410e92aab20f702d64160e2891aaaed881f27d59ff518078d1 12313 {}", GIVER_V2_KEY))?;
    // run_command_and_decode_json(r#"proposal decode 0:28a3738f08f5b3410e92aab20f702d64160e2891aaaed881f27d59ff518078d1 12313"#)?;
    let command = format!(
        "run --abi tests/samples/fakeDepool.abi.json {} getDat {{}}",
        GIVER_ADDR
    );
    run_command_and_decode_json(&command)?;
    let command = format!(
        "runx --abi tests/samples/fakeDepool.abi.json --addr {} gtData ",
        GIVER_ADDR
    );
    run_command_and_decode_json(&command)?;
    run_command_and_decode_json(r#"runget --boc tests/account_fift.boc past_election_is"#)?;
    run_command_and_decode_json(r#"send --abi tests/samples/fakeDepool.abi.json 65465"#)?;
    run_command_and_decode_json(
        r#"debug call 0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94 sendGram1 '{"dest":"0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94","amount":1111111}' --abi tests/samples/giver.abi.json -c tests/config.boc"#,
    )?;
    run_command_and_decode_json(
        r#" --optionss call 0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94 sendGram '{"dest":"0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94","amount":1111111}' --abi tests/samples/giver.abi.json"#,
    )?;
    Ok(())
}
