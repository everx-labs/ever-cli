use assert_cmd::Command;
use predicates::prelude::*;
// uncomment for debug
// use std::io::Write;
mod common;
use common::{BIN_NAME, giver, grep_address};

fn get_debot_paths(name: &str) -> (String, String, String) {
    (
        format!("tests/samples/{}.tvc", name),
        format!("tests/samples/{}.abi.json", name),
        format!("tests/{}.keys.json", name),
    )
}

fn deploy_debot(name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let (tvc, abi, keys) = get_debot_paths(name);
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    let out = cmd
        .arg("genaddr")
        .arg("--genkey")
        .arg(&keys)
        .arg(&tvc)
        .arg(&abi)
        .output()
        .expect("Failed to generate address.");
    let addr = grep_address(&out.stdout);
    giver(&addr);

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("deploy")
        .arg(&tvc)
        .arg("{}")
        .arg("--abi")
        .arg(&abi)
        .arg("--sign")
        .arg(&keys);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(&addr))
        .stdout(predicate::str::contains("Transaction succeeded."));

    let abi_string = std::fs::read_to_string(&abi).unwrap();
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("call")
        .arg("--abi")
        .arg(&abi)
        .arg("--sign")
        .arg(&keys)
        .arg(&addr)
        .arg("setABI")
        .arg(format!(r#"{{"dabi":"{}"}}"#, hex::encode(abi_string)));
    cmd.assert()
        .success();

    Ok(addr)
}

#[test]
fn test_signing_box_interface() -> Result<(), Box<dyn std::error::Error>> {
    let addr = deploy_debot("sample1")?;
    let (_, _, keys) = get_debot_paths("sample1");

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.timeout(std::time::Duration::from_secs(2))
        .write_stdin(format!("y\n{}", keys))
        .arg("debot")
        .arg("fetch")
        .arg(&addr);
    let _cmd = cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("Enter my signing keys:"))
        .stdout(predicate::str::contains("Signing Box Handle:"))
        .stdout(predicate::str::contains("test sign hash passed"));
    // uncomment for debug
    // let out = cmd.get_output();
    // std::io::stdout().lock().write_all(&out.stdout)?;
    Ok(())
}

#[test]
fn test_userinfo() -> Result<(), Box<dyn std::error::Error>> {
    let addr = deploy_debot("sample2")?;
    let (_, abi, keys) = get_debot_paths("sample2");
    let wallet = format!("0:{:064}", 1);
    let key = format!("0x{:064}", 2);
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--wallet")
        .arg(&wallet)
        .arg("--pubkey")
        .arg(&key);
    cmd.assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("call")
        .arg("--abi")
        .arg(abi)
        .arg("--sign")
        .arg(keys)
        .arg(&addr)
        .arg("setParams")
        .arg(format!(r#"{{"wallet":"{}","key":"{}"}}"#, wallet, key));
    cmd.assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.timeout(std::time::Duration::from_secs(2))
        .write_stdin(format!("y\n"))
        .arg("debot")
        .arg("start")
        .arg(&addr);
    let _cmd = cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("Account is valid"))
        .stdout(predicate::str::contains("Public key is valid"));
    Ok(())
}
