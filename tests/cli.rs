use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

const BIN_NAME: &str = "tonos-cli";

#[test]
fn test_config() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--url")
        .arg("http://0.0.0.0")
        .arg("--retries")
        .arg("10")
        .arg("--timeout")
        .arg("25000")
        .arg("--wc")
        .arg("-2");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Succeeded"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--list");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#""url": "http://0.0.0.0""#))
        .stdout(predicate::str::contains(r#""retries": 10"#))
        .stdout(predicate::str::contains(r#""timeout": 25000"#))
        .stdout(predicate::str::contains(r#""wc": -2"#));
    
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--wc")
        .arg("0")
        .assert()
        .success();
    Ok(())
}

#[test]
fn test_call_giver() -> Result<(), Box<dyn std::error::Error>> {
    let giver_abi_name = "tests/samples/giver.abi.json";
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("call")
        .arg("--abi")
        .arg(giver_abi_name)
        .arg("0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94")
        .arg("sendGrams")
        .arg(r#"{"dest":"0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94","amount":1000000000}"#);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Succeeded"));

    Ok(())
}

#[test]
fn test_genaddr_genkey() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("genaddr")
        .arg("tests/samples/wallet.tvc")
        .arg("tests/samples/wallet.abi.json")
        .arg("--genkey")
        .arg("tests/samples/wallet.keys.json");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Raw address"))
        .stdout(predicate::str::contains("Seed phrase"))
        .stdout(predicate::str::contains("Succeeded"));
    Ok(())
}

#[test]
fn test_genaddr() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("genaddr")
        .arg("tests/samples/wallet.tvc")
        .arg("tests/samples/wallet.abi.json")
        .arg("--verbose");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Input arguments:"))
        .stdout(predicate::str::contains("tvc:"))
        .stdout(predicate::str::contains("wc:"))
        .stdout(predicate::str::contains("keys:"))
        .stdout(predicate::str::contains("Raw address"))
        .stdout(predicate::str::contains("Seed phrase"))
        .stdout(predicate::str::contains("Succeeded"));
    Ok(())
}

#[test]
fn test_genaddr_setkey() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("genaddr")
        .arg("tests/samples/wallet.tvc")
        .arg("tests/samples/wallet.abi.json")
        .arg("--setkey")
        .arg("tests/samples/wallet.keys.json");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Raw address"))
        .stdout(predicate::str::contains("Succeeded"));
    Ok(())
}

#[test]
fn test_genaddr_wc() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("genaddr")
        .arg("tests/samples/wallet.tvc")
        .arg("tests/samples/wallet.abi.json")
        .arg("--wc")
        .arg("-1");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Raw address: -1"))
        .stdout(predicate::str::contains("Succeeded"));
    Ok(())
}

#[test]
fn test_genaddr_initdata() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("genaddr")
        .arg("tests/data.tvc")
        .arg("tests/data.abi.json")
        .arg("--genkey")
        .arg("key1")
        .arg("--save")
        .arg("--data")
        .arg(r#"{"m_id":"0x2e0d054dfe43198d971c0f8eaa5f98ca8d08928ecb48a362a900997faecff2e5"}"#);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("TVC file updated"))
        .stdout(predicate::str::contains("Succeeded"));

    Ok(())
}

#[test]
fn test_getkeypair() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("getkeypair")
        .arg("tests/samples/tmp.json")
        .arg("ghost frost pool buzz rival mad naive rare shell tooth smart praise");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Succeeded."));

    let checked_keys = std::fs::read_to_string("tests/samples/tmp.json").unwrap();
    let expected_keys = std::fs::read_to_string("tests/samples/exp.json").unwrap();
    assert_eq!(expected_keys, checked_keys);

    Ok(())
}

#[test]
fn test_deploy() -> Result<(), Box<dyn std::error::Error>> {
    let giver_abi_name = "tests/samples/giver.abi.json";
    let precalculated_addr = "0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e";
    let seed_phrase = "blanket time net universe ketchup maid way poem scatter blur limit drill";
    
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("call")
        .arg("--abi")
        .arg(giver_abi_name)
        .arg("0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94")
        .arg("sendGrams")
        // use precalculated wallet address
        .arg(r#"{"dest":"0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e","amount":1000000000}"#);
    cmd.assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("deploy")
        .arg("tests/samples/wallet.tvc")
        .arg("{}")
        .arg("--abi")
        .arg("tests/samples/wallet.abi.json")
        .arg("--sign")
        .arg(seed_phrase);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(precalculated_addr))
        .stdout(predicate::str::contains("Transaction succeeded."));

    Ok(())
}