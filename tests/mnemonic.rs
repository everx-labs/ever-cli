use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

mod common;
use common::create::BIN_NAME;

#[test]
fn test_has_mnemonic_checks() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("genpubkey")
        .arg("abuse boss fly battle rubber wasp afraid hamster guide essence vibrant tattoo");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Succeeded."))
        .stdout(predicate::str::contains(
            "Public key: 0cd34af58fc9cef235e1ad3aafb3d9c18c388b99c7089842eb9a49538e18d67d",
        ));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("genpubkey");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains(
            "The following required arguments were not provided:",
        ))
        .stderr(predicate::str::contains("<PHRASE>"));

    //just test check exits all other checks in test_invalid_mnemonic
    const WRONG_SEED: &str =
        "unit follow zone decline glare flower crisp vocal adapt magic much mesh cherry ";
    const WRONG_SEED_ERROR_TEXT: &str = "Invalid bip39 phrase";

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("genpubkey").arg(WRONG_SEED);
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains(WRONG_SEED_ERROR_TEXT));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("getkeypair")
        .arg("-o")
        .arg("test.json")
        .arg("-p")
        .arg(WRONG_SEED);
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains(WRONG_SEED_ERROR_TEXT));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("call")
        .arg("0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13")
        .arg("sendTransaction")
        .arg(r#"{"dest":"0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94","value":1000000000,"bounce":true}"#)
        .arg("--abi")
        .arg("./tests/samples/wallet.abi.json")
        .arg("--sign")
        .arg(WRONG_SEED);
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains(WRONG_SEED_ERROR_TEXT));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("callx")
        .arg("-m")
        .arg("sendTransaction")
        .arg("--addr")
        .arg("0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13")
        .arg("--abi")
        .arg("./tests/samples/wallet.abi.json")
        .arg("--keys")
        .arg(WRONG_SEED)
        .arg("--dest")
        .arg("0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94")
        .arg("--value")
        .arg("1000000000")
        .arg("--bounce")
        .arg("true");
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains(WRONG_SEED_ERROR_TEXT));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("deploy")
        .arg("tests/samples/wallet.tvc")
        .arg("{}")
        .arg("--abi")
        .arg("tests/samples/wallet.abi.json")
        .arg("--sign")
        .arg(WRONG_SEED);
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains(WRONG_SEED_ERROR_TEXT));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("config")
        .arg("--addr")
        .arg("0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("replenish")
        .arg("--wallet")
        .arg("0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13")
        .arg("--value")
        .arg("1000000")
        .arg("-s")
        .arg(WRONG_SEED);
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains(WRONG_SEED_ERROR_TEXT));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("stake")
        .arg("ordinary")
        .arg("--wallet")
        .arg("0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13")
        .arg("--value")
        .arg("1000000")
        .arg("-s")
        .arg(WRONG_SEED);
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains(WRONG_SEED_ERROR_TEXT));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("stake")
        .arg("lock")
        .arg("--wallet")
        .arg("0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13")
        .arg("--beneficiary")
        .arg("0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13")
        .arg("--value")
        .arg("1000000")
        .arg("--total")
        .arg("30")
        .arg("--withdrawal")
        .arg("30")
        .arg("--sign")
        .arg(WRONG_SEED);
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains(WRONG_SEED_ERROR_TEXT));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("stake")
        .arg("vesting")
        .arg("--wallet")
        .arg("0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13")
        .arg("--beneficiary")
        .arg("0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13")
        .arg("--value")
        .arg("1000000")
        .arg("--total")
        .arg("30")
        .arg("--withdrawal")
        .arg("30")
        .arg("--sign")
        .arg(WRONG_SEED);
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains(WRONG_SEED_ERROR_TEXT));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("stake")
        .arg("remove")
        .arg("--wallet")
        .arg("0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13")
        .arg("--value")
        .arg("1000000")
        .arg("--sign")
        .arg(WRONG_SEED);
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains(WRONG_SEED_ERROR_TEXT));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("stake")
        .arg("transfer")
        .arg("--wallet")
        .arg("0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13")
        .arg("--dest")
        .arg("0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13")
        .arg("--value")
        .arg("1000000")
        .arg("--sign")
        .arg(WRONG_SEED);
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains(WRONG_SEED_ERROR_TEXT));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("depool")
        .arg("stake")
        .arg("withdrawPart")
        .arg("--wallet")
        .arg("0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13")
        .arg("--value")
        .arg("1000000")
        .arg("--sign")
        .arg(WRONG_SEED);
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains(WRONG_SEED_ERROR_TEXT));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("multisig")
        .arg("send")
        .arg("--addr")
        .arg("0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13")
        .arg("--dest")
        .arg("0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13")
        .arg("--purpose")
        .arg("")
        .arg("--value")
        .arg("1000000")
        .arg("--sign")
        .arg(WRONG_SEED);
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains(WRONG_SEED_ERROR_TEXT));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("nodeid").arg("--keypair").arg(WRONG_SEED);
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains(WRONG_SEED_ERROR_TEXT));

    let mut cmd = Command::cargo_bin(BIN_NAME)?;
    cmd.arg("message")
        .arg("0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13")
        .arg("sendTransaction")
        .arg(r#"{"dest":"0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94","value":1000000000,"bounce":true}"#)
        .arg("--abi")
        .arg("tests/samples/wallet.abi.json")
        .arg("--lifetime")
        .arg("3600")
        .arg("--output")
        .arg("tests/samples/result.json")
        .arg("--sign")
        .arg(WRONG_SEED);
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains(WRONG_SEED_ERROR_TEXT));
    //TODO debot
    //TODO proposal
    Ok(())
}
