use predicates::prelude::*;
use assert_cmd::Command;
use std::env;
use std::time::Duration;
use std::thread::sleep;
use std::fs;

mod common;
use common::{BIN_NAME, NETWORK, giver_v2, grep_address, set_config, GIVER_V2_ABI,
             GIVER_V2_ADDR, GIVER_V2_KEY, generate_key_and_address, GIVER_ABI,
             generate_phrase_and_key};

const DEPOOL_ABI: &str = "tests/samples/fakeDepool.abi.json";
const DEPOOL_TVC: &str = "tests/samples/fakeDepool.tvc";
const SAFEMSIG_ABI: &str = "tests/samples/SafeMultisigWallet.abi.json";
const SAFEMSIG_TVC: &str = "tests/samples/SafeMultisigWallet.tvc";
const SAFEMSIG_SEED: &str = "blanket time net universe ketchup maid way poem scatter blur limit drill";
const SAFEMSIG_ADDR: &str = "0:d5f5cfc4b52d2eb1bd9d3a8e51707872c7ce0c174facddd0e06ae5ffd17d2fcd";
const SAFEMSIG_CONSTR_ARG: &str = r#"{"owners":["0xc8bd66f90d61f7e1e1a6151a0dbe9d8640666920d8c0cf399cbfb72e089d2e41"],"reqConfirms":1}"#;

fn now_ms() -> u64 {
    chrono::prelude::Utc::now().timestamp_millis() as u64
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

fn deploy_safe_msig(key_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    deploy_contract(key_path, SAFEMSIG_TVC, SAFEMSIG_ABI, SAFEMSIG_CONSTR_ARG)
}

fn deploy_depool(key_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    deploy_contract(key_path, DEPOOL_TVC, DEPOOL_ABI, "{}")
}

fn deploy_contract(
    key_path: &str,
    tvc_path: &str,
    abi_path: &str,
    constructor_params: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let address = generate_key_and_address(key_path, tvc_path, abi_path)?;
    giver_v2(&address);

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("deploy")
        .arg(tvc_path)
        .arg(constructor_params)
        .arg("--abi")
        .arg(abi_path)
        .arg("--sign")
        .arg(key_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(&address))
        .stdout(predicate::str::contains("Transaction succeeded."));

    Ok(address)
}

fn wait_for_cmd_res(cmd: &mut Command, expected: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut attempts = 10;
    loop {
        // let mut cmd = Command::cargo_bin(BIN_NAME)?;
        let res = cmd.assert()
            .success();
        match String::from_utf8(res.get_output().stdout.clone()) {
            Ok(res) => {
                if res.contains(expected) {
                    break;
                }
            },
            Err(_) => {
                return Err(string_error::into_err("Failed to decode command output.".to_string()));
            }
        }
        attempts -= 1;
        if attempts == 0 {
            return Err(string_error::into_err("Failed to fetch command result.".to_string()));
        }
        sleep(Duration::new(1, 0));
    }
    Ok(())
}


#[test]
fn test_config_1() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = "test1.config";
    set_config(
        &["--url", "--retries", "--timeout", "--wc"],
        &[&*NETWORK, "10", "25000", "-2"],
        Some(config_path)
    )?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("config")
        .arg("--list");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(format!(r#""url": "{}""#, &*NETWORK)))
        .stdout(predicate::str::contains(r#""retries": 10"#))
        .stdout(predicate::str::contains(r#""timeout": 25000"#))
        .stdout(predicate::str::contains(r#""wc": -2"#));

    fs::remove_file(config_path)?;
    Ok(())
}

#[test]
fn test_config_endpoints() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = "test2.config";
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("config")
        .arg("endpoint")
        .arg("reset");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("main.ton.dev"))
        .stdout(predicate::str::contains("https://rbx01.main.everos.dev"))
        .stdout(predicate::str::contains("http://127.0.0.1/"))
        .stdout(predicate::str::contains("net.ton.dev"))
        .stdout(predicate::str::contains("https://gra01.net.everos.dev"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("config")
        .arg("clear");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Succeeded"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("config")
        .arg("--url")
        .arg("main.ton.dev");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#""url": "main.ton.dev","#))
        .stdout(predicate::str::contains(r#""endpoints": [
    "https://eri01.main.everos.dev",
    "https://gra01.main.everos.dev",
    "https://gra02.main.everos.dev",
    "https://lim01.main.everos.dev",
    "https://rbx01.main.everos.dev"
  ]"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("config")
        .arg("endpoint")
        .arg("add")
        .arg("myownhost")
        .arg("[1.1.1.1,my.net.com]");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("main.ton.dev"))
        .stdout(predicate::str::contains("https://rbx01.main.everos.dev"))
        .stdout(predicate::str::contains("http://127.0.0.1/"))
        .stdout(predicate::str::contains("net.ton.dev"))
        .stdout(predicate::str::contains("https://gra01.net.everos.dev"))
        .stdout(predicate::str::contains("myownhost"))
        .stdout(predicate::str::contains("1.1.1.1"))
        .stdout(predicate::str::contains("my.net.com"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("config")
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
    cmd.arg("--config")
        .arg(config_path)
        .arg("config")
        .arg("endpoint")
        .arg("add")
        .arg("myownhost")
        .arg("[1.1.1.1,my.net.com,tonlabs.net]");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("main.ton.dev"))
        .stdout(predicate::str::contains("https://rbx01.main.everos.dev"))
        .stdout(predicate::str::contains("http://127.0.0.1/"))
        .stdout(predicate::str::contains("net.ton.dev"))
        .stdout(predicate::str::contains("https://gra01.net.everos.dev"))
        .stdout(predicate::str::contains("myownhost"))
        .stdout(predicate::str::contains("1.1.1.1"))
        .stdout(predicate::str::contains("my.net.com"))
        .stdout(predicate::str::contains("tonlabs.net"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("config")
        .arg("--url")
        .arg("myownhost");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#""url": "myownhost","#))
        .stdout(predicate::str::contains("1.1.1.1"))
        .stdout(predicate::str::contains("my.net.com"))
        .stdout(predicate::str::contains("tonlabs.net"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("config")
        .arg("endpoint")
        .arg("remove")
        .arg("main.ton.dev");
    cmd.assert()
        .success()
        .stdout(predicate::function(|s: &str| !s.contains("main.ton.dev")))
        .stdout(predicate::function(|s: &str| !s.contains("https://rbx01.main.everos.dev")))
        .stdout(predicate::str::contains("http://127.0.0.1/"))
        .stdout(predicate::str::contains("net.ton.dev"))
        .stdout(predicate::str::contains("https://gra01.net.everos.dev"))
        .stdout(predicate::str::contains("myownhost"))
        .stdout(predicate::str::contains("1.1.1.1"))
        .stdout(predicate::str::contains("my.net.com"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("config")
        .arg("endpoint")
        .arg("print");
    cmd.assert()
        .success()
        .stdout(predicate::function(|s: &str| !s.contains("main.ton.dev")))
        .stdout(predicate::function(|s: &str| !s.contains("https://rbx01.main.everos.dev")))
        .stdout(predicate::str::contains("http://127.0.0.1/"))
        .stdout(predicate::str::contains("net.ton.dev"))
        .stdout(predicate::str::contains("https://gra01.net.everos.dev"))
        .stdout(predicate::str::contains("myownhost"))
        .stdout(predicate::str::contains("1.1.1.1"))
        .stdout(predicate::str::contains("my.net.com"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("config")
        .arg("endpoint")
        .arg("reset");
    cmd.assert()
        .success()
        .stdout(predicate::function(|s: &str| !s.contains("myownhost")))
        .stdout(predicate::function(|s: &str| !s.contains("my.net.com")))
        .stdout(predicate::str::contains("http://127.0.0.1/"))
        .stdout(predicate::str::contains("net.ton.dev"))
        .stdout(predicate::str::contains("https://gra01.net.everos.dev"))
        .stdout(predicate::str::contains("main.ton.dev"))
        .stdout(predicate::str::contains("https://rbx01.main.everos.dev"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("config")
        .arg("--list");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#""url": "myownhost","#))
        .stdout(predicate::str::contains("1.1.1.1"))
        .stdout(predicate::str::contains("my.net.com"))
        .stdout(predicate::str::contains("tonlabs.net"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("config")
        .arg("clear");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Succeeded"));

    fs::remove_file(config_path)?;
    Ok(())
}

#[test]
fn test_call_giver() -> Result<(), Box<dyn std::error::Error>> {
    giver_v2(GIVER_V2_ADDR);
    Ok(())
}

#[test]
fn test_fee() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("fee")
        .arg("storage")
        .arg(GIVER_V2_ADDR)
        .assert()
        .success()
        .stdout(predicate::str::contains("Storage fee per 31536000 seconds: "));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("fee")
        .arg("storage")
        .arg(GIVER_V2_ADDR)
        .arg("--period")
        .arg("10000")
        .assert()
        .success()
        .stdout(predicate::str::contains("Storage fee per 10000 seconds: "));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("fee")
        .arg("call")
        .arg("--abi")
        .arg(GIVER_V2_ABI)
        .arg(GIVER_V2_ADDR)
        .arg("--sign")
        .arg(GIVER_V2_KEY)
        .arg("sendTransaction")
        .arg(format!(r#"{{"dest":"{}","value":100000000000,"bounce":false}}"#, GIVER_V2_ADDR))
        .assert()
        .success()
        .stdout(predicate::str::contains(r#"  "in_msg_fwd_fee":"#))
        .stdout(predicate::str::contains(r#"  "storage_fee":"#))
        .stdout(predicate::str::contains(r#"  "gas_fee":"#))
        .stdout(predicate::str::contains(r#"  "out_msgs_fwd_fee":"#))
        .stdout(predicate::str::contains(r#"  "total_account_fees":"#))
        .stdout(predicate::str::contains(r#"  "total_output":"#))
        .stdout(predicate::str::contains(r#"Succeeded."#));

    let key_path = "deploy_test_fee.key";
    let _ = generate_phrase_and_key(key_path)?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("fee")
        .arg("deploy")
        .arg(SAFEMSIG_TVC)
        .arg(SAFEMSIG_CONSTR_ARG)
        .arg("--abi")
        .arg(SAFEMSIG_ABI)
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
    fs::remove_file(key_path)?;
    Ok(())
}

#[test]
fn test_genaddr_genkey() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("genaddr")
        .arg("tests/samples/wallet.tvc")
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
        .arg("--genkey")
        .arg("/dev/null");
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
        .arg("--abi")
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
        .arg("-o")
        .arg("tests/samples/tmp.json")
        .arg("-p")
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
    let key_path = "tests/async_deploy_test.key";
    let config_path = "async.config";

    let addr = generate_key_and_address(key_path, wallet_tvc, wallet_abi)?;
    giver_v2(&addr);

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("account")
        .arg(addr.clone());

    wait_for_cmd_res(&mut cmd, "acc_type:      Uninit")?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("config")
        .arg("--async_call")
        .arg("true")
        .arg("--url")
        .arg(&*NETWORK)
        .assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("deploy")
        .arg(wallet_tvc)
        .arg("{}")
        .arg("--abi")
        .arg(wallet_abi)
        .arg("--sign")
        .arg(key_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(addr.clone()));

    fs::remove_file(config_path)?;
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("account")
        .arg(addr.clone());

    wait_for_cmd_res(&mut cmd, "acc_type:      Active")?;

    fs::remove_file(key_path)?;
    Ok(())
}

#[test]
fn test_deploy() -> Result<(), Box<dyn std::error::Error>> {
    let wallet_tvc = "tests/samples/wallet.tvc";
    let wallet_abi = "tests/samples/wallet.abi.json";
    let key_path = "tests/deploy_test.key";
    let config_path = "deploy.conf";

    let addr = generate_key_and_address(key_path, wallet_tvc, wallet_abi)?;

    giver_v2(&addr);

    set_config(&["--balance_in_tons", "--url"], &["true", &*NETWORK], Some(config_path))?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("account")
        .arg(&addr)
        .assert()
        .success()
        .stdout(predicate::str::contains(" ton"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("-j")
        .arg("--config")
        .arg(config_path)
        .arg("account")
        .arg(&addr)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"balance\": \""));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("account")
        .arg(&addr)
        .assert()
        .success()
        .stdout(predicate::str::contains(" nanoton"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("-j")
        .arg("account")
        .arg(&addr)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"balance\": \""));

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

    let tvc_path = "tests/samples/fakeDepool.tvc";
    let tvc_path2 = "tests/samples/fakeDepool.tvc2";

    let _ = std::fs::copy(tvc_path, tvc_path2)?;
    let abi_path = "tests/samples/fakeDepool.abi.json";

    let time = now_ms();
    let data_str = format!(r#"{{"m_seed":{}}}"#, time);

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    let out = cmd.arg("genaddr")
        .arg(tvc_path2)
        .arg("--abi")
        .arg(abi_path)
        .arg("--data")
        .arg(data_str)
        .arg("--save")
        .output()
        .expect("Failed to generate address.");

    let addr = grep_address(&out.stdout);
    giver_v2(&addr);

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("deploy")
        .arg(tvc_path2)
        .arg("{}")
        .arg("--abi")
        .arg(abi_path)
        .assert()
        .success()
        .stdout(predicate::str::contains(addr))
        .stdout(predicate::str::contains("Transaction succeeded."));

    fs::remove_file(tvc_path2)?;
    fs::remove_file(config_path)?;
    Ok(())
}

#[test]
fn test_genaddr_seed() -> Result<(), Box<dyn std::error::Error>> {
    let key_path = "tests/genaddr_seed.key";

    let seed = generate_phrase_and_key(key_path)?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    let out = cmd.arg("genaddr")
        .arg("--setkey")
        .arg(key_path)
        .arg(SAFEMSIG_TVC)
        .assert()
        .success();

    let msig_addr = grep_address(&out.get_output().stdout);

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    let out = cmd.arg("genaddr")
        .arg("--setkey")
        .arg(seed)
        .arg(SAFEMSIG_TVC)
        .assert()
        .success();

    let msig_addr2 = grep_address(&out.get_output().stdout);

    assert_eq!(msig_addr, msig_addr2);

    fs::remove_file(key_path)?;

    Ok(())
}

#[test]
fn test_callex() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("callex")
        .arg("sendTransaction")
        .arg(GIVER_V2_ADDR)
        .arg(GIVER_V2_ABI)
        .arg(GIVER_V2_KEY)
        .arg("--dest")
        .arg("0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e")
        .arg("--value")
        .arg("0.2T")
        .arg("--bounce")
        .arg("false");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#""dest":"0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e""#))
        .stdout(predicate::str::contains(r#""value":"200000000""#))
        .stdout(predicate::str::contains("Succeeded"));


    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("callex")
        .arg("sendTransaction")
        .arg(GIVER_V2_ADDR)
        .arg(GIVER_V2_ABI)
        .arg(GIVER_V2_KEY)
        .arg("--dest")
        .arg("0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e")
        .arg("--value")
        .arg("1000000000")
        .arg("--bounce")
        .arg("false");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#""dest":"0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e""#))
        .stdout(predicate::str::contains(r#""value":"1000000000""#))
        .stdout(predicate::str::contains("Succeeded"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("callex")
        .arg("sendTransaction")
        .arg(GIVER_V2_ADDR)
        .arg(GIVER_V2_ABI)
        .arg(GIVER_V2_KEY)
        .arg("--dest")
        .arg("0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e")
        .arg("--value")
        .arg("0x10000")
        .arg("--bounce")
        .arg("false");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#""dest":"0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e""#))
        .stdout(predicate::str::contains(r#""value":"0x10000""#))
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
        .stdout(predicate::str::contains("Url: https://test.ton.dev"));
    // config from env variable
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.env("TONOSCLI_CONFIG", "./tests/conf2.json")
        .arg("account")
        .arg("0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94");
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("Url: https://test2.ton.dev"));

    // config from cmd line has higher priority than env variable
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg("tests/conf1.json")
        .env("TONOSCLI_CONFIG", "./tests/conf2.json")
        .arg("account")
        .arg("0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94");
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("Url: https://test.ton.dev"));

    // if there is neither --config nor env variable then config file is searched in current working dir
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("account")
        .arg("0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(format!("Url: {}", &*NETWORK)));
    Ok(())
}

#[test]
fn test_sendfile() -> Result<(), Box<dyn std::error::Error>> {
    let msg_path = "call.boc";
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
        .arg(msg_path);
    cmd.assert()
        .success();

    let config_path = "send.config";
    set_config(&["--async_call"], &["true"], Some(config_path))?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--url")
        .arg(&*NETWORK)
        .arg("--config")
        .arg(config_path)
        .arg("sendfile")
        .arg(msg_path);
    cmd.assert()
        .success();

    fs::remove_file(config_path)?;
    fs::remove_file(msg_path)?;

    Ok(())
}


#[test]
fn test_account_command() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("RUST_LOG", "debug");
    let config_path = "devnet.config";

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("config")
        .arg("--url")
        .arg("https://net.ton.dev")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("account")
        .arg("-1:3333333333333333333333333333333333333333333333333333333333333333");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("acc_type:      Active"))
        .stdout(predicate::str::contains("balance:"))
        .stdout(predicate::str::contains("last_paid:"))
        .stdout(predicate::str::contains("last_trans_lt:"))
        .stdout(predicate::str::contains("data(boc):"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("account")
        .arg("1:3333333333333333333333333333333333333333333333333333333333333333");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Account not found"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("account")
        .arg("--boc")
        .arg("tests/account.boc");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("acc_type:      Active"))
        .stdout(predicate::str::contains("balance:"))
        .stdout(predicate::str::contains("last_paid:"))
        .stdout(predicate::str::contains("last_trans_lt:"))
        .stdout(predicate::str::contains("data(boc):"));

    fs::remove_file(config_path)?;
    Ok(())
}


#[test]
fn test_config_wc() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = "test_wc.config";
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("config")
        .arg("--url")
        .arg("https://net.ton.dev")
        .arg("--wc")
        .arg("-1");
    cmd.assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("account")
        .arg("3333333333333333333333333333333333333333333333333333333333333333");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("acc_type:      Active"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("config")
        .arg("--wc")
        .arg("1");
    cmd.assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("account")
        .arg("3333333333333333333333333333333333333333333333333333333333333333");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Account not found"));

    fs::remove_file(config_path)?;
    Ok(())
}


#[test]
fn test_account_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
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
    let config_path  = "decode.conf";
    set_config(&["--abi"], &["tests/samples/Subscription.abi.json"], Some(config_path))?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("decode")
        .arg("body")
        .arg("te6ccgEBAgEAkwABW1ByqHsAAAF1QnI+qZ/1tsdEUQb8jxj9vr/H4WuiQwfD5ESNbO4lcz2Kca2KavABAMAQYZcjaCLLbO1phXFWOD/kmlkZ1g7FyjgSIEHRpXeeIDiQ3f7FKVd+oeq6VxVlAti+jigqVmtrn8wmBEgbyT8P+5iyVBuoBWSPJetGndR2b83eA6LP5vtB2MFXHClAfKM=");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("constructor: {"))
        .stdout(predicate::str::contains("\"wallet\": \"-1:adb63a228837e478c7edf5fe3f0b5d12183e1f22246b67712b99ec538d6c5357\""));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("--json")
        .arg("decode")
        .arg("body")
        .arg("te6ccgEBAgEAkwABW1ByqHsAAAF1QnI+qZ/1tsdEUQb8jxj9vr/H4WuiQwfD5ESNbO4lcz2Kca2KavABAMAQYZcjaCLLbO1phXFWOD/kmlkZ1g7FyjgSIEHRpXeeIDiQ3f7FKVd+oeq6VxVlAti+jigqVmtrn8wmBEgbyT8P+5iyVBuoBWSPJetGndR2b83eA6LP5vtB2MFXHClAfKM=");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"BodyCall\":"))
        .stdout(predicate::str::contains("\"constructor\":"))
        .stdout(predicate::str::contains("\"wallet\":"))
        .stdout(predicate::str::contains("-1:adb63a228837e478c7edf5fe3f0b5d12183e1f22246b67712b99ec538d6c5357"));


    //test that abi in commandline is preferred
    set_config(&["--abi"], &["tests/samples/wallet.abi.json"], Some(config_path))?;
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("--json")
        .arg("decode")
        .arg("body")
        .arg("te6ccgEBAgEAkwABW1ByqHsAAAF1QnI+qZ/1tsdEUQb8jxj9vr/H4WuiQwfD5ESNbO4lcz2Kca2KavABAMAQYZcjaCLLbO1phXFWOD/kmlkZ1g7FyjgSIEHRpXeeIDiQ3f7FKVd+oeq6VxVlAti+jigqVmtrn8wmBEgbyT8P+5iyVBuoBWSPJetGndR2b83eA6LP5vtB2MFXHClAfKM=")
        .arg("--abi")
        .arg("tests/samples/Subscription.abi.json");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"BodyCall\":"))
        .stdout(predicate::str::contains("\"constructor\":"))
        .stdout(predicate::str::contains("\"wallet\":"))
        .stdout(predicate::str::contains("-1:adb63a228837e478c7edf5fe3f0b5d12183e1f22246b67712b99ec538d6c5357"));

    //test error on wrong body
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("--json")
        .arg("decode")
        .arg("body")
        .arg("\"123\"")
        .arg("--abi")
        .arg("tests/samples/Subscription.abi.json");
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("body is not a valid base64 string"));

    //test error on empty body
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("--json")
        .arg("decode")
        .arg("body")
        .arg("")
        .arg("--abi")
        .arg("tests/samples/Subscription.abi.json");
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("failed to decode body"));

    fs::remove_file(config_path)?;
    Ok(())
}

#[test]
fn test_error() -> Result<(), Box<dyn std::error::Error>> {
    let key_path = "error_test.key";
    let depool_addr = deploy_depool(key_path)?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("call")
        .arg("--abi")
        .arg(DEPOOL_ABI)
        .arg(&depool_addr)
        .arg("error")
        .arg(r#"{"code":101}"#);
    cmd.assert()
        .stdout(predicate::str::contains(r#""exit_code": 101,"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("call")
        .arg("--abi")
        .arg(DEPOOL_ABI)
        .arg(&depool_addr)
        .arg("outOfGas")
        .arg(r#"{}"#);
    cmd.assert()
        .stdout(predicate::str::contains(r#""exit_code": -14,"#));

    fs::remove_file(key_path)?;
    Ok(())
}

#[test]
fn test_depool_body() -> Result<(), Box<dyn std::error::Error>> {
    let key_path = "depool_body.key";
    let depool_addr = deploy_depool(key_path)?;
    let wallet_addr = deploy_safe_msig(key_path)?;
    fs::remove_file(key_path)?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("body")
        .arg("--abi")
        .arg(DEPOOL_ABI)
        .arg("addOrdinaryStake")
        .arg(r#"{"stake":65535}"#);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("te6ccgEBAQEADgAAGAqsGP0AAAAAAAD//w=="));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(DEPOOL_ABI)
        .arg(&depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"stake": "0"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("call")
        .arg("--abi")
        .arg(SAFEMSIG_ABI)
        .arg("--sign")
        .arg(SAFEMSIG_SEED)
        .arg(&wallet_addr)
        .arg("sendTransaction")
        .arg(format!(r#"{{"dest":"{}","value":1000000000,"bounce":true,"flags":1,"payload":"te6ccgEBAQEADgAAGAqsGP0AAAAAAAD//w=="}}"#, &depool_addr));
    cmd.assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(DEPOOL_ABI)
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
    let key_path = "depool_1.key";
    let depool_addr = deploy_depool(key_path)?;
    let wallet_addr = deploy_safe_msig(key_path)?;
    fs::remove_file(key_path)?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("--addr")
        .arg(&depool_addr)
        .arg("replenish")
        .arg("--wallet")
        .arg(&wallet_addr)
        .arg("--value")
        .arg("2")
        .arg("--sign")
        .arg(SAFEMSIG_SEED);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"Succeeded."#));

    sleep(Duration::new(1, 0));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(DEPOOL_ABI)
        .arg(&depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"value": "2000000000"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("--addr")
        .arg(&depool_addr)
        .arg("ticktock")
        .arg("--wallet")
        .arg(&wallet_addr)
        .arg("--sign")
        .arg(SAFEMSIG_SEED);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"Succeeded."#));

    sleep(Duration::new(1, 0));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(DEPOOL_ABI)
        .arg(&depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"value": "1000000000"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("--addr")
        .arg(&depool_addr)
        .arg("events");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"StakeSigningRequested"#))
        .stdout(predicate::str::contains(r#"{"electionId":"1","proxy":"0:0000000000000000000000000000000000000000000000000000000000000002"}"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("--addr")
        .arg(&depool_addr)
        .arg("events")
        .arg("-w");
    cmd.assert()
        .success();

    Ok(())
}

#[test]
fn test_depool_2() -> Result<(), Box<dyn std::error::Error>> {
    let key_path = "depool_2.key";
    let depool_addr = deploy_depool(key_path)?;
    let wallet_addr = deploy_safe_msig(key_path)?;
    fs::remove_file(key_path)?;

    let config_path = "fee.conf";
    set_config(&["--depool_fee", "--url"], &["0.7", &*NETWORK], Some(config_path))?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("depool")
        .arg("--addr")
        .arg(&depool_addr)
        .arg("withdraw")
        .arg("off")
        .arg("--wallet")
        .arg(&wallet_addr)
        .arg("--sign")
        .arg(SAFEMSIG_SEED)
        .arg("--wait-answer");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"Answer status: SUCCESS"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(DEPOOL_ABI)
        .arg(&depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"reinvest": true"#))
        .stdout(predicate::str::contains(r#"value": "700000000"#));

    set_config(&["--depool_fee"], &["0.8"], Some(config_path))?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("depool")
        .arg("--addr")
        .arg(&depool_addr)
        .arg("withdraw")
        .arg("on")
        .arg("--wallet")
        .arg(&wallet_addr)
        .arg("--sign")
        .arg(SAFEMSIG_SEED)
        .arg("-a");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"Answer status: SUCCESS"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(DEPOOL_ABI)
        .arg(&depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"reinvest": false"#))
        .stdout(predicate::str::contains(r#"value": "800000000"#));

    fs::remove_file(config_path)?;
    Ok(())
}

#[test]
fn test_depool_3() -> Result<(), Box<dyn std::error::Error>> {
    let key_path = "depool_3.key";
    let depool_addr = deploy_depool(key_path)?;
    let wallet_addr = deploy_safe_msig(key_path)?;
    fs::remove_file(key_path)?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("--addr")
        .arg(&depool_addr)
        .arg("--wait-answer")
        .arg("stake")
        .arg("ordinary")
        .arg("--wallet")
        .arg(&wallet_addr)
        .arg("--sign")
        .arg(SAFEMSIG_SEED)
        .arg("--value")
        .arg("1");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"Answer status: TOTAL_PERIOD_MORE_18YEARS"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(DEPOOL_ABI)
        .arg(&depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(format!(r#"sender": "{}"#, &wallet_addr)))
        .stdout(predicate::str::contains(r#"stake": "1000000000"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("--addr")
        .arg(&depool_addr)
        .arg("stake")
        .arg("lock")
        .arg("--wallet")
        .arg(&wallet_addr)
        .arg("--sign")
        .arg(SAFEMSIG_SEED)
        .arg("--beneficiary")
        .arg(GIVER_V2_ADDR)
        .arg("--total")
        .arg("1")
        .arg("--withdrawal")
        .arg("1")
        .arg("--value")
        .arg("2");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"Succeeded."#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(DEPOOL_ABI)
        .arg(&depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(format!(r#"sender": "{}"#, &wallet_addr)))
        .stdout(predicate::str::contains(r#"stake": "2000000000"#))
        .stdout(predicate::str::contains(format!(r#"receiver": "{}"#, GIVER_V2_ADDR)))
        .stdout(predicate::str::contains(r#"withdrawal": "86400"#))
        .stdout(predicate::str::contains(r#"total": "86400"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("--addr")
        .arg(&depool_addr)
        .arg("stake")
        .arg("vesting")
        .arg("--wallet")
        .arg(&wallet_addr)
        .arg("--sign")
        .arg(SAFEMSIG_SEED)
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
        .stdout(predicate::str::contains(r#"Succeeded."#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(DEPOOL_ABI)
        .arg(&depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(format!(r#"sender": "{}"#, &wallet_addr)))
        .stdout(predicate::str::contains(r#"stake": "4000000000"#))
        .stdout(predicate::str::contains(r#"receiver": "0:0123456789012345012345678901234501234567890123450123456789012345"#))
        .stdout(predicate::str::contains(r#"withdrawal": "172800"#))
        .stdout(predicate::str::contains(r#"total": "172800"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("--addr")
        .arg(&depool_addr)
        .arg("stake")
        .arg("transfer")
        .arg("--wallet")
        .arg(&wallet_addr)
        .arg("--sign")
        .arg(SAFEMSIG_SEED)
        .arg("--dest")
        .arg(GIVER_V2_ADDR)
        .arg("--value")
        .arg("2");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"Succeeded."#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(DEPOOL_ABI)
        .arg(&depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(format!(r#"sender": "{}"#, &wallet_addr)))
        .stdout(predicate::str::contains(r#"stake": "2000000000"#))
        .stdout(predicate::str::contains(format!(r#"receiver": "{}"#, GIVER_V2_ADDR)));

    Ok(())
}

#[test]
fn test_depool_4() -> Result<(), Box<dyn std::error::Error>> {
    let key_path = "depool_4.key";
    let depool_addr = deploy_depool(key_path)?;
    let wallet_addr = deploy_safe_msig(key_path)?;
    fs::remove_file(key_path)?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("--addr")
        .arg(&depool_addr)
        .arg("stake")
        .arg("remove")
        .arg("--wallet")
        .arg(&wallet_addr)
        .arg("--sign")
        .arg(SAFEMSIG_SEED)
        .arg("--value")
        .arg("3");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"Succeeded."#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(DEPOOL_ABI)
        .arg(&depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(format!(r#"sender": "{}"#, &wallet_addr)))
        .stdout(predicate::str::contains(r#"stake": "3000000000"#))
        .stdout(predicate::str::contains(r#"value": "500000000"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("--addr")
        .arg(&depool_addr)
        .arg("stake")
        .arg("withdrawPart")
        .arg("--wallet")
        .arg(&wallet_addr)
        .arg("--sign")
        .arg(SAFEMSIG_SEED)
        .arg("--value")
        .arg("4");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"Succeeded."#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(DEPOOL_ABI)
        .arg(&depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(format!(r#"sender": "{}"#, &wallet_addr)))
        .stdout(predicate::str::contains(r#"stake": "4000000000"#))
        .stdout(predicate::str::contains(r#"value": "500000000"#));

    Ok(())
}

#[test]
fn test_depool_5() -> Result<(), Box<dyn std::error::Error>> {
    let key_path = "depool_5.key";
    let depool_addr = deploy_depool(key_path)?;
    let wallet_addr = deploy_safe_msig(key_path)?;
    fs::remove_file(key_path)?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("--addr")
        .arg(&depool_addr)
        .arg("donor")
        .arg("lock")
        .arg("--wallet")
        .arg(&wallet_addr)
        .arg("--sign")
        .arg(SAFEMSIG_SEED)
        .arg("--donor")
        .arg("0:0123456789012345012345678901234501234567890123450123456789012345");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"Succeeded."#));


    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(DEPOOL_ABI)
        .arg(&depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"receiver": "0:0123456789012345012345678901234501234567890123450123456789012345"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("--addr")
        .arg(&depool_addr)
        .arg("donor")
        .arg("vesting")
        .arg("--wallet")
        .arg(&wallet_addr)
        .arg("--sign")
        .arg(SAFEMSIG_SEED)
        .arg("--donor")
        .arg("0:0123456789012345012345678901234501234567890123450123456789012346");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"Succeeded."#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(DEPOOL_ABI)
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
        .stdout(predicate::str::contains(r#"version": "sol 0.52.0"#))
        .stdout(predicate::str::contains(r#"code_depth": "7"#))
        .stdout(predicate::str::contains(r#"data_depth": "2"#));

    Ok(())
}

#[test]
fn test_gen_deploy_message() -> Result<(), Box<dyn std::error::Error>> {
    let output = "test_gen_deploy_message_raw.out";
    let key_path = "gen_deploy_test.key";

    let addr = generate_key_and_address(key_path, SAFEMSIG_TVC, SAFEMSIG_ABI)?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("deploy_message")
        .arg(SAFEMSIG_TVC)
        .arg(SAFEMSIG_CONSTR_ARG)
        .arg("--abi")
        .arg(SAFEMSIG_ABI)
        .arg("--sign")
        .arg(key_path)
        .arg("--output")
        .arg(output)
        .arg("--raw");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(addr))
        .stdout(predicate::str::contains("Succeeded"));

    std::fs::remove_file(output)?;
    std::fs::remove_file(key_path)?;
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
        .stdout(predicate::str::contains(r#"version": "sol 0.52.0"#))
        .stdout(predicate::str::contains(r#"code_depth": "7"#))
        .stdout(predicate::str::contains(r#"data_depth": "2"#));

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
    cmd.arg("--url")
        .arg("main.ton.dev")
        .arg("dump")
        .arg("config")
        .arg(config_path)
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
    let config_path = "async_call.conf";
    set_config(&["--url", "--async_call"], &[&*NETWORK, "false"], Some(config_path))?;

    let time = now_ms();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("call")
        .arg("0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94")
        .arg("sendGrams")
        .arg(r#"{"dest":"0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94","amount":1000000000}"#)
        .arg("--abi")
        .arg(GIVER_ABI)
        .assert()
        .success()
        .stdout(predicate::str::contains("Succeeded"));

    let duration = now_ms() - time;

    set_config(&["--async_call"], &["true"], Some(config_path))?;

    let time = now_ms();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("call")
        .arg("0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94")
        .arg("sendGrams")
        .arg(r#"{"dest":"0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94","amount":1000000000}"#)
        .arg("--abi")
        .arg(GIVER_ABI)
        .assert()
        .success()
        .stdout(predicate::str::contains("Succeeded"));

    assert!(duration > now_ms() - time);

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("run")
        .arg("--abi")
        .arg(SAFEMSIG_ABI)
        .arg(SAFEMSIG_ADDR)
        .arg("getParameters")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#""maxQueuedTransactions": "5","#));

    set_config(&["--local_run", "--async_call"], &["true", "false"], Some(config_path))?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("run")
        .arg("--abi")
        .arg(SAFEMSIG_ABI)
        .arg(SAFEMSIG_ADDR)
        .arg("getParameters")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#""maxQueuedTransactions": "5","#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("call")
        .arg("--abi")
        .arg(GIVER_V2_ABI)
        .arg(GIVER_V2_ADDR)
        .arg("--sign")
        .arg(GIVER_V2_KEY)
        .arg("sendTransaction")
        .arg(format!(r#"{{"dest":"{}","value":100000000000,"bounce":false}}"#, GIVER_V2_ADDR))
        .assert()
        .success()
        .stdout(predicate::str::contains("Local run succeeded"));

    fs::remove_file(config_path)?;
    Ok(())
}

#[test]
fn test_multisig() -> Result<(), Box<dyn std::error::Error>> {
    let key_path = "msig_test.key";
    let setcode_msig_abi = "tests/samples/SetcodeMultisigWallet.abi.json";

    let _ = generate_phrase_and_key(key_path)?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    let out = cmd.arg("multisig")
        .arg("deploy")
        .arg("-k")
        .arg(key_path)
        .output()
        .expect("Failed to generate multisig address.");
    let output = &out.stdout;
    let mut addr = String::from_utf8_lossy(output).to_string();
    addr.replace_range(..addr.find("0:").unwrap_or(0), "");
    addr.replace_range(addr.find("Connecting").unwrap_or(addr.len())-1.., "");

    giver_v2(&addr);

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
        .arg(SAFEMSIG_ABI)
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
    let output = &out.stdout;
    let mut addr = String::from_utf8_lossy(output).to_string();
    addr.replace_range(..addr.find("0:").unwrap_or(0), "");
    addr.replace_range(addr.find("Connecting").unwrap_or(addr.len())-1.., "");
    giver_v2(&addr);

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
    let output = &out.stdout;
    let mut addr = String::from_utf8_lossy(output).to_string();
    addr.replace_range(..addr.find("0:").unwrap_or(0), "");
    addr.replace_range(addr.find("Connecting").unwrap_or(addr.len())-1.., "");

    giver_v2(&addr);

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

    let owners_string = format!(r#"{},{},{}"]"#, key1, key2, key3);

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("multisig")
        .arg("deploy")
        .arg("-k")
        .arg(key_path2)
        .arg("--owners")
        .arg(owners_string)
        .arg("--confirms")
        .arg("2")
        .arg("-l")
        .arg("5000000000")
        .assert()
        .success()
        .stdout(predicate::str::contains("Wallet successfully deployed"));

    let owners_string = format!(r#"0x{},"0x{}',"{}""]"#, key1, key2, key3);

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("multisig")
        .arg("deploy")
        .arg("-k")
        .arg(key_path3)
        .arg("--owners")
        .arg(owners_string)
        .arg("--confirms")
        .arg("2")
        .arg("-l")
        .arg("5000000000")
        .assert()
        .success()
        .stdout(predicate::str::contains("Wallet successfully deployed"));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(SAFEMSIG_ABI)
        .arg(addr.clone())
        .arg("getCustodians")
        .arg("{}")
        .assert()
        .success()
        .stdout(predicate::str::contains(key1))
        .stdout(predicate::str::contains(key2))
        .stdout(predicate::str::contains(key3));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(SAFEMSIG_ABI)
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
    let output = &out.stdout;
    let mut addr = String::from_utf8_lossy(output).to_string();
    addr.replace_range(..addr.find("0:").unwrap_or(0), "");
    addr.replace_range(addr.find("Connecting").unwrap_or(addr.len())-1.., "");

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("call")
        .arg(addr)
        .arg("sendTransaction")
        .arg(r#"{"dest":"0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94","value":100000,"bounce":"false","flags":1,"payload":""}"#)
        .arg("--abi")
        .arg(SAFEMSIG_ABI)
        .arg("--sign")
        .arg(key_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Succeeded"));

    fs::remove_file(key_path)?;
    Ok(())
}

#[test]
fn test_alternative_syntax() -> Result<(), Box<dyn std::error::Error>> {
    let boc_path = "tests/depool_acc.boc";

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("runx")
        .arg("--boc")
        .arg("--addr")
        .arg(boc_path)
        .arg("-m")
        .arg("getData")
        .arg("--abi")
        .arg(DEPOOL_ABI)
        .assert()
        .success()
        .stdout(predicate::str::contains("Succeeded."))
        .stdout(predicate::str::contains("Result: {"))
        .stdout(predicate::str::contains(r#""reinvest": false,"#));

    let key_path = "alternative_test.key";

    let address = generate_key_and_address(key_path, SAFEMSIG_TVC, SAFEMSIG_ABI)?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("callx")
        .arg("--keys")
        .arg(GIVER_V2_KEY)
        .arg("--abi")
        .arg(GIVER_V2_ABI)
        .arg("--addr")
        .arg(GIVER_V2_ADDR)
        .arg("-m")
        .arg("sendTransaction")
        .arg("--dest")
        .arg(address.clone())
        .arg("--value")
        .arg("1000000000")
        .arg("--bounce")
        .arg("false");
    cmd.assert()
        .success();

    let config_path = "alternative.config";
    set_config(
        &["--url", "--abi", "--addr", "--keys"],
        &[&*NETWORK, SAFEMSIG_ABI, &address, key_path],
        Some(config_path)
    )?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("deployx")
        .arg(SAFEMSIG_TVC)
        .arg("--owners")
        .arg(r#"["0xc8bd66f90d61f7e1e1a6151a0dbe9d8640666920d8c0cf399cbfb72e089d2e41"]"#)
        .arg("--reqConfirms")
        .arg("1");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(address))
        .stdout(predicate::str::contains("Transaction succeeded."));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("runx")
        .arg("-m")
        .arg("getParameters");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Succeeded."));

    set_config(&["--method"], &["getParameters"], Some(config_path))?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("runx");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Succeeded."));

    set_config(
        &["--method", "--keys", "--abi", "--addr", "--parameters"],
        &["sendTransaction", GIVER_V2_KEY, GIVER_V2_ABI, GIVER_V2_ADDR,
            &format!("{{\"dest\":\"{}\",\"value\":1000000000,\"bounce\":\"false\"}}", GIVER_V2_ADDR)],
        Some(config_path))?;

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("--config")
        .arg(config_path)
        .arg("callx");
    cmd.assert()
        .success();

    fs::remove_file(config_path)?;
    fs::remove_file(key_path)?;

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
