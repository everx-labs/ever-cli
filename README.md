# TONOS-CLI

TONOS-CLI is a multi-platform command line interface for TON OS.
It allows you to work with keys and seed phrases, deploy contracts, call any of their methods, generate and broadcast messages.
It supports specific commands for DeBot, DePool and Multisig contracts, as well as a number of supplementary functions.

To access built-in help, use `--help` or `-h` flag:

```bash
tonos-cli --help
tonos-cli <subcommand> -h
```

# Table of contents
- [TONOS-CLI](#tonos-cli)
- [Table of contents](#table-of-contents)
- [1. Installation](#1-installation)
  - [Install compiled executable](#install-compiled-executable)
    - [Ubuntu 22 troubleshooting](#ubuntu-22-troubleshooting)
  - [Install through EVERDEV](#install-through-everdev)
  - [Build from source](#build-from-source)
    - [Prerequisites](#prerequisites)
    - [Build from source on Linux and Mac OS](#build-from-source-on-linux-and-mac-os)
    - [Build from source on Windows](#build-from-source-on-windows)
    - [Windows debug build troubleshooting](#windows-debug-build-troubleshooting)
    - [Tails OS secure environment](#tails-os-secure-environment)
    - [Put TONOS-CLI into system environment](#put-tonos-cli-into-system-environment)
  - [Check version](#check-version)
  - [A note on Windows syntax](#a-note-on-windows-syntax)
- [2. Configuration](#2-configuration)
  - [2.1. Set the network and parameter values](#21-set-the-network-and-parameter-values)
  - [2.2. Check configuration](#22-check-configuration)
  - [2.3. Clear configuration](#23-clear-configuration)
  - [2.4. Configure endpoints map](#24-configure-endpoints-map)
  - [2.5. Override configuration file location](#25-override-configuration-file-location)
  - [2.6. Override network settings](#26-override-network-settings)
  - [2.7. Force json output](#27-force-json-output)
- [3. Cryptographic commands](#3-cryptographic-commands)
  - [3.1. Create seed phrase](#31-create-seed-phrase)
  - [3.2. Generate public key](#32-generate-public-key)
  - [3.3. Generate key pair file](#33-generate-key-pair-file)
- [4. Smart contract commands](#4-smart-contract-commands)
  - [4.1. Generate contract address](#41-generate-contract-address)
  - [4.2. Deploy contract](#42-deploy-contract)
  - [4.3. Generate deploy message offline](#43-generate-deploy-message-offline)
  - [4.3. Get contract status](#43-get-contract-status)
  - [4.4. Call method](#44-call-method)
    - [4.4.1. Call contract on the blockchain](#441-call-contract-on-the-blockchain)
    - [4.4.2. Run contract method locally](#442-run-contract-method-locally)
    - [4.4.3. Run funC get-method](#443-run-func-get-method)
    - [4.4.4. Run contract method locally for saved account BOC](#444-run-contract-method-locally-for-saved-account-boc)
  - [4.5. Generate encrypted message offline](#45-generate-encrypted-message-offline)
  - [4.6. Broadcast previously generated message](#46-broadcast-previously-generated-message)
  - [4.7. Broadcast previously generated message from a file](#47-broadcast-previously-generated-message-from-a-file)
  - [4.8. Decode commands](#48-decode-commands)
    - [4.8.1. Decode BOC file](#481-decode-boc-file)
    - [4.8.2. Decode message body](#482-decode-message-body)
    - [4.8.3. Decode account commands](#483-decode-account-commands)
      - [4.8.3.1. Decode account data fields](#4831-decode-account-data-fields)
      - [4.8.3.2. Decode data from the account BOC file](#4832-decode-data-from-the-account-boc-file)
    - [4.8.4. Decode stateInit fields](#484-decode-stateinit-fields)
  - [4.9. Generate payload for internal function call](#49-generate-payload-for-internal-function-call)
  - [4.10. Alternative syntax for call, deploy and run commands](#410-alternative-syntax-for-call-deploy-and-run-commands)
- [5. DeBot commands](#5-debot-commands)
- [6. Multisig commands](#6-multisig-commands)
  - [6.1. Send tokens](#61-send-tokens)
  - [6.2. Deploy wallet](#62-deploy-wallet)
- [7. DePool commands](#7-depool-commands)
  - [7.1. Configure TONOS-CLI for DePool operations](#71-configure-tonos-cli-for-depool-operations)
  - [7.2. Deposit stakes](#72-deposit-stakes)
    - [7.2.1. Ordinary stake](#721-ordinary-stake)
    - [7.2.2. Vesting stake](#722-vesting-stake)
    - [7.2.3. Lock stake](#723-lock-stake)
  - [7.3. Remove stakes](#73-remove-stakes)
  - [7.4. Transfer stakes](#74-transfer-stakes)
  - [7.5. Withdraw Stakes](#75-withdraw-stakes)
    - [7.5.1. Withdraw entire stake](#751-withdraw-entire-stake)
    - [7.5.2. Withdraw part of the stake](#752-withdraw-part-of-the-stake)
  - [7.6. Reinvest Stakes](#76-reinvest-stakes)
  - [7.7. Read DePool answers](#77-read-depool-answers)
  - [7.8. View DePool events](#78-view-depool-events)
  - [7.9. Replenish DePool balance](#79-replenish-depool-balance)
  - [7.10. Send ticktock to DePool](#710-send-ticktock-to-depool)
- [8. Proposal commands](#8-proposal-commands)
  - [8.1. Create proposal and cast the first vote](#81-create-proposal-and-cast-the-first-vote)
  - [8.2. Vote for proposal](#82-vote-for-proposal)
  - [8.3. Decode proposal comment](#83-decode-proposal-comment)
- [9. Supplementary commands](#9-supplementary-commands)
  - [9.1. Get global config](#91-get-global-config)
  - [9.2. NodeID](#92-nodeid)
  - [9.3. Dump blockchain config](#93-dump-blockchain-config)
  - [9.4. Dump several account states](#94-dump-several-account-states)
  - [9.5. Update global config parameter](#95-update-global-config-parameter)
  - [9.6. Wait for an account change](#96-wait-for-an-account-change)
  - [9.7 Make a raw GraphQL query](#97-make-a-raw-graphql-query)
- [10. Fetch and replay](#10-fetch-and-replay)
  - [10.1. How to unfreeze account](#101-how-to-unfreeze-account)
- [11. Debug commands](#11-debug-commands)
  - [11.1. Debug transaction](#111-debug-transaction)
  - [11.2. Debug call](#112-debug-call)
  - [11.3. Debug run](#113-debug-run)
  - [11.4. Debug replay transaction on the saved account state](#114-debug-replay-transaction-on-the-saved-account-state)
  - [11.5. Debug deploy](#115-debug-deploy)
  - [11.6. Debug message](#116-debug-message)
  - [11.7. Render UML sequence diagram](#117-render-uml-sequence-diagram)

# 1. Installation

## Install compiled executable

Create a folder. Download the `.zip` file from the latest release from here: [https://github.com/tonlabs/tonos-cli/releases](https://github.com/tonlabs/tonos-cli/releases) to this folder. Extract it.

### Ubuntu 22 troubleshooting

Ubuntu 22 has upgraded to OpenSSL 3.0 and this breaks execution of compiled tonos-cli releases. To fix this problem one
should install old version of libssl. To do it one can download amd64 package from
(packages.debian.org)[https://packages.debian.org/stretch/libssl1.1] and install it with dpkg:

```bash
sudo dpkg -i libssl1.1*.deb
```

## Install through EVERDEV

You can use [EVERDEV](https://github.com/tonlabs/everdev) to install the latest version of TONOS-CLI.

```bash
everdev tonos-cli install
```

The installer requires [NPM](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm) to be installed, so it can install packages globally without using sudo. In case of error, manually set environment variable `PATH=$PATH:$HOME./everdev/solidity`

This command updates TONOS-CLI installed through EVERDEV to the latest version:

```bash
everdev tonos-cli update
```

This command specifies TONOS-CLI version to use and downloads it if needed:

```bash
everdev tonos-cli set --version 0.8.0
```

## Build from source

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) latest version
- OpenSSL

For Linux:

```bash
sudo apt-get install libssl-dev (openssl-devel on Fedora)
sudo apt-get install pkg-config
```

### Build from source on Linux and Mac OS

Install Cargo: [https://github.com/rust-lang/cargo#compiling-from-source](https://github.com/rust-lang/cargo#compiling-from-source)

Build TONOS-CLI tool from source:

```bash
git clone https://github.com/tonlabs/tonos-cli.git
cd tonos-cli
cargo update
cargo build --release
cd target/release
```

The `tonos-cli` executable is built in the `tonos-cli/target/release` folder. Create a folder elsewhere. Copy the `tonos-cli` executable into the new folder you have created.

### Build from source on Windows

Install Cargo: [https://github.com/rust-lang/cargo#compiling-from-source](https://github.com/rust-lang/cargo#compiling-from-source)

Build TONOS-CLI tool from source:

```bash
> git clone https://github.com/tonlabs/tonos-cli.git
> cd tonos-cli
> cargo update
> cargo build --release
> cd target/release
```

The `tonos-cli` executable is built in the `tonos-cli/target/release` folder. Create a folder elsewhere. Copy the `tonos-cli` executable into the new folder you have created.

### Windows debug build troubleshooting

Default debug executable built after `cargo build` command may have an issue with binary default stack size:

```bash
> cargo build
Finished dev [unoptimized + debuginfo] target(s) in 0.66s
> .\target\debug\tonos-cli.exe --version

thread 'main' has overflowed its stack
```

User can fix this issue by using [editbin tool from MSVC Tools](https://docs.microsoft.com/ru-ru/cpp/build/reference/editbin-reference?view=msvc-170). This tool allows user to
increase binary stack reserve. Increase it by 2 times will help to fix tonos-cli:

```bash
> editbin /STACK:2097152 tonos-cli.exe
Microsoft (R) COFF/PE Editor Version 14.28.29914.0
Copyright (C) Microsoft Corporation.  All rights reserved.

> tonos-cli.exe --version
tonos_cli 0.26.7
COMMIT_ID: 1e1397b5561ea79d2fd7cce47cd033450b123f25
BUILD_DATE: Unknown
COMMIT_DATE: 2022-05-13 14:15:47 +0300
GIT_BRANCH: master
```

### Tails OS secure environment

For maximum security while working with offline TONOS-CLI features (such as cryptographic commands or encrypted message generation), you can use the [Tails OS](https://tails.boum.org/).

### Put TONOS-CLI into system environment

Optional, Linux/Mac OS. Use the following command to put the utility into system environment:

```bash
export PATH="<tonos_folder_path>:$PATH"
```

This step can be skipped, if TONOS-CLI was installed through EVERDEV. Otherwise, if you skip this step, make sure you always run the utility from folder containing the utility:

```bash
./tonos-cli <command> <options>
```

## Check version

You can check version of the current TONOS-CLI installation with the following command:

```bash
tonos-cli version
```

Output example:

```bash
$ tonos-cli version
Config: default
tonos-cli 0.2.0
COMMIT_ID: 21ebd53c35bf22696bf1eb434e408ed33318136a
BUILD_DATE: 2021-01-26 15:06:18 +0300
COMMIT_DATE: 2021-01-14 16:13:32 +0300
GIT_BRANCH: master
```

## A note on Windows syntax

When using Windows command line, the following syntax should be used for all TONOS-CLI commands:

1. Never use the `./` symbols before `tonos-cli`:

```bash
> tonos-cli <command_name> <options>
```

2. For all commands with nested quotes, the outer single quotes should be changed to double quotes, and the inner double quotes should be shielded by a preceding `\`. Example:

```bash
> tonos-cli deploy SafeMultisigWallet.tvc "{\"owners\":[\"0x723b2f0fa217cd10fe21326634e66106678f15d5a584babe4f576dffe9dcbb1b\",\"0x127e3ca223ad429ddaa053a39fecd21131df173bb459a4438592493245b695a3\",\"0xc2dd3682ffa9df97a968bef90b63da90fc92b22163f558b63cb7e52bfcd51bbb\"],\"reqConfirms\":2}" --abi SafeMultisigWallet.abi.json --sign deploy.keys.json
```

If this is not done, `arguments are not in json format: key must be a string at line 1 column` error may occur.

# 2. Configuration

## 2.1. Set the network and parameter values

TONOS-CLI can remember some parameter values and use it automatically in various subcommands.

After that you can omit the corresponding parameters in subsequent subcommands.

`tonos-cli.config.json` configuration file will be created in the current working directory. All subsequent calls of the utility will use this file.

Use the following command to create a configuration file:

```bash
tonos-cli config <--option> <option_value>
```

All other TONOS-CLI commands will indicate the configuration file currently used.

List of available options:

```bash
--abi <ABI>                                   Path to the contract ABI file.
--addr <ADDR>                                 Contract address.
--async_call <ASYNC_CALL>                     Disables wait for transaction to appear in the network after call command.
--balance_in_tons <BALANCE_IN_TONS>           Print balance for account command in tons. If false balance is printed in nanotons.
--debug_fail <DEBUG_FAIL>                     When enabled tonos-cli executes debug command on fail of run or call command. Can be enabled with values 'full' or 'minimal' which set the trace level for debug run and disabled with value 'none'.
--depool_fee <DEPOOL_FEE>                     Value added to the message sent to depool to cover its fees (change will be returned).
--is_json <IS_JSON>                           Cli prints output in json format.
--keys <KEYS>                                 Path to the file with keypair.
--lifetime <LIFETIME>                         Period of time in seconds while message is valid. Change of this parameter may affect "out_of_sync" parameter, because "lifetime" should be at least 2 times greater than "out_of_sync".
--local_run <LOCAL_RUN>                       Enable preliminary local run before deploy and call commands.
--method <METHOD>                             Method name that can be saved to be used by some commands (runx, callx).
--message_processing_timeout <MSG_TIMEOUT>    Network message processing timeout in ms.
--no-answer <NO_ANSWER>                       Flag whether to wait for depool answer when calling a depool function.
--out_of_sync <OUT_OF_SYNC>                   Network connection "out_of_sync_threshold" parameter in seconds. Mind that it cant exceed half of the "lifetime" parameter.
--parameters <PARAMETERS>                     Function parameters that can be saved to be used by some commands (runx, callx).
--pubkey <PUBKEY>                             User public key. Used by DeBot Browser.
--retries <RETRIES>                           Number of attempts to call smart contract function if previous attempt was unsuccessful.
--timeout <TIMEOUT>                           Network `wait_for` timeout in ms.
--url <URL>                                   Url to connect.
--wallet <WALLET>                             Multisig wallet address.
--wc <WC>                                     Workchain id.
```

Example:

```bash
$ tonos-cli config --url https://main.ton.dev --wc -1 --keys key.json --abi SafeMultisigWallet.abi.json --lifetime 3600 --local_run true --retries 3 --timeout 600
Config: /home/user/TONLabs/tonos-cli/tonos-cli.conf.json
Succeeded.
{
  "url": "main.ton.dev",
  "wc": -1,
  "addr": null,
  "wallet": null,
  "pubkey": null,
  "abi_path": "SafeMultisigWallet.abi.json",
  "keys_path": "key.json",
  "retries": 3,
  "timeout": 600,
  "message_processing_timeout": 40000,
  "out_of_sync_threshold": 15,
  "is_json": false,
  "depool_fee": 0.5,
  "lifetime": 3600,
  "no_answer": true,
  "balance_in_tons": false,
  "local_run": true,
  "async_call": false,
  "debug_fail": "None",
  "endpoints": [
    "https://eri01.main.everos.dev",
    "https://gra01.main.everos.dev",
    "https://gra02.main.everos.dev",
    "https://lim01.main.everos.dev",
    "https://rbx01.main.everos.dev"
  ]
}
```

Some frequently used networks:

`https://net.ton.dev` - developer sandbox for testing. TONOS-CLI connects to it by default.

`https://main.ton.dev` - main Free TON network.

`https://rustnet.ton.dev` - test network running on Rust nodes.

TONOS-CLI supports the use of multiple endpoints for networks: if several endpoints are [specified in the endpoint map](#24-configure-endpoints-map) for a network, TONOS-CLI will use them all when accessing it. Otherwise, the network URL will be treated as the only endpoint.

`main.ton.dev` and `net.ton.dev` networks already have their current endpoints specified in the default endpoint map.
See [section 2.4 below](#24-configure-endpoints-map) on how to edit and add endpoints to the endpoint map.

> **Note**: This change was introduced in version 0.16.1 and is fully compatible with scripts written for previous versions, where main.ton.dev and net.ton.dev networks were specified with a single url. TONOS-CLI will simply use the default endpoint map to access these networks.


Network configuration can be [overridden](#26-override-network-settings) for any single subcommand.

To connect to a [DApp Server](https://github.com/tonlabs/TON-OS-DApp-Server) you are running, it should have domain name and a DNS record. Then its URL may be used to access it with TONOS-CLI:

```bash
tonos-cli config --url <dapp_server_url>
```

> Note: Either run tonos-cli utility only from the directory where tonos-cli.config.json is placed, or use one of the available methods (see [section 2.5](#25-override-configuration-file-location)) to make the utility look for the file elsewhere.

## 2.2. Check configuration

You can check the current configuration parameters with the following command:

```bash
tonos-cli config --list
```

## 2.3. Clear configuration

Use the following command to reset configuration to default values:

```bash
tonos-cli config clear
```

## 2.4. Configure endpoints map

TONOS-CLI config file also stores an endpoints map that can be updated by the user.
Each time user [changes the url](#21-set-the-network-and-parameter-values), endpoints also change in accordance to endpoints map.
To print the map use the following command:

```bash
tonos-cli config endpoint print
```

User can reset map to the default state:

```bash
tonos-cli config endpoint reset
```

Default state of the map:

```bash
{
  "net.ton.dev": [
    "https://eri01.net.everos.dev",
    "https://rbx01.net.everos.dev"
    "https://gra01.net.everos.dev"
  ],
  "main.ton.dev": [
    "https://eri01.main.everos.dev",
    "https://gra01.main.everos.dev",
    "https://gra02.main.everos.dev",
    "https://lim01.main.everos.dev",
    "https://rbx01.main.everos.dev"
  ],
  "http://127.0.0.1/": [
    "http://0.0.0.0/",
    "http://127.0.0.1/",
    "http://localhost/"
  ]
}
```

Map can be changed with `remove` and `add` subcommands:

```bash
tonos-cli config endpoint remove <url>
tonos-cli config endpoint add <url> <list_of_endpoints>
```

Example:

```bash
tonos-cli config endpoint remove main.ton.dev
tonos-cli config endpoint add main.ton.dev "https://eri01.main.everos.dev","https://gra01.main.everos.dev","https://gra02.main.everos.dev","https://lim01.main.everos.dev","https://rbx01.main.everos.dev"
```

> **Note**: If url used in add command already exists, endpoints lists will be merged.

If a network that doesn't have mapped endpoints is [specified in the config file](#21-set-the-network-and-parameter-values), its url will be automatically treated as the only endpoint. For example, configuring TONOS-CLI to connect to RustNet with the command `tonos-cli config --url https://rustnet.ton.dev` will result in TONOS-CLI using this url as a single endpoint, without the user having to specify it in the endpoints map additionally.


## 2.5. Override configuration file location

You can move the `tonos-cli.config.json` configuration file to any other convenient location and/or rename it. There are several ways you can point the utility to the new location of the file:

- **define environment variable** `TONOSCLI_CONFIG` with the path to your configuration file:

```bash
export TONOSCLI_CONFIG=<path_to_config_file>
```

Example:

```bash
export TONOSCLI_CONFIG=/home/user/config.json
```

- **define global option** `--config <path_to_config_file>` before any other subcommand:

```bash
tonos-cli --config <path_to_config_file> <any_subcommand>
```

Example:

```bash
tonos-cli --config /home/user/config.json account <address>
```

The `--config` global option has higher priority than the `TONOSCLI_CONFIG` environment variable.

> Note: You cannot use the config subcommand to create or edit a configuration file located outside the current working directory. It should either be called from the directory containing the file, or the file should be manually copied to the desired directory later.

> However, config --list subcommand displays the parameters of the currently used configuration file, wherever it is located.

## 2.6. Override network settings

You can also separately override [preconfigured network settings](#21-set-the-network-and-parameter-values) for a single subcommand. Use the `--url <network_url>` global option for this purpose:

```bash
tonos-cli --url <network_url> <any_subcommand>
```

Example:

```bash
tonos-cli --url https://main.ton.dev account <address>
```

## 2.7. Force json output

You can force TONOS-CLi to print output in json format. To do so, add `--json` flag before a subcommand:

```bash
tonos-cli --json <any_subcommand>
```

## 2.8. Debug on fail option

You can force TONOS-CLi to debug call and run executions if they fail with error code 414.

```bash
tonos-cli config --debug_fail <trace_level>
```

Possible <trace_level> values:
- 'full'
- 'minimal'
- 'none'

# 3. Cryptographic commands

## 3.1. Create seed phrase

To generate a mnemonic seed phrase enter the following command:

```bash
tonos-cli genphrase
```

Example:

```bash
$ tonos-cli genphrase
Config: /home/user/tonos-cli.conf.json
Succeeded.
Seed phrase: "rule script joy unveil chaos replace fox recipe hedgehog heavy surge online"
```

## 3.2. Generate public key

To generate a public key from a seed phrase enter the following command with the seed phrase in quotes:

```bash
tonos-cli genpubkey "rule script joy unveil chaos replace fox recipe hedgehog heavy surge online"
```

The generated QR code also contains the public key.

Example:

```bash
$ tonos-cli genpubkey "rule script joy unveil chaos replace fox recipe hedgehog heavy surge online"
Config: /home/user/tonos-cli.conf.json
Succeeded.
Public key: 88c541e9a1c173069c89bcbcc21fa2a073158c1bd21ca56b3eb264bba12d9340

<QR code with key>

```

## 3.3. Generate key pair file

To create a key pair file from a seed phrase use the following command:

```bash
tonos-cli getkeypair [-o <keyfile.json>] [-p "<seed_phrase>"]
```

`<keyfile.json>` - the file the key pair will be written to. If not specified keys will be printed to the stdout.
`"<seed_phrase>"` - seed phrase or secret key. If not specified a new phrase will be generated.
Example:

```bash
$ tonos-cli getkeypair -o key.json -p "rule script joy unveil chaos replace fox recipe hedgehog heavy surge online"
Config: /home/user/TONLabs/tonos-cli/tonos-cli.conf.json
Input arguments:
key_file: key.json
  phrase: rule script joy unveil chaos replace fox recipe hedgehog heavy surge online
Keypair successfully saved to key.json.
Succeeded.

$ tonos-cli getkeypair -o key.json 
Config: /home/user/TONLabs/tonos-cli/tonos-cli.conf.json
Input arguments:
key_file: key.json
  phrase: None
Generating seed phrase.
Seed phrase: "elephant tone error jazz scrap wise kick walk panda snake right feature"
Keypair successfully saved to key.json.
Succeeded.


$ tonos-cli getkeypair 
Config: /home/user/TONLabs/tonos-cli/tonos-cli.conf.json
Input arguments:
key_file: None
  phrase: None
Generating seed phrase.
Seed phrase: "behave early mammal cart grape wolf pulse once helmet shop kit this"
Keypair: {
  "public": "d5218be1502c98019a2c08ae588f73abd56b4c72411e8d2ee37e5c2d821e075f",
  "secret": "842bd2b9df2ec4ed07b6b66d6d0c2858769ba4ed9005ffe58cba26783504a3ff"
}
Succeeded.

$ tonos-cli -j getkeypair
{
  "public": "09889cd2f085a693ef04a6dad4b6533c7019014a7e0ca9b5b146e66e550973d9",
  "secret": "021196259435d54dfb5c41970db5bcfc2306d59877665c3b573486d441cf021a"
}
```

# 4. Smart contract commands

When working with smart contracts, TONOS-CLI requires the following files:

- **ABI file** - a .json file that describes the contract interface, the methods and parameters used to interact with it.
- **TVC file** - the compiled smart contract file. Used only when generating contract address and deploying contract code to the blockchain.
- **Key pair file** - used in contracts with implemented authorization. It is the file containing [private and public keys](#3-cryptographic-commands) authorized to access the contract. In `--sign` parameter the corresponding seed phrase may be used instead of it.

By default, the utility looks for these files in the current working directory.

## 4.1. Generate contract address

Contract address uniquely identifies the contract on the blockchain. Contract balance is attached to its address, the address is used for any interactions with the contract, such as calling contract functions, sending messages, etc.

Contract address is generated based on contract TVC file and selected keys. To get a different address for the same type of contract, use different keys.

> **Note**: If your contract has static variables, they can be initialized through [TVM linker](https://github.com/tonlabs/TVM-linker#5-initialize-static-variables-in-compiled-contract) before deployment.

Use the following command to generate the contract address:

```bash
tonos-cli genaddr [--genkey|--setkey <keyfile.json>] [--wc <int8>] [--abi <contract.abi.json>] <contract.tvc>
```

- `--genkey <keyfile.json>` - generate new `keyfile.json` key pair file and use it to calculate the contract address.

> Note: if you use --genkey, the corresponding seed phrase will be displayed. Write it down, if you mean to keep using this key pair.

- `--abi <contract.abi.json>` - contract ABI interface file. If not specified tonos-cli can use ABI path from config of obtained from tvc path (for `<contrac>.tvc` checks `<contract>.abi.json`).
- `--setkey <keyfile.json>` - use already [existing](#33-generate-key-pair-file) `keyfile.json` key pair file to calculate the contract address. Seed phrase cannot be used instead of the file.
- `--wc <int8>`  ID of the workchain the contract will be deployed to (`-1` for masterchain, `0` for basechain). By default, this value is set to 0.

  `<contract.tvc>` - compiled smart contract file.


As result the utility displays the new contract address (`Raw address`).

Example ([multisignature wallet](https://github.com/tonlabs/ton-labs-contracts/tree/master/solidity/safemultisig) address generation for the masterchain):

```bash
$ tonos-cli genaddr --genkey key.json --wc -1 SafeMultisigWallet.tvc --abi SafeMultisigWallet.abi.json
Config: /home/user/tonos-cli.conf.json
Input arguments:
     tvc: SafeMultisigWallet.tvc
      wc: -1
    keys: key.json
init_data: None
is_update_tvc: None

Seed phrase: "chimney nice diet engage hen sing vocal upgrade column address consider word"
Raw address: -1:a021414a79539001ed35d615a646dc8b89df29ccccf143c30df15c7fbcaff086
testnet:
Non-bounceable address (for init): 0f-gIUFKeVOQAe011hWmRtyLid8pzMzxQ8MN8Vx_vK_whkeM
Bounceable address (for later access): kf-gIUFKeVOQAe011hWmRtyLid8pzMzxQ8MN8Vx_vK_whhpJ
mainnet:
Non-bounceable address (for init): Uf-gIUFKeVOQAe011hWmRtyLid8pzMzxQ8MN8Vx_vK_whvwG
Bounceable address (for later access): Ef-gIUFKeVOQAe011hWmRtyLid8pzMzxQ8MN8Vx_vK_whqHD
Succeeded
```

## 4.2. Deploy contract

> **Note**: If your contract has static variables, they can be initialized through [TVM linker](https://github.com/tonlabs/TVM-linker#5-initialize-static-variables-in-compiled-contract) before deployment.

Use the following command to deploy a contract:

```bash
tonos-cli deploy [--sign <deploy_seed_or_keyfile>] [--wc <int8>] [--abi <contract.abi.json>] <contract.tvc> <params>
```

`<deploy_seed_or_keyfile>` - can either be the seed phrase used to generate the deployment key pair file or the key pair file itself. If seed phrase is used, enclose it in double quotes.

Example:

- `--sign "flip uncover dish sense hazard smile gun mom vehicle chapter order enact"`

or

- `--sign deploy.keys.json`
- `--wc <int8>` ID of the workchain the wallet will be deployed to (`-1` for masterchain, `0` for basechain). By default, this value is set to 0.

`<contract.abi.json>` - contract interface file.

`<contract.tvc>` - compiled smart contract file.

`<params>` - deploy command parameters, depend on the contract.
Sometimes it can be not obvious in which way method parameters should be specified,
especially if it is a large structure with different and complex fields.
It is generally described in [abi doc](https://github.com/tonlabs/ton-labs-abi/blob/master/docs/ABI_2.1_spec.md).
Example ([multisignature wallet](https://github.com/tonlabs/ton-labs-contracts/tree/master/solidity/safemultisig) contract deployment to the masterchain):

```bash
$ tonos-cli deploy --sign key.json --wc -1 --abi SafeMultisigWallet.abi.json SafeMultisigWallet.tvc '{"owners":["0x88c541e9a1c173069c89bcbcc21fa2a073158c1bd21ca56b3eb264bba12d9340"],"reqConfirms":1}'
Config: /home/user/tonos-cli.conf.json
Input arguments:
     tvc: SafeMultisigWallet.tvc
  params: {"owners":["0x88c541e9a1c173069c89bcbcc21fa2a073158c1bd21ca56b3eb264bba12d9340"],"reqConfirms":1}
     abi: SafeMultisigWallet.abi.json
    keys: key.json
      wc: -1
Connecting to net.ton.dev
Deploying...
Transaction succeeded.
Contract deployed at address: -1:0c5d5215317ec8eef1b84c43cbf08523c33f69677365de88fe3d96a0b31b59c6
```

## 4.3. Generate deploy message offline

If needed, signed deploy message can be generated without immediately broadcasting it to the blockchain. Generated message can be broadcasted later<link to broadcast section>.

```bash
tonos-cli deploy_message [--raw] [--output <path_to_file>] [--sign <deploy_seed_or_keyfile>] [--wc <int8>] [--abi <contract.abi.json>] <contract.tvc> <params>
```

`--raw` - use to create raw message boc.

`--output <path_to_file>` - specify path to file where the raw message should be written to, instead of printing it to terminal.

`<deploy_seed_or_keyfile>` - can either be the seed phrase used to generate the deployment key pair file or the key pair file itself. If seed phrase is used, enclose it in double quotes.

Example:

- `--sign "flip uncover dish sense hazard smile gun mom vehicle chapter order enact"`

or

- `--sign deploy.keys.json`
- `--wc <int8>` ID of the workchain the wallet will be deployed to (`-1` for masterchain, `0` for basechain). By default, this value is set to 0.

`<contract.abi.json>` - contract interface file.

`<contract.tvc>` - compiled smart contract file.

`<params>` - deploy command parameters, depend on the contract.

Example (saving to a file [multisignature wallet](https://github.com/tonlabs/ton-labs-contracts/tree/master/solidity/safemultisig) contract deployment message to the masterchain):

```bash
$ tonos-cli deploy_message --raw --output deploy.boc --sign key.json --wc -1 --abi SafeMultisigWallet.abi.json SafeMultisigWallet.tvc '{"owners":["0x88c541e9a1c173069c89bcbcc21fa2a073158c1bd21ca56b3eb264bba12d9340"],"reqConfirms":1}'
Config: /home/user/tonos-cli.conf.json
Input arguments:
     tvc: SafeMultisigWallet.tvc
  params: {"owners":["0x88c541e9a1c173069c89bcbcc21fa2a073158c1bd21ca56b3eb264bba12d9340"],"reqConfirms":1}
     abi: SafeMultisigWallet.abi.json
    keys: key.json
      wc: -1

MessageId: 51da1b8840bd12f9ef5152639bd1fe9062d77ed91829301043bb85b4a4d610ea
Expire at: unknown
Message saved to file deploy.boc
Contract's address: -1:0c5d5215317ec8eef1b84c43cbf08523c33f69677365de88fe3d96a0b31b59c6
Succeeded.
```

## 4.3. Get contract status

You may use the following command to check the current status of a contract:

```bash
tonos-cli account <list_of_addresses> [--dumptvc <tvc_path>] [--dumpboc <boc_path>]
```

`<list_of_addresses>` - contract [addresses](#41-generate-contract-address).
`--dumptvc <tvc_path>` - this flag can be specified to dump account StateInit to the <tvc_path> file.
`--dumpboc <boc_path>` - this flag can be specified to dump account boc to the <boc_path> file.

Example:

```bash
$ tonos-cli  account 0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13 0:14014af4a374bdd13dae2379063ea2597634c2c2fc8e99ca9eab431a7ab6f566  0:f89d946b5b4b8a06f01dc20dceef30caff844d5285abea8a21ad3730c0f3dd12
Config: /home/user/TONLabs/tonos-cli/tonos-cli.conf.json
Input arguments:
addresses: 0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13, 0:14014af4a374bdd13dae2379063ea2597634c2c2fc8e99ca9eab431a7ab6f566, 0:f89d946b5b4b8a06f01dc20dceef30caff844d5285abea8a21ad3730c0f3dd12
Connecting to net.ton.dev
Processing...
Succeeded.
address:       0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13
acc_type:      Active
balance:       11466383488239689 nanoton
last_paid:     1640619135
last_trans_lt: 0x530197a143
data(boc):     b5ee9c720101060100b000014195c06aa743d1f9000dd64b75498f106af4b7e7444234d7de67ea26988f6181dfe001020120050202012004030052bf874da2f56d034e11773c58331900e0e1e91a137e1b4c2ca15607634c2d63e1af0000000061c9dca50052bfbddf9156dc04cca88cf25d9c766b1bd2f1ab7d0878c4d761862fc524758767f10000000061c9dc820053bfd627d55f960de2235b3f1537884d5968e5e486c58c581bc9ea4068c8da164ce18000000030e4ee49c0
code_hash:     ccbfc821853aa641af3813ebd477e26818b51e4ca23e5f6d34509215aa7123d9

address:       0:14014af4a374bdd13dae2379063ea2597634c2c2fc8e99ca9eab431a7ab6f566
acc_type:      Active
balance:       2082745497066 nanoton
last_paid:     1640619517
last_trans_lt: 0x530a3c2782
data(boc):     b5ee9c7201020c0100022e000373000000befe45557e0000000000000000000000000002faf04e577e5cf5b28c2a81afc5ae534a0f3f494cc4ee62ef675ca8e36af911a3c8767a400b0a010183801f13b28d6b697140de03b841b9dde6195ff089aa50b57d514435a6e6181e7baba318b50f6f18c9d307d500216c80d6ecd77d13e437bdfcaf0b4fa6b9204b7847500203a1c00b620939e214cadb7481682034e58a853a77874f473c69cc7d3b1ad9da7f0bafa0000000280000000c0000000bddcfa66622a7b9c955271c779b92448cff442b8efead77d43bd7f50b07a45f380030010706030203cca005040045b41bda168cd2322b5dcd28989176a9eae590288db4d548f2b6948d214de0c9bdb372700045b6554f714ca768f21ad18cff20c7af62091e9fc2d40c06d32d1ace7495f5dd1605781000bda90017d76e405363a8a494a3a8d8c38fcadd4f2c7fb550244fd6d2a77ac12eb029bce000000000000255400000000000000000000000000000034c3babc06000000000000000000000000000000000000000000000000000000000000000100201200908009bbfe85a3348c8ad7734a26245daa7ab9640a236d35523cada523485378326f6cdc9800000000000106f0000000000000000000000000002035ac0000000000000000000000000000000187c4b00e0007bbffdc5329da3c86b4633fc831ebd88247a7f0b50301b4cb46b39d257d7745815e0000000000000095500000000000000000000000002f8eb24987c490760000454310010546f6b656e202331
code_hash:     eee7d3331153dce4aa938e3bcdc922467fa215c77f56bbea1debfa8583d22f9c

0:f89d946b5b4b8a06f01dc20dceef30caff844d5285abea8a21ad3730c0f3dd12 not found


$ tonos-cli  account 0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13
Config: /home/user/TONLabs/tonos-cli/tonos-cli.conf.json
Input arguments:
addresses: 0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13
Connecting to net.ton.dev
Processing...
Succeeded.
address:       0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13
acc_type:      Active
balance:       11463682795615708 nanoton
last_paid:     1640624439
last_trans_lt: 0x5379939282
data(boc):     b5ee9c7201010401008100014195c06aa743d1f9000dd64b75498f106af4b7e7444234d7de67ea26988f6181dfe00102012003020053bfde8d98393e5db0ea2f609ed9266cf61a7487759d679ea9792adbdcfc137f6caf8000000030e4f89dc00053bfc8658b6b027767d9addd720a0bf8b157379a9b0e9208bab53ad4ee54358c6ce98000000030e4f89dc0
code_hash:     ccbfc821853aa641af3813ebd477e26818b51e4ca23e5f6d34509215aa7123d9

```

## 4.4. Call method

### 4.4.1. Call contract on the blockchain

```bash
tonos-cli call [--abi <contract.abi.json>] [--sign <seed_or_keyfile>] <address> <method> <params>
```

`<contract.abi.json>` - contract interface file.

`<seed_or_keyfile>` - can either be the seed phrase or the corresponding key pair file. If seed phrase is used, enclose it in double quotes.

Example:

- `--sign "flip uncover dish sense hazard smile gun mom vehicle chapter order enact"`

or

- `--sign keyfile.json`

`<address>` - contract [address](#41-generate-contract-address).

`<method>` - the method being called.

`<params>` - parameters of the called method.
Sometimes it can be not obvious in which way method parameters should be specified,
especially if it is a large structure with different and complex fields.
It is generally described in [abi doc](https://github.com/tonlabs/ton-labs-abi/blob/master/docs/ABI_2.1_spec.md).

Example (transaction creation in a [multisignature wallet](https://github.com/tonlabs/ton-labs-contracts/tree/master/solidity/safemultisig) contract):

```bash
$ tonos-cli call 0:a4629d617df931d8ad86ed24f4cac3d321788ba082574144f5820f2894493fbc submitTransaction '{"dest":"-1:0c5d5215317ec8eef1b84c43cbf08523c33f69677365de88fe3d96a0b31b59c6","value":234000000,"bounce":false,"allBalance":false,"payload":""}' --abi SetcodeMultisigWallet.abi.json --sign k1.keys.json
Config: /home/user/tonos-cli.conf.json
Input arguments:
 address: 0:a4629d617df931d8ad86ed24f4cac3d321788ba082574144f5820f2894493fbc
  method: submitTransaction
  params: {"dest":"-1:0c5d5215317ec8eef1b84c43cbf08523c33f69677365de88fe3d96a0b31b59c6","value":234000000,"bounce":false,"allBalance":false,"payload":""}
     abi: SetcodeMultisigWallet.abi.json
    keys: k1.keys.json
lifetime: None
  output: None
Connecting to net.ton.dev
Generating external inbound message...

MessageId: c6baac843fefe6b9e8dc3609487a63ef21207e4fdde9ec253b9a47f7f5a88d01
Expire at: Sat, 08 May 2021 14:52:23 +0300
Processing...
Succeeded.
Result: {
  "transId": "6959885776551137793"
}
```

**Note**: If your function is marked as [responsible](https://github.com/tonlabs/TON-Solidity-Compiler/blob/master/API.md#external-function-calls), TONOS-CLI expects `_answer_id` field, and you may encounter errors, if it's missing.

### 4.4.2. Run contract method locally

```bash
tonos-cli run [--abi <contract.abi.json>] <address> <method> <params>
```

`<contract.abi.json>` - contract interface file.

`<address>` - contract [address](#41-generate-contract-address).

`<method>` - the method being called.

`<params>` - parameters of the called method.
Sometimes it can be not obvious in which way method parameters should be specified,
especially if it is a large structure with different and complex fields.
It is generally described in [abi doc](https://github.com/tonlabs/ton-labs-abi/blob/master/docs/ABI_2.1_spec.md).

Example of a transaction list request in a [multisignature wallet](https://github.com/tonlabs/ton-labs-contracts/tree/master/solidity/safemultisig):

```bash
$ tonos-cli run 0:a4629d617df931d8ad86ed24f4cac3d321788ba082574144f5820f2894493fbc getTransactions {} --abi SafeMultisigWallet.abi.json
Config: /home/user/tonos-cli.conf.json
Input arguments:
 address: 0:a4629d617df931d8ad86ed24f4cac3d321788ba082574144f5820f2894493fbc
  method: getTransactions
  params: {}
     abi: SafeMultisigWallet.abi.json
    keys: None
lifetime: None
  output: None
Connecting to net.ton.dev
Generating external inbound message...

MessageId: ff8b8a73b1a7803a735eb4f620cade78ed45fd1530992fd3bedb91f3c66eacc5
Expire at: Sat, 08 May 2021 15:16:59 +0300
Running get-method...
Succeeded.
Result: {
  "transactions": [
    {
      "id": "6959890394123980993",
      "confirmationsMask": "1",
      "signsRequired": "4",
      "signsReceived": "1",
      "creator": "0x849ee401fde65ad8cda6d937bdc81e2beba0f36ba2f87115f4a2d24a15568203",
      "index": "0",
      "dest": "-1:0c5d5215317ec8eef1b84c43cbf08523c33f69677365de88fe3d96a0b31b59c6",
      "value": "234000000",
      "sendFlags": "3",
      "payload": "te6ccgEBAQEAAgAAAA==",
      "bounce": false
    }
  ]
}
```

### 4.4.3. Run funC get-method

```bash
tonos-cli runget [--boc] [--tvc] <address> <method> [<params>...] [--bc_config <config_path>]
```

`<address>` - contract [address](#41-generate-contract-address) or path to the file with:
* account boc (It can be obtained from the TON Live) if `--boc` option is used;
* account state init if flag `--tvc` is used.

`<method>` - the method being called.

`<params>` - parameters of the called method. Can have multiple values: one for each function parameter.
Parameters should be specified separately without json wrap and argument names.

`--bc_config <config_path>` - this option can be used with `--boc` option to specify the file with the blockchain config
BOC. It can be obtained with [dump blockchain config](#94-dump-blockchain-config) command.

Example:

```bash
$ tonos-cli runget -1:3333333333333333333333333333333333333333333333333333333333333333 active_election_id
Config: /home/user/tonos-cli.conf.json
Input arguments:
 address: -1:3333333333333333333333333333333333333333333333333333333333333333
  method: active_election_id
  params: None
Connecting to net.ton.dev
Running get-method...
Succeded.
Result: ["1619901678"]

$ tonos-cli runget --boc acc.boc compute_returned_stake 0x0166d0181a19f87af9397040a68671e1b239f12152824f7d987fd6897d6a9587
Config: /home/user/TONLabs/tonos-cli/tonos-cli.conf.json
Input arguments:
 address: acc.boc
  method: compute_returned_stake
  params: ["0x0166d0181a19f87af9397040a68671e1b239f12152824f7d987fd6897d6a9587"]
Connecting to main.ton.dev
Running get-method...
Succeeded.
Result: ["125387107580525"]

$ tonos-cli runget --tvc acc.tvc compute_returned_stake 0x0166d0181a19f87af9397040a68671e1b239f12152824f7d987fd6897d6a9587
Config: /home/user/TONLabs/tonos-cli/tonos-cli.conf.json
Input arguments:
 address: acc.boc
  method: compute_returned_stake
  params: ["0x0166d0181a19f87af9397040a68671e1b239f12152824f7d987fd6897d6a9587"]
Connecting to main.ton.dev
Running get-method...
Succeeded.
Result: ["125387107580525"]
```



### 4.4.4. Run contract method locally for saved account BOC

```bash
tonos-cli run [--boc] [--tvc] [--abi <contract.abi.json>] <account> <method> <params> [--bc_config <config_path>]
```

`<contract.abi.json>` - contract interface file.

`<account>` - path to the file with account boc for flag `--boc` or account state init for flag `--tvc`
(they can be obtained from the network with `account` command).

`<method>` - the method being called.

`<params>` - parameters of the called method.

`--bc_config <config_path>` - this option can be used with `--boc` option to specify the file with the blockchain config
BOC. It can be obtained with [dump blockchain config](#94-dump-blockchain-config) command.

Example:

```bash
$ tonos-cli run --boc tests/depool_acc.boc getData '{}' --abi tests/samples/fakeDepool.abi.json
Config: /home/user/TONLabs/tonos-cli/tonos-cli.conf.json
Input arguments:
 account: tests/depool_acc.boc
  method: getData
  params: {}
     abi: tests/samples/fakeDepool.abi.json
Generating external inbound message...
Succeeded.
Result: {
  "stake": "65535",
  "sender": "0:1e0739795a20263747ba659785a791fc2761295593a694f53116ab53439cc0a4",
  "receiver": "0:0123456789012345012345678901234501234567890123450123456789012346",
  "withdrawal": "172800",
  "total": "172800",
  "reinvest": false,
  "value": "1000000000"
}

$ tonos-cli run --tvc tests/depool_acc.tvc getData '{}' --abi tests/samples/fakeDepool.abi.json
Config: /home/user/TONLabs/tonos-cli/tonos-cli.conf.json
Input arguments:
 account: tests/depool_acc.boc
  method: getData
  params: {}
     abi: tests/samples/fakeDepool.abi.json
Generating external inbound message...
Succeeded.
Result: {
  "stake": "65535",
  "sender": "0:1e0739795a20263747ba659785a791fc2761295593a694f53116ab53439cc0a4",
  "receiver": "0:0123456789012345012345678901234501234567890123450123456789012346",
  "withdrawal": "172800",
  "total": "172800",
  "reinvest": false,
  "value": "1000000000"
}

```

## 4.5. Generate encrypted message offline

An internet connection is not required to create an encrypted message. Use the following command to do it:

```bash
tonos-cli message [--raw] [--output <path_to_file>] [--abi <contract.abi.json>] [--sign <seed_or_keyfile>] <address> <method> <params> [--lifetime <seconds>]
```

`--raw` - use to create raw message boc.

`--output <path_to_file>` - specify path to file where the raw message should be written to, instead of printing it to terminal.

`<contract.abi.json>` - contract interface file.

`<seed_or_keyfile>` - can either be the seed phrase or the corresponding key pair file. If seed phrase is used, enclose it in double quotes.

Example:

- `--sign "flip uncover dish sense hazard smile gun mom vehicle chapter order enact"`

or

- `--sign keyfile.json`

`<address>` - contract [address](#41-generate-contract-address).

`<method>` - the method being called.

`<params>` - parameters of the called method.

`lifetime` – message lifetime in seconds. Once this time elapses, the message will not be accepted by the contract.

The TONOS-CLI utility displays encrypted message text and a QR code that also contains the message.Copy the message text or scan the QR code and broadcast the message online.

Example (raw boc of create new multisig transaction message with a lifetime of 1 hour saved to file):

```bash
$ tonos-cli message --raw --output message.boc --sign k1.keys.json --abi SafeMultisigWallet.abi.json 0:a4629d617df931d8ad86ed24f4cac3d321788ba082574144f5820f2894493fbc submitTransaction '{"dest":"-1:0c5d5215317ec8eef1b84c43cbf08523c33f69677365de88fe3d96a0b31b59c6","value":234000000,"bounce":false,"allBalance":false,"payload":""}' --lifetime 3600
Config: /home/user/tonos-cli.conf.json
Input arguments:
 address: 0:a4629d617df931d8ad86ed24f4cac3d321788ba082574144f5820f2894493fbc
  method: submitTransaction
  params: {"dest":"-1:0c5d5215317ec8eef1b84c43cbf08523c33f69677365de88fe3d96a0b31b59c6","value":234000000,"bounce":false,"allBalance":false,"payload":""}
     abi: SafeMultisigWallet.abi.json
    keys: k1.keys.json
lifetime: 3600
  output: message.boc
Generating external inbound message...

MessageId: 59d698efe871cf9ffa8f6eb4c784b294538cd2223b4c876bb4e999a8edf8d410
Expire at: Sat, 08 May 2021 16:42:03 +0300
Message saved to file message.boc
```

## 4.6. Broadcast previously generated message

Use the following command to broadcast a previously generated message, that is not in raw format, and not in a file:

```bash
tonos-cli send [--abi <contract.abi.json>] "<message_text>"
```

`<contract.abi.json>` - contract interface file.

`Message` – the content of the message generated by the TONOS-CLI utility during message creation. It should be enclosed in double quotes.

Example:

```bash
$ tonos-cli send --abi SafeMultisigWallet.abi.json "7b226d7367223a7b226d6573736167655f6964223a2266363364666332623030373065626264386365643265333865373832386630343837326465643036303735376665373430376534393037646266663338626261222c226d657373616765223a227465366363674542424145413051414252596742534d553677767679593746624464704a365a5748706b4c7846304545726f4b4a36775165555369536633674d41514868757856507a324c5376534e663344454a2f374866653165562f5a78324d644e6b4b727770323865397a7538376a4d6e7275374c48685965367642523141756c48784b44446e4e62344f47686768386e6b6b7a48386775456e7551422f655a61324d326d32546539794234723636447a61364c34635258306f744a4b465661434177414141586c4d464e7077594a61616b524d64677332414341574f663459757151715976325233654e776d49655834517048686e37537a75624c76524838657931425a6a617a6a414141414141414141414141414141414a4d61735142414d4141413d3d222c22657870697265223a313632303438323730352c2261646472657373223a22303a61343632396436313764663933316438616438366564323466346361633364333231373838626130383235373431343466353832306632383934343933666263227d2c226d6574686f64223a227375626d69745472616e73616374696f6e227d"
Config: /home/user/tonos-cli.conf.json
Input arguments:
 message: 7b226d7367223a7b226d6573736167655f6964223a2266363364666332623030373065626264386365643265333865373832386630343837326465643036303735376665373430376534393037646266663338626261222c226d657373616765223a227465366363674542424145413051414252596742534d553677767679593746624464704a365a5748706b4c7846304545726f4b4a36775165555369536633674d41514868757856507a324c5376534e663344454a2f374866653165562f5a78324d644e6b4b727770323865397a7538376a4d6e7275374c48685965367642523141756c48784b44446e4e62344f47686768386e6b6b7a48386775456e7551422f655a61324d326d32546539794234723636447a61364c34635258306f744a4b465661434177414141586c4d464e7077594a61616b524d64677332414341574f663459757151715976325233654e776d49655834517048686e37537a75624c76524838657931425a6a617a6a414141414141414141414141414141414a4d61735142414d4141413d3d222c22657870697265223a313632303438323730352c2261646472657373223a22303a61343632396436313764663933316438616438366564323466346361633364333231373838626130383235373431343466353832306632383934343933666263227d2c226d6574686f64223a227375626d69745472616e73616374696f6e227d
     abi: SafeMultisigWallet.abi.json
Connecting to net.ton.dev

MessageId: f63dfc2b0070ebbd8ced2e38e7828f04872ded060757fe7407e4907dbff38bba
Expire at: Sat, 08 May 2021 17:05:05 +0300
Calling method submitTransaction with parameters:
{
  "dest": "-1:0c5d5215317ec8eef1b84c43cbf08523c33f69677365de88fe3d96a0b31b59c6",
  "value": "1234000000",
  "bounce": false,
  "allBalance": false,
  "payload": "te6ccgEBAQEAAgAAAA=="
}
Processing...
Processing...
Succeded.
Result: {
  "transId": "6959904904053506881"
}
```

## 4.7. Broadcast previously generated message from a file

Use the following command to broadcast a previously generated message, that is stored in a .boc file:

```bash
tonos-cli sendfile <path_to_boc_file>
```

`<path_to_boc_file>` – path to the file where the message was saved.

Example:

```bash
$ tonos-cli sendfile /home/user/ton/message.boc
Config: /home/user/tonos-cli.conf.json
Input arguments:
     boc: /home/user/ton/message.boc
Connecting to net.ton.dev
Sending message to account 0:a4629d617df931d8ad86ed24f4cac3d321788ba082574144f5820f2894493fbc
Succeded.
```

## 4.8. Decode commands

### 4.8.1. Decode BOC file

Use the following command to decode previously generated messages in .boc files.

```bash
tonos-cli decode msg --abi <contract.abi.json> <path_to_boc_file>
```

`<contract.abi.json>` - contract interface file.

`<path_to_boc_file>` – path to the file where the message was saved.

Example:

```bash
$ tonos-cli decode msg --abi SafeMultisigWallet.abi.json /home/user/ton/message.boc
Config: /home/user/tonos-cli.conf.json
Input arguments:
     msg: /home/user/ton/message.boc
     abi: SafeMultisigWallet.abi.json
 "Type": "external inbound message",
 "Header": {
   "source": "",
   "destination": "0:a4629d617df931d8ad86ed24f4cac3d321788ba082574144f5820f2894493fbc",
   "import_fee": "0"
 },
 "Body": "te6ccgEBAwEAqwAB4diOBnSVls3D8/zEb/Uj6hIfwKrdG2uRyCWmWx+mpFtdbaZNBcTW3yS3QiwLR8NgoqLcqoDsGwDA/RbrJLen+wXhJ7kAf3mWtjNptk3vcgeK+ug82ui+HEV9KLSShVWggMAAAF5S//FEWCWlSsTHYLNgAQFjn+GLqkKmL9kd3jcJiHl+EKR4Z+0s7my70R/HstQWY2s4wAAAAAAAAAAAAAAAAb5R0AQCAAA=",
submitTransaction: {
  "dest": "-1:0c5d5215317ec8eef1b84c43cbf08523c33f69677365de88fe3d96a0b31b59c6",
  "value": "234000000",
  "bounce": false,
  "allBalance": false,
  "payload": "te6ccgEBAQEAAgAAAA=="
}
```

### 4.8.2. Decode message body

Use the following command to decode previously generated message body (can be obtained by decoding message .boc file).

```bash
tonos-cli decode body --abi <contract.abi.json> "<message_body>"
```

`<contract.abi.json>` - contract interface file.

`<message_body>` - Message body encoded as base64.

```bash
$ tonos-cli decode body --abi SafeMultisigWallet.abi.json "te6ccgEBAwEAqwAB4diOBnSVls3D8/zEb/Uj6hIfwKrdG2uRyCWmWx+mpFtdbaZNBcTW3yS3QiwLR8NgoqLcqoDsGwDA/RbrJLen+wXhJ7kAf3mWtjNptk3vcgeK+ug82ui+HEV9KLSShVWggMAAAF5S//FEWCWlSsTHYLNgAQFjn+GLqkKmL9kd3jcJiHl+EKR4Z+0s7my70R/HstQWY2s4wAAAAAAAAAAAAAAAAb5R0AQCAAA="
Config: /home/user/tonos-cli.conf.json
Input arguments:
    body: te6ccgEBAwEAqwAB4diOBnSVls3D8/zEb/Uj6hIfwKrdG2uRyCWmWx+mpFtdbaZNBcTW3yS3QiwLR8NgoqLcqoDsGwDA/RbrJLen+wXhJ7kAf3mWtjNptk3vcgeK+ug82ui+HEV9KLSShVWggMAAAF5S//FEWCWlSsTHYLNgAQFjn+GLqkKmL9kd3jcJiHl+EKR4Z+0s7my70R/HstQWY2s4wAAAAAAAAAAAAAAAAb5R0AQCAAA=
     abi: SafeMultisigWallet.abi.json
submitTransaction: {
  "dest": "-1:0c5d5215317ec8eef1b84c43cbf08523c33f69677365de88fe3d96a0b31b59c6",
  "value": "234000000",
  "bounce": false,
  "allBalance": false,
  "payload": "te6ccgEBAQEAAgAAAA=="
}
```

### 4.8.3. Decode account commands

#### 4.8.3.1. Decode account data fields

Use the following command to decode data fields of the contract.

```bash
tonos-cli decode account data --abi <contract.abi.json> --addr <contract_address>
tonos-cli decode account data --abi <contract.abi.json> --tvc <contract_file>
```

`<contract.abi.json>` - contract interface file.

Contract address on blockchain or path to the file with contract's StateInit can be specified
with options `--addr` and `--tvc` respectively.

```bash
$ tonos-cli decode account data --abi tests/test_abi_v2.1.abi.json --tvc tests/decode_fields.tvc
Config: /home/user/TONLabs/tonos-cli/tonos-cli.conf.json
Input arguments:
     tvc: tests/decode_fields.tvc
     abi: tests/test_abi_v2.1.abi.json
TVC fields:
{
  "__pubkey": "0xe8b1d839abe27b2abb9d4a2943a9143a9c7e2ae06799bd24dec1d7a8891ae5dd",
  "__timestamp": "1626254942358",
  "fun": "22",
  "opt": "48656c6c6f",
  "big": {
    "value0": "0x0000000000000000000000000000000000000000000000000000000000000002",
    "value1": "0x0000000000000000000000000000000000000000000000000000000000000008",
    "value2": "0x0000000000000000000000000000000000000000000000000000000000000002",
    "value3": "0x0000000000000000000000000000000000000000000000000000000000000000"
  },
  "a": "I like it.",
  "b": "",
  "length": "0x000000000000000000000000000000000000000000000000000000000000000f"
}
```

#### 4.8.3.2. Decode data from the account BOC file

Use the following command to decode data from the file with BOC of the account and save
StateInit to a separate file if needed.

```bash
tonos-cli decode account boc <boc_file> [--dumptvc <tvc_path>]
```

`<boc_file>` - path to the file with BOC of the account. E.g. it can be obtained from
the TON Live.
`--dumptvc <tvc_path>` - this flag can be specified to dump account StateInit to the <tvc_path> file.

```bash
$ tonos-cli decode account boc tests/account.boc --dumptvc acc.tvc
Config: /home/user/TONLabs/tonos-cli/tonos-cli.conf.json
Input arguments:
     boc: tests/account.boc
tvc_path: acc.tvc
address:       0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13
acc_type:      Active
balance:       908097175476967754
last_paid:     1626706323
last_trans_lt: 246923000003
code_hash:     4e92716de61d456e58f16e4e867e3e93a7548321eace86301b51c8b80ca6239b
state_init:
 split_depth: None
 special: None
 data: te6ccgEBAgEAUAABQZXAaqdD0fkADdZLdUmPEGr0t+dEQjTX3mfqJpiPYYHf4AEAU6AFqBJgJG7Ncw9arsqLrQ5Aoeenp6RgXcbQ7vUibecz0mAAAAAMHrI00A==
 code: te6ccgECFAEAA6EAAib/APSkICLAAZL0oOGK7VNYMPShAwEBCvSkIPShAgAAAgEgBgQB/P9/Ie1E0CDXScIBn9P/0wD0Bfhqf/hh+Gb4Yo4b9AVt+GpwAYBA9A7yvdcL//hicPhjcPhmf/hh4tMAAY4SgQIA1xgg+QFY+EIg+GX5EPKo3iP4QvhFIG6SMHDeuvLgZSHTP9MfNDH4IyEBvvK5IfkAIPhKgQEA9A4gkTHeswUATvLgZvgAIfhKIgFVAcjLP1mBAQD0Q/hqIwRfBNMfAfAB+EdukvI83gIBIAwHAgFYCwgBCbjomPxQCQH++EFujhLtRNDT/9MA9AX4an/4Yfhm+GLe0XBtbwL4SoEBAPSGlQHXCz9/k3BwcOKRII43IyMjbwJvIsgizwv/Ic8LPzExAW8iIaQDWYAg9ENvAjQi+EqBAQD0fJUB1ws/f5NwcHDiAjUzMehfA8iCEHdEx+KCEIAAAACxzwsfIQoAom8iAssf9ADIglhgAAAAAAAAAAAAAAAAzwtmgQOYIs8xAbmWcc9AIc8XlXHPQSHN4iDJcfsAWzDA/44S+ELIy//4Rs8LAPhKAfQAye1U3n/4ZwDFuRar5/8ILdHG3aiaBBrpOEAz+n/6YB6Avw1P/ww/DN8MUcN+gK2/DU4AMAgegd5XuuF//wxOHwxuHwzP/ww8W98I0l5Gcm4/DNxfABo/CFkZf/8I2eFgHwlAPoAZPaqP/wzwAgEgDw0B17sV75NfhBbo4S7UTQ0//TAPQF+Gp/+GH4Zvhi3vpA1w1/ldTR0NN/39cMAJXU0dDSAN/RIiIic8hxzwsBIs8KAHPPQCTPFiP6AoBpz0Byz0AgySL7AF8F+EqBAQD0hpUB1ws/f5NwcHDikSCA4Ako4t+CMiAbuf+EojASEBgQEA9FswMfhq3iL4SoEBAPR8lQHXCz9/k3BwcOICNTMx6F8DXwP4QsjL//hGzwsA+EoB9ADJ7VR/+GcCASAREADHuORhh18ILdHCXaiaGn/6YB6Avw1P/ww/DN8MW9qaPwhfCKQN0kYOG9deXAy/AB8IWRl//wjZ4WAfCUA+gBk9qp8B5B9ghBodo92qfgBGHwhZGX//CNnhYB8JQD6AGT2qj/8M8AIC2hMSAC2vhCyMv/+EbPCwD4SgH0AMntVPgP8gCAB1pwIccAnSLQc9ch1wsAwAGQkOLgIdcNH5LyPOFTEcAAkODBAyKCEP////28sZLyPOAB8AH4R26S8jzeg=
 lib:

```

### 4.8.4. Decode stateInit fields

StateInit can be decoded for network account or file with account BOC or TVC.

```bash
tonos-cli decode stateinit [--tvc] [--boc] <input>
```

`<input>` - depending on the flags this parameter should contain:
- path to the file with account BOC if `--boc` flag is specified;
- path to the TVC file if `--tvc` flag is specified;
- contract network address otherwise.

```bash
$ tonos-cli decode stateinit --boc account.boc
Config: /home/user/TONLabs/tonos-cli/tonos-cli.conf.json
Input arguments:
   input: account.boc
Decoded data:
{
  "split_depth": "None",
  "special": "None",
  "data": "te6ccgEBAgEAkQABowWvkA5qHmFvsIUxqyOHGegsw+mhvvuZc5taNDPm+bI8AAABfFtnzLOAAAAAAAAAAEAMpbXqnWxVq2MH9mu2c3ABPAlgHxYzBcVVGea3KTKb6UgBAHOAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAO5rKAI",
  "code": "te6ccgECKwEABs0ABCSK7VMg4wMgwP/jAiDA/uMC8gsoAgEqAuDtRNDXScMB+GaNCGAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAT4aSHbPNMAAZ+BAgDXGCD5AVj4QvkQ8qje0z8B+EMhufK0IPgjgQPoqIIIG3dAoLnytPhj0x8B+CO88rnTHwHbPPI8CwMDUu1E0NdJwwH4ZiLQ0wP6QDD4aak4ANwhxwDjAiHXDR/yvCHjAwHbPPI8JycDAiggghB7lnbGu+MCIIIQf7YUIrrjAgYEAh4w+Eby4EzT/9HbPOMA8gAFJQAKcrYJ8vAEUCCCEBM3c0q74wIgghBJt6tBu+MCIIIQaETH67vjAiCCEHuWdsa74wIcEwwHBFAgghBotV8/uuMCIIIQcXluqLrjAiCCEHTvWym64wIgghB7lnbGuuMCChYICAMoMPhG8uBM+EJu4wDTP9HbPNs88gAjCSUAcPhJ+Gtopv5g+HD4anAg+EnIz4WIzo0FkB1vNFQAAAAAAAAAAAAAAAAAH4hPIkDPFssfyz/JcPsAAiIw+EJu4wD4RvJz0fgA2zzyAAslAfTtRNDXScIBio5vcO1E0PQFcPhqjQhgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAE+GuNCGAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAT4bHD4bXD4bnD4b3D4cIBA9A7yvdcL//hicPhj4iMEUCCCEE5mPkG64wIgghBX1CDIuuMCIIIQaBC/TrrjAiCCEGhEx+u64wIRFA8NAyQw+Eby4Ez4Qm7jANHbPNs88gAjDiUAFPhJ+Gtopv5g+HADSjD4RvLgTPhCbuMA+kGV1NHQ+kDf1w0/ldTR0NM/39HbPNs88gAjECUAdPhJ+Gtopv5g+HD4avhscCD4ScjPhYjOjQWQHW80VAAAAAAAAAAAAAAAAAAfiE8iQM8Wyx/LP8lw+wACGjD4RvLgTNHbPOMA8gASJQAybXCVIIEnD7ueVHABWMjL/1mBAQD0QzLoWwRQIIIQKICYI7rjAiCCEDsU9ku64wIgghBAegYiuuMCIIIQSberQbrjAhoYFhQDNjD4RvLgTPhCbuMA+kGV1NHQ+kDf0ds82zzyACMVJQBc+GxwIPhJyM+FiM6NBZAdbzRUAAAAAAAAAAAAAAAAAB+ITyJAzxbLH8s/yXD7AANiMPhG8uBM+EJu4wDTP/pBldTR0PpA39cNH5XU0dDTH9/XDR+V1NHQ0x/f0ds82zzyACMXJQCE+En4a2im/mD4cFUC+GpY+GwB+G34bnAg+EnIz4WIzo0FkB1vNFQAAAAAAAAAAAAAAAAAH4hPIkDPFssfyz/JcPsAA5Aw+Eby4Ez4Qm7jANHbPCeOLynQ0wH6QDAxyM+HIM5xzwthXmDIz5LsU9kuyz/OVUDIzssfyx/KAMt/zc3JcPsAkl8H4uMA8gAjGSUAHPhK+Ev4TPhN+E74T/hQAyQw+Eby4Ez4Qm7jANHbPNs88gAjGyUA1vhJ+Gtopv5g+HD4ScjPhYjOi/F7AAAAAAAAAAAAAAAAABDPFslx+wCNCGAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSNBXAAAAAAAAAAAAAAAAARRY3EgAAAAGDIzs7JcPsABE4ggghUryu64wIgghAKrBj9uuMCIIIQEvQDcLrjAiCCEBM3c0q64wIkIR8dAyQw+Eby4Ez4Qm7jANHbPNs88gAjHiUAcvhJ+Gtopv5g+HB/+G9wIPhJyM+FiM6NBZAdbzRUAAAAAAAAAAAAAAAAAB+ITyJAzxbLH8s/yXD7AAMkMPhG8uBM+EJu4wDR2zzbPPIAIyAlAHL4SfhraKb+YPhwcPhvcCD4ScjPhYjOjQWQHW80VAAAAAAAAAAAAAAAAAAfiE8iQM8Wyx/LP8lw+wADKDD4RvLgTPhCbuMA0z/R2zzbPPIAIyIlAHb4avhJ+Gtopv5g+HCBAN6AC/hJyM+FiM6NBZAdbzRUAAAAAAAAAAAAAAAAAB+ITyJAzxbLH8s/yXD7AABc7UTQ0//TP9MAMdM/+kDU0dD6QNMf0x/SANN/0fhw+G/4bvht+Gz4a/hq+GP4YgIaMPhG8uBM0ds84wDyACYlAFj4UPhP+E74TfhM+Ev4SvhD+ELIy//LP8+Dyz/OVUDIzssfyx/KAMt/zcntVABcgQDegAv4ScjPhYjOjQVOYloAAAAAAAAAAAAAAAAAAB+ITyJAzxbLH8s/yXD7AAAK+Eby4EwCCvSkIPShKikAFHNvbCAwLjUxLjAAAA==",
  "code_hash": "82236b6062da156069b3cbf5020daf1a17b76869d676df216177fca950ab37df",
  "data_hash": "7197d8544363ac2b2718240a84448584a675727ec8d42efd3726e82a4c8a3853",
  "code_depth": "7",
  "data_depth": "1",
  "version": "sol 0.51.0",
  "lib":  ""
}

$ tonos-cli decode stateinit --tvc fakeDepool.tvc
Config: default
Input arguments:
   input: fakeDepool.tvc
Decoded data:
{
  "split_depth": "None",
  "special": "None",
  "data": "te6ccgEBAgEAKAABAcABAEPQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAg",
  "code": "te6ccgECKwEABs0ABCSK7VMg4wMgwP/jAiDA/uMC8gsoAgEqAuDtRNDXScMB+GaNCGAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAT4aSHbPNMAAZ+BAgDXGCD5AVj4QvkQ8qje0z8B+EMhufK0IPgjgQPoqIIIG3dAoLnytPhj0x8B+CO88rnTHwHbPPI8CwMDUu1E0NdJwwH4ZiLQ0wP6QDD4aak4ANwhxwDjAiHXDR/yvCHjAwHbPPI8JycDAiggghB7lnbGu+MCIIIQf7YUIrrjAgYEAh4w+Eby4EzT/9HbPOMA8gAFJQAKcrYJ8vAEUCCCEBM3c0q74wIgghBJt6tBu+MCIIIQaETH67vjAiCCEHuWdsa74wIcEwwHBFAgghBotV8/uuMCIIIQcXluqLrjAiCCEHTvWym64wIgghB7lnbGuuMCChYICAMoMPhG8uBM+EJu4wDTP9HbPNs88gAjCSUAcPhJ+Gtopv5g+HD4anAg+EnIz4WIzo0FkB1vNFQAAAAAAAAAAAAAAAAAH4hPIkDPFssfyz/JcPsAAiIw+EJu4wD4RvJz0fgA2zzyAAslAfTtRNDXScIBio5vcO1E0PQFcPhqjQhgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAE+GuNCGAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAT4bHD4bXD4bnD4b3D4cIBA9A7yvdcL//hicPhj4iMEUCCCEE5mPkG64wIgghBX1CDIuuMCIIIQaBC/TrrjAiCCEGhEx+u64wIRFA8NAyQw+Eby4Ez4Qm7jANHbPNs88gAjDiUAFPhJ+Gtopv5g+HADSjD4RvLgTPhCbuMA+kGV1NHQ+kDf1w0/ldTR0NM/39HbPNs88gAjECUAdPhJ+Gtopv5g+HD4avhscCD4ScjPhYjOjQWQHW80VAAAAAAAAAAAAAAAAAAfiE8iQM8Wyx/LP8lw+wACGjD4RvLgTNHbPOMA8gASJQAybXCVIIEnD7ueVHABWMjL/1mBAQD0QzLoWwRQIIIQKICYI7rjAiCCEDsU9ku64wIgghBAegYiuuMCIIIQSberQbrjAhoYFhQDNjD4RvLgTPhCbuMA+kGV1NHQ+kDf0ds82zzyACMVJQBc+GxwIPhJyM+FiM6NBZAdbzRUAAAAAAAAAAAAAAAAAB+ITyJAzxbLH8s/yXD7AANiMPhG8uBM+EJu4wDTP/pBldTR0PpA39cNH5XU0dDTH9/XDR+V1NHQ0x/f0ds82zzyACMXJQCE+En4a2im/mD4cFUC+GpY+GwB+G34bnAg+EnIz4WIzo0FkB1vNFQAAAAAAAAAAAAAAAAAH4hPIkDPFssfyz/JcPsAA5Aw+Eby4Ez4Qm7jANHbPCeOLynQ0wH6QDAxyM+HIM5xzwthXmDIz5LsU9kuyz/OVUDIzssfyx/KAMt/zc3JcPsAkl8H4uMA8gAjGSUAHPhK+Ev4TPhN+E74T/hQAyQw+Eby4Ez4Qm7jANHbPNs88gAjGyUA1vhJ+Gtopv5g+HD4ScjPhYjOi/F7AAAAAAAAAAAAAAAAABDPFslx+wCNCGAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSNBXAAAAAAAAAAAAAAAAARRY3EgAAAAGDIzs7JcPsABE4ggghUryu64wIgghAKrBj9uuMCIIIQEvQDcLrjAiCCEBM3c0q64wIkIR8dAyQw+Eby4Ez4Qm7jANHbPNs88gAjHiUAcvhJ+Gtopv5g+HB/+G9wIPhJyM+FiM6NBZAdbzRUAAAAAAAAAAAAAAAAAB+ITyJAzxbLH8s/yXD7AAMkMPhG8uBM+EJu4wDR2zzbPPIAIyAlAHL4SfhraKb+YPhwcPhvcCD4ScjPhYjOjQWQHW80VAAAAAAAAAAAAAAAAAAfiE8iQM8Wyx/LP8lw+wADKDD4RvLgTPhCbuMA0z/R2zzbPPIAIyIlAHb4avhJ+Gtopv5g+HCBAN6AC/hJyM+FiM6NBZAdbzRUAAAAAAAAAAAAAAAAAB+ITyJAzxbLH8s/yXD7AABc7UTQ0//TP9MAMdM/+kDU0dD6QNMf0x/SANN/0fhw+G/4bvht+Gz4a/hq+GP4YgIaMPhG8uBM0ds84wDyACYlAFj4UPhP+E74TfhM+Ev4SvhD+ELIy//LP8+Dyz/OVUDIzssfyx/KAMt/zcntVABcgQDegAv4ScjPhYjOjQVOYloAAAAAAAAAAAAAAAAAAB+ITyJAzxbLH8s/yXD7AAAK+Eby4EwCCvSkIPShKikAFHNvbCAwLjUxLjAAAA==",
  "code_hash": "82236b6062da156069b3cbf5020daf1a17b76869d676df216177fca950ab37df",
  "data_hash": "55a703465a160dce20481375de2e5b830c841c2787303835eb5821d62d65ca9d",
  "code_depth": "7",
  "data_depth": "1",
  "version": "sol 0.51.0",
  "lib":  ""
}

$ tonos-cli decode stateinit 989439e29664a71e57a21bff0ff9896b5e58018fcac32e83fade913c4f43479e
Config: /home/user/TONLabs/tonos-cli/tonos-cli.conf.json
Input arguments:
   input: 989439e29664a71e57a21bff0ff9896b5e58018fcac32e83fade913c4f43479e
Connecting to http://127.0.0.1/
Decoded data:
{
  "split_depth": "None",
  "special": "None",
  "data": "te6ccgEBAQEASwAAkWOlCuhADbJ3v+8vaQu9RUczWADX7uP05UFjmpt/sOAVAAABfF7iC8SAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMsA=",
  "code": "te6ccgECDwEAAakABCSK7VMg4wMgwP/jAiDA/uMC8gsMAgEOApztRNDXScMB+GYh2zzTAAGOEoECANcYIPkBWPhCIPhl+RDyqN7TPwH4QyG58rQg+COBA+iogggbd0CgufK0+GPTHwH4I7zyudMfAds88jwFAwNK7UTQ10nDAfhmItDXCwOpOADcIccA4wIh1w0f8rwh4wMB2zzyPAsLAwM8IIIQJe+yCLrjAiCCEETv7Oy64wIgghBotV8/uuMCBwYEAkgw+EJu4wD4RvJz0fhC8uBl+EUgbpIwcN74Qrry4Gb4ANs88gAFCAFK7UTQ10nCAYqOGnDtRND0BXD4aoBA9A7yvdcL//hicPhjcPhq4goBUDDR2zz4SiGOHI0EcAAAAAAAAAAAAAAAADE7+zsgyM7L/8lw+wDe8gAKAygw+Eby4Ez4Qm7jANP/0ds82zzyAAoJCAAk+Er4Q/hCyMv/yz/Pg8v/ye1UACr4RSBukjBw3vhCuvLgZvgA+Eqg+GoAJu1E0NP/0z/TADHT/9H4avhj+GIACvhG8uBMAgr0pCD0oQ4NABRzb2wgMC41MS4wAAA=",
  "code_hash": "d840258803b9d7472f2d959a5db7bb42d246f5e8f0dc6a94bb459ebb730a0e01",
  "data_hash": "0ea45bfc864790ee1d66301059fa2cbdaba7a75e9e4f4bc1d2fbffd8401ee798",
  "code_depth": "5",
  "data_depth": "0",
  "version": "sol 0.51.0",
  "lib":  ""
}
```

## 4.9. Generate payload for internal function call

Use the following command to generate payload for internal function call:

```bash
tonos-cli body [--abi <contract.abi.json>] <method> <params>
```

`<contract.abi.json>` - contract interface file.

`<method>` - the method being called.

`<params>` - parameters of the called method.

Example:

```bash
$ tonos-cli body submitTransaction '{"dest":"-1:0c5d5215317ec8eef1b84c43cbf08523c33f69677365de88fe3d96a0b31b59c6","value":234000000,"bounce":false,"allBalance":false,"payload":""}' --abi SetcodeMultisigWallet.abi.json
Config: /home/user/tonos-cli.conf.json
Input arguments:
  method: submitTransaction
  params: {"dest":"-1:0c5d5215317ec8eef1b84c43cbf08523c33f69677365de88fe3d96a0b31b59c6","value":234000000,"bounce":false,"allBalance":false,"payload":""}
     abi: SetcodeMultisigWallet.abi.json
  output: None
Message body: te6ccgEBAgEAOwABaxMdgs2f4YuqQqYv2R3eNwmIeX4QpHhn7SzubLvRH8ey1BZjazjAAAAAAAAAAAAAAAABvlHQBAEAAA==
```

## 4.10. Alternative syntax for call, deploy and run commands

To facilitate usage of tonos-cli use commands `callx`, `runx` and `deployx` instead of `call`, `run` and `deploy`.
These alternative syntax commands have almost the same syntax as classic, but allow to specify address, abi and keys
options in the config file. Also, this commands allow to skip params option if command doesn't need it.
Examples:

```bash
# specify options manually
tonos-cli callx --keys giver.key --abi giver.abi.json --addr 0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94 -m sendGrams --dest 841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94 --amount 1000000000

# options are taken from the config
tonos-cli config --abi giver.abi.json --addr 0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94 --keys giver.key
tonos-cli callx -m sendGrams --dest 841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94 --amount 1000000000

# if contract function or constructor doesn't take arguments, parameters can be skipped
tonos-cli deployx contract.tvc
tonos-cli runx -m getParameters

# method and parameters can be specified in config
tonos-cli config --method add --parameters '{"value":1}' --addr 0:41af055743c85ba58fcaead78fa45b017f265c9351b5275ad76bf58be11760fd --abi ../samples/1_Accumulator.abi.json --keys keys/key0
tonos-cli callx
tonos-cli config --method sum --parameters '{}'
tonos-cli runx
```

# 5. DeBot commands

TONOS-CLI has a built-in DeBot <link to DeBots repo> browser, which is regularly updated with the most recent versions of DEngine <link to DEngine>.

To call a DeBot, use the following command:

```bash
tonos-cli debot fetch <--debug> <debot_address>
```

`<debot_address>` - address of the DeBot contract.

`<--debug>` - runs DeBot in verbose mode.

Example:

```bash
$ tonos-cli debot fetch 0:09403116d2d04f3d86ab2de138b390f6ec1b0bc02363dbf006953946e807051e
Config: /home/user/tonos-cli.conf.json
Connecting to net.ton.dev
DeBot Info:
Name   : Multisig
Version: 1.2.0
Author : TON Labs
Publisher: TON Labs
Support: 0:66e01d6df5a8d7677d9ab2daf7f258f1e2a7fe73da5320300395f99e01dc3b5f
Description: DeBot for multisig wallets
Hi, I will help you work with multisig wallets that can have multiple custodians.
Run the DeBot (y/n)?
y

Which wallet do you want to work with?
```

Further input depends on the DeBot, which usually explains any actions it offers you to perform.

# 6. Multisig commands

Multisig commands allow you to work with any existing Multisig wallets <link to repo> in a more convenient way and with no need of ABI files.

## 6.1. Send tokens

Use the following command to send tokens to any recipient:

```bash
tonos-cli multisig send --addr <sender_address> --dest <recipient_address> --purpose <"text_in_quotes"> --sign <path_to_keys_or_seed_phrase> --value *number*
```

`<sender_address>` - address of the multisig wallet that tokens are sent from.

`<recipient_address>` - address of the account tokens are sent to.

`<"text_in_quotes">` - accompanying message. Only the recipient will be able to decrypt and read it.

`<path_to_keys_or_seed_phrase>` - path to sender wallet key file or the corresponding seed phrase in quotes.

`--value *number*` - value to be transferred (in tokens).

Example:

```bash
$ tonos-cli multisig send --addr 0:255a3ad9dfa8aa4f3481856aafc7d79f47d50205190bd56147138740e9b177f3 --dest 0:a4629d617df931d8ad86ed24f4cac3d321788ba082574144f5820f2894493fbc --purpose "test transaction" --sign key.json --value 6
Config: /home/user/tonos-cli.conf.json
Connecting to net.ton.dev
Generating external inbound message...

MessageId: 62b1420ac98e586f29bf79bc2917a0981bb3f15c4757e8dca65370c19146e327
Expire at: Thu, 13 May 2021 13:26:06 +0300
Processing...
Succeeded.
Result: {
  "transId": "0"
}.
```

# 6.2. Deploy wallet

Use the following command to deploy a multisignature wallet:

```bash
tonos-cli multisig deploy [--setcode] [--owners <owners_list>] [--confirms <confirms_cnt>] [--local <local_giver_value>] --keys <path_to_keys_or_seed_phrase>
```

`--setcode` - flag that changes type of the wallet to the SetcodeMultisigWallet. If not specified, a SafeMultisigWallet is deployed.

`--owners <owners_list>` - option that sets wallet owners. If not specified, the only owner is deploy signed (set with --keys option).
List of owners must be specified by their public keys in hex format, split by the `,`.

`--confirms <confirms_cnt>` - option that sets required number of confirmations. If not specified, is set to 1.

`--local <local_giver_value>` - value that should be transferred from the local giver if wallet is deployed onto the Node SE (in nanotons).

`--keys <path_to_keys_or_seed_phrase>` - path to the wallet key file or the corresponding seed phrase in quotes.


Example:

```bash
$ tonos-cli multisig deploy -k "young tell target alter sport dignity enforce improve pottery fashion alert genuine" --local 1_000_000_000
Config: /home/user/TONLabs/tonos-cli/tonos-cli.conf.json
Wallet address: 0:4d892e63989c1c0ad64b0bbe22e8d036b0da271c19b6686d01bd29a99dcbc86d
Connecting to http://127.0.0.1/
Expire at: Mon, 13 Sep 2021 14:55:29 +0300
MessageId: 3c3537e36e2a4a4018b7463db2bf57efad5dc0dc0233b040c2f5e165cb43e887
MessageId: 8102067efc190b2e728d91d632c985634fc4717b7ae1137a4bbcf756c4cf8705
Wallet successfully deployed
Wallet address: 0:4d892e63989c1c0ad64b0bbe22e8d036b0da271c19b6686d01bd29a99dcbc86d

# deploy with owners
tonos-cli multisig deploy -l 5000000000 -c 2 -o 8b445b0feab10b9abf4e039d649348ec8662e3673fe9c37b7208c4d9d04c9b3f,ddc5bc7198c90feb75d9ce09e1b1f25a7e14a252fef31b50fac048c6ac3ee46c -k test.key
```

# 7. DePool commands

## 7.1. Configure TONOS-CLI for DePool operations

For all commands listed below, the DePool address, the wallet making the stake, the amount of fee to pay for DePool's services and the path to the keyfile/seed phrase may be specified in the TONOS-CLI config file in advance:

```bash
tonos-cli config --addr <address> --wallet <address> --no-answer true | false --keys <path_to_keys or seed_phrase> --depool_fee <depool_fee>
```

`--addr <address>` - the address of the DePool

`--wallet <address>` - the address of the wallet making the stake

`--no-answer true | false` - no-answer flag, which determines, whether TONOS-CLI waits for DePool answer when performing various actions and prints it out, or simply generates and sends a transaction through the specified multisig wallet, without monitoring transaction results in the DePool. By default is set to `true`. Setting to false can be useful for catching rejected stakes or other errors on the DePool side.

`<path_to_keys or seed_phrase>` - either the keyfile for the wallet making the stake, or the seed phrase in quotes

`--depool_fee <depool_fee>` - value in tons, that is additionally attached to the message sent to the DePool to cover its fees. Change is returned to the sender. The default value, used if this option isn't configured, is 0.5 tons. It should be increased only if it proves insufficient and DePool begins to run out of gas on execution.

Example:

```bash
tonos-cli config --addr 0:37fbcb6e3279cbf5f783d61c213ed20fee16e0b1b94a48372d20a2596b700ace --wallet 0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e --no-answer false --keys "dizzy modify exotic daring gloom rival pipe disagree again film neck fuel" --depool_fee 0.8
```

In this case all DePool commands allow to omit `--addr`, `--wallet`, `--wait-answer` and `--sign` options.

Below is an example of similar DePool commands with and without waiting for DePool answer.

With waiting for DePool answer:

```bash
$ tonos-cli depool --addr 0:127ae93241278304fff6b7e5b7b182fd382b6e95b200551061a7354e032e50bf stake ordinary --value 25 --wallet 0:255a3ad9dfa8aa4f3481856aafc7d79f47d50205190bd56147138740e9b177f3 --sign key.json --wait-answer
Config: /home/user/tonos-cli.conf.json
Input arguments:
  depool: 0:127ae93241278304fff6b7e5b7b182fd382b6e95b200551061a7354e032e50bf
  wallet: 0:255a3ad9dfa8aa4f3481856aafc7d79f47d50205190bd56147138740e9b177f3
   stake: 25
    keys: key.json
Connecting to https://net.ton.dev
Generating external inbound message...

MessageId: bf3cfc02dd8eff3edbd7a70e63ce3e91e61340676bf46c43cf534ccbebc9865a
Expire at: unknown
Multisig message processing...

Message was successfully sent to the multisig, waiting for message to be sent to the depool...

Request was successfully sent to depool.

Waiting for depool answer...

Answer:
Id: 453c03c3ad4985330237ed16998e3f7a5b6936c717b2aac753967fd9c03f2926
Value: 25.489215000
Created at: 1620907654 (2021-05-13 12:07:34.000)
Decoded body:
receiveAnswer {"errcode":"1","comment":"100000000000"}

Answer status: STAKE_TOO_SMALL
Comment: 100000000000

Done
```

Same command without waiting for DePool answer:

```bash
$ tonos-cli depool --addr 0:127ae93241278304fff6b7e5b7b182fd382b6e95b200551061a7354e032e50bf stake ordinary --value 25 --wallet 0:255a3ad9dfa8aa4f3481856aafc7d79f47d50205190bd56147138740e9b177f3 --sign key.json
Config: /home/user/tonos-cli.conf.json
Input arguments:
  depool: 0:127ae93241278304fff6b7e5b7b182fd382b6e95b200551061a7354e032e50bf
  wallet: 0:255a3ad9dfa8aa4f3481856aafc7d79f47d50205190bd56147138740e9b177f3
   stake: 25
    keys: key.json
Connecting to https://net.ton.dev
Generating external inbound message...

MessageId: e1b0aba39233e07daf6a65c2426e273e9d68a75e3b440893251fbce56c6a756d
Expire at: Thu, 13 May 2021 15:09:43 +0300
Processing...
Succeeded.
Result: {
  "transId": "0"
}
```

In both cases the stake is rejected for being too small, but with `no-answer` set to `false` it isn't immediately apparent, as only the results of the sussecful multisig transaction are displayed.

## 7.2. Deposit stakes

### 7.2.1. Ordinary stake

Ordinary stake is the most basic type of stake. It and the rewards from it belong to the wallet that made it.

It is invested completely in the current pooling round, and can be reinvested every second round (as odd and even rounds are handled by DePool separately). Thus to participate in every DePool round, an ordinary stake should be invested in two consecutive rounds, so it can later be reinvested in odd and even rounds both.

Ordinary stake must exceed DePool minimum stake. Check DePool's page on [ton.live](https://ton.live/dePools) to find out the minimum stake.

```bash
tonos-cli depool [--addr <depool_address>] stake ordinary [--wallet <msig_address>] --value <number> [--sign <key_file or seed_phrase>] [--wait-answer]
```

`<depool_address>` - address of the DePool contract.

`<msig_address>` - address of the wallet making a stake.

all --value parameters must be defined in tons, like this: `--value 10.5`, which means the value is 10,5 tons.

`<key_file or seed_phrase>` - either the keyfile for the wallet making the stake, or the seed phrase in quotes.

`--wait-answer` - optional flag, which forces TONOS-CLI to wait for DePool answer when performing the action and print out the resulting message. Without it only the results of the multisig transaction to DePool will be displayed.

Example:

```bash
tonos-cli depool --addr 0:37fbcb6e3279cbf5f783d61c213ed20fee16e0b1b94a48372d20a2596b700ace stake ordinary --wallet 0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e --value 100.5 --sign "dizzy modify exotic daring gloom rival pipe disagree again film neck fuel"
```

### 7.2.2. Vesting stake

A wallet can make a vesting stake and define a target participant address (beneficiary) who will own this stake, provided the beneficiary has previously indicated the donor as its vesting donor address. This condition prevents unauthorized vestings from blocking the beneficiary from receiving an expected vesting stake from a known address.

**To receive a vesting stake beneficiary must**:

- already have an ordinary stake of any amount in the DePool
- set the donor address with the following command:

```bash
tonos-cli depool [--addr <depool_address>] donor vesting [--wallet <beneficiary_address>] --donor <donor_address> [--sign <key_file or seed_phrase>] [--wait-answer]
```

`<depool_address>` - address of the DePool contract.

`<beneficiary_address>` - address of the beneficiary wallet .

`<donor_address>` - address of the donor wallet.

`<key_file or seed_phrase>` - either the keyfile for the beneficiary wallet, or the seed phrase in quotes.

`--wait-answer` - optional flag, which forces TONOS-CLI to wait for DePool answer when performing the action and print out the resulting message. Without it only the results of the multisig transaction to DePool will be displayed.

Example:

```bash
tonos-cli depool --addr 0:3187b4d738d69776948ca8543cb7d250c042d7aad1e0aa244d247531590b9147 donor vesting --wallet 0:255a3ad9dfa8aa4f3481856aafc7d79f47d50205190bd56147138740e9b177f3 --donor 0:279afdbd7b2cbf9e65a5d204635a8630aec2baec60916ffdc9c79a09d2d2893d --sign "deal hazard oak major glory meat robust teach crush plastic point edge"
```

Not the whole stake is available to the beneficiary at once. Instead it is split into parts and the next part of stake becomes available to the beneficiary (is transformed into beneficiary's ordinary stake) at the end of the round that coincides with the end of the next withdrawal period. Rewards from vesting stake are always added to the beneficiary's ordinary stake. To withdraw these funds, beneficiary should use use one of the [withdrawal functions](#75-withdraw-stakes).

Please note, that the vesting stake is split into two equal parts by the DePool, to be used in both odd and even rounds, so to ensure DePool can participate in elections with just one vesting stake where validator wallet is beneficiary, the stake should exceed `validatorAssurance` *2. Similarly, to ensure any vesting stake is accepted, make sure it exceeds `minStake` *2.

**Donor uses the following command to make a vesting stake:**

```bash
tonos-cli depool [--addr <depool_address>] stake vesting [--wallet <msig_address>] --value <number> --total <days> --withdrawal <days> --beneficiary <address> [--sign <key_file or seed_phrase>] [--wait-answer]
```

Where

`<depool_address>` - address of the DePool contract.

`<msig_address>` - address of the donor wallet making a stake.

all `--value` parameters must be defined in tons, like this: `--value 10.5`, which means the value is 10,5 tons.

`total <days>` - total period, for which the stake is made.

`withdrawal <days>` - withdrawal period (each time a withdrawal period ends, a portion of the stake is released to the beneficiary).

> There are limitations for period settings: withdrawalPeriod should be <= totalPeriod, totalPeriod cannot exceed 18 years or be <=0, totalPeriod should be exactly divisible by withdrawalPeriod.

`beneficiary <address>` - address of the beneficiary (wallet that will receive rewards from the stake and, in parts over time, the vesting stake itself). Cannot be the same as the wallet making the stake.

`<key_file or seed_phrase>` - either the keyfile for the donor wallet making the stake, or the seed phrase in quotes.

`--wait-answer` - optional flag, which forces TONOS-CLI to wait for DePool answer when performing the action and print out the resulting message. Without it only the results of the multisig transaction to DePool will be displayed.

Example:

```bash
tonos-cli depool --addr 0:37fbcb6e3279cbf5f783d61c213ed20fee16e0b1b94a48372d20a2596b700ace stake vesting --wallet 0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e --value 1000 --total 360 --withdrawal 30 --beneficiary 0:f22e02a1240dd4b5201f8740c38f2baf5afac3cedf8f97f3bd7cbaf23c7261e3 --sign "dizzy modify exotic daring gloom rival pipe disagree again film neck fuel"
```

> Note: Each participant can concurrently be the beneficiary of only one vesting stake. Once the current vesting stake expires, another can be made for the participant.

### 7.2.3. Lock stake

A wallet can make a lock stake, in which it locks its funds in DePool for a defined period, but rewards from this stake will be payed to another target participant (beneficiary). As with vesting, the beneficiary has to indicate the donor as its lock donor address before receiving a lock stake. This condition prevents unauthorized lock stakes from blocking the beneficiary from receiving an expected lock stake from a known address.

**To receive a lock stake beneficiary must**:

- already have an ordinary stake of any amount in the DePool
- set the donor address with the following command:

```bash
tonos-cli depool [--addr <depool_address>] donor lock [--wallet <beneficiary_address>] --donor <donor_address> [--sign <key_file or seed_phrase>] [--wait-answer]
```

Where

`<depool_address>` - address of the DePool contract.

`<beneficiary_address>` - address of the beneficiary wallet .

`<donor_address>` - address of the donor wallet.

`<key_file or seed_phrase>` - either the keyfile for the beneficiary wallet, or the seed phrase in quotes.

`--wait-answer` - optional flag, which forces TONOS-CLI to wait for DePool answer when performing the action and print out the resulting message. Without it only the results of the multisig transaction to DePool will be displayed.

Example:

```bash
tonos-cli depool --addr 0:3187b4d738d69776948ca8543cb7d250c042d7aad1e0aa244d247531590b9147 donor lock --wallet 0:255a3ad9dfa8aa4f3481856aafc7d79f47d50205190bd56147138740e9b177f3 --donor 0:279afdbd7b2cbf9e65a5d204635a8630aec2baec60916ffdc9c79a09d2d2893d --sign "deal hazard oak major glory meat robust teach crush plastic point edge"
```

Like vesting stake, lock stake can be configured to be unlocked in parts at the end of each round that coincides with the end of the next withdrawal period. At the end of each period the Lock Stake is returned to the wallet which locked it. The rewards of a lock stake are always added to the ordinary stake of the beneficiary. To withdraw these funds, beneficiary should use use one of the [withdrawal functions](#75-withdraw-stakes).

Please note that the lock stake is split into two equal parts by the DePool, to be used in both odd and even rounds, so to ensure DePool can participate in elections with just one lock stake where validator wallet is beneficiary, the stake should equal `validatorAssurance` *2. Similarly, to ensure any vesting stake is accepted, make sure it exceeds `minStake` *2.

**Donor uses the following command to make a lock stake:**

```bash
tonos-cli depool [--addr <depool_address>] stake lock [--wallet <msig_address>] --value <number> --total <days> --withdrawal <days> --beneficiary <address> [--sign <key_file or seed_phrase>] [--wait-answer]
```

Where

`<depool_address>` - address of the DePool contract.

`<msig_address>` - address of the donor wallet making a stake.

all `--value` parameters must be defined in tons, like this: `--value 10.5`, which means the value is 10,5 tons.

`total <days>` - total period, for which the stake is made.

`withdrawal <days>` - withdrawal period (each time a withdrawal period ends, a portion of the stake is returned to the wallet that made the stake).

> There are limitations for period settings: withdrawalPeriod should be <= totalPeriod, totalPeriod cannot exceed 18 years or be <=0, totalPeriod should be exactly divisible by withdrawalPeriod.

`beneficiary <address>`address of the beneficiary (wallet that will receive rewards from the stake). Cannot be the same as the wallet making the stake.

`key_file or seed_phrase` - either the keyfile for the donor wallet making the stake, or the seed phrase in quotes.

`--wait-answer` - optional flag, which forces TONOS-CLI to wait for DePool answer when performing the action and print out the resulting message. Without it only the results of the multisig transaction to DePool will be displayed.

Example:

```bash
tonos-cli depool --addr 0:37fbcb6e3279cbf5f783d61c213ed20fee16e0b1b94a48372d20a2596b700ace stake lock --wallet 0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e --value 1000 --total 360 --withdrawal 30 --beneficiary 0:f22e02a1240dd4b5201f8740c38f2baf5afac3cedf8f97f3bd7cbaf23c7261e3 --sign "dizzy modify exotic daring gloom rival pipe disagree again film neck fuel"
```

> Note: Each participant can concurrently be the beneficiary of only one lock stake. Once the current lock stake expires, another can be made for the participant.

## 7.3. Remove stakes

This command removes an ordinary stake from a pooling round (while it has not been staked in the Elector yet):

```bash
tonos-cli depool [--addr <depool_address>] stake remove [--wallet <msig_address>] --value <number> [--sign <key_file or seed_phrase>] [--wait-answer]
```

Where

`<depool_address>` - address of the DePool contract.

`<msig_address>` - address of the wallet that made the stake.

all `--value` parameters must be defined in tons, like this: `--value 10.5`, which means the value is 10,5 tons.

`<key_file or seed_phrase>` - either the keyfile for the wallet making the stake, or the seed phrase in quotes.

`--wait-answer` - optional flag, which forces TONOS-CLI to wait for DePool answer when performing the action and print out the resulting message. Without it only the results of the multisig transaction to DePool will be displayed.

Example:

```bash
tonos-cli depool --addr 0:37fbcb6e3279cbf5f783d61c213ed20fee16e0b1b94a48372d20a2596b700ace stake remove --wallet 0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e --value 100 --sign "dizzy modify exotic daring gloom rival pipe disagree again film neck fuel"
```

## 7.4. Transfer stakes

The following command assigns an existing ordinary stake or its part to another participant wallet. If the entirety of the stake is transferred, the transferring wallet is removed from the list of participants in the DePool. If the receiving wallet isn't listed among the participants, it will become a participant as the result of the command.

```bash
tonos-cli depool [--addr <depool_address>] stake transfer [--wallet <msig_address>] --value <number> --dest <address> [--sign <key_file or seed_phrase>] [--wait-answer]
```

Where

`<depool_address>` - address of the DePool contract.

`<msig_address>` - address of the wallet that made the stake.

all `--value` parameters must be defined in tons, like this: `--value 10.5`, which means the value is 10,5 tons.

`dest <address>` - address of the new owner of the stake.

`<key_file or seed_phrase>` - either the keyfile for the wallet making the stake, or the seed phrase in quotes.

`--wait-answer` - optional flag, which forces TONOS-CLI to wait for DePool answer when performing the action and print out the resulting message. Without it only the results of the multisig transaction to DePool will be displayed.

Example:

```bash
tonos-cli depool --addr 0:37fbcb6e3279cbf5f783d61c213ed20fee16e0b1b94a48372d20a2596b700ace stake transfer --wallet 0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e --value 1000 --dest 0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e --sign "dizzy modify exotic daring gloom rival pipe disagree again film neck fuel"
```

> Note: Stakes cannot be transferred from or to DePool's validator wallet, and between any wallets during round completion step.

## 7.5. Withdraw Stakes

### 7.5.1. Withdraw entire stake

The following command allows to withdraw an ordinary stake to the wallet that owns it, as soon as the stake becomes available. Use `withdraw on` to receive the stake, once it's unlocked. If you then make another stake, and want to keep reinvesting it every round, run the command with `withdraw off`.

```bash
tonos-cli depool [--addr <depool_address>] withdraw on | off [--wallet <msig_address>] [--sign <key_file or seed_phrase>] [--wait-answer]
```

Where

`<depool_address>` - address of the DePool contract.

`<msig_address>` - address of the wallet that made the stake.

`<key_file or seed_phrase>` - either the keyfile for the wallet that made the stake, or the seed phrase in quotes.

`--wait-answer` - optional flag, which forces TONOS-CLI to wait for DePool answer when performing the action and print out the resulting message. Without it only the results of the multisig transaction to DePool will be displayed.

Example:

```bash
tonos-cli depool --addr 0:37fbcb6e3279cbf5f783d61c213ed20fee16e0b1b94a48372d20a2596b700ace withdraw on --wallet 0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e --sign "dizzy modify exotic daring gloom rival pipe disagree again film neck fuel"
```

### 7.5.2. Withdraw part of the stake

The following command allows to withdraw part of an ordinary stake to the wallet that owns it, as soon as the stake becomes available. If, as result of this withdrawal, participant's ordinary stake becomes less than `minStake`, then participant's whole stake is sent to participant.

```bash
tonos-cli depool [--addr <depool_address>] stake withdrawPart [--wallet <msig_address>] --value <number> [--sign <key_file or seed_phrase>] [--wait-answer]
```

Where

`<depool_address>` - address of the DePool contract.

`<msig_address>` - address of the wallet that made the stake.

all `--value` parameters must be defined in tons, like this: `--value 10.5`, which means the value is 10,5 tons.

`<key_file or seed_phrase>` - either the keyfile for the wallet that made the stake, or the seed phrase in quotes.

`--wait-answer` - optional flag, which forces TONOS-CLI to wait for DePool answer when performing the action and print out the resulting message. Without it only the results of the multisig transaction to DePool will be displayed.

Example:

```bash
tonos-cli depool --addr 0:37fbcb6e3279cbf5f783d61c213ed20fee16e0b1b94a48372d20a2596b700ace stake withdrawPart --wallet 0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e --value 1000 --sign "dizzy modify exotic daring gloom rival pipe disagree again film neck fuel"
```

## 7.6. Reinvest Stakes

[Ordinary](#721-ordinary-stake) stake reinvestment is controlled by the DePool reinvest flag. By default this flag is set to `yes`, and the the participant's available ordinary stake will be reinvested every round, no additional action required. It gets set to `no` when [withdrawing the entire stake](#751-withdraw-entire-stake). After stake withdrawal it remains set to `no`. To re-enable ordinary stake reinvesting after withdrawing a stake, run the withdraw command with option `off`:

```bash
tonos-cli depool [--addr <depool_address>] withdraw off [--wallet <msig_address>] [--sign <key_file or seed_phrase>] [--wait-answer]
```

`<depool_address>` - address of the DePool contract.

`<msig_address>` - address of the wallet that made the stake.

`<key_file or seed_phrase>` - either the keyfile for the wallet that made the stake, or the seed phrase in quotes.

`--wait-answer` - optional flag, which forces TONOS-CLI to wait for DePool answer when performing the action and print out the resulting message. Without it only the results of the multisig transaction to DePool will be displayed.

Example:

```bash
tonos-cli depool --addr 0:37fbcb6e3279cbf5f783d61c213ed20fee16e0b1b94a48372d20a2596b700ace withdraw off --wallet 0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e --sign "dizzy modify exotic daring gloom rival pipe disagree again film neck fuel"
```

**Note:**

[Withdrawing a part of the stake](#752-withdraw-part-of-the-stake) does not affect the reinvest flag.

[Lock](#723-lock-stake) and [vesting](#722-vesting-stake) stakes are reinvested according to their initial settings for the full duration of the staking period. There is no way to change these settings once lock and vesting stakes are made.

## 7.7. Read DePool answers

Every time anything happens with the participant stake in the DePool, e.g. a round completes and rewards are distributed, DePool sends the participant a message with the relevant details. Use the following command to read these messages:

```bash
tonos-cli depool --addr <depool_address> answers --wallet <msig_address> [--since <unixtime>]
```

`<depool_address>` - address of the DePool contract.

`<msig_address>` - address of the wallet that made the stake.

`<unixtime>` - unixtime, since which you want to view DePool answers. If `--since` is omitted, all DePool answers are printed.

Example:

```bash
$ tonos-cli depool --addr 0:127ae93241278304fff6b7e5b7b182fd382b6e95b200551061a7354e032e50bf answers --wallet 0:255a3ad9dfa8aa4f3481856aafc7d79f47d50205190bd56147138740e9b177f3
Config: /home/user/tonos-cli.conf.json
Connecting to net.ton.dev
34 answers found
Answer:
Id: 7cacf43d2e748a5c9209e93c41c0aeccc71a5b05782dbfb3c8ac538948b67c49
Value: 0.000000001
Created at: 1619803878 (2021-04-30 17:31:18.000)
Decoded body:
onRoundComplete {"roundId":"104","reward":"2907725565","ordinaryStake":"211269425171","vestingStake":"0","lockStake":"0","reinvest":true,"reason":"5"}
```

## 7.8. View DePool events

Various events occurring in the DePool are broadcasted to the blockchain and can be monitored. use the following command to view them:

```bash
tonos-cli depool [--addr <depool_address>] events [--since <unixtime>]
```

`<depool_address>` - address of the DePool contract.

`<unixtime>` - unixtime, since which you want to view DePool events. If `--since` is omitted, all DePool events are printed.

Example:

```bash
$ tonos-cli depool --addr 0:127ae93241278304fff6b7e5b7b182fd382b6e95b200551061a7354e032e50bf events --since 1619803870
Config: /home/user/tonos-cli.conf.json
Input arguments:
  depool: 0:127ae93241278304fff6b7e5b7b182fd382b6e95b200551061a7354e032e50bf
   since: 1619803870
Connecting to net.ton.dev
3 events found
event ba71ce0889adb4740515dd714c0ce5757373448abe20835990a7c19910bcedaf
RoundStakeIsAccepted 1619803936 (2021-04-30 17:32:16.000)
{"queryId":"1619803887","comment":"0"}

event 9c5fca5548a57809cadac1b8943ac5c60f24cf8132cf6221023e7076373764e1
StakeSigningRequested 1619803878 (2021-04-30 17:31:18.000)
{"electionId":"1619836142","proxy":"-1:ed1976efa2bc727e49079de13620881fb63a1d1ca688cb9e6300da9c157e4a19"}

event b705534fed098b49897591cedc76a71c6d0c71988454dd34730be97c0cfbf604
RoundCompleted 1619803878 (2021-04-30 17:31:18.000)
{"round":{"id":"104","supposedElectedAt":"1619705070","unfreeze":"1619803374","stakeHeldFor":"32768","vsetHashInElectionPhase":"0x000000000000000000000000000000000000000000000000000000006089bcee","step":"8","completionReason":"5","stake":"412362311390363","recoveredStake":"95976319560878","unused":"322363311390363","isValidatorStakeCompleted":false,"participantReward":"5675390246734","participantQty":"7","validatorStake":"328635369379831","validatorRemainingStake":"0","handledStakesAndRewards":"0"}}

Done
```

To wait for a new event, use the following command:

```bash
tonos-cli depool [--addr <depool_address>] events --wait-one
```

TONOS-CLI waits until new event will be emitted and then prints it to terminal.

## 7.9. Replenish DePool balance

To operate correctly, DePool needs to maintain a balance over 20 tokens. Normally, this happens automatically, but in some cases, when normal operation is interrupted, DePool balance may drop lower. Use the following command to replenish DePool balance (this is not counted towards any stake):

```bash
tonos-cli depool [--addr <depool_address>] replenish --value *number* [--wallet <msig_address>] [--sign <key_file_or_seed_phrase>]
```

`<depool_address>` - address of the DePool contract.

all `--value` parameters must be defined in tons, like this: `--value 150.5`, which means the value is 150,5 tons.

`<msig_address>` - address of the wallet that made the stake.

`<key_file_or_seed_phrase>` - either the keyfile for the wallet, or the seed phrase in quotes.

Example:

```bash
$ tonos-cli depool --addr 0:127ae93241278304fff6b7e5b7b182fd382b6e95b200551061a7354e032e50bf replenish --value 5 --wallet 0:255a3ad9dfa8aa4f3481856aafc7d79f47d50205190bd56147138740e9b177f3 --sign key.json
Config: /home/user/tonos-cli.conf.json
Input arguments:
  depool: 0:127ae93241278304fff6b7e5b7b182fd382b6e95b200551061a7354e032e50bf
  wallet: 0:255a3ad9dfa8aa4f3481856aafc7d79f47d50205190bd56147138740e9b177f3
   stake: 5
    keys: key.json
Connecting to net.ton.dev
Generating external inbound message...

MessageId: 43f45f2590ba3c7afec1974f3a2bcc726f98d8ed0bcf216656ea321606f5bf60
Expire at: Thu, 13 May 2021 14:17:44 +0300
Processing...
Succeeded.
Result: {
  "transId": "0"
}
```

## 7.10. Send ticktock to DePool

To operate correctly, DePool needs to receive regular ticktock (state update) calls. One way to set them up, is through a TONOS-CLI with the use of a multisig wallet. Use the following command to send a ticktock call (you may set up a script to run this command regularly):

```bash
tonos-cli depool [--addr <depool_address>] ticktock [--wallet <msig_address>] [--sign <path_to_keys_or_seed_phrase>]
```

- `--addr <depool_address>` - the address of the DePool
- `--wallet <msig_address>` - the address of the multisig wallet used to call DePool
- `--sign <path_to_keys_or_seed_phrase>` - either the keyfile for the wallet, or the seed phrase in quotes

1 token is always attached to this call. Change will be returned.

Example:

```bash
$ tonos-cli depool --addr 0:127ae93241278304fff6b7e5b7b182fd382b6e95b200551061a7354e032e50bf ticktock --wallet 0:255a3ad9dfa8aa4f3481856aafc7d79f47d50205190bd56147138740e9b177f3 --sign key.json
Config: /home/user/tonos-cli.conf.json
Input arguments:
  depool: 0:127ae93241278304fff6b7e5b7b182fd382b6e95b200551061a7354e032e50bf
  wallet: 0:255a3ad9dfa8aa4f3481856aafc7d79f47d50205190bd56147138740e9b177f3
    keys: key.json
Connecting to https://net.ton.dev
Generating external inbound message...

MessageId: 903bb44b8286fc679e4cd08178fcaac1fb126519ecf6f7cea5794db7337645c4
Expire at: Thu, 20 May 2021 12:59:25 +0300
Processing...
Succeeded.
Result: {
  "transId": "0"
}
```

# 8. Proposal commands

The following commands are used when voting for various FreeTON proposals at [https://gov.freeton.org/](https://gov.freeton.org/)

## 8.1. Create proposal and cast the first vote

Use the following command:

```bash
tonos-cli proposal create <msig_address> <proposal_address> "<comment>" <path_to_keyfile_or_seed_phrase>
```

`<msig_address>` -  address of judge wallet.

`<proposal_address>` - address of proposal contract.

`"<comment>"` - proposal description (max symbols: 382). Should be enclosed in double quotes.

`<path_to_keyfile_or_seed_phrase>` - path to key file or seed phrase for the judge wallet. Seed phrase should be enclosed in double quotes.

The utility generates the proposal transaction ID and casts the first vote for the proposal.

The proposal transaction ID can be used to vote for the proposal by all other wallet custodians and should be communicated to them.

## 8.2. Vote for proposal

Receive proposal transaction ID and use the following command to cast a vote:

```bash
tonos-cli proposal vote <msig_address> <proposal_id> <path_to_keyfile_or_seed_phrase>
```

`<msig_address>` - address of judge wallet.

`<proposal_id>` - proposal transaction ID.

`"<seed_phrase>"` - path to key file or seed phrase for the judge wallet. Seed phrase should be enclosed in double quotes.

Once the proposal transaction receives the required amount of votes (depends on judge wallet configuration), the transaction is executed and the proposal is considered approved.

## 8.3. Decode proposal comment

Use the following command to read the proposal comment added when the proposal transaction was created:

```bash
tonos-cli proposal decode <msig_address> <proposal_id>
```

`<msig_address>` - address of judge wallet.

`<proposal_id>` - proposal transaction ID.

# 9. Supplementary commands

## 9.1. Get global config

```bash
tonos-cli getconfig <index>
```

`<index>` - number of the [global config parameter](https://docs.ton.dev/86757ecb2/v/0/p/35a3f3-field-descriptions) (equals the numeric part of the config parameter field name).

Example (requesting the maximum and minimum numbers of validators on the blockchain):

```bash
$ tonos-cli getconfig 16
Config: /home/user/tonos-cli.conf.json
Input arguments:
   index: 16
Connecting to net.ton.dev
Config p16: {
  "max_validators": 1000,
  "max_main_validators": 100,
  "min_validators": 13
}
```

## 9.2. NodeID

The following command calculates node ID from validator public key:

```bash
tonos-cli nodeid --pubkey <validator_public_key> | --keypair <path_to_key_or_seed_phrase>
```

`<validator_public_key>` - public key of the validator wallet.

`<path_to_key_or_seed_phrase>` - path to validator wallet keyfile or the corresponding seed phrase in quotes.

Example:

```bash
$ tonos-cli nodeid ---keypair "dizzy modify exotic daring gloom rival pipe disagree again film neck fuel"
Config: /home/user/tonos-cli.conf.json
Input arguments:
     key: None
 keypair: dizzy modify exotic daring gloom rival pipe disagree again film neck fuel
50232655f2ad44f026b03ec1834ae8316bfa1f3533732da1e19b3b31c0f04143
```

## 9.3. Dump blockchain config

```bash
tonos-cli dump config <path>
```

`<path>` - path where to save the blockchain config dump.

Example:

```bash
$ tonos-cli dump config config.boc
Config: /home/user/TONLabs/tonos-cli/tonos-cli.conf.json
Input arguments:
    path: config.boc
Connecting to main.ton.dev
Config successfully saved to config.boc
```

## 9.4. Dump several account states

Dumps the list of accounts. Files will have address without workchain id as a name.

```bash
tonos-cli dump account <list_of_addresses> [--path <dir_path>]
```

`<list_of_addresses>` - list of account addresses. Addresses should be specified separately with space delimiter.
Example: `0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13 0:14014af4a374bdd13dae2379063ea2597634c2c2fc8e99ca9eab431a7ab6f566  f89d946b5b4b8a06f01dc20dceef30caff844d5285abea8a21ad3730c0f3dd12 3333333333333333333333333333333333333333333333333333333333333333`.

`<dir_path>` - path to the directory where to save dumps. Defaults to current directory.

Example:

```bash
$ tonos-cli dump account 0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13 0:14014af4a374bdd13dae2379063ea2597634c2c2fc8e99ca9eab431a7ab6f566  f89d946b5b4b8a06f01dc20dceef30caff844d5285abea8a21ad3730c0f3dd12 3333333333333333333333333333333333333333333333333333333333333333
Config: /home/user/TONLabs/tonos-cli/tonos-cli.conf.json
Input arguments:
addresses: 0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13, 0:14014af4a374bdd13dae2379063ea2597634c2c2fc8e99ca9eab431a7ab6f566, 0:f89d946b5b4b8a06f01dc20dceef30caff844d5285abea8a21ad3730c0f3dd12, 0:3333333333333333333333333333333333333333333333333333333333333333
    path: None
Connecting to net.ton.dev
Processing...
./2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13.boc successfully dumped.
./14014af4a374bdd13dae2379063ea2597634c2c2fc8e99ca9eab431a7ab6f566.boc successfully dumped.
0:f89d946b5b4b8a06f01dc20dceef30caff844d5285abea8a21ad3730c0f3dd12 was not found.
0:3333333333333333333333333333333333333333333333333333333333333333 was not found.
Succeeded.
```

## 9.5. Update global config parameter

Use the following command to update one parameter of global config, that is stored in a .json file:

```bash
tonos-cli update_config <seqno> <config_master_key_file> <new_param_file>
```

`<seqno>` – current seqno of config contract. It can get from command `seqno` on config account.

`<config_master_key_file>` – prefix of config master files. There should be two files: `<config_master_key_file>.addr` with address of config master and `<config_master_key_file>.pk` with private key of config master.

`<new_param_file>` – json with new config configuration. 

Example of new_param_file

```json
{
  "p8": {
    "version": 10,
    "capabilities": 8238
  }
}

```

Example:

```bash
$ tonos-cli update_config 9 config-master example.json
Config: /home/user/tonos-cli/tonos-cli.conf.json
Input arguments:
   seqno: 9
config_master: config-master
new_param: example.json
Message: b5ee9c720101020100850001e589feaaaaaaaaaaaaa...

```

## 9.6. Wait for an account change

The command `account-wait` waits for the change of the `last_trans_lt` account field. It exits with zero exit code upon success (the field has changed before timeout). Otherwise, it exits with non-zero code.

```bash
tonos-cli account-wait <address> [--timeout <timeout_in_secs>]
```

`<address>` - address of account to wait for.

`<timeout_in_secs>` - timeout in seconds (the default is 30).

Example:

```bash
$ tonos-cli account-wait --timeout 10 0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13
...
Succeeded.
$ echo $?
0
```

## 9.7. Make a raw GraphQL query

The command `query-raw` executes a raw network query by directly calling the `ton_client::net::query_collection` SDK interface.

```bash
tonos-cli query-raw <collection> <result> [--filter <filter>] [--limit <limit>] [--order <order>]
```

See relevant SDK documentation to learn about the command's parameters.

Examples:

```bash
$ tonos-cli --json query-raw accounts "id bits cells" --filter '{ "id": { "eq": "0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13" } }'
[
  {
    "id": "0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13",
    "bits": "0x20bc",
    "cells": "0x25"
  }
]

$ tonos-cli --json query-raw accounts "id bits cells" --order '[ { "path": "balance", "direction": "DESC" } ]' --limit 3
[
  {
    "id": "-1:7777777777777777777777777777777777777777777777777777777777777777",
    "bits": "0xe635",
    "cells": "0x6f"
  },
  {
    "id": "0:5a70f26b94d500a5dc25c6f1b19d802beb97b89f702001dc46bfaf08922d4a6f",
    "bits": "0x87",
    "cells": "0x1"
  },
  {
    "id": "0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13",
    "bits": "0x20ba",
    "cells": "0x25"
  }
]
```

For more information and syntax read docs section on [playground](https://ever-live-playground.web.app/).

## 10. Fetch and replay

These two commands are commonly used in pairs to recover a state of the account at the specific point before a given transaction.

Example:

1) Dump blockchain config history to the file.

```bash
$ tonos-cli fetch -- -1:5555555555555555555555555555555555555555555555555555555555555555 config.txns
```

2) Dump account transactions from the network to the file.

```bash
$ tonos-cli fetch 0:570ddeb8f632e5f9fde198dd4a799192f149f01c8fd360132b38b04bb7761c5d 570ddeb8.txns
```
where `0:570ddeb8f632e5f9fde198dd4a799192f149f01c8fd360132b38b04bb7761c5d` is an example of account address, `570ddeb8.txns` - name of the output file.

```bash
$ tonos-cli replay [-e] [-c config.txns] 570ddeb8.txns 197ee1fe7876d4e2987b5dd24fb6701e76d76f9d08a5eeceb7fe8ca73d9b8270
```

Transaction can be replayed with config using option `-c` or with the current network config (option `-e`).

where `197ee1fe7876d4e2987b5dd24fb6701e76d76f9d08a5eeceb7fe8ca73d9b8270` is a txn id before which account state should be restored.

Note 1: last command generates 3 files. The file with the longest name in the form of `<addr>-<txn_id>.boc` is a replayed and serialized Account state.

Note 2: to get StateInit (tvc) from Account state use `tonos-cli decode account boc` command with `--dumptvc` option.

### 10.1. How to unfreeze account

- 1) Dump Account state before transaction in which account changed state from Active to Frozen.

- 2) Extract tvc from the generated Account state.

1) Use contract deployer (address in mainnet: `0:51616debd4296a4598530d57c10a630db6dc677ecbe1500acaefcfdb9c596c64`) to deploy the extracted tvc to the frozen account. Send 1 ton to its address and then run its `deploy` method.

    Example:
    `tonos-cli --url main.ton.dev call 0:51616debd4296a4598530d57c10a630db6dc677ecbe1500acaefcfdb9c596c64 deploy --abi deployer.abi.json "{\"stateInit\":\"$(cat state.tvc | base64 -w 0)\",\"value\":500000000,\"dest\":\"-1:618272d6b15fd8f1eaa3cdb61ab9d77ae47ebbfcf7f28d495c727d0e98d523eb\"}"`
    where `dest` - an address of frozen account, `state.tvc` - extracted account StateInit in step 2.

Deployer.abi.json:
```json
{
	"ABI version": 2,
	"header": ["time", "expire"],
	"functions": [
		{
			"name": "deploy",
			"inputs": [
				{"name":"stateInit","type":"cell"},
				{"name":"value","type":"uint128"},
				{"name":"dest","type":"address"}
			],
			"outputs": [
			]
		},
		{
			"name": "constructor",
			"inputs": [
			],
			"outputs": [
			]
		}
	],
	"data": [
	],
	"events": [
	]
}
```

# 11. Debug commands

Debug commands allow user to replay transaction locally or execute a function call locally and obtain TVM trace.

## 11.1. Debug transaction

```bash
tonos-cli debug transaction [FLAGS] [OPTIONS] <tx_id>
```

FLAGS:

`--dump_config`           Dump the replayed config contract account state.

`--dump_contract`         Dump the replayed target contract account state.

`-e, --empty_config`      Replay transaction without full dump of the config contract.

`--min_trace`             Flag that changes trace to minimal version.

OPTIONS:

`-c, --config <CONFIG_PATH>`        Path to the file with saved config contract transactions. If not set transactions will be fetched to file "config.txns".

`-t, --contract <CONTRACT_PATH>`    Path to the file with saved target contract transactions. If not set transactions will be fetched to file "contract.txns".

`-d, --dbg_info <DBG_INFO>`         Path to the file with debug info.

`--decode_abi <DECODE_ABI>`         Path to the ABI file used to decode output messages.

`-o, --output <LOG_PATH>`           Path where to store the trace. Default path is "./trace.log". Note: old file will be removed.

ARGUMENTS:

`<tx_id>`      ID of the transaction that should be replayed.

This command allows user to replay remote transaction locally and obtain TVM trace.
Full replay requires transactions dump of the debugged contract and of the config contract.
This command fetches them automatically, but config contract may have too many transactions and full dump of them can
take a very long time, that's why user can use option `--empty_config` to limit number of the queried transactions and
speed up the execution if the debugged contract doesn't check network configuration parameters. Another way to speed up
execution if the contract needs config is to reuse dump of config transactions by passing the file with
`--config <CONFIG_PATH>` option.

Example:

```bash
$ tonos-cli debug transaction -o tvm_trace.log 74acbd354e605519d799c7e1e90e52030e8f9e781453e48ecad18bb035fe1586 --empty-config
Config: /home/user/TONLabs/sol2tvm/scripts/tonos-cli.conf.json
Input arguments:
 address: 0:e5b3856d4d6b45f33ea625b9c4d949c601b8b6fb60fe6b968c5c0e5000a6aa78
   tx_id: 74acbd354e605519d799c7e1e90e52030e8f9e781453e48ecad18bb035fe1586
trace_path: tvm_trace.log
config_path: None
contract_path: None
Fetching config contract transactions...
Fetching contract transactions...
Replaying the last transactions...
DONE
Log saved to tvm_trace.log.
```

## 11.2. Debug call

```bash
tonos-cli debug call [FLAGS] [OPTIONS] <address> <method> <params>
```

FLAGS:

`--boc`          Flag that changes behavior of the command to work with the saved account state (account BOC).

`--min_trace`    Flag that changes trace to minimal version.

`--tvc`          Flag that changes behavior of the command to work with the saved contract state (stateInit TVC).

OPTIONS:

`--abi <ABI>`                             Path to the contract ABI file. Can be specified in the config file.

`--tvc_address <ACCOUNT_ADDRESS>`         Account address for account constructed from TVC.

`-c, --config <CONFIG_PATH>`              Path to the file with saved config contract state.

`-d, --dbg_info <DBG_INFO>`               Path to the file with debug info.

`--decode_abi <DECODE_ABI>`               Path to the ABI file used to decode output messages.

`-o, --output <LOG_PATH>`                 Path where to store the trace. Default path is "./trace.log". Note: old file will be removed.

`--now <NOW>`                             Now timestamp (in milliseconds) for execution. If not set it is equal to the current timestamp.

`--sign <SIGN>`                           Seed phrase or path to the file with keypair used to sign the message. Can be specified in the config.

ARGUMENTS:

`<address>`    Contract address or path the file with saved contract state if corresponding flag is used.

`<method>`     Name of the function being called.

`<params>`     Function arguments. Can be specified with a filename, which contains json data.

This command allows user locally emulate contract call and obtain TVM trace.
Command can work with contract in the network by querying its boc and running message on it or with saved account state
in format of account BOC or pure StateInit TVC. If contract is passed via TVC file, contract address can be specified
with `--address <tvc_address>` option. Also, execution timestamp can be specified with option `--now <timestamp>`.

```bash
$ tonos-cli debug call --abi ../samples/1_Accumulator.abi.jso
n --sign keys/key0 0:e5b3856d4d6b45f33ea625b9c4d949c601b8b6fb60fe6b968c5c0e5000a6aa78 add2 '{"value":1}'
Config: /home/user/TONLabs/sol2tvm/scripts/tonos-cli.conf.json
Input arguments:
   input: 0:e5b3856d4d6b45f33ea625b9c4d949c601b8b6fb60fe6b968c5c0e5000a6aa78
  method: add2
  params: {"value":1}
    sign: keys/key0
     abi: ../samples/1_Accumulator.abi.json
  output: ./trace.log
Execution finished.
Log saved to ./trace.log
```

## 11.3. Debug run

```bash
tonos-cli debug run [FLAGS] [OPTIONS] <address> <method> <params>
```

FLAGS:

`--boc`          Flag that changes behavior of the command to work with the saved account state (account BOC).

`--min_trace`    Flag that changes trace to minimal version.

`--tvc`          Flag that changes behavior of the command to work with the saved contract state (stateInit TVC).

OPTIONS:

`--abi <ABI>`                             Path to the contract ABI file. Can be specified in the config file.

`--tvc_address <ACCOUNT_ADDRESS>`         Account address for account constructed from TVC.

`-c, --config <CONFIG_PATH>`              Path to the file with saved config contract state.

`-d, --dbg_info <DBG_INFO>`               Path to the file with debug info.

`--decode_abi <DECODE_ABI>`               Path to the ABI file used to decode output messages.

`-o, --output <LOG_PATH>`                 Path where to store the trace. Default path is "./trace.log". Note: old file will be removed.

`--now <NOW>`                             Now timestamp (in milliseconds) for execution. If not set it is equal to the current timestamp.

ARGUMENTS:

`<address>`    Contract address or path the file with saved contract state if corresponding flag is used.

`<method>`     Name of the function being called.

`<params>`     Function arguments. Can be specified with a filename, which contains json data.

This command is similar to `tonos-cli debug call` but allows user to debug get methods.

```bash
$ tonos-cli debug run --abi ../sol2tvm/samples/1_Accumulator.abi.json 0:04a12b2e001a4144b70ddb244838f8627f8d11cade399dc0892aded7b82d6d68 sum '{}'
Config: /home/user/TONLabs/tonos-cli/tonos-cli.conf.json
Input arguments:
   input: 0:04a12b2e001a4144b70ddb244838f8627f8d11cade399dc0892aded7b82d6d68
  method: sum
  params: {}
    sign: None
 opt_abi: ../sol2tvm/samples/1_Accumulator.abi.json
  output: ./trace.log
debug_info: None
Execution finished.
Log saved to ./trace.log
```

## 11.4. Debug replay transaction on the saved account state

```bash
    tonos-cli debug replay [FLAGS] [OPTIONS] <TX_ID> <INPUT>
```

FLAGS:

`--min_trace`       Flag that changes trace to minimal version.

`--update_state`    Update state of the contract.

OPTIONS:

`-c, --config <CONFIG_PATH>`       Path to the file with saved config contract state.

`-d, --dbg_info <DBG_INFO>`        Path to the file with debug info.

`--decode_abi <DECODE_ABI>`        Path to the ABI file used to decode output messages.file.

`-o, --output <LOG_PATH>`          Path where to store the trace. Default path is "./trace.log". Note: old file will be removed.

ARGS:

`<TX_ID>`    ID of the transaction that should be replayed.

`<INPUT>`    Path to the saved account state.

This command allows replay transaction on the saved account state. This can be useful if user wants to check
transaction execution on the contract state, whose code was replaced to a new one using TVM_LINKER.

```bash
$ tonos-cli debug replay --min_trace --update_state -d 2_StorageClient.dbg.json2 --decode_abi 2_UintStorage.abi.json -o trace2.log 82733d3ddf7cae1d3fa07ec5ce288b7febf3bffd9d229a8e538f62fac10eec3e contract.boc
Config: default
Input arguments:
   input: contract.boc
   tx_id: 82733d3ddf7cae1d3fa07ec5ce288b7febf3bffd9d229a8e538f62fac10eec3e
  output: trace2.log
config_path: None
debug_info: 2_StorageClient.dbg.json2
Contract state was updated.
Execution finished.
Log saved to trace2.log
```

## 11.5. Debug deploy

```bash
tonos-cli debug deploy [FLAGS] [OPTIONS] <tvc> <params>
```

FLAGS:

`--full_trace`      Flag that changes trace to full version.

`--init_balance`    Do not fetch account from the network, but create dummy account with big balance.

OPTIONS:

`--abi <ABI>`                       Path to the contract ABI file. Can be specified in the config file.

`-c, --config <CONFIG_PATH>`        Path to the file with saved config contract state.

`-d, --dbg_info <DBG_INFO>`         Path to the file with debug info.

`--decode_abi <DECODE_ABI>`         Path to the ABI file used to decode output messages. Can be specified in the config file.

`-o, --output <LOG_PATH>`           Path where to store the trace. Default path is "./trace.log". Note: old file will be removed.

`--now <NOW>`                       Now timestamp (in milliseconds) for execution. If not set it is equal to the current timestamp.

`--sign <SIGN>`                     Seed phrase or path to the file with keypair used to sign the message. Can be specified in the config.

`--wc <WC>`                         Workchain ID

ARGUMENTS:

`<tvc>`       Path to the tvc file with contract StateInit.

`<params>`    Constructor arguments.

This command allows user locally emulate contract deploy.
Command can work with prepared network account or create a dummy one with big balance (if --init_balance flag is
specified).

## 11.6. Debug message

```bash
$ tonos-cli debug message [--boc] <address_or_path> [-u] [-o <log_path>] <message_in_base64_or_path_to_file>
```

FLAGS:

`--boc`               Flag that changes behavior of the command to work with the saved account state (account BOC).

`--full_trace`        Flag that changes trace to full version.

`-u, --update`        Update contract BOC after execution

OPTIONS:

`-c, --config <CONFIG_PATH>`        Path to the file with saved config contract state.

`-d, --dbg_info <DBG_INFO>`         Path to the file with debug info.

`--decode_abi <DECODE_ABI>`         Path to the ABI file used to decode output messages. Can be specified in the config file.

`-o, --output <LOG_PATH>`           Path where to store the trace. Default path is "./trace.log". Note: old file will be removed.

`--now <NOW>`                       Now timestamp (in milliseconds) for execution. If not set it is equal to the current timestamp.

ARGUMENTS:

`<address_or_path>`                       Contract address or path the file with saved contract state if corresponding flag is used.

`<message_in_base64_or_path_to_file>`     Message in Base64 or path to fil with message.

This command allows to play message on the contract state locally with trace.
It can be useful when user wants to play contract interaction locally. User can call one contract locally with
`tonos-cli debug call` and find output messages in trace log:

```log
Output messages:
----------------

{
  "Type": "internal message",
  "Header": {
    "ihr_disabled": "true",
    "bounce": "true",
    "bounced": "false",
    "source": "0:c015125ec7788fe31c8ff246ad58ca3dda476f74d544f6b161535b7e8ad995e3",
    "destination": "0:9677580d26bd9d316323470526d94186354698092f62ea63e65cebcd5c6ad7a8",
    "value": "9000000",
    "ihr_fee": "0",
    "fwd_fee": "666672",
    "created_lt": "4786713000003",
    "created_at": "1652270294"
  },
  "Body": "te6ccgEBAQEAJgAASEhEWrgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA3g==",
  "BodyCall": "Undefined",
  "Message_base64": "te6ccgEBAQEAfgAA92gBgCokvY7xH8Y5H+SNWrGUe7SO3umqie1iwqa2/RWzK8cAJZ3WA0mvZ0xYyNHBSbZQYY1RpgJL2LqY+Zc681cateoOJVEABhRYYAAACLT8p/CGxPdJrCQiLVwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAb0A="
}
```

`Message_base64` then can be passed to `tonos-cli debug message` to play it on another account.

## 11.7. Render UML sequence diagram

```bash
    tonos-cli debug sequence-diagram <address_list>
```

`<address_list>`    File containing a list of account addresses, one address per line. Blank lines and lines starting with # character are ignored.

This command generates a `.plantuml` text file which describes a sequence diagram of messages and transactions
for a provided list of accounts. See PlantUML documentation for a complete guide on rendering an image out of .plantuml.
To render an SVG the following command can be used:

```bash
    java -jar plantuml.jar accounts.plantuml -tsvg
```

### Caveat

Sequence diagrams are well suited for describing synchronous interactions. However, transactions (and messages which spawn them) of the blockchain are inherently asynchronous. In particular, sequence diagram arrows can only be horizontal, and there is no way to make them curve down towards the destination, skipping other transactions and thus depicting asynchronicity.

Practically, this means that one should look cautiously at the point of transaction spawn, being aware that the spawning message can be located somewhere above.
