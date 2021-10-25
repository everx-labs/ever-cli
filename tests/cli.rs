use predicates::prelude::*;
use assert_cmd::Command;
use std::env;
use std::time::Duration;
use std::thread::sleep;
use std::fs;
mod common;
use common::{BIN_NAME, NETWORK, get_config};

fn now_ms() -> u64 {
    chrono::prelude::Utc::now().timestamp_millis() as u64
}

fn generate_phrase_and_key(key_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    let out = cmd.arg("genphrase")
        .output()
        .expect("Failed to generate a seed phrase.");
    let mut seed = String::from_utf8_lossy(&out.stdout).to_string();
    seed.replace_range(..seed.find('"').unwrap_or(0), "");
    seed.retain(|c| c != '\n' && c != '"');

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("getkeypair")
        .arg(key_path)
        .arg(seed.clone())
        .assert()
        .success();

    Ok(seed)
}

fn generate_public_key(seed: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    let out = cmd.arg("genpubkey")
        .arg(seed)
        .output()
        .expect("Failed to generate a public key phrase.");
    let mut key = String::from_utf8_lossy(&out.stdout).to_string();
    key.replace_range(..key.find("Public key: ").unwrap_or(0) + "Public key: ".len(), "");
    key.replace_range(key.find("\n\n").unwrap_or(key.len())-1.., "");

    Ok(key)
}

fn generate_key_and_address(
    key_path: &str,
    tvc_path: &str,
    abi_path: &str
) -> Result<String, Box<dyn std::error::Error>> {
    generate_phrase_and_key(key_path)?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    let out = cmd.arg("genaddr")
        .arg("--setkey")
        .arg(key_path)
        .arg(tvc_path)
        .arg(abi_path)
        .output()
        .expect("Failed to generate address.");

    let mut addr = String::from_utf8_lossy(&out.stdout).to_string();
    addr.replace_range(..addr.find("0:").unwrap_or(0), "");
    addr.replace_range(addr.find("testnet").unwrap_or(addr.len())-1.., "");

    Ok(addr)
}

fn ask_giver(target: &str, amount: u64) -> Result<(), Box<dyn std::error::Error>> {
    let giver_abi = "tests/samples/giver.abi.json";
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--url")
        .arg(&*NETWORK)
        .assert()
        .success();

    let arg_string = format!(r#"{{"dest":"{}","amount":{}}}"#, target, amount);

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("call")
        .arg("0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94")
        .arg("sendGrams")
        .arg(arg_string)
        .arg("--abi")
        .arg(giver_abi)
        .assert()
        .success()
        .stdout(predicate::str::contains("Succeeded"));

    Ok(())
}


#[test]
fn test_config_1() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--url")
        .arg(&*NETWORK)
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
        .stdout(predicate::str::contains(format!(r#""url": "{}""#, &*NETWORK)))
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
fn test_config_endpoints() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("endpoint")
        .arg("reset");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("main.ton.dev"))
        .stdout(predicate::str::contains("https://main2.ton.dev"))
        .stdout(predicate::str::contains("http://127.0.0.1/"))
        .stdout(predicate::str::contains("net.ton.dev"))
        .stdout(predicate::str::contains("https://net1.ton.dev"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("clear");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Succeeded"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--url")
        .arg("main.ton.dev");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#""url": "main.ton.dev","#))
        .stdout(predicate::str::contains(r#""endpoints": [
    "https://main2.ton.dev",
    "https://main3.ton.dev",
    "https://main4.ton.dev"
  ]"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("endpoint")
        .arg("add")
        .arg("myownhost")
        .arg("[1.1.1.1,my.net.com]");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("main.ton.dev"))
        .stdout(predicate::str::contains("https://main2.ton.dev"))
        .stdout(predicate::str::contains("http://127.0.0.1/"))
        .stdout(predicate::str::contains("net.ton.dev"))
        .stdout(predicate::str::contains("https://net1.ton.dev"))
        .stdout(predicate::str::contains("myownhost"))
        .stdout(predicate::str::contains("1.1.1.1"))
        .stdout(predicate::str::contains("my.net.com"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--url")
        .arg("myownhost");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#""url": "myownhost","#))
        .stdout(predicate::str::contains(r#""endpoints": [
    "1.1.1.1",
    "my.net.com"
  ]"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("endpoint")
        .arg("add")
        .arg("myownhost")
        .arg("[1.1.1.1,my.net.com,tonlabs.net]");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("main.ton.dev"))
        .stdout(predicate::str::contains("https://main2.ton.dev"))
        .stdout(predicate::str::contains("http://127.0.0.1/"))
        .stdout(predicate::str::contains("net.ton.dev"))
        .stdout(predicate::str::contains("https://net1.ton.dev"))
        .stdout(predicate::str::contains("myownhost"))
        .stdout(predicate::str::contains("1.1.1.1"))
        .stdout(predicate::str::contains("my.net.com"))
        .stdout(predicate::str::contains("tonlabs.net"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--url")
        .arg("myownhost");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#""url": "myownhost","#))
        .stdout(predicate::str::contains("1.1.1.1"))
        .stdout(predicate::str::contains("my.net.com"))
        .stdout(predicate::str::contains("tonlabs.net"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("endpoint")
        .arg("remove")
        .arg("main.ton.dev");
    cmd.assert()
        .success()
        .stdout(predicate::function(|s: &str| !s.contains("main.ton.dev")))
        .stdout(predicate::function(|s: &str| !s.contains("https://main2.ton.dev")))
        .stdout(predicate::str::contains("http://127.0.0.1/"))
        .stdout(predicate::str::contains("net.ton.dev"))
        .stdout(predicate::str::contains("https://net1.ton.dev"))
        .stdout(predicate::str::contains("myownhost"))
        .stdout(predicate::str::contains("1.1.1.1"))
        .stdout(predicate::str::contains("my.net.com"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("endpoint")
        .arg("print");
    cmd.assert()
        .success()
        .stdout(predicate::function(|s: &str| !s.contains("main.ton.dev")))
        .stdout(predicate::function(|s: &str| !s.contains("https://main2.ton.dev")))
        .stdout(predicate::str::contains("http://127.0.0.1/"))
        .stdout(predicate::str::contains("net.ton.dev"))
        .stdout(predicate::str::contains("https://net1.ton.dev"))
        .stdout(predicate::str::contains("myownhost"))
        .stdout(predicate::str::contains("1.1.1.1"))
        .stdout(predicate::str::contains("my.net.com"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("endpoint")
        .arg("reset");
    cmd.assert()
        .success()
        .stdout(predicate::function(|s: &str| !s.contains("myownhost")))
        .stdout(predicate::function(|s: &str| !s.contains("my.net.com")))
        .stdout(predicate::str::contains("http://127.0.0.1/"))
        .stdout(predicate::str::contains("net.ton.dev"))
        .stdout(predicate::str::contains("https://net1.ton.dev"))
        .stdout(predicate::str::contains("main.ton.dev"))
        .stdout(predicate::str::contains("https://main2.ton.dev"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--list");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#""url": "myownhost","#))
        .stdout(predicate::str::contains("1.1.1.1"))
        .stdout(predicate::str::contains("my.net.com"))
        .stdout(predicate::str::contains("tonlabs.net"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("clear");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Succeeded"));

    Ok(())
}

#[test]
fn test_call_giver() -> Result<(), Box<dyn std::error::Error>> {
    ask_giver("0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94", 1000000000)
}

#[test]
fn test_fee() -> Result<(), Box<dyn std::error::Error>> {
    let giver_abi_name = "tests/samples/giver.abi.json";
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("fee")
        .arg("storage")
        .arg("0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94")
        .assert()
        .success()
        .stdout(predicate::str::contains("Storage fee per 31536000 seconds: "));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("fee")
        .arg("storage")
        .arg("0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94")
        .arg("--period")
        .arg("10000")
        .assert()
        .success()
        .stdout(predicate::str::contains("Storage fee per 10000 seconds: "));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("fee")
        .arg("call")
        .arg("0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94")
        .arg("sendGrams")
        .arg(r#"{"dest":"0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94","amount":1000000000}"#)
        .arg("--abi")
        .arg(giver_abi_name)
        .assert()
        .success()
        .stdout(predicate::str::contains(r#"  "in_msg_fwd_fee":"#))
        .stdout(predicate::str::contains(r#"  "storage_fee":"#))
        .stdout(predicate::str::contains(r#"  "gas_fee":"#))
        .stdout(predicate::str::contains(r#"  "out_msgs_fwd_fee":"#))
        .stdout(predicate::str::contains(r#"  "total_account_fees":"#))
        .stdout(predicate::str::contains(r#"  "total_output":"#))
        .stdout(predicate::str::contains(r#"Succeeded."#));

    let wallet_tvc = "tests/samples/wallet.tvc";
    let wallet_abi = "tests/samples/wallet.abi.json";
    let key_path = "tests/deploy_test.key";

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("fee")
        .arg("deploy")
        .arg(wallet_tvc)
        .arg("{}")
        .arg("--abi")
        .arg(wallet_abi)
        .arg("--sign")
        .arg(key_path)
        .assert()
        .success()
        .stdout(predicate::str::contains(r#"  "in_msg_fwd_fee":"#))
        .stdout(predicate::str::contains(r#"  "storage_fee":"#))
        .stdout(predicate::str::contains(r#"  "gas_fee":"#))
        .stdout(predicate::str::contains(r#"  "out_msgs_fwd_fee":"#))
        .stdout(predicate::str::contains(r#"  "total_account_fees":"#))
        .stdout(predicate::str::contains(r#"  "total_output":"#));

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
        .arg("tests/samples/wallet.abi.json");
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
    fs::copy("tests/data.tvc", "tests/data2.tvc")?;
    cmd.arg("genaddr")
        .arg("tests/data2.tvc")
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
    fs::remove_file("tests/data2.tvc")?;
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
fn test_async_deploy() -> Result<(), Box<dyn std::error::Error>> {
    let wallet_tvc = "tests/samples/wallet.tvc";
    let wallet_abi = "tests/samples/wallet.abi.json";
    let key_path = "tests/deploy_test.key";

    let addr = generate_key_and_address(key_path, wallet_tvc, wallet_abi)?;

    ask_giver(&addr, 1000000000)?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("account")
        .arg(addr.clone())
        .assert()
        .success()
        .stdout(predicate::str::contains("acc_type:      Uninit"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--async_call")
        .arg("true")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("deploy")
        .arg(wallet_tvc)
        .arg("{}")
        .arg("--abi")
        .arg(wallet_abi)
        .arg("--sign")
        .arg(key_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(addr.clone()));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--async_call")
        .arg("false")
        .assert()
        .success();

    sleep(Duration::new(1, 0));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("account")
        .arg(addr.clone())
        .assert()
        .success()
        .stdout(predicate::str::contains("acc_type:      Active"));

    Ok(())
}

#[test]
fn test_deploy() -> Result<(), Box<dyn std::error::Error>> {
    let wallet_tvc = "tests/samples/wallet.tvc";
    let wallet_abi = "tests/samples/wallet.abi.json";
    let key_path = "tests/deploy_test.key";

    let addr = generate_key_and_address(key_path, wallet_tvc, wallet_abi)?;

    ask_giver(&addr, 1000000000)?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("deploy")
        .arg(wallet_tvc)
        .arg("{}")
        .arg("--abi")
        .arg(wallet_abi)
        .arg("--sign")
        .arg(key_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(addr))
        .stdout(predicate::str::contains("Transaction succeeded."));

    Ok(())
}

#[test]
fn test_genaddr_seed() -> Result<(), Box<dyn std::error::Error>> {
    let msig_abi = "tests/samples/SafeMultisigWallet.abi.json";
    let msig_tvc = "tests/samples/SafeMultisigWallet.tvc";
    let key_path = "tests/deploy_test.key";

    let seed = generate_phrase_and_key(key_path)?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    let out = cmd.arg("genaddr")
        .arg("--setkey")
        .arg(key_path)
        .arg(msig_tvc)
        .arg(msig_abi)
        .output()
        .expect("Failed to generate address.");

    let mut msig_addr = String::from_utf8_lossy(&out.stdout).to_string();
    msig_addr.replace_range(..msig_addr.find("0:").unwrap_or(0), "");
    msig_addr.replace_range(msig_addr.find("testnet").unwrap_or(msig_addr.len())-1.., "");

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    let out = cmd.arg("genaddr")
        .arg("--setkey")
        .arg(seed)
        .arg(msig_tvc)
        .arg(msig_abi)
        .output()
        .expect("Failed to generate address.");

    let mut msig_addr2 = String::from_utf8_lossy(&out.stdout).to_string();
    msig_addr2.replace_range(..msig_addr2.find("0:").unwrap_or(0), "");
    msig_addr2.replace_range(msig_addr2.find("testnet").unwrap_or(msig_addr2.len())-1.., "");

    assert_eq!(msig_addr, msig_addr2);

    Ok(())
}

#[test]
fn test_callex() -> Result<(), Box<dyn std::error::Error>> {
    let giver_abi_name = "tests/samples/giver.abi.json";

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--url")
        .arg(&*NETWORK)
        .assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("callex")
        .arg("sendGrams")
        .arg("0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94")
        .arg(giver_abi_name)
        .arg("--")
        .arg("--dest")
        .arg("0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e")
        .arg("--amount")
        .arg("0.2T");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#""dest":"0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e""#))
        .stdout(predicate::str::contains(r#""amount":"200000000""#))
        .stdout(predicate::str::contains("Succeeded"));


    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("callex")
        .arg("sendGrams")
        .arg("0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94")
        .arg(giver_abi_name)
        .arg("--")
        .arg("--dest")
        .arg("0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e")
        .arg("--amount")
        .arg("1000000000");
    cmd.assert()
        .success();
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#""dest":"0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e""#))
        .stdout(predicate::str::contains(r#""amount":"1000000000""#))
        .stdout(predicate::str::contains("Succeeded"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("callex")
        .arg("sendGrams")
        .arg("0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94")
        .arg(giver_abi_name)
        .arg("--")
        .arg("--dest")
        .arg("0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e")
        .arg("--amount")
        .arg("0x10000");
    cmd.assert()
        .success();
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#""dest":"0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e""#))
        .stdout(predicate::str::contains(r#""amount":"0x10000""#))
        .stdout(predicate::str::contains("Succeeded"));

    Ok(())
}

#[test]
fn test_nodeid() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("nodeid")
        .arg("--pubkey")
        .arg("cde8fbf86c44e4ed2095f83b6f3c97b7aec55a77e06e843f8b9ffeab66ad4b32");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("cdae19f3d5a96d016e74d656ef15e35839b554ae2590bec0dce5e6608cb7f837"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("nodeid")
        .arg("--keypair")
        .arg("tests/samples/exp.json");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("e8c5df53b6205e8db639629d2cd2552b354501021a9f223bb72e81e75f37f64a"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("nodeid")
        .arg("--keypair")
        .arg("ghost frost pool buzz rival mad naive rare shell tooth smart praise");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("e8c5df53b6205e8db639629d2cd2552b354501021a9f223bb72e81e75f37f64a"));

    Ok(())
}

#[test]
fn test_override_config_path() -> Result<(), Box<dyn std::error::Error>> {

    // config from cmd lime
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg("tests/conf1.json")
        .arg("account")
        .arg("0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94");
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("Connecting to https://test.ton.dev"));
    // config from env variable
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.env("TONOSCLI_CONFIG", "./tests/conf2.json")
        .arg("account")
        .arg("0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94");
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("Connecting to https://test2.ton.dev"));

    // config from cmd line has higher priority than env variable
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg("tests/conf1.json")
        .env("TONOSCLI_CONFIG", "./tests/conf2.json")
        .arg("account")
        .arg("0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94");
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("Connecting to https://test.ton.dev"));

    // if there is neither --config nor env variable then config file is searched in current working dir
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("account")
        .arg("0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(format!("Connecting to {}", &*NETWORK)));
    Ok(())
}

#[test]
fn test_sendfile() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--url")
        .arg(&*NETWORK)
        .arg("message")
        .arg("0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94")
        .arg("sendTransaction")
        .arg(r#"{"dest":"0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94","value":1000000000,"bounce":true}"#)
        .arg("--abi")
        .arg("./tests/samples/wallet.abi.json")
        .arg("--raw")
        .arg("--output")
        .arg("call.boc");
    cmd.assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--async_call")
        .arg("true")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--url")
        .arg(&*NETWORK)
        .arg("sendfile")
        .arg("call.boc");
    cmd.assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--async_call")
        .arg("false")
        .assert()
        .success();

    Ok(())
}


#[test]
fn test_account_command() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("RUST_LOG", "debug");

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--url")
        .arg("https://net.ton.dev");
    cmd.assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("account")
        .arg("-1:3333333333333333333333333333333333333333333333333333333333333333");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("acc_type:      Active"))
        .stdout(predicate::str::contains("balance:"))
        .stdout(predicate::str::contains("last_paid:"))
        .stdout(predicate::str::contains("last_trans_lt:"))
        .stdout(predicate::str::contains("data(boc):"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("account")
        .arg("1:3333333333333333333333333333333333333333333333333333333333333333");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Account not found"));
    Ok(())
}


#[test]
fn test_config_wc() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--url")
        .arg("https://net.ton.dev")
        .arg("--wc")
        .arg("-1");
    cmd.assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("account")
        .arg("3333333333333333333333333333333333333333333333333333333333333333");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("acc_type:      Active"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--wc")
        .arg("1");
    cmd.assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("account")
        .arg("3333333333333333333333333333333333333333333333333333333333333333");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Account not found"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--wc")
        .arg("0");
    cmd.assert()
        .success();
    Ok(())
}


#[test]
fn test_account_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--url")
        .arg("https://net.ton.dev");
    cmd.assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("account");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("The following required arguments were not provided:"))
        .stderr(predicate::str::contains("<ADDRESS>"));

    Ok(())
}

#[test]
fn test_decode_msg() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--json")
        .arg("decode")
        .arg("msg")
        .arg("tests/samples/wallet.boc")
        .arg("--abi")
        .arg("tests/samples/wallet.abi.json")
        .assert()
        .success()
        .stdout(predicate::str::contains("sendTransaction"))
        .stdout(predicate::str::contains("dest"))
        .stdout(predicate::str::contains("value"))
        .stdout(predicate::str::contains("bounce"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--json")
        .arg("decode")
        .arg("msg")
        .arg("tests/deploy_msg.boc")
        .arg("--abi")
        .arg("tests/samples/wallet.abi.json")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#"Init": {"#))
        .stdout(predicate::str::contains(r#""StateInit": {"#))
        .stdout(predicate::str::contains(r#""data": "te6ccgEBAgEAKAABAcABAEPQAZ6jzp01QGBqmSPd8SBPy4vE1I8GSisk4ihjvGiRJP7g""#));
    Ok(())
}

#[test]
fn test_decode_body() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--json").arg("decode")
        .arg("body").arg("te6ccgEBAQEARAAAgwAAALqUCTqWL8OX7JivfJrAAzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMQAAAAAAAAAAAAAAAEeGjADA==")
        .arg("--abi").arg("tests/samples/wallet.abi.json");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("sendTransaction"))
        .stdout(predicate::str::contains("dest"))
        .stdout(predicate::str::contains("value"))
        .stdout(predicate::str::contains("bounce"));
    Ok(())
}

#[test]
fn test_decode_body_constructor_for_minus_workchain() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("decode")
        .arg("body").arg("te6ccgEBAgEAkwABW1ByqHsAAAF1QnI+qZ/1tsdEUQb8jxj9vr/H4WuiQwfD5ESNbO4lcz2Kca2KavABAMAQYZcjaCLLbO1phXFWOD/kmlkZ1g7FyjgSIEHRpXeeIDiQ3f7FKVd+oeq6VxVlAti+jigqVmtrn8wmBEgbyT8P+5iyVBuoBWSPJetGndR2b83eA6LP5vtB2MFXHClAfKM=")
        .arg("--abi").arg("tests/samples/Subscription.abi.json");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("constructor: {"))
        .stdout(predicate::str::contains("\"wallet\": \"-1:adb63a228837e478c7edf5fe3f0b5d12183e1f22246b67712b99ec538d6c5357\""));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--json").arg("decode")
        .arg("body").arg("te6ccgEBAgEAkwABW1ByqHsAAAF1QnI+qZ/1tsdEUQb8jxj9vr/H4WuiQwfD5ESNbO4lcz2Kca2KavABAMAQYZcjaCLLbO1phXFWOD/kmlkZ1g7FyjgSIEHRpXeeIDiQ3f7FKVd+oeq6VxVlAti+jigqVmtrn8wmBEgbyT8P+5iyVBuoBWSPJetGndR2b83eA6LP5vtB2MFXHClAfKM=")
        .arg("--abi").arg("tests/samples/Subscription.abi.json");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"BodyCall\":"))
        .stdout(predicate::str::contains("\"constructor\":"))
        .stdout(predicate::str::contains("\"wallet\":"))
        .stdout(predicate::str::contains("-1:adb63a228837e478c7edf5fe3f0b5d12183e1f22246b67712b99ec538d6c5357"));


    //test getting ABI from config

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--abi")
        .arg("tests/samples/Subscription.abi.json");
    cmd.assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("decode")
        .arg("body").arg("te6ccgEBAgEAkwABW1ByqHsAAAF1QnI+qZ/1tsdEUQb8jxj9vr/H4WuiQwfD5ESNbO4lcz2Kca2KavABAMAQYZcjaCLLbO1phXFWOD/kmlkZ1g7FyjgSIEHRpXeeIDiQ3f7FKVd+oeq6VxVlAti+jigqVmtrn8wmBEgbyT8P+5iyVBuoBWSPJetGndR2b83eA6LP5vtB2MFXHClAfKM=");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("constructor: {"))
        .stdout(predicate::str::contains("\"wallet\": \"-1:adb63a228837e478c7edf5fe3f0b5d12183e1f22246b67712b99ec538d6c5357\""));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--json").arg("decode")
        .arg("body").arg("te6ccgEBAgEAkwABW1ByqHsAAAF1QnI+qZ/1tsdEUQb8jxj9vr/H4WuiQwfD5ESNbO4lcz2Kca2KavABAMAQYZcjaCLLbO1phXFWOD/kmlkZ1g7FyjgSIEHRpXeeIDiQ3f7FKVd+oeq6VxVlAti+jigqVmtrn8wmBEgbyT8P+5iyVBuoBWSPJetGndR2b83eA6LP5vtB2MFXHClAfKM=");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"BodyCall\":"))
        .stdout(predicate::str::contains("\"constructor\":"))
        .stdout(predicate::str::contains("\"wallet\":"))
        .stdout(predicate::str::contains("-1:adb63a228837e478c7edf5fe3f0b5d12183e1f22246b67712b99ec538d6c5357"));


    //test that abi in commandline is preferred
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--abi")
        .arg("tests/samples/wallet.abi.json");
    cmd.assert()
        .success();
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--json").arg("decode")
        .arg("body").arg("te6ccgEBAgEAkwABW1ByqHsAAAF1QnI+qZ/1tsdEUQb8jxj9vr/H4WuiQwfD5ESNbO4lcz2Kca2KavABAMAQYZcjaCLLbO1phXFWOD/kmlkZ1g7FyjgSIEHRpXeeIDiQ3f7FKVd+oeq6VxVlAti+jigqVmtrn8wmBEgbyT8P+5iyVBuoBWSPJetGndR2b83eA6LP5vtB2MFXHClAfKM=")
        .arg("--abi").arg("tests/samples/Subscription.abi.json");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"BodyCall\":"))
        .stdout(predicate::str::contains("\"constructor\":"))
        .stdout(predicate::str::contains("\"wallet\":"))
        .stdout(predicate::str::contains("-1:adb63a228837e478c7edf5fe3f0b5d12183e1f22246b67712b99ec538d6c5357"));

    //test error on wrong body
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--json").arg("decode")
        .arg("body").arg("\"123\"")
        .arg("--abi").arg("tests/samples/Subscription.abi.json");
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("body is not a valid base64 string"));

    //test error on empty body
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--json").arg("decode")
        .arg("body").arg("")
        .arg("--abi").arg("tests/samples/Subscription.abi.json");
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("failed to decode body"));

    Ok(())
}


#[test]
fn test_depool_0() -> Result<(), Box<dyn std::error::Error>> {
    let depool_abi = "tests/samples/fakeDepool.abi.json";
    let depool_tvc = "tests/samples/fakeDepool.tvc";
    let msig_abi = "tests/samples/SafeMultisigWallet.abi.json";
    let msig_tvc = "tests/samples/SafeMultisigWallet.tvc";
    let key_path = "tests/deploy_test.key";

    let msig_addr = generate_key_and_address(key_path, msig_tvc, msig_abi)?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    let out = cmd.arg("genaddr")
        .arg("--setkey")
        .arg(key_path)
        .arg(depool_tvc)
        .arg(depool_abi)
        .output()
        .expect("Failed to generate address.");

    let mut depool_addr = String::from_utf8_lossy(&out.stdout).to_string();
    depool_addr.replace_range(..depool_addr.find("0:").unwrap_or(0), "");
    depool_addr.replace_range(depool_addr.find("testnet").unwrap_or(depool_addr.len())-1.., "");

    ask_giver(&depool_addr, 10000000000)?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("deploy")
        .arg(depool_tvc)
        .arg("{}")
        .arg("--abi")
        .arg(depool_abi)
        .arg("--sign")
        .arg(key_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(&depool_addr))
        .stdout(predicate::str::contains("Transaction succeeded."));

    ask_giver(&msig_addr, 30000000000)?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("deploy")
        .arg(msig_tvc)
        .arg(r#"{"owners":["0xc8bd66f90d61f7e1e1a6151a0dbe9d8640666920d8c0cf399cbfb72e089d2e41"],"reqConfirms":1}"#)
        .arg("--abi")
        .arg(msig_abi)
        .arg("--sign")
        .arg(key_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(&msig_addr))
        .stdout(predicate::str::contains("Transaction succeeded."));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--addr")
        .arg(&depool_addr)
        .arg("--wallet")
        .arg(&msig_addr)
        .assert()
        .success();

    Ok(())
}

#[test]
fn test_error() -> Result<(), Box<dyn std::error::Error>> {
    let depool_abi = "tests/samples/fakeDepool.abi.json";

    let config = get_config().unwrap();
    let depool_addr = config["addr"].as_str().unwrap();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("call")
        .arg("--abi")
        .arg(&depool_abi)
        .arg(&depool_addr)
        .arg("error")
        .arg(r#"{"code":101}"#);
    cmd.assert()
        .stdout(predicate::str::contains(r#""exit_code": 101,"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("call")
        .arg("--abi")
        .arg(&depool_abi)
        .arg(&depool_addr)
        .arg("outOfGas")
        .arg(r#"{}"#);
    cmd.assert()
        .stdout(predicate::str::contains(r#""exit_code": -14,"#));

    Ok(())
}

#[test]
fn test_depool_body() -> Result<(), Box<dyn std::error::Error>> {
    let depool_abi = "tests/samples/fakeDepool.abi.json";
    let msig_abi = "tests/samples/SafeMultisigWallet.abi.json";
    let seed_phrase = "blanket time net universe ketchup maid way poem scatter blur limit drill";

    let config = get_config().unwrap();
    let depool_addr = config["addr"].as_str().unwrap();
    let msig_addr = config["wallet"].as_str().unwrap();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("body")
        .arg("--abi")
        .arg(depool_abi)
        .arg("addOrdinaryStake")
        .arg(r#"{"stake":65535}"#);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("te6ccgEBAQEADgAAGAqsGP0AAAAAAAD//w=="));

    sleep(Duration::new(2, 0));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(depool_abi)
        .arg(&depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"stake": "4000000000"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("call")
        .arg("--abi")
        .arg(msig_abi)
        .arg("--sign")
        .arg(seed_phrase)
        .arg(&msig_addr)
        .arg("sendTransaction")
        .arg(format!(r#"{{"dest":"{}","value":1000000000,"bounce":true,"flags":1,"payload":"te6ccgEBAQEADgAAGAqsGP0AAAAAAAD//w=="}}"#, &depool_addr));
    cmd.assert()
        .success();

    sleep(Duration::new(2, 0));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(depool_abi)
        .arg(&depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"stake": "65535"#));

    Ok(())
}

#[test]
fn test_depool_1() -> Result<(), Box<dyn std::error::Error>> {
    let depool_abi = "tests/samples/fakeDepool.abi.json";
    let seed_phrase = "blanket time net universe ketchup maid way poem scatter blur limit drill";

    let config = get_config().unwrap();
    let depool_addr = config["addr"].as_str().unwrap();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("replenish")
        .arg("--value")
        .arg("2")
        .arg("--sign")
        .arg(seed_phrase);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"Succeeded."#));

    sleep(Duration::new(2, 0));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(depool_abi)
        .arg(&depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"value": "2000000000"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("ticktock")
        .arg("--sign")
        .arg(seed_phrase);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"Succeeded."#));

    sleep(Duration::new(2, 0));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(depool_abi)
        .arg(&depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"value": "1000000000"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("events");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"StakeSigningRequested"#))
        .stdout(predicate::str::contains(r#"{"electionId":"1","proxy":"0:0000000000000000000000000000000000000000000000000000000000000002"}"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("events")
        .arg("-w");
    cmd.assert()
        .success();

    Ok(())
}

#[test]
fn test_depool_2() -> Result<(), Box<dyn std::error::Error>> {
    let depool_abi = "tests/samples/fakeDepool.abi.json";
    let seed_phrase = "blanket time net universe ketchup maid way poem scatter blur limit drill";

    let config = get_config().unwrap();
    let depool_addr = config["addr"].as_str().unwrap();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--depool_fee")
        .arg("0.7")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("withdraw")
        .arg("off")
        .arg("--sign")
        .arg(seed_phrase)
        .arg("--wait-answer");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"Answer status: SUCCESS"#));

    sleep(Duration::new(2, 0));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(depool_abi)
        .arg(&depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"reinvest": true"#))
        .stdout(predicate::str::contains(r#"value": "700000000"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--depool_fee")
        .arg("0.8")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("withdraw")
        .arg("on")
        .arg("--sign")
        .arg(seed_phrase)
        .arg("-a");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"Answer status: SUCCESS"#));

    sleep(Duration::new(2, 0));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(depool_abi)
        .arg(&depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"reinvest": false"#))
        .stdout(predicate::str::contains(r#"value": "800000000"#));

    Ok(())
}

#[test]
fn test_depool_3() -> Result<(), Box<dyn std::error::Error>> {
    let giver_addr = "0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94";
    let depool_abi = "tests/samples/fakeDepool.abi.json";
    let seed_phrase = "blanket time net universe ketchup maid way poem scatter blur limit drill";

    let config = get_config().unwrap();
    let depool_addr = config["addr"].as_str().unwrap();
    let msig_addr = config["wallet"].as_str().unwrap();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("--wait-answer")
        .arg("stake")
        .arg("lock")
        .arg("--sign")
        .arg(seed_phrase)
        .arg("--beneficiary")
        .arg(giver_addr)
        .arg("--total")
        .arg("1")
        .arg("--withdrawal")
        .arg("1")
        .arg("--value")
        .arg("2");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"Answer status: SUCCESS"#));

    sleep(Duration::new(2, 0));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(depool_abi)
        .arg(&depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(format!(r#"sender": "{}"#, &msig_addr)))
        .stdout(predicate::str::contains(r#"stake": "2000000000"#))
        .stdout(predicate::str::contains(format!(r#"receiver": "{}"#, giver_addr)))
        .stdout(predicate::str::contains(r#"withdrawal": "86400"#))
        .stdout(predicate::str::contains(r#"total": "86400"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("-a")
        .arg("stake")
        .arg("vesting")
        .arg("--sign")
        .arg(seed_phrase)
        .arg("--beneficiary")
        .arg("0:0123456789012345012345678901234501234567890123450123456789012345")
        .arg("--total")
        .arg("2")
        .arg("--withdrawal")
        .arg("2")
        .arg("--value")
        .arg("4");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"Answer status: SUCCESS"#));

    sleep(Duration::new(2, 0));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(depool_abi)
        .arg(&depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(format!(r#"sender": "{}"#, &msig_addr)))
        .stdout(predicate::str::contains(r#"stake": "4000000000"#))
        .stdout(predicate::str::contains(r#"receiver": "0:0123456789012345012345678901234501234567890123450123456789012345"#))
        .stdout(predicate::str::contains(r#"withdrawal": "172800"#))
        .stdout(predicate::str::contains(r#"total": "172800"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("stake")
        .arg("transfer")
        .arg("--sign")
        .arg(seed_phrase)
        .arg("--dest")
        .arg(giver_addr)
        .arg("--value")
        .arg("2");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"Succeeded."#));

    sleep(Duration::new(2, 0));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(depool_abi)
        .arg(&depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(format!(r#"sender": "{}"#, &msig_addr)))
        .stdout(predicate::str::contains(r#"stake": "2000000000"#))
        .stdout(predicate::str::contains(format!(r#"receiver": "{}"#, giver_addr)));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("--wait-answer")
        .arg("stake")
        .arg("ordinary")
        .arg("--sign")
        .arg(seed_phrase)
        .arg("--value")
        .arg("1");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"Answer status: TOTAL_PERIOD_MORE_18YEARS"#));

    sleep(Duration::new(2, 0));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(depool_abi)
        .arg(&depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(format!(r#"sender": "{}"#, &msig_addr)))
        .stdout(predicate::str::contains(r#"stake": "1000000000"#));

    Ok(())
}

#[test]
fn test_depool_4() -> Result<(), Box<dyn std::error::Error>> {
    let depool_abi = "tests/samples/fakeDepool.abi.json";
    let seed_phrase = "blanket time net universe ketchup maid way poem scatter blur limit drill";

    let config = get_config().unwrap();
    let depool_addr = config["addr"].as_str().unwrap();
    let msig_addr = config["wallet"].as_str().unwrap();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("stake")
        .arg("remove")
        .arg("--sign")
        .arg(seed_phrase)
        .arg("--value")
        .arg("3");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"Succeeded."#));

    sleep(Duration::new(2, 0));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(depool_abi)
        .arg(&depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(format!(r#"sender": "{}"#, &msig_addr)))
        .stdout(predicate::str::contains(r#"stake": "3000000000"#))
        .stdout(predicate::str::contains(r#"value": "800000000"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("stake")
        .arg("withdrawPart")
        .arg("--sign")
        .arg(seed_phrase)
        .arg("--value")
        .arg("4");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"Succeeded."#));

    sleep(Duration::new(2, 0));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(depool_abi)
        .arg(&depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(format!(r#"sender": "{}"#, &msig_addr)))
        .stdout(predicate::str::contains(r#"stake": "4000000000"#))
        .stdout(predicate::str::contains(r#"value": "800000000"#));

    Ok(())
}

#[test]
fn test_depool_5() -> Result<(), Box<dyn std::error::Error>> {
    let seed_phrase = "blanket time net universe ketchup maid way poem scatter blur limit drill";
    let config = get_config().unwrap();
    let depool_abi = "tests/samples/fakeDepool.abi.json";
    let depool_addr = config["addr"].as_str().unwrap();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("--wait-answer")
        .arg("donor")
        .arg("lock")
        .arg("--sign")
        .arg(seed_phrase)
        .arg("--donor")
        .arg("0:0123456789012345012345678901234501234567890123450123456789012345");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"Answer status: SUCCESS"#));

    sleep(Duration::new(2, 0));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(depool_abi)
        .arg(&depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"receiver": "0:0123456789012345012345678901234501234567890123450123456789012345"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("donor")
        .arg("vesting")
        .arg("--wait-answer")
        .arg("--sign")
        .arg(seed_phrase)
        .arg("--donor")
        .arg("0:0123456789012345012345678901234501234567890123450123456789012346");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"Answer status: SUCCESS"#));

    sleep(Duration::new(2, 0));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(depool_abi)
        .arg(&depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"receiver": "0:0123456789012345012345678901234501234567890123450123456789012346"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("decode")
        .arg("stateinit")
        .arg(&depool_addr);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"version": "sol 0.51.0"#))
        .stdout(predicate::str::contains(r#"code_depth": "7"#))
        .stdout(predicate::str::contains(r#"data_depth": "1"#));

    Ok(())
}

#[test]
fn test_gen_deploy_message() -> Result<(), Box<dyn std::error::Error>> {
    let output = "test_gen_deploy_message_raw.out";
    let wallet_tvc = "tests/samples/wallet.tvc";
    let wallet_abi = "tests/samples/wallet.abi.json";
    let key_path = "tests/deploy_test.key";

    let addr = generate_key_and_address(key_path, wallet_tvc, wallet_abi)?;

    let _ = std::fs::remove_file(output);
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("deploy_message")
        .arg(wallet_tvc)
        .arg("{}")
        .arg("--abi")
        .arg(wallet_abi)
        .arg("--sign")
        .arg(key_path)
        .arg("--output")
        .arg(output)
        .arg("--raw");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(addr))
        .stdout(predicate::str::contains("Succeeded"));

    let _ = std::fs::remove_file(output);
    Ok(())
}

#[test]
fn test_decode_tvc() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("decode")
        .arg("account")
        .arg("data")
        .arg("--abi")
        .arg("tests/test_abi_v2.1.abi.json")
        .arg("--tvc")
        .arg("tests/decode_fields.tvc")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""__pubkey": "0xe8b1d839abe27b2abb9d4a2943a9143a9c7e2ae06799bd24dec1d7a8891ae5dd","#))
        .stdout(predicate::str::contains(r#" "a": "I like it.","#));

    let boc_path = "tests/account.boc";
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("decode")
        .arg("stateinit")
        .arg("--boc")
        .arg(boc_path)
        .assert()
        .success()
        .stdout(predicate::str::contains(r#"version": "sol 0.51.0"#))
        .stdout(predicate::str::contains(r#"code_depth": "7"#))
        .stdout(predicate::str::contains(r#"data_depth": "1"#));


    let tvc_path = "tests/samples/fakeDepool.tvc";
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("decode")
        .arg("stateinit")
        .arg("--tvc")
        .arg(tvc_path)
        .assert()
        .success()
        .stdout(predicate::str::contains(r#"version": "sol 0.51.0"#))
        .stdout(predicate::str::contains(r#"code_depth": "7"#))
        .stdout(predicate::str::contains(r#"data_depth": "1"#));

    Ok(())
}

#[test]
fn test_dump_tvc() -> Result<(), Box<dyn std::error::Error>> {
    let tvc_path = "giver.tvc";
    let giver_addr = "0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94";

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("account")
        .arg("--dumptvc")
        .arg(tvc_path)
        .arg(giver_addr)
        .assert()
        .success()
        .stdout(predicate::str::contains("Saved contract to"));

    fs::remove_file(tvc_path)?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("account")
        .arg("--dumpboc")
        .arg(tvc_path)
        .arg(giver_addr)
        .assert()
        .success()
        .stdout(predicate::str::contains("Saved account to"));

    fs::remove_file(tvc_path)?;

    let boc_path = "tests/account.boc";

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("decode")
        .arg("account")
        .arg("boc")
        .arg(boc_path)
        .arg("--dumptvc")
        .arg(tvc_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("balance: "))
        .stdout(predicate::str::contains("state_init"));

    fs::remove_file(tvc_path)?;
    Ok(())
}

#[test]
fn test_run_account() -> Result<(), Box<dyn std::error::Error>> {
    let boc_path = "tests/depool_acc.boc";
    let tvc_path = "tests/depool_acc.tvc";
    let abi_path = "tests/samples/fakeDepool.abi.json";

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--boc")
        .arg(boc_path)
        .arg("getData")
        .arg("{}")
        .arg("--abi")
        .arg(abi_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Succeeded."))
        .stdout(predicate::str::contains("Result: {"))
        .stdout(predicate::str::contains(r#""reinvest": false,"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--tvc")
        .arg(tvc_path)
        .arg("getData")
        .arg("{}")
        .arg("--abi")
        .arg(abi_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Succeeded."))
        .stdout(predicate::str::contains("Result: {"))
        .stdout(predicate::str::contains(r#""reinvest": false,"#));

    let config_path = "tests/block_config.boc";

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--url")
        .arg("main.ton.dev")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("dump")
        .arg("config")
        .arg(config_path)
        .assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--url")
        .arg(&*NETWORK)
        .assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--boc")
        .arg(boc_path)
        .arg("getData")
        .arg("{}")
        .arg("--abi")
        .arg(abi_path)
        .arg("--bc_config")
        .arg(config_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Succeeded."))
        .stdout(predicate::str::contains("Result: {"))
        .stdout(predicate::str::contains(r#""reinvest": false,"#));

    let boc_path = "tests/account_fift.boc";
    let tvc_path = "tests/account_fift.tvc";

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("runget")
        .arg("--boc")
        .arg(boc_path)
        .arg("past_election_ids")
        .assert()
        .success()
        .stdout(predicate::str::contains("Succeeded."))
        .stdout(predicate::str::contains(r#"Result: [["1633273052",["1633338588",null]]]"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("runget")
        .arg("--tvc")
        .arg(tvc_path)
        .arg("past_election_ids")
        .assert()
        .success()
        .stdout(predicate::str::contains("Succeeded."))
        .stdout(predicate::str::contains(r#"Result: [["1634489970",["1634555506",null]]]"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("runget")
        .arg("--boc")
        .arg(boc_path)
        .arg("past_election_ids")
        .arg("--bc_config")
        .arg(config_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Succeeded."))
        .stdout(predicate::str::contains(r#"Result: [["1633273052",["1633338588",null]]]"#));

    fs::remove_file(config_path)?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("runget")
        .arg("--boc")
        .arg(boc_path)
        .arg("compute_returned_stake")
        .arg("0x0166d0181a19f87af9397040a68671e1b239f12152824f7d987fd6897d6a9587")
        .assert()
        .success()
        .stdout(predicate::str::contains("Succeeded."))
        .stdout(predicate::str::contains(r#"Result: ["125387107580525"]"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("-j")
        .arg("runget")
        .arg("--boc")
        .arg(boc_path)
        .arg("participant_list_extended")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#"value0": "0","#))
        .stdout(predicate::str::contains(r#"value1": "0","#))
        .stdout(predicate::str::contains(r#"value2": "0","#))
        .stdout(predicate::str::contains(r#"value3": "0","#))
        .stdout(predicate::str::contains(r#"value4": null,"#))
        .stdout(predicate::str::contains(r#"value5": "0","#))
        .stdout(predicate::str::contains(r#"value6": "0""#));

    Ok(())
}

#[test]
fn test_run_async_call() -> Result<(), Box<dyn std::error::Error>> {
    let giver_abi_name = "tests/samples/giver.abi.json";

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--url")
        .arg(&*NETWORK)
        .assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--async_call")
        .arg("false")
        .assert()
        .success();

    let time = now_ms();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("call")
        .arg("0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94")
        .arg("sendGrams")
        .arg(r#"{"dest":"0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94","amount":1000000000}"#)
        .arg("--abi")
        .arg(giver_abi_name)
        .assert()
        .success()
        .stdout(predicate::str::contains("Succeeded"));

    let duration = now_ms() - time;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--async_call")
        .arg("true")
        .assert()
        .success();

    let time = now_ms();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("call")
        .arg("0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94")
        .arg("sendGrams")
        .arg(r#"{"dest":"0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94","amount":1000000000}"#)
        .arg("--abi")
        .arg(giver_abi_name)
        .assert()
        .success()
        .stdout(predicate::str::contains("Succeeded"));

    assert!(duration > now_ms() - time);

    let config = get_config().unwrap();
    let depool_abi = "tests/samples/fakeDepool.abi.json";
    let depool_addr = config["addr"].as_str().unwrap();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(depool_abi)
        .arg(&depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"receiver": "0:0123456789012345012345678901234501234567890123450123456789012346"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--async_call")
        .arg("false")
        .arg("--local_run")
        .arg("true")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(depool_abi)
        .arg(&depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"receiver": "0:0123456789012345012345678901234501234567890123450123456789012346"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("call")
        .arg("0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94")
        .arg("sendGrams")
        .arg(r#"{"dest":"0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94","amount":1000000000}"#)
        .arg("--abi")
        .arg(giver_abi_name)
        .assert()
        .success()
        .stdout(predicate::str::contains("Local run succeeded"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--async_call")
        .arg("false")
        .arg("--local_run")
        .arg("true")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(depool_abi)
        .arg(&depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"receiver": "0:0123456789012345012345678901234501234567890123450123456789012346"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("call")
        .arg("0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94")
        .arg("sendGrams")
        .arg(r#"{"dest":"0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94","amount":1000000000}"#)
        .arg("--abi")
        .arg(giver_abi_name)
        .assert()
        .success()
        .stdout(predicate::str::contains("Succeeded"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--async_call")
        .arg("false")
        .arg("--local_run")
        .arg("false")
        .assert()
        .success();

    Ok(())
}

#[test]
fn test_multisig() -> Result<(), Box<dyn std::error::Error>> {
    let key_path = "tests/deploy_test.key";
    let safe_msig_abi = "tests/samples/SafeMultisigWallet.abi.json";
    let setcode_msig_abi = "tests/samples/SetcodeMultisigWallet.abi.json";

    generate_phrase_and_key(key_path)?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    let out = cmd.arg("multisig")
        .arg("deploy")
        .arg("-k")
        .arg(key_path)
        .output()
        .expect("Failed to generate multisig address.");
    let mut addr = String::from_utf8_lossy(&out.stdout).to_string();
    addr.replace_range(..addr.find("0:").unwrap_or(0), "");
    addr.replace_range(addr.find("Connecting").unwrap_or(addr.len())-1.., "");

    ask_giver(&addr, 1000000000)?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("multisig")
        .arg("deploy")
        .arg("-k")
        .arg(key_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Wallet successfully deployed"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("call")
        .arg(addr)
        .arg("sendTransaction")
        .arg(r#"{"dest":"0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94","value":100000,"bounce":"false","flags":1,"payload":""}"#)
        .arg("--abi")
        .arg(safe_msig_abi)
        .arg("--sign")
        .arg(key_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Succeeded"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    let out = cmd.arg("multisig")
        .arg("deploy")
        .arg("--setcode")
        .arg("-k")
        .arg(key_path)
        .output()
        .expect("Failed to generate multisig address.");
    let mut addr = String::from_utf8_lossy(&out.stdout).to_string();
    addr.replace_range(..addr.find("0:").unwrap_or(0), "");
    addr.replace_range(addr.find("Connecting").unwrap_or(addr.len())-1.., "");

    ask_giver(&addr, 1000000000)?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("multisig")
        .arg("deploy")
        .arg("--setcode")
        .arg("-k")
        .arg(key_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Wallet successfully deployed"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg(addr)
        .arg("getUpdateRequests")
        .arg("{}")
        .arg("--abi")
        .arg(setcode_msig_abi)
        .assert()
        .success()
        .stdout(predicate::str::contains("Succeeded"));

    let key_path1 = "key1";
    let seed = generate_phrase_and_key(key_path1)?;
    let key1 = generate_public_key(&seed)?;

    let key_path2 = "key2";
    let seed = generate_phrase_and_key(key_path2)?;
    let key2 = generate_public_key(&seed)?;

    let key_path3 = "key3";
    let seed = generate_phrase_and_key(key_path3)?;
    let key3 = generate_public_key(&seed)?;

    let owners_string = format!(r#"["0x{}","0x{}","0x{}"]"#, key1, key2, key3);

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    let out = cmd.arg("multisig")
        .arg("deploy")
        .arg("-k")
        .arg(key_path1)
        .arg("--owners")
        .arg(owners_string.clone())
        .arg("--confirms")
        .arg("2")
        .output()
        .expect("Failed to generate multisig address.");
    let mut addr = String::from_utf8_lossy(&out.stdout).to_string();
    addr.replace_range(..addr.find("0:").unwrap_or(0), "");
    addr.replace_range(addr.find("Connecting").unwrap_or(addr.len())-1.., "");

    ask_giver(&addr, 1000000000)?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("multisig")
        .arg("deploy")
        .arg("-k")
        .arg(key_path1)
        .arg("--owners")
        .arg(owners_string)
        .arg("--confirms")
        .arg("2")
        .assert()
        .success()
        .stdout(predicate::str::contains("Wallet successfully deployed"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(safe_msig_abi)
        .arg(addr.clone())
        .arg("getCustodians")
        .arg("{}")
        .assert()
        .success()
        .stdout(predicate::str::contains(key1.clone()))
        .stdout(predicate::str::contains(key2.clone()))
        .stdout(predicate::str::contains(key3.clone()));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(safe_msig_abi)
        .arg(addr.clone())
        .arg("getParameters")
        .arg("{}")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""requiredTxnConfirms": "2""#));

    fs::remove_file(key_path1)?;
    fs::remove_file(key_path2)?;
    fs::remove_file(key_path3)?;

    let seed = generate_phrase_and_key(key_path)?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    let out = cmd.arg("multisig")
        .arg("deploy")
        .arg("-k")
        .arg(seed)
        .arg("--local")
        .arg("1_000_000_000")
        .output()
        .expect("Failed to generate multisig address.");
    let mut addr = String::from_utf8_lossy(&out.stdout).to_string();
    addr.replace_range(..addr.find("0:").unwrap_or(0), "");
    addr.replace_range(addr.find("Connecting").unwrap_or(addr.len())-1.., "");

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("call")
        .arg(addr)
        .arg("sendTransaction")
        .arg(r#"{"dest":"0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94","value":100000,"bounce":"false","flags":1,"payload":""}"#)
        .arg("--abi")
        .arg(safe_msig_abi)
        .arg("--sign")
        .arg(key_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Succeeded"));
    Ok(())
}

#[test]
fn test_alternative_syntax() -> Result<(), Box<dyn std::error::Error>> {
    let boc_path = "tests/depool_acc.boc";
    let abi_path = "tests/samples/fakeDepool.abi.json";

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("runx")
        .arg("--boc")
        .arg("--addr")
        .arg(boc_path)
        .arg("getData")
        .arg("--abi")
        .arg(abi_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Succeeded."))
        .stdout(predicate::str::contains("Result: {"))
        .stdout(predicate::str::contains(r#""reinvest": false,"#));

    let giver_abi_name = "tests/samples/giver.abi.json";
    let wallet_tvc = "tests/samples/SafeMultisigWallet.tvc";
    let wallet_abi = "tests/samples/SafeMultisigWallet.abi.json";
    let key_path = "tests/deploy_test.key";

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--url")
        .arg(&*NETWORK)
        .assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    let out = cmd.arg("genphrase")
        .output()
        .expect("Failed to generate a seed phrase.");
    let mut seed = String::from_utf8_lossy(&out.stdout).to_string();
    seed.replace_range(..seed.find('"').unwrap_or(0), "");
    seed.retain(|c| c != '\n' && c != '"');

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("getkeypair")
        .arg(key_path)
        .arg(seed)
        .assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    let out = cmd.arg("genaddr")
        .arg("--setkey")
        .arg(key_path)
        .arg(wallet_tvc)
        .arg(wallet_abi)
        .output()
        .expect("Failed to generate address.");

    let mut addr = String::from_utf8_lossy(&out.stdout).to_string();
    addr.replace_range(..addr.find("0:").unwrap_or(0), "");
    addr.replace_range(addr.find("testnet").unwrap_or(addr.len())-1.., "");

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("callx")
        .arg("--abi")
        .arg(giver_abi_name)
        .arg("--addr")
        .arg("0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94")
        .arg("sendGrams")
        .arg("--dest")
        .arg(addr.clone())
        .arg("--amount")
        .arg("1000000000");
    cmd.assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--abi")
        .arg(wallet_abi)
        .arg("--addr")
        .arg(addr.clone())
        .arg("--keys")
        .arg(key_path)
        .assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("deployx")
        .arg(wallet_tvc)
        .arg("--owners")
        .arg(r#"["0xc8bd66f90d61f7e1e1a6151a0dbe9d8640666920d8c0cf399cbfb72e089d2e41"]"#)
        .arg("--reqConfirms")
        .arg("1");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(addr))
        .stdout(predicate::str::contains("Transaction succeeded."));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("runx")
        .arg("getParameters");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Succeeded."));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("clear")
        .assert()
        .success();

    Ok(())
}

#[test]
fn test_convert() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("convert")
        .arg("tokens")
        .arg("0.1234567890")
        .output()
        .expect("Error: invalid fractional part");

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("convert")
        .arg("tokens")
        .arg("0.123456789")
        .assert()
        .success()
        .stdout(predicate::str::contains("123456789"));


    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("convert")
        .arg("tokens")
        .arg("0.01")
        .assert()
        .success()
        .stdout(predicate::str::contains("10000000"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("-j")
        .arg("convert")
        .arg("tokens")
        .arg("0.123456789")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""value": "123456789""#));

    Ok(())
}
