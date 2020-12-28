use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use lazy_static::*;
use std::env;

const BIN_NAME: &str = "tonos-cli";

lazy_static! {
    static ref NETWORK: String = env::var("TON_NETWORK_ADDRESS")
        .unwrap_or("http://127.0.0.1".to_string());
}

#[test]
fn test_config() -> Result<(), Box<dyn std::error::Error>> {
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
fn test_call_giver() -> Result<(), Box<dyn std::error::Error>> {
    let giver_abi_name = "tests/samples/giver.abi.json";
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--url")
        .arg(&*NETWORK)
        .assert()
        .success();
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
    cmd.arg("config")
        .arg("--url")
        .arg(&*NETWORK)
        .assert()
        .success();

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

#[test]
fn test_depool() -> Result<(), Box<dyn std::error::Error>> {
    let giver_abi_name = "tests/samples/giver.abi.json";
    let giver_addr = "0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94";
    let depool_abi = "tests/samples/fakeDepool.abi.json";
    let depool_tvc = "tests/samples/fakeDepool.tvc";
    let depool_addr = "0:cf72b41b704b7173467ffcd2c7bbc2a30d251996e3e3d848a74f9f72c8bc65e6";
    let msig_abi = "ton-labs-contracts/solidity/safemultisig/SafeMultisigWallet.abi.json";
    let msig_tvc = "ton-labs-contracts/solidity/safemultisig/SafeMultisigWallet.tvc";
    let msig_addr = "0:507fc74745d5a259b9939dfbdfd97cd186d13e8a7160206f3054746c1f0518cd";
    let seed_phrase = "blanket time net universe ketchup maid way poem scatter blur limit drill";

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--url")
        .arg(&*NETWORK)
        .assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("call")
        .arg("--abi")
        .arg(giver_abi_name)
        .arg(giver_addr)
        .arg("sendGrams")
        // use precalculated depool address
        .arg(r#"{"dest":"0:cf72b41b704b7173467ffcd2c7bbc2a30d251996e3e3d848a74f9f72c8bc65e6","amount":10000000000}"#);
    cmd.assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("deploy")
        .arg(depool_tvc)
        .arg("{}")
        .arg("--abi")
        .arg(depool_abi)
        .arg("--sign")
        .arg(seed_phrase);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(depool_addr))
        .stdout(predicate::str::contains("Transaction succeeded."));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("call")
        .arg("--abi")
        .arg(giver_abi_name)
        .arg(giver_addr)
        .arg("sendGrams")
        // use precalculated msig address
        .arg(r#"{"dest":"0:507fc74745d5a259b9939dfbdfd97cd186d13e8a7160206f3054746c1f0518cd","amount":30000000000}"#);
    cmd.assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("deploy")
        .arg(msig_tvc)
        .arg(r#"{"owners":["0xc8bd66f90d61f7e1e1a6151a0dbe9d8640666920d8c0cf399cbfb72e089d2e41"],"reqConfirms":1}"#)
        .arg("--abi")
        .arg(msig_abi)
        .arg("--sign")
        .arg(seed_phrase);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(msig_addr))
        .stdout(predicate::str::contains("Transaction succeeded."));

    Ok(())
}

#[test]
fn test_depool_1() -> Result<(), Box<dyn std::error::Error>> {
    let depool_abi = "tests/samples/fakeDepool.abi.json";
    let depool_addr = "0:cf72b41b704b7173467ffcd2c7bbc2a30d251996e3e3d848a74f9f72c8bc65e6";
    let msig_addr = "0:507fc74745d5a259b9939dfbdfd97cd186d13e8a7160206f3054746c1f0518cd";
    let seed_phrase = "blanket time net universe ketchup maid way poem scatter blur limit drill";
    
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--wallet")
        .arg(msig_addr)
        .arg("--addr")
        .arg(depool_addr)
        .assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("replenish")
        .arg("--value")
        .arg("2")
        .arg("--sign")
        .arg(seed_phrase);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"transId": "0"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(depool_abi)
        .arg(depool_addr)
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
        .stdout(predicate::str::contains(r#"transId": "0"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(depool_abi)
        .arg(depool_addr)
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

    Ok(())
}

#[test]
fn test_depool_2() -> Result<(), Box<dyn std::error::Error>> {
    let depool_abi = "tests/samples/fakeDepool.abi.json";
    let depool_addr = "0:cf72b41b704b7173467ffcd2c7bbc2a30d251996e3e3d848a74f9f72c8bc65e6";
    let seed_phrase = "blanket time net universe ketchup maid way poem scatter blur limit drill";
    
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
        .arg(seed_phrase);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"transId": "0"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(depool_abi)
        .arg(depool_addr)
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
        .arg(seed_phrase);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"transId": "0"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(depool_abi)
        .arg(depool_addr)
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
    let depool_addr = "0:cf72b41b704b7173467ffcd2c7bbc2a30d251996e3e3d848a74f9f72c8bc65e6";
    let seed_phrase = "blanket time net universe ketchup maid way poem scatter blur limit drill";
    
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
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
        .stdout(predicate::str::contains(r#"transId": "0"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(depool_abi)
        .arg(depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"sender": "0:507fc74745d5a259b9939dfbdfd97cd186d13e8a7160206f3054746c1f0518cd"#))
        .stdout(predicate::str::contains(r#"stake": "2000000000"#))
        .stdout(predicate::str::contains(r#"receiver": "0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94"#))
        .stdout(predicate::str::contains(r#"withdrawal": "86400"#))
        .stdout(predicate::str::contains(r#"total": "86400"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
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
        .stdout(predicate::str::contains(r#"transId": "0"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(depool_abi)
        .arg(depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"sender": "0:507fc74745d5a259b9939dfbdfd97cd186d13e8a7160206f3054746c1f0518cd"#))
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
        .stdout(predicate::str::contains(r#"transId": "0"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(depool_abi)
        .arg(depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"sender": "0:507fc74745d5a259b9939dfbdfd97cd186d13e8a7160206f3054746c1f0518cd"#))
        .stdout(predicate::str::contains(r#"stake": "2000000000"#))
        .stdout(predicate::str::contains(r#"receiver": "0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("stake")
        .arg("ordinary")
        .arg("--sign")
        .arg(seed_phrase)
        .arg("--value")
        .arg("1");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"transId": "0"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(depool_abi)
        .arg(depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"sender": "0:507fc74745d5a259b9939dfbdfd97cd186d13e8a7160206f3054746c1f0518cd"#))
        .stdout(predicate::str::contains(r#"stake": "1000000000"#));

    Ok(())
}

#[test]
fn test_depool_4() -> Result<(), Box<dyn std::error::Error>> {
    let depool_abi = "tests/samples/fakeDepool.abi.json";
    let depool_addr = "0:cf72b41b704b7173467ffcd2c7bbc2a30d251996e3e3d848a74f9f72c8bc65e6";
    let seed_phrase = "blanket time net universe ketchup maid way poem scatter blur limit drill";
    
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
        .stdout(predicate::str::contains(r#"transId": "0"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(depool_abi)
        .arg(depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"sender": "0:507fc74745d5a259b9939dfbdfd97cd186d13e8a7160206f3054746c1f0518cd"#))
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
        .stdout(predicate::str::contains(r#"transId": "0"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(depool_abi)
        .arg(depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"sender": "0:507fc74745d5a259b9939dfbdfd97cd186d13e8a7160206f3054746c1f0518cd"#))
        .stdout(predicate::str::contains(r#"stake": "4000000000"#))
        .stdout(predicate::str::contains(r#"value": "800000000"#));
    
    Ok(())
}

#[test]
fn test_depool_5() -> Result<(), Box<dyn std::error::Error>> {
    let depool_abi = "tests/samples/fakeDepool.abi.json";
    let depool_addr = "0:cf72b41b704b7173467ffcd2c7bbc2a30d251996e3e3d848a74f9f72c8bc65e6";
    let seed_phrase = "blanket time net universe ketchup maid way poem scatter blur limit drill";
    
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("donor")
        .arg("lock")
        .arg("--sign")
        .arg(seed_phrase)
        .arg("--donor")
        .arg("0:0123456789012345012345678901234501234567890123450123456789012345");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"transId": "0"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(depool_abi)
        .arg(depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"receiver": "0:0123456789012345012345678901234501234567890123450123456789012345"#));
    
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("donor")
        .arg("vesting")
        .arg("--sign")
        .arg(seed_phrase)
        .arg("--donor")
        .arg("0:0123456789012345012345678901234501234567890123450123456789012346");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"transId": "0"#));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("run")
        .arg("--abi")
        .arg(depool_abi)
        .arg(depool_addr)
        .arg("getData")
        .arg("{}");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"receiver": "0:0123456789012345012345678901234501234567890123450123456789012346"#));
    
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
        .stdout(predicate::str::contains(r#""amount":"0200000000""#))
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
        .arg("https://net.ton.dev")
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
    cmd.arg("--url")
        .arg("https://net.ton.dev")
        .arg("sendfile")
        .arg("call.boc");
    cmd.assert()
        .success();
    Ok(())
}


#[test]
fn test_account_command() -> Result<(), Box<dyn std::error::Error>> {
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
        .arg("0:3333333333333333333333333333333333333333333333333333333333333333");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Account not found"));
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
    cmd.arg("--json").arg("decode")
        .arg("msg").arg("tests/samples/wallet.boc")
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