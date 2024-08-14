# EVER-CLI

EVER-CLI is a multi-platform command line interface for TVM compatible networks (Everscale, Venom, Gosh, TON).

It allows user to work with keys and seed phrases, deploy contracts, call any of their methods, generate and broadcast
messages. It supports specific commands for DeBot, DePool and Multisignature Wallet contracts, as well as a number of
supplementary functions.

To access built-in help, use `--help` or `-h` flag:

```bash
ever-cli --help
ever-cli <subcommand> -h
```

# Table of contents
- [EVER-CLI](#ever-cli)
- [Table of contents](#table-of-contents)
- [1. Installation](#1-installation)
  - [Install compiled executable](#install-compiled-executable)
  - [Install through EVERDEV](#install-through-everdev)
  - [Build from source](#build-from-source)
    - [Prerequisites](#prerequisites)
    - [Build from source on Linux and macOS](#build-from-source-on-linux-and-macos)
    - [Build from source on Windows](#build-from-source-on-windows)
    - [Tails OS secure environment](#tails-os-secure-environment)
    - [Put EVER-CLI into system environment](#put-ever-cli-into-system-environment)
    - [Install ever-cli, completion script and bind them](#install-ever-cli-completion-script-and-bind-them)
    - [Windows debug build troubleshooting](#windows-debug-build-troubleshooting)
  - [Ubuntu 22 troubleshooting](#ubuntu-22-troubleshooting)
  - [Check version](#check-version)
  - [A note on Windows syntax](#a-note-on-windows-syntax)
- [2. Configuration](#2-configuration)
  - [2.1. Set the network and parameter values](#21-set-the-network-and-parameter-values)
    - [2.1.1. Troubleshooting network connectivity problems](#211-troubleshooting-network-connectivity-problems)
  - [2.2. Check configuration](#22-check-configuration)
  - [2.3. Clear configuration](#23-clear-configuration)
  - [2.4. Configure endpoints map](#24-configure-endpoints-map)
  - [2.5. Override configuration file location](#25-override-configuration-file-location)
  - [2.6. Override network settings](#26-override-network-settings)
  - [2.7. Force json output](#27-force-json-output)
  - [2.8. Debug on fail option](#28-debug-on-fail-option)
  - [2.9 Configure aliases map](#29-configure-aliases-map)
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
  - [7.1. Configure EVER-CLI for DePool operations](#71-configure-ever-cli-for-depool-operations)
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
  - [9.7. Make a raw GraphQL query](#97-make-a-raw-graphql-query)
  - [9.8. Fee commands](#98-fee-commands)
    - [9.8.1. Call fee command](#981-call-fee-command)
    - [9.8.2. Deploy fee command](#982-deploy-fee-command)
    - [9.8.3. Storage fee command](#983-storage-fee-command)
- [10. Fetch and replay](#10-fetch-and-replay)
  - [10.1. How to unfreeze account](#101-how-to-unfreeze-account)
- [11. Debug commands](#11-debug-commands)
  - [11.1. Debug transaction](#111-debug-transaction)
  - [11.2. Debug call](#112-debug-call)
  - [11.3. Debug run](#113-debug-run)
  - [11.4. Debug replay transaction on the saved account state](#114-debug-replay-transaction-on-the-saved-account-state)
  - [11.5. Debug deploy](#115-debug-deploy)
  - [11.6. Debug message](#116-debug-message)
  - [11.7. Debug account](#117-debug-account)
  - [11.8. Render UML sequence diagram](#118-render-uml-sequence-diagram)
- [12. Alias functionality](#12-alias-functionality)
- [13. Evercloud authentication](#13-evercloud-authentication)

# 1. Installation

## Install compiled executable

Create a folder. Download the `.zip` file from the latest release from here:
[https://github.com/everx-labs/ever-cli/releases](https://github.com/everx-labs/ever-cli/releases) to this folder. Extract
it.

## Install through EVERDEV

You can use [EVERDEV](https://github.com/everx-labs/everdev) to install the latest version of EVER-CLI.

```bash
everdev ever-cli install
```

The installer requires [NPM](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm) to be installed, so it
can install packages globally without using sudo. In case of error, manually set environment variable
`PATH=$PATH:$HOME./everdev/solidity`

This command updates EVER-CLI installed through EVERDEV to the latest version:

```bash
everdev ever-cli update
```

This command specifies EVER-CLI version to use and downloads it if needed:

```bash
everdev ever-cli set --version 0.8.0
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

### Build from source on Linux and macOS

Install Cargo: [https://github.com/rust-lang/cargo#compiling-from-source](https://github.com/rust-lang/cargo#compiling-from-source)

Build EVER-CLI tool from source:

```bash
git clone https://github.com/everx-labs/ever-cli.git
cd ever-cli
cargo update
cargo build --release
cd target/release
```

The `ever-cli` executable is built in the `ever-cli/target/release` folder.
Create a folder elsewhere. Copy the `ever-cli` executable into the new folder you have created.
Or just add `ever-cli/target/release` to the PATH local variable.

### Build from source on Windows

Install Cargo: [https://github.com/rust-lang/cargo#compiling-from-source](https://github.com/rust-lang/cargo#compiling-from-source)

Build EVER-CLI tool from source:

```bash
> git clone https://github.com/everx-labs/ever-cli.git
> cd ever-cli
> cargo update
> cargo build --release
> cd target/release
```

The `ever-cli` executable is built in the `ever-cli/target/release` folder.
Create a folder elsewhere. Copy the `ever-cli` executable into the new folder you have created.
Or just add `ever-cli/target/release` to the PATH local variable.

### Tails OS secure environment

For maximum security while working with offline EVER-CLI features (such as cryptographic commands or encrypted message
generation), you can use the [Tails OS](https://tails.boum.org/).

### Put EVER-CLI into system environment

Optional, Linux/macOS. Use the following command to put the utility into system environment:

```bash
export PATH="<ever_folder_path>:$PATH"
```

This step can be skipped, if EVER-CLI was installed through EVERDEV. Otherwise, if you skip this step, make sure you
always run the utility from folder containing the utility:

```bash
./ever-cli <command> <options>
```

### Install ever-cli, completion script and bind them

On Linux ever-cli can be installed with a completion script by using such commands:

```bash
cd ever-cli
cargo install --force --path .
complete -C __ever-cli_completion ever-cli
```

After adding completion script, user can use `<Tab>` key to complete `--addr` option with aliases saved in the config
file and `-m/--method` option with methods loaded from the ABI file.

### Windows debug build troubleshooting

Default debug executable built after `cargo build` command may have an issue with binary default stack size:

```bash
> cargo build
Finished dev [unoptimized + debuginfo] target(s) in 0.66s
> .\target\debug\ever-cli.exe --version

thread 'main' has overflowed its stack
```

User can fix this issue by using [editbin tool from MSVC Tools](https://docs.microsoft.com/ru-ru/cpp/build/reference/editbin-reference?view=msvc-170).
This tool allows user to increase binary stack reserve. Increase it by 2 times will help to fix ever-cli:

```bash
> editbin /STACK:2097152 ever-cli.exe
Microsoft (R) COFF/PE Editor Version 14.28.29914.0
Copyright (C) Microsoft Corporation.  All rights reserved.

> ever-cli.exe --version
ever_cli 0.26.7
COMMIT_ID: 1e1397b5561ea79d2fd7cce47cd033450b123f25
BUILD_DATE: Unknown
COMMIT_DATE: 2022-05-13 14:15:47 +0300
GIT_BRANCH: master
```

## Ubuntu 22 troubleshooting

Ubuntu 22 has upgraded to OpenSSL 3.0 and this breaks execution of compiled ever-cli releases. To fix this problem one
should install old version of libssl. To do it one can download amd64 package from
(packages.debian.org)[https://packages.debian.org/stretch/libssl1.1] and install it with dpkg:

```bash
sudo dpkg -i libssl1.1*.deb
```

## Check version

You can check version of the current EVER-CLI installation with the following command:

```bash
ever-cli version
```

Output example:

```bash
$ ever-cli version
Config: default
ever-cli 0.2.0
COMMIT_ID: 21ebd53c35bf22696bf1eb434e408ed33318136a
BUILD_DATE: 2021-01-26 15:06:18 +0300
COMMIT_DATE: 2021-01-14 16:13:32 +0300
GIT_BRANCH: master
```

## A note on Windows syntax

When using Windows command line, the following syntax should be used for all EVER-CLI commands:

1) Never use the `./` symbols before `ever-cli`:

```bash
> ever-cli <command_name> <options>
```

2) For all commands with nested quotes, the outer single quotes should be changed to double quotes, and the inner double
quotes should be shielded by a preceding `\`. Example:

```bash
> ever-cli deploy SafeMultisigWallet.tvc "{\"owners\":[\"0x723b2f0fa217cd10fe21326634e66106678f15d5a584babe4f576dffe9dcbb1b\",\"0x127e3ca223ad429ddaa053a39fecd21131df173bb459a4438592493245b695a3\",\"0xc2dd3682ffa9df97a968bef90b63da90fc92b22163f558b63cb7e52bfcd51bbb\"],\"reqConfirms\":2}" --abi SafeMultisigWallet.abi.json --sign deploy.keys.json
```

If this is not done, `arguments are not in json format: key must be a string at line 1 column` error may occur.

# 2. Configuration

## 2.1. Set the network and parameter values

EVER-CLI can store some parameter values in the ever-cli configuration file and use it automatically in various
subcommands.

After that you can omit the corresponding parameters in subsequent subcommands.

Default path for the configuration file is `./ever-cli.config.json`. It is created in the current working directory.
User can set up path to the configuration file [manually](#25-override-configuration-file-location).
All subsequent calls of the utility will use this file by default.

Use the following command to create a configuration file:

```bash
ever-cli config [--global] <--option> <option_value>
```

All other EVER-CLI commands will indicate the configuration file currently used.

Default values for options that were not specified are taken from the global configuration file. It has name
`.ever-cli.global.conf.json` and is located in the folder, where the `ever-cli` executable lies. This global
configuration file can be configured as the ordinary one, but the option `--global` must be used for the `config`
subcommand.

List of available options:

```bash
--abi <ABI>                                   Path or link to the contract ABI file or pure json ABI data.
--access_key <ACCESS_KEY>                     Project secret or JWT in Evercloud (dashboard.evercloud.dev).
--addr <ADDR>                                 Contract address.
--async_call <ASYNC_CALL>                     Disables wait for transaction to appear in the network after call command.
--balance_in_tons <BALANCE_IN_TONS>           Print balance for account command in tons. If false balance is printed in nanotons.
--debug_fail <DEBUG_FAIL>                     When enabled ever-cli executes debug command on fail of run or call command. Can be enabled with values 'full' or 'minimal' which set the trace level for debug run and disabled with value 'none'.
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
--project_id <PROJECT_ID>                     Project Id in Evercloud (dashboard.evercloud.dev).
--pubkey <PUBKEY>                             User public key. Used by DeBot Browser.
--retries <RETRIES>                           Number of attempts to call smart contract function if previous attempt was unsuccessful.
--timeout <TIMEOUT>                           Network `wait_for` timeout in ms. This value is also used as timeout for remote files (specified with link, e.g. ABI file) loading.
--url <URL>                                   Url to connect.
--wallet <WALLET>                             Multisig wallet address.
--wc <WC>                                     Workchain id.
```

Example:

```bash
$ ever-cli config --url https://main.evercloud.dev --wc -1 --keys key.json --abi SafeMultisigWallet.abi.json --lifetime 3600 --local_run true --retries 3 --timeout 600
Succeeded.
{
  "url": "main.evercloud.dev",
  "wc": -1,
  "addr": null,
  "method": null,
  "parameters": null,
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
  "project_id": null,
  "access_key": null,
  "endpoints": [
    "https://mainnet.evercloud.dev"
  ]
}
```

Some frequently used networks:

`http://127.0.0.1/` - Node SE local node.

`https://devnet.evercloud.dev` - developer sandbox for testing. EVER-CLI connects to it by default.

`https://mainnet.evercloud.dev` - main Free TON network.

EVER-CLI supports the use of multiple endpoints for networks: if several endpoints are
[specified in the endpoint map](#24-configure-endpoints-map) for a network, EVER-CLI will use them all when accessing
it. Otherwise, the network URL will be treated as the only endpoint.

`https://mainnet.evercloud.dev` and `https://devnet.evercloud.dev` networks already have their current endpoints specified in the default endpoint map.
See [section 2.4 below](#24-configure-endpoints-map) on how to edit and add endpoints to the endpoint map.

> **Note**: This change was introduced in version 0.16.1 and is fully compatible with scripts written for previous versions, where `https://mainnet.evercloud.dev` and `https://devnet.evercloud.dev` networks were specified with a single url. EVER-CLI will simply use the default endpoint map to access these networks.


Network configuration can be [overridden](#26-override-network-settings) for any single subcommand.

To connect to a [DApp Server](https://github.com/everx-labs/TON-OS-DApp-Server) you are running, it should have domain name
and a DNS record. Then its URL may be used to access it with EVER-CLI:

```bash
ever-cli config --url <dapp_server_url>
```

> Note: Either run ever-cli utility only from the directory where ever-cli.config.json is placed, or use one of the available methods (see [section 2.5](#25-override-configuration-file-location)) to make the utility look for the file elsewhere.

### 2.1.1. Troubleshooting network connectivity problems

Most part of the network connectivity problems can be fixed by using right network endpoints and authentication
credentials.
ever-cli reads network endpoints settings from the configuration file, so ensure that you have set them properly.
[Here](https://docs.everos.dev/ever-sdk/reference/ever-os-api/networks) you can get the list of current network
endpoints.
ever-cli usually has the latest list of endpoints, but old endpoints can be saved in the configuration or global
configuration files, so it's better to clear the config files after upgrading the ever-cli:

```bash
$ ever-cli config --global clear
$ ever-cli config --global endpoint reset
$ ever-cli config clear
$ ever-cli config endpoint reset
```

If your network connection can't be established with such error description:

```bash
$ ever-cli account -1:3333333333333333333333333333333333333333333333333333333333333333
Input arguments:
addresses: -1:3333333333333333333333333333333333333333333333333333333333333333
Connecting to:
        Url: main.evercloud.dev
        Endpoints: ["https://mainnet.evercloud.dev"]

Processing...
Error: failed to query account info: Query failed: Can not send http request: Server responded with code 401
Error: 1
```

it can be caused by absence of authentication credentials. Set them up as described in
[this section](#13-evercloud-authentication).

## 2.2. Check configuration

You can print the current or the global configuration parameters with the following command:

```bash
ever-cli config --list
ever-cli config --global --list
```

## 2.3. Clear configuration

Use the following command to reset configuration to default values:

```bash
ever-cli config clear
```

The same options as in ordinary `congfig` command can be used to clear only the specified parametes.

```bash
$ ever-cli config clear --url --addr --wallet
Succeeded.
{
  "url": "net.evercloud.dev",
  "wc": 0,
  "addr": null,
  "method": null,
  "parameters": null,
  "wallet": null,
  "pubkey": null,
  "abi_path": null,
  "keys_path": null,
  "retries": 5,
  "timeout": 40000,
  "message_processing_timeout": 40000,
  "out_of_sync_threshold": 15,
  "is_json": false,
  "depool_fee": 0.5,
  "lifetime": 60,
  "no_answer": true,
  "balance_in_tons": false,
  "local_run": false,
  "async_call": false,
  "debug_fail": "None",
  "project_id": null,
  "access_key": null,
  "endpoints": [
    "https://devnet.evercloud.dev"
  ]
}
```

## 2.4. Configure endpoints map

EVER-CLI configuration file also stores the endpoints map that can be updated by the user.
Each time user [changes the url](#21-set-the-network-and-parameter-values), endpoints also change in accordance to the
endpoints map.
To print the map use the following command:

```bash
ever-cli config endpoint print
```

User can reset map to the default state:

```bash
ever-cli config endpoint reset
```

Default state of the map:

```bash
{
  "http://127.0.0.1/": [
    "http://0.0.0.0",
    "http://127.0.0.1",
    "http://localhost"
  ],
  "main.evercloud.dev": [
    "https://mainnet.evercloud.dev"
  ],
  "net.evercloud.dev": [
    "https://devnet.evercloud.dev"
  ]
}
```

Map can be changed with `remove` and `add` subcommands:

```bash
ever-cli config endpoint remove <url>
ever-cli config endpoint add <url> <list_of_endpoints>
```

Example:

```bash
ever-cli config endpoint remove main.evercloud.dev
ever-cli config endpoint add main.evercloud.dev "https://mainnet.evercloud.dev"
```

> **Note**: If url used in the add command already exists, endpoints lists will be merged.

If a network that doesn't have mapped endpoints is
[specified in the config file](#21-set-the-network-and-parameter-values), its url will be automatically treated as the
only endpoint. For example, configuring EVER-CLI to connect to RustNet with the command
`ever-cli config --url https://rustnet.ton.dev` will result in EVER-CLI using this url as a single endpoint, without
the user having to specify it in the endpoints map additionally.


## 2.5. Override configuration file location

You can move the `ever-cli.config.json` configuration file to any other convenient location and/or rename it. There are
several ways you can point the utility to the new location of the file:

- **define environment variable** `EVER_CLI_CONFIG` with the path to your configuration file:

```bash
export EVER_CLI_CONFIG=<path_to_config_file>
```

Example:

```bash
export EVER_CLI_CONFIG=/home/user/config.json
```

- **define direct option** `--config <path_to_config_file>` before any other subcommand:

```bash
ever-cli --config <path_to_config_file> <any_subcommand>
```

Example:

```bash
ever-cli --config /home/user/config.json account <address>
```

The `--config` direct option has higher priority than the `EVER_CLI_CONFIG` environment variable.

> Note: You can use the config subcommand to create or edit a configuration file located outside the current working directory.

## 2.6. Override network settings

You can also separately override [preconfigured network settings](#21-set-the-network-and-parameter-values) for a single subcommand. Use the `--url <network_url>` direct option for this purpose:

```bash
ever-cli --url <network_url> <any_subcommand>
```

Example:

```bash
ever-cli --url main.evercloud.dev account <address>
```

## 2.7. Force json output

You can force EVER-CLi to print output in json format. To do so, add `--json` flag before a subcommand:

```bash
ever-cli --json <any_subcommand>
```

This option can also be saved in the ever-cli configuration file:

```bash
ever-cli config --is_json true
{
  "url": "http://127.0.0.1/",
  "wc": 0,
  "addr": "0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13",
  "method": null,
  "parameters": null,
  "wallet": "0:0000000000000000000000000000000000000000000000000000000000000001",
  "pubkey": "0x0000000000000000000000000000000000000000000000000000000000000002",
  "abi_path": null,
  "keys_path": null,
  "retries": 5,
  "timeout": 40000,
  "message_processing_timeout": 40000,
  "out_of_sync_threshold": 15,
  "is_json": true,
  "depool_fee": 0.5,
  "lifetime": 60,
  "no_answer": true,
  "balance_in_tons": false,
  "local_run": false,
  "async_call": false,
  "debug_fail": "None",
  "endpoints": [
    "http://0.0.0.0/",
    "http://127.0.0.1/",
    "http://localhost/"
  ]
}
```

## 2.8. Debug on fail option

You can force EVER-CLi to debug call and run executions if they fail with error code **414**.

```bash
ever-cli config --debug_fail <trace_level>
```

Possible <trace_level> values:
- 'full'
- 'minimal'
- 'none'

## 2.9. Configure aliases map

Yoo can explore and configure current aliases map with the list of commands

```bash
ever-cli config alias add [--addr <contract_address>] [--abi <contract_abi>] [--keys <contract_keys>] <alias>  # add entity to the map
ever-cli config alias remove <alias>  # remove entity
ever-cli config alias reset  # clear the map
ever-cli config alias print  # print the current state of the map
```

Options:
- `<alias>` - alias name of the contract;
- `<contract_address>` - address of the contract;
- `<contract_abi>` - path to the contract abi file;
- `<contract_keys>` - seed phrase or path to the file with contract key pair.

Example:

```bash
$ ever-cli config alias add msig --addr 0:d5f5cfc4b52d2eb1bd9d3a8e51707872c7ce0c174facddd0e06ae5ffd17d2fcd --abi samples/SafeMultisigWallet.abi.json --keys key0.keys.json
Config: /home/user/ever-cli/ever-cli.conf.json
{
  "msig": {
    "abi_path": "samples/SafeMultisigWallet.abi.json",
    "address": "0:d5f5cfc4b52d2eb1bd9d3a8e51707872c7ce0c174facddd0e06ae5ffd17d2fcd",
    "key_path": "key0.keys.json"
  }
}
$ ever-cli config alias print
Config: /home/user/ever-cli/ever-cli.conf.json
{
  "msig": {
    "abi_path": "samples/SafeMultisigWallet.abi.json",
    "address": "0:d5f5cfc4b52d2eb1bd9d3a8e51707872c7ce0c174facddd0e06ae5ffd17d2fcd",
    "key_path": "key0.keys.json"
  }
}
$ ever-cli config alias add msig2 --addr 0:eef5cfc4b52d2eb1bd9d3a8e51707872c7ce0c174facddd0e06ae5ffd17d2fff --abi samples/SafeMultisigWallet.abi.json --keys key1.keys.json
Config: /home/user/ever-cli/ever-cli.conf.json
{
  "msig": {
    "abi_path": "samples/SafeMultisigWallet.abi.json",
    "address": "0:d5f5cfc4b52d2eb1bd9d3a8e51707872c7ce0c174facddd0e06ae5ffd17d2fcd",
    "key_path": "key0.keys.json"
  },
  "msig2": {
    "abi_path": "samples/SafeMultisigWallet.abi.json",
    "address": "0:eef5cfc4b52d2eb1bd9d3a8e51707872c7ce0c174facddd0e06ae5ffd17d2fff",
    "key_path": "key1.keys.json"
  }
}
$ ever-cli config alias remove msig
Config: /home/user/ever-cli/ever-cli.conf.json
{
  "msig2": {
    "abi_path": "samples/SafeMultisigWallet.abi.json",
    "address": "0:eef5cfc4b52d2eb1bd9d3a8e51707872c7ce0c174facddd0e06ae5ffd17d2fff",
    "key_path": "key1.keys.json"
  }
}
$ ever-cli config alias reset
Config: /home/user/ever-cli/ever-cli.conf.json
{}
$ ever-cli config alias print
Config: /home/user/ever-cli/ever-cli.conf.json
{}
```

## 2.10. Enabling verbose mode for SDK execution

User can increase log level of the tool execution to see more messages. To do it one need to specify environment
variable `RUST_LOG=debug`:

```bash
$ ever-cli callx --addr 0:75186644bf5157d1b638390889ec2ba297a12250f6e90d935618918cb82d12c3 --abi ../samples/1_Accumulator.abi.json --keys keys/key0 -m add --value 1
Input arguments:
 address: 0:75186644bf5157d1b638390889ec2ba297a12250f6e90d935618918cb82d12c3
  method: add
  params: {"value":"1"}
     abi: ../samples/1_Accumulator.abi.json
    keys: keys/key0
Connecting to:
        Url: net.evercloud.dev
        Endpoints: ["https://devnet.evercloud.dev/b2ad82504ee54fccb5bc6db8cbb3df1e"]

MessageId: b3e24321924526dbfdc8ffdd9cc94aeb2da80edca7d87bd7f16f4a0a2afbfa20
Succeeded.
Result: {}

# Enable verbose mode
$ export RUST_LOG=debug

$ ever-cli callx --addr 0:75186644bf5157d1b638390889ec2ba297a12250f6e90d935618918cb82d12c3 --abi ../samples/1_Accumulator.abi.json --keys keys/key0 -m add --value 1
Input arguments:
 address: 0:75186644bf5157d1b638390889ec2ba297a12250f6e90d935618918cb82d12c3
  method: add
  params: {"value":"1"}
     abi: ../samples/1_Accumulator.abi.json
    keys: keys/key0
Connecting to:
        Url: net.evercloud.dev
        Endpoints: ["https://devnet.evercloud.dev/b2ad82504ee54fccb5bc6db8cbb3df1e"]

starting new connection: https://devnet.evercloud.dev/
Last block "76657141a65727996dadf9b929d40887cc3c78df09a97b9daecabd8b3e01327a"
MessageId: 64c98e8fbf5aa9ccf9d6526c6275bc617f6eb6f747b616f82e85cda7403c165b
message_expiration_time 1664983987
fetch_block_timeout 88688
1664983931: block received {
  "id": "b3ab65d5b8503dedfa1250d72b7d8247e802551dfedabb80b82454abb6e755ce",
  "gen_utime": 1664983924,
  "after_split": false,
  "workchain_id": 0,
  "shard": "7800000000000000",
  "in_msg_descr": []
}
fetch_block_timeout 85461
1664983933: block received {
  "id": "8dc7cc2c4ab9be6b4ac0e9d3bcd6aac3179825683f4c8df02f76e9929a649ffc",
  "gen_utime": 1664983926,
  "after_split": false,
  "workchain_id": 0,
  "shard": "7800000000000000",
  "in_msg_descr": []
}
fetch_block_timeout 83209
1664983936: block received {
  "id": "b6977305cf28b86a0547a7fd34c03ad0534a94fb5453c3639e5f28e18a0c5d6b",
  "gen_utime": 1664983929,
  "after_split": false,
  "workchain_id": 0,
  "shard": "7800000000000000",
  "in_msg_descr": [
    {
      "msg_id": "64c98e8fbf5aa9ccf9d6526c6275bc617f6eb6f747b616f82e85cda7403c165b",
      "transaction_id": "796ebf67fab053ea88bdf9a971d088fc6dbcb47b106f420c740815246f28c8b7"
    }
  ]
}
Succeeded.
Result: {}
```

# 3. Cryptographic commands

## 3.1. Create seed phrase

To generate a mnemonic seed phrase enter the following command:

```bash
ever-cli genphrase [--dump <path>]
```

Options:

`--dump <path>` - Path where to dump keypair generated from the phrase.

Example:

```bash
$ ever-cli genphrase
Config: /home/user/ever-cli.conf.json
Succeeded.
Seed phrase: "rule script joy unveil chaos replace fox recipe hedgehog heavy surge online"

$ ever-cli genphrase --dump /tmp/1.key
Succeeded.
Seed phrase: "resist immune key jar lunar snake real vintage chicken radar famous cinnamon"
Keypair successfully saved to /tmp/1.key.
Succeeded.
Keypair saved to /tmp/1.key
```

## 3.2. Generate public key

To generate a public key from a seed phrase enter the following command with the seed phrase in quotes:

```bash
ever-cli genpubkey "rule script joy unveil chaos replace fox recipe hedgehog heavy surge online"
```

The generated QR code also contains the public key.

Example:

```bash
$ ever-cli genpubkey "rule script joy unveil chaos replace fox recipe hedgehog heavy surge online"
Config: /home/user/ever-cli.conf.json
Succeeded.
Public key: 88c541e9a1c173069c89bcbcc21fa2a073158c1bd21ca56b3eb264bba12d9340

<QR code with key>

```

## 3.3. Generate key pair file

To create a key pair file from a seed phrase use the following command:

```bash
ever-cli getkeypair [-o <keyfile.json>] [-p "<seed_phrase>"]
```

`<keyfile.json>` - the file the key pair will be written to. If not specified keys will be printed to the stdout.
`"<seed_phrase>"` - seed phrase or secret key. If not specified a new phrase will be generated.
Example:

```bash
$ ever-cli getkeypair -o key.json -p "rule script joy unveil chaos replace fox recipe hedgehog heavy surge online"
Config: /home/user/ever-cli/ever-cli.conf.json
Input arguments:
key_file: key.json
  phrase: rule script joy unveil chaos replace fox recipe hedgehog heavy surge online
Keypair successfully saved to key.json.
Succeeded.

$ ever-cli getkeypair -o key.json
Config: /home/user/ever-cli/ever-cli.conf.json
Input arguments:
key_file: key.json
  phrase: None
Generating seed phrase.
Seed phrase: "elephant tone error jazz scrap wise kick walk panda snake right feature"
Keypair successfully saved to key.json.
Succeeded.


$ ever-cli getkeypair
Config: /home/user/ever-cli/ever-cli.conf.json
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

$ ever-cli -j getkeypair
{
  "public": "09889cd2f085a693ef04a6dad4b6533c7019014a7e0ca9b5b146e66e550973d9",
  "secret": "021196259435d54dfb5c41970db5bcfc2306d59877665c3b573486d441cf021a"
}
```

# 4. Smart contract commands

When working with smart contracts, EVER-CLI requires the following files:

- **ABI file** - a .json file that describes the contract interface, the methods and parameters used to interact with it.
- **TVC file** - the compiled smart contract file. Used only when generating contract address and deploying contract code to the blockchain.
- **Key pair file** - used in contracts with implemented authorization. It is the file containing [private and public keys](#3-cryptographic-commands) authorized to access the contract. In `--sign` parameter the corresponding seed phrase may be used instead of it.

By default, the utility looks for these files in the current working directory.

## 4.1. Generate contract address

Contract address uniquely identifies the contract on the blockchain. Contract balance is attached to its address, the address is used for any interactions with the contract, such as calling contract functions, sending messages, etc.

Contract address is generated based on contract TVC file and selected keys. To get a different address for the same type of contract, use different keys.

> **Note**:  For contracts with ABI 2.4, you should use the flag `--save` to insert the deployment public key into the TCV file.

> **Note**: If your contract has static variables, they can be initialized through [TVM linker](https://github.com/everx-labs/TVM-linker#5-initialize-static-variables-in-compiled-contract) before deployment.

Use the following command to generate the contract address:

```bash
ever-cli genaddr [--genkey|--setkey <keyfile.json>] [--wc <int8>] [--abi <contract.abi.json>] [--save] [--data <data>] <contract.tvc>
```

Options:

`--genkey <keyfile.json>` - generate new `keyfile.json` key pair file and use it to calculate the contract address.

> Note: if you use --genkey, the corresponding seed phrase will be displayed. Write it down, if you mean to keep using this key pair.

`--abi <contract.abi.json>` - contract ABI interface file. If not specified ever-cli can use ABI path from config of obtained from tvc path (for `<contrac>.tvc` checks `<contract>.abi.json`).

`--setkey <keyfile.json>` - use already [existing](#33-generate-key-pair-file) `keyfile.json` key pair file to calculate the contract address. Seed phrase cannot be used instead of the file.

`--wc <int8>` - ID of the workchain the contract will be deployed to (`-1` for masterchain, `0` for basechain). By default, this value is set to 0.

`--save` - If this flag is specified, modifies the tvc file with the keypair and initial data.

`--data <data>` - Initial data to insert into the contract. Should be specified in json format.

`<contract.tvc>` - compiled smart contract file.

As a result ever-cli displays the new contract address (`Raw address`).

Example ([multisignature wallet](https://github.com/everx-labs/ton-labs-contracts/tree/master/solidity/safemultisig) address generation for the masterchain):

```bash
$ ever-cli genaddr --genkey key.json --wc -1 SafeMultisigWallet.tvc --abi SafeMultisigWallet.abi.json
Config: /home/user/ever-cli.conf.json
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

> **Note**:  For contracts using ABI 2.4, it is necessary to first insert the deployment public key into the TCV file. This can be achieved using the `genaddr` function.

> **Note**: If your contract has static variables, they can be initialized with [genaddr command](#41-generate-contract-address) before deployment.

Use the following command to deploy a contract:

```bash
ever-cli deploy [--sign <deploy_seed_or_keyfile>] [--wc <int8>] [--abi <contract.abi.json>] [--alias <alias>] <contract.tvc> <params>
```

`<deploy_seed_or_keyfile>` - can either be the seed phrase used to generate the deployment key pair file or the key pair file itself. If seed phrase is used, enclose it in double quotes.

Example:
  `--sign "flip uncover dish sense hazard smile gun mom vehicle chapter order enact"`
or
  `--sign deploy.keys.json`

`--wc <int8>` ID of the workchain the wallet will be deployed to (`-1` for masterchain, `0` for basechain). By default, this value is set to 0.

`--abi <contract.abi.json>` - Path or link to the contract ABI file or pure json ABI data. Can be specified in the config file.

`--alias <alias>` - allows to save contract parameters (address, abi, keys) to use them easier with `callx` or `runx` commands.

`<contract.tvc>` - compiled smart contract file.

`<params>` - deploy command parameters, depend on the contract.

Sometimes it can be not obvious in which way method parameters should be specified,
especially if it is a large structure with different and complex fields.
It is generally described in [abi doc](https://github.com/everx-labs/ever-abi/blob/master/docs/ABI_2.1_spec.md).
Example ([multisignature wallet](https://github.com/everx-labs/ton-labs-contracts/tree/master/solidity/safemultisig)
contract deployment to the masterchain):

```bash
$ ever-cli deploy --sign key.json --wc -1 --abi SafeMultisigWallet.abi.json SafeMultisigWallet.tvc '{"owners":["0x88c541e9a1c173069c89bcbcc21fa2a073158c1bd21ca56b3eb264bba12d9340"],"reqConfirms":1}'
Config: /home/user/ever-cli.conf.json
Input arguments:
     tvc: SafeMultisigWallet.tvc
  params: {"owners":["0x88c541e9a1c173069c89bcbcc21fa2a073158c1bd21ca56b3eb264bba12d9340"],"reqConfirms":1}
     abi: SafeMultisigWallet.abi.json
    keys: key.json
      wc: -1
Connecting to net.evercloud.dev
Deploying...
Transaction succeeded.
Contract deployed at address: -1:0c5d5215317ec8eef1b84c43cbf08523c33f69677365de88fe3d96a0b31b59c6
```

## 4.3. Generate deploy message offline

If needed, signed deploy message can be generated without immediately broadcasting it to the blockchain. Generated
message can be sent later.

```bash
ever-cli deploy_message [--raw] [--output <path_to_file>] [--signature_id <value>] [--sign <deploy_seed_or_keyfile>] [--wc <int8>] [--abi <contract.abi.json>] <contract.tvc> <params>
```

`--raw` - use to create raw message boc.

`--output <path_to_file>` - specify path to file where the raw message should be written to, instead of printing it to terminal.

`--signature_id <value>` - use this option to designate a specific signature_id for signing your message. For an automated online retrieval of the signature_id, set the `value` to `online`, which will fetch it from the network configuration. Alternatively, input a specific numerical value to facilitate offline message signing.

`<deploy_seed_or_keyfile>` - can either be the seed phrase used to generate the deployment key pair file or the key pair file itself. If seed phrase is used, enclose it in double quotes.

Example:

`--sign "flip uncover dish sense hazard smile gun mom vehicle chapter order enact"`

or

`--sign deploy.keys.json`

`--wc <int8>` ID of the workchain the wallet will be deployed to (`-1` for masterchain, `0` for basechain). By default, this value is set to 0.

`<contract.abi.json>` - contract interface file.

`<contract.tvc>` - compiled smart contract file.

`<params>` - deploy command parameters, depend on the contract.

Example (saving to a file [multisignature wallet](https://github.com/everx-labs/ton-labs-contracts/tree/master/solidity/safemultisig) contract deployment message to the masterchain):

```bash
$ ever-cli deploy_message --raw --output deploy.boc --sign key.json --wc -1 --abi SafeMultisigWallet.abi.json SafeMultisigWallet.tvc '{"owners":["0x88c541e9a1c173069c89bcbcc21fa2a073158c1bd21ca56b3eb264bba12d9340"],"reqConfirms":1}'
Config: /home/user/ever-cli.conf.json
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
ever-cli account [--boc] <list_of_addresses> [--dumptvc <tvc_path>] [--dumpboc <boc_path>]
```

`<list_of_addresses>` - contract [addresses](#41-generate-contract-address), if not specified address is taken from the config file.
`--dumptvc <tvc_path>` - this flag can be specified to dump account StateInit to the <tvc_path> file.
`--dumpboc <boc_path>` - this flag can be specified to dump account boc to the <boc_path> file.
`--boc` - flag that changes behaviour of the command to work with the saved account state from the BOC file. In this case path to the boc file should be specified instead of address.

Example:

```bash
$ ever-cli  account 0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13 0:14014af4a374bdd13dae2379063ea2597634c2c2fc8e99ca9eab431a7ab6f566  0:f89d946b5b4b8a06f01dc20dceef30caff844d5285abea8a21ad3730c0f3dd12
Config: /home/user/ever-cli/ever-cli.conf.json
Input arguments:
addresses: 0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13, 0:14014af4a374bdd13dae2379063ea2597634c2c2fc8e99ca9eab431a7ab6f566, 0:f89d946b5b4b8a06f01dc20dceef30caff844d5285abea8a21ad3730c0f3dd12
Connecting to net.evercloud.dev
Processing...
Succeeded.
address:       0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13
acc_type:      Active
balance:       11466383488239689 nanoton
last_paid:     1640619135
last_trans_lt: 0x530197a143
data_boc:      b5ee9c720101060100b000014195c06aa743d1f9000dd64b75498f106af4b7e7444234d7de67ea26988f6181dfe001020120050202012004030052bf874da2f56d034e11773c58331900e0e1e91a137e1b4c2ca15607634c2d63e1af0000000061c9dca50052bfbddf9156dc04cca88cf25d9c766b1bd2f1ab7d0878c4d761862fc524758767f10000000061c9dc820053bfd627d55f960de2235b3f1537884d5968e5e486c58c581bc9ea4068c8da164ce18000000030e4ee49c0
code_hash:     ccbfc821853aa641af3813ebd477e26818b51e4ca23e5f6d34509215aa7123d9

address:       0:14014af4a374bdd13dae2379063ea2597634c2c2fc8e99ca9eab431a7ab6f566
acc_type:      Active
balance:       2082745497066 nanoton
last_paid:     1640619517
last_trans_lt: 0x530a3c2782
data_boc:      b5ee9c7201020c0100022e000373000000befe45557e0000000000000000000000000002faf04e577e5cf5b28c2a81afc5ae534a0f3f494cc4ee62ef675ca8e36af911a3c8767a400b0a010183801f13b28d6b697140de03b841b9dde6195ff089aa50b57d514435a6e6181e7baba318b50f6f18c9d307d500216c80d6ecd77d13e437bdfcaf0b4fa6b9204b7847500203a1c00b620939e214cadb7481682034e58a853a77874f473c69cc7d3b1ad9da7f0bafa0000000280000000c0000000bddcfa66622a7b9c955271c779b92448cff442b8efead77d43bd7f50b07a45f380030010706030203cca005040045b41bda168cd2322b5dcd28989176a9eae590288db4d548f2b6948d214de0c9bdb372700045b6554f714ca768f21ad18cff20c7af62091e9fc2d40c06d32d1ace7495f5dd1605781000bda90017d76e405363a8a494a3a8d8c38fcadd4f2c7fb550244fd6d2a77ac12eb029bce000000000000255400000000000000000000000000000034c3babc06000000000000000000000000000000000000000000000000000000000000000100201200908009bbfe85a3348c8ad7734a26245daa7ab9640a236d35523cada523485378326f6cdc9800000000000106f0000000000000000000000000002035ac0000000000000000000000000000000187c4b00e0007bbffdc5329da3c86b4633fc831ebd88247a7f0b50301b4cb46b39d257d7745815e0000000000000095500000000000000000000000002f8eb24987c490760000454310010546f6b656e202331
code_hash:     eee7d3331153dce4aa938e3bcdc922467fa215c77f56bbea1debfa8583d22f9c

0:f89d946b5b4b8a06f01dc20dceef30caff844d5285abea8a21ad3730c0f3dd12 not found


$ ever-cli  account 0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13
Config: /home/user/ever-cli/ever-cli.conf.json
Input arguments:
addresses: 0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13
Connecting to net.evercloud.dev
Processing...
Succeeded.
address:       0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13
acc_type:      Active
balance:       11463682795615708 nanoton
last_paid:     1640624439
last_trans_lt: 0x5379939282
data_boc:      b5ee9c7201010401008100014195c06aa743d1f9000dd64b75498f106af4b7e7444234d7de67ea26988f6181dfe00102012003020053bfde8d98393e5db0ea2f609ed9266cf61a7487759d679ea9792adbdcfc137f6caf8000000030e4f89dc00053bfc8658b6b027767d9addd720a0bf8b157379a9b0e9208bab53ad4ee54358c6ce98000000030e4f89dc0
code_hash:     ccbfc821853aa641af3813ebd477e26818b51e4ca23e5f6d34509215aa7123d9

```

## 4.4. Call method

### 4.4.1. Call contract on the blockchain

```bash
ever-cli call [--abi <contract.abi.json>] [--sign <seed_or_keyfile>] [--saved_config <config_contract_path>] <address> <method> <params>
```

`<contract.abi.json>` - contract interface file.

`<config_contract_path>` - path to the file with saved config contract state. Is used for debug on fail.

`<seed_or_keyfile>` - can either be the seed phrase or the corresponding key pair file. If seed phrase is used, enclose it in double quotes.

Example:

- `--sign "flip uncover dish sense hazard smile gun mom vehicle chapter order enact"`

or

- `--sign keyfile.json`

`<address>` - contract [address](#41-generate-contract-address).

`<method>` - the method being called.

`<params>` - parameters of the called method. Can be specified by a path to the file, which contains parameters in json
format.
Sometimes it can be not obvious in which way method parameters should be specified,
especially if it is a large structure with different and complex fields.
It is generally described in [abi doc](https://github.com/everx-labs/ever-abi/blob/master/docs/ABI_2.1_spec.md).

Example (transaction creation in a [multisignature wallet](https://github.com/everx-labs/ton-labs-contracts/tree/master/solidity/safemultisig) contract):

```bash
$ ever-cli call 0:a4629d617df931d8ad86ed24f4cac3d321788ba082574144f5820f2894493fbc submitTransaction '{"dest":"-1:0c5d5215317ec8eef1b84c43cbf08523c33f69677365de88fe3d96a0b31b59c6","value":234000000,"bounce":false,"allBalance":false,"payload":""}' --abi SetcodeMultisigWallet.abi.json --sign k1.keys.json
Config: /home/user/ever-cli.conf.json
Input arguments:
 address: 0:a4629d617df931d8ad86ed24f4cac3d321788ba082574144f5820f2894493fbc
  method: submitTransaction
  params: {"dest":"-1:0c5d5215317ec8eef1b84c43cbf08523c33f69677365de88fe3d96a0b31b59c6","value":234000000,"bounce":false,"allBalance":false,"payload":""}
     abi: SetcodeMultisigWallet.abi.json
    keys: k1.keys.json
lifetime: None
  output: None
Connecting to net.evercloud.dev
Generating external inbound message...

MessageId: c6baac843fefe6b9e8dc3609487a63ef21207e4fdde9ec253b9a47f7f5a88d01
Expire at: Sat, 08 May 2021 14:52:23 +0300
Processing...
Succeeded.
Result: {
  "transId": "6959885776551137793"
}
```

**Note**: If your function is marked as [responsible](https://github.com/everx-labs/TON-Solidity-Compiler/blob/master/API.md#external-function-calls), EVER-CLI expects `_answer_id` field, and you may encounter errors, if it's missing.

### 4.4.2. Run contract method locally

```bash
ever-cli run [--abi <contract.abi.json>] <address> <method> <params>
```

`<contract.abi.json>` - contract interface file.

`<address>` - contract [address](#41-generate-contract-address).

`<method>` - the method being called.

`<params>` - parameters of the called method.
Sometimes it can be not obvious in which way method parameters should be specified,
especially if it is a large structure with different and complex fields.
It is generally described in [abi doc](https://github.com/everx-labs/ever-abi/blob/master/docs/ABI_2.1_spec.md).

Example of a transaction list request in a [multisignature wallet](https://github.com/everx-labs/ton-labs-contracts/tree/master/solidity/safemultisig):

```bash
$ ever-cli run 0:a4629d617df931d8ad86ed24f4cac3d321788ba082574144f5820f2894493fbc getTransactions {} --abi SafeMultisigWallet.abi.json
Config: /home/user/ever-cli.conf.json
Input arguments:
 address: 0:a4629d617df931d8ad86ed24f4cac3d321788ba082574144f5820f2894493fbc
  method: getTransactions
  params: {}
     abi: SafeMultisigWallet.abi.json
    keys: None
lifetime: None
  output: None
Connecting to net.evercloud.dev
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
ever-cli runget [--boc] [--tvc] <address> <method> [<params>...] [--bc_config <config_path>]
```

`<address>` - contract [address](#41-generate-contract-address) or path to the file with:
* account boc (It can be obtained from the TON Live) if `--boc` option is used;
* account state init if flag `--tvc` is used.

`<method>` - the method being called.

`<params>` - parameters of the called method. Can have multiple values: one for each function parameter.
Parameters should be specified separately without json wrap and argument names.

`--bc_config <config_path>` - this option can be used with `--boc` option to specify the file with the blockchain config
BOC. It can be obtained with [dump blockchain config](#93-dump-blockchain-config) command.

Example:

```bash
$ ever-cli runget -1:3333333333333333333333333333333333333333333333333333333333333333 active_election_id
Config: /home/user/ever-cli.conf.json
Input arguments:
 address: -1:3333333333333333333333333333333333333333333333333333333333333333
  method: active_election_id
  params: None
Connecting to net.evercloud.dev
Running get-method...
Succeded.
Result: ["1619901678"]

$ ever-cli runget --boc acc.boc compute_returned_stake 0x0166d0181a19f87af9397040a68671e1b239f12152824f7d987fd6897d6a9587
Config: /home/user/ever-cli/ever-cli.conf.json
Input arguments:
 address: acc.boc
  method: compute_returned_stake
  params: ["0x0166d0181a19f87af9397040a68671e1b239f12152824f7d987fd6897d6a9587"]
Connecting to main.evercloud.dev
Running get-method...
Succeeded.
Result: ["125387107580525"]

$ ever-cli runget --tvc acc.tvc compute_returned_stake 0x0166d0181a19f87af9397040a68671e1b239f12152824f7d987fd6897d6a9587
Config: /home/user/ever-cli/ever-cli.conf.json
Input arguments:
 address: acc.boc
  method: compute_returned_stake
  params: ["0x0166d0181a19f87af9397040a68671e1b239f12152824f7d987fd6897d6a9587"]
Connecting to main.evercloud.dev
Running get-method...
Succeeded.
Result: ["125387107580525"]
```



### 4.4.4. Run contract method locally for saved account BOC

```bash
ever-cli run [--boc] [--tvc] [--abi <contract.abi.json>] <account> <method> <params> [--bc_config <config_path>] [--saved_config <config_contract_path>]
```

`<contract.abi.json>` - contract interface file.

`<account>` - path to the file with account boc for flag `--boc` or account state init for flag `--tvc`
(they can be obtained from the network with `account` command).

`<config_contract_path>` - path to the file with saved config contract state. Is used for debug on fail.

`<method>` - the method being called.

`<params>` - parameters of the called method.

`--bc_config <config_path>` - this option can be used with `--boc` option to specify the file with the blockchain config
BOC. It can be obtained with [dump blockchain config](#93-dump-blockchain-config) command.

Example:

```bash
$ ever-cli run --boc tests/depool_acc.boc getData '{}' --abi tests/samples/fakeDepool.abi.json
Config: /home/user/ever-cli/ever-cli.conf.json
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

$ ever-cli run --tvc tests/depool_acc.tvc getData '{}' --abi tests/samples/fakeDepool.abi.json
Config: /home/user/ever-cli/ever-cli.conf.json
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
ever-cli message [--raw] [--output <path_to_file>] [--signature_id <value>] [--abi <contract.abi.json>] [--sign <seed_or_keyfile>] <address> <method> <params> [--lifetime <seconds>]
```

`--raw` - use to create raw message boc.

`--output <path_to_file>` - specify path to file where the raw message should be written to, instead of printing it to terminal.

`--signature_id <value>` - use this option to designate a specific signature_id for signing your message. For an automated online retrieval of the signature_id, set the `value` to `online`, which will fetch it from the network configuration. Alternatively, input a specific numerical value to facilitate offline message signing.

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

The EVER-CLI utility displays encrypted message text and a QR code that also contains the message.Copy the message text or scan the QR code and broadcast the message online.

Example (raw boc of create new multisig transaction message with a lifetime of 1 hour saved to file):

```bash
$ ever-cli message --raw --output message.boc --sign k1.keys.json --abi SafeMultisigWallet.abi.json 0:a4629d617df931d8ad86ed24f4cac3d321788ba082574144f5820f2894493fbc submitTransaction '{"dest":"-1:0c5d5215317ec8eef1b84c43cbf08523c33f69677365de88fe3d96a0b31b59c6","value":234000000,"bounce":false,"allBalance":false,"payload":""}' --lifetime 3600
Config: /home/user/ever-cli.conf.json
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

Use the following command to send a previously generated message, that is not in raw format, and not in a file:

```bash
ever-cli send [--abi <contract.abi.json>] "<message_text>"
```

`<contract.abi.json>` - contract interface file.

`<message_text>` – the content of the message generated by the EVER-CLI utility during message creation. It should be enclosed in double quotes.

Example:

```bash
$ ever-cli send --abi SafeMultisigWallet.abi.json "7b226d7367223a7b226d6573736167655f6964223a2266363364666332623030373065626264386365643265333865373832386630343837326465643036303735376665373430376534393037646266663338626261222c226d657373616765223a227465366363674542424145413051414252596742534d553677767679593746624464704a365a5748706b4c7846304545726f4b4a36775165555369536633674d41514868757856507a324c5376534e663344454a2f374866653165562f5a78324d644e6b4b727770323865397a7538376a4d6e7275374c48685965367642523141756c48784b44446e4e62344f47686768386e6b6b7a48386775456e7551422f655a61324d326d32546539794234723636447a61364c34635258306f744a4b465661434177414141586c4d464e7077594a61616b524d64677332414341574f663459757151715976325233654e776d49655834517048686e37537a75624c76524838657931425a6a617a6a414141414141414141414141414141414a4d61735142414d4141413d3d222c22657870697265223a313632303438323730352c2261646472657373223a22303a61343632396436313764663933316438616438366564323466346361633364333231373838626130383235373431343466353832306632383934343933666263227d2c226d6574686f64223a227375626d69745472616e73616374696f6e227d"
Config: /home/user/ever-cli.conf.json
Input arguments:
 message: 7b226d7367223a7b226d6573736167655f6964223a2266363364666332623030373065626264386365643265333865373832386630343837326465643036303735376665373430376534393037646266663338626261222c226d657373616765223a227465366363674542424145413051414252596742534d553677767679593746624464704a365a5748706b4c7846304545726f4b4a36775165555369536633674d41514868757856507a324c5376534e663344454a2f374866653165562f5a78324d644e6b4b727770323865397a7538376a4d6e7275374c48685965367642523141756c48784b44446e4e62344f47686768386e6b6b7a48386775456e7551422f655a61324d326d32546539794234723636447a61364c34635258306f744a4b465661434177414141586c4d464e7077594a61616b524d64677332414341574f663459757151715976325233654e776d49655834517048686e37537a75624c76524838657931425a6a617a6a414141414141414141414141414141414a4d61735142414d4141413d3d222c22657870697265223a313632303438323730352c2261646472657373223a22303a61343632396436313764663933316438616438366564323466346361633364333231373838626130383235373431343466353832306632383934343933666263227d2c226d6574686f64223a227375626d69745472616e73616374696f6e227d
     abi: SafeMultisigWallet.abi.json
Connecting to net.evercloud.dev

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

Use the following command to send a previously generated message, that is stored in a .boc file:

```bash
ever-cli sendfile <path_to_boc_file>
```

`<path_to_boc_file>` – path to the file where the message was saved.

Example:

```bash
$ ever-cli sendfile /home/user/ton/message.boc
Config: /home/user/ever-cli.conf.json
Input arguments:
     boc: /home/user/ton/message.boc
Connecting to net.evercloud.dev
Sending message to account 0:a4629d617df931d8ad86ed24f4cac3d321788ba082574144f5820f2894493fbc
Succeded.
```

## 4.8. Decode commands

### 4.8.1. Decode BOC file

Use the following command to decode previously generated messages in .boc files.

```bash
ever-cli decode msg --abi <contract.abi.json> <path_to_boc_file>
```

`<contract.abi.json>` - contract ABI file.

`<path_to_boc_file>` – path to the file where the message was saved.

Example:

```bash
$ ever-cli decode msg --abi SafeMultisigWallet.abi.json /home/user/ton/message.boc
Config: /home/user/ever-cli.conf.json
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
ever-cli decode body --abi <contract.abi.json> "<message_body>"
```

`<contract.abi.json>` - contract interface file.

`<message_body>` - Message body encoded as base64.

```bash
$ ever-cli decode body --abi SafeMultisigWallet.abi.json "te6ccgEBAwEAqwAB4diOBnSVls3D8/zEb/Uj6hIfwKrdG2uRyCWmWx+mpFtdbaZNBcTW3yS3QiwLR8NgoqLcqoDsGwDA/RbrJLen+wXhJ7kAf3mWtjNptk3vcgeK+ug82ui+HEV9KLSShVWggMAAAF5S//FEWCWlSsTHYLNgAQFjn+GLqkKmL9kd3jcJiHl+EKR4Z+0s7my70R/HstQWY2s4wAAAAAAAAAAAAAAAAb5R0AQCAAA="
Config: /home/user/ever-cli.conf.json
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
ever-cli decode account data --abi <contract.abi.json> --addr <contract_address>
ever-cli decode account data --abi <contract.abi.json> --tvc <contract_file>
```

`<contract.abi.json>` - contract interface file.

Contract address on blockchain or path to the file with contract's StateInit can be specified
with options `--addr` and `--tvc` respectively.

```bash
$ ever-cli decode account data --abi tests/test_abi_v2.1.abi.json --tvc tests/decode_fields.tvc
Config: /home/user/ever-cli/ever-cli.conf.json
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
ever-cli decode account boc <boc_file> [--dumptvc <tvc_path>]
```

`<boc_file>` - path to the file with BOC of the account. E.g. it can be obtained from
the TON Live.
`--dumptvc <tvc_path>` - this flag can be specified to dump account StateInit to the <tvc_path> file.

```bash
$ ever-cli decode account boc tests/account.boc --dumptvc acc.tvc
Config: /home/user/ever-cli/ever-cli.conf.json
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
ever-cli decode stateinit [--tvc] [--boc] <input>
```

`<input>` - depending on the flags this parameter should contain:
- path to the file with account BOC if `--boc` flag is specified;
- path to the TVC file if `--tvc` flag is specified;
- contract network address otherwise.

```bash
$ ever-cli decode stateinit --boc account.boc
Config: /home/user/ever-cli/ever-cli.conf.json
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

$ ever-cli decode stateinit --tvc fakeDepool.tvc
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

$ ever-cli decode stateinit 989439e29664a71e57a21bff0ff9896b5e58018fcac32e83fade913c4f43479e
Config: /home/user/ever-cli/ever-cli.conf.json
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
ever-cli body [--abi <contract.abi.json>] <method> <params>
```

`<contract.abi.json>` - contract interface file.

`<method>` - the method being called.

`<params>` - parameters of the called method.

Example:

```bash
$ ever-cli body submitTransaction '{"dest":"-1:0c5d5215317ec8eef1b84c43cbf08523c33f69677365de88fe3d96a0b31b59c6","value":234000000,"bounce":false,"allBalance":false,"payload":""}' --abi SetcodeMultisigWallet.abi.json
Config: /home/user/ever-cli.conf.json
Input arguments:
  method: submitTransaction
  params: {"dest":"-1:0c5d5215317ec8eef1b84c43cbf08523c33f69677365de88fe3d96a0b31b59c6","value":234000000,"bounce":false,"allBalance":false,"payload":""}
     abi: SetcodeMultisigWallet.abi.json
  output: None
Message body: te6ccgEBAgEAOwABaxMdgs2f4YuqQqYv2R3eNwmIeX4QpHhn7SzubLvRH8ey1BZjazjAAAAAAAAAAAAAAAABvlHQBAEAAA==
```

## 4.10. Alternative syntax for call, deploy and run commands

To facilitate usage of ever-cli use commands `callx`, `runx` and `deployx` instead of `call`, `run` and `deploy`.
These alternative syntax commands have almost the same syntax as classic, but allow to specify address, abi and keys
options in the config file. Also, this commands allow to skip params option if command doesn't need it.
Examples:

```bash
# specify options manually
ever-cli callx --keys giver.key --abi giver.abi.json --addr 0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94 -m sendGrams --dest 841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94 --amount 1000000000

# options are taken from the config
ever-cli config --abi giver.abi.json --addr 0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94 --keys giver.key
ever-cli callx -m sendGrams --dest 841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94 --amount 1000000000

# if contract function or constructor doesn't take arguments, parameters can be skipped
ever-cli deployx contract.tvc
ever-cli runx -m getParameters

# method and parameters can be specified in config
ever-cli config --method add --parameters '{"value":1}' --addr 0:41af055743c85ba58fcaead78fa45b017f265c9351b5275ad76bf58be11760fd --abi ../samples/1_Accumulator.abi.json --keys keys/key0
ever-cli callx
ever-cli config --method sum --parameters '{}'
ever-cli runx
```

If some parameters have names equal to options, use can tell the ever-cli that you have started mentioning parameters
by using empty `--`. Examples:

```bash
# abi, addr, keys and method are specified as options and after `--` they are specified again as arguments.
ever-cli callx --abi arguments.abi.json --addr 0:62c2040f7f7406732037c1856e91732be3f9907b94fb34f53ba664ba94b228f6 --keys argument.key --method add -- --addr 2 --keys 3 --abi 4 --method 5
# abi, addr, key and method are specified as arguments because `--` is specified in the beginning. Abi, addr, keys and method options are taken from the config.
ever-cli callx -- --addr 2 --keys 3 --abi 4 --method 5
```

# 5. DeBot commands

EVER-CLI has a built-in DeBot <link to DeBots repo> browser, which is regularly updated with the most recent versions of DEngine <link to DEngine>.

To call a DeBot, use the following command:

```bash
ever-cli debot fetch <--debug> <debot_address>
```

`<debot_address>` - address of the DeBot contract.

`<--debug>` - runs DeBot in verbose mode.

Example:

```bash
$ ever-cli debot fetch 0:09403116d2d04f3d86ab2de138b390f6ec1b0bc02363dbf006953946e807051e
Config: /home/user/ever-cli.conf.json
Connecting to net.evercloud.dev
DeBot Info:
Name   : Multisig
Version: 1.2.0
Author : EverX
Publisher: EverX
Support: 0:66e01d6df5a8d7677d9ab2daf7f258f1e2a7fe73da5320300395f99e01dc3b5f
Description: DeBot for multisig wallets
Hi, I will help you work with multisig wallets that can have multiple custodians.
Run the DeBot (y/n)?
y

Which wallet do you want to work with?
```

Further input depends on the DeBot, which usually explains any actions it offers you to perform.

# 6. Multisig commands

Multisig commands allow you to work with any existing Multisig wallets <link to repo> in a more convenient way and with
no need of ABI files.

## 6.1. Send tokens

Use the following command to send tokens to any recipient:

```bash
ever-cli multisig send --addr <sender_address> --dest <recipient_address> --purpose <"text_in_quotes"> --sign <path_to_keys_or_seed_phrase> --value *number* [--v2]
```

`<sender_address>` - address of the multisig wallet that tokens are sent from.

`<recipient_address>` - address of the account tokens are sent to.

`<"text_in_quotes">` - accompanying message. Only the recipient will be able to decrypt and read it.

`<path_to_keys_or_seed_phrase>` - path to sender wallet key file or the corresponding seed phrase in quotes.

`--value *number*` - value to be transferred (in tokens).

`--v2` - optional flag, force to interpret wallet as multisig v2.

Example:

```bash
$ ever-cli multisig send --addr 0:255a3ad9dfa8aa4f3481856aafc7d79f47d50205190bd56147138740e9b177f3 --dest 0:a4629d617df931d8ad86ed24f4cac3d321788ba082574144f5820f2894493fbc --purpose "test transaction" --sign key.json --value 6
Config: /home/user/ever-cli.conf.json
Connecting to net.evercloud.dev
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
ever-cli multisig deploy [--setcode] [--v2] [--owners <owners_list>] [--confirms <confirms_cnt>] [--local <local_giver_value>] --keys <path_to_keys_or_seed_phrase>
```

`--setcode` - flag that changes type of the wallet to the SetcodeMultisigWallet. If not specified, a SafeMultisigWallet is deployed.

`--owners <owners_list>` - option that sets wallet owners. If not specified, the only owner is the key deployment message was signed with (set with --keys option).
List of owners must be specified by their public keys in hex format, split by the `,`.

`--confirms <confirms_cnt>` - option that sets required number of confirmations. If not specified, is set to 1.

`--local <local_giver_value>` - value that should be transferred from the local giver if wallet is deployed onto the Node SE (in nanotons).

`--keys <path_to_keys_or_seed_phrase>` - path to the wallet key file or the corresponding seed phrase in quotes.

`--v2` - optional flag, force to deploy multisig v2.


Example:

```bash
$ ever-cli multisig deploy -k "young tell target alter sport dignity enforce improve pottery fashion alert genuine" --local 1_000_000_000
Config: /home/user/ever-cli/ever-cli.conf.json
Wallet address: 0:4d892e63989c1c0ad64b0bbe22e8d036b0da271c19b6686d01bd29a99dcbc86d
Connecting to http://127.0.0.1/
Expire at: Mon, 13 Sep 2021 14:55:29 +0300
MessageId: 3c3537e36e2a4a4018b7463db2bf57efad5dc0dc0233b040c2f5e165cb43e887
MessageId: 8102067efc190b2e728d91d632c985634fc4717b7ae1137a4bbcf756c4cf8705
Wallet successfully deployed
Wallet address: 0:4d892e63989c1c0ad64b0bbe22e8d036b0da271c19b6686d01bd29a99dcbc86d

# deploy with owners
ever-cli multisig deploy -l 5000000000 -c 2 -o 8b445b0feab10b9abf4e039d649348ec8662e3673fe9c37b7208c4d9d04c9b3f,ddc5bc7198c90feb75d9ce09e1b1f25a7e14a252fef31b50fac048c6ac3ee46c -k test.key
```

# 7. DePool commands

## 7.1. Configure EVER-CLI for DePool operations

For all commands listed below, the DePool address, the wallet making the stake, the amount of fee to pay for DePool's
services and the path to the keyfile/seed phrase may be specified in the EVER-CLI config file in advance:

```bash
ever-cli config --addr <address> --wallet <address> --no-answer true | false --keys <path_to_keys or seed_phrase> --depool_fee <depool_fee>
```

`--addr <address>` - the address of the DePool

`--wallet <address>` - the address of the wallet making the stake

`--no-answer true | false` - no-answer flag, which determines, whether EVER-CLI waits for DePool answer when performing various actions and prints it out, or simply generates and sends a transaction through the specified multisig wallet, without monitoring transaction results in the DePool. By default, is set to `true`. Setting to false can be useful for catching rejected stakes or other errors on the DePool side.

`<path_to_keys or seed_phrase>` - either the keyfile for the wallet making the stake, or the seed phrase in quotes

`--depool_fee <depool_fee>` - value in tons, that is additionally attached to the message sent to the DePool to cover its fees. Change is returned to the sender. The default value, used if this option isn't configured, is 0.5 tons. It should be increased only if it proves insufficient and DePool begins to run out of gas on execution.

Example:

```bash
ever-cli config --addr 0:37fbcb6e3279cbf5f783d61c213ed20fee16e0b1b94a48372d20a2596b700ace --wallet 0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e --no-answer false --keys "dizzy modify exotic daring gloom rival pipe disagree again film neck fuel" --depool_fee 0.8
```

In this case all DePool commands allow to omit `--addr`, `--wallet`, `--wait-answer` and `--sign` options.

Below is an example of similar DePool commands with and without waiting for DePool answer.

With waiting for DePool answer:

```bash
$ ever-cli depool --addr 0:127ae93241278304fff6b7e5b7b182fd382b6e95b200551061a7354e032e50bf stake ordinary --value 25 --wallet 0:255a3ad9dfa8aa4f3481856aafc7d79f47d50205190bd56147138740e9b177f3 --sign key.json --wait-answer
Config: /home/user/ever-cli.conf.json
Input arguments:
  depool: 0:127ae93241278304fff6b7e5b7b182fd382b6e95b200551061a7354e032e50bf
  wallet: 0:255a3ad9dfa8aa4f3481856aafc7d79f47d50205190bd56147138740e9b177f3
   stake: 25
    keys: key.json
Connecting to https://net.evercloud.dev
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
$ ever-cli depool --addr 0:127ae93241278304fff6b7e5b7b182fd382b6e95b200551061a7354e032e50bf stake ordinary --value 25 --wallet 0:255a3ad9dfa8aa4f3481856aafc7d79f47d50205190bd56147138740e9b177f3 --sign key.json
Config: /home/user/ever-cli.conf.json
Input arguments:
  depool: 0:127ae93241278304fff6b7e5b7b182fd382b6e95b200551061a7354e032e50bf
  wallet: 0:255a3ad9dfa8aa4f3481856aafc7d79f47d50205190bd56147138740e9b177f3
   stake: 25
    keys: key.json
Connecting to https://net.evercloud.dev
Generating external inbound message...

MessageId: e1b0aba39233e07daf6a65c2426e273e9d68a75e3b440893251fbce56c6a756d
Expire at: Thu, 13 May 2021 15:09:43 +0300
Processing...
Succeeded.
Result: {
  "transId": "0"
}
```

In both cases the stake is rejected for being too small, but with `no-answer` set to `false` it isn't immediately
apparent, as only the results of the sussecful multisig transaction are displayed.

## 7.2. Deposit stakes

### 7.2.1. Ordinary stake

Ordinary stake is the most basic type of stake. It and the rewards from it belong to the wallet that made it.

It is invested completely in the current pooling round, and can be reinvested every second round (as odd and even rounds
are handled by DePool separately). Thus, to participate in every DePool round, an ordinary stake should be invested in
two consecutive rounds, so it can later be reinvested in odd and even rounds both.

Ordinary stake must exceed DePool minimum stake. Check DePool's page on [ton.live](https://ton.live/dePools) to find out
the minimum stake.

```bash
ever-cli depool [--addr <depool_address>] stake ordinary [--wallet <msig_address>] --value <number> [--sign <key_file or seed_phrase>] [--wait-answer] [--v2]
```

`<depool_address>` - address of the DePool contract.

`<msig_address>` - address of the wallet making a stake.

all --value parameters must be defined in tons, like this: `--value 10.5`, which means the value is 10,5 tons.

`<key_file or seed_phrase>` - either the keyfile for the wallet making the stake, or the seed phrase in quotes.

`--wait-answer` - optional flag, which forces EVER-CLI to wait for DePool answer when performing the action and print out the resulting message. Without it only the results of the multisig transaction to DePool will be displayed.

`--v2` - optional flag, force to interpret wallet as multisig v2.

Example:

```bash
ever-cli depool --addr 0:37fbcb6e3279cbf5f783d61c213ed20fee16e0b1b94a48372d20a2596b700ace stake ordinary --wallet 0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e --value 100.5 --sign "dizzy modify exotic daring gloom rival pipe disagree again film neck fuel"
```

### 7.2.2. Vesting stake

A wallet can make a vesting stake and define a target participant address (beneficiary) who will own this stake,
provided the beneficiary has previously indicated the donor as its vesting donor address. This condition prevents
unauthorized vestings from blocking the beneficiary from receiving an expected vesting stake from a known address.

**To receive a vesting stake beneficiary must**:

- already have an ordinary stake of any amount in the DePool
- set the donor address with the following command:

```bash
ever-cli depool [--addr <depool_address>] donor vesting [--wallet <beneficiary_address>] --donor <donor_address> [--sign <key_file or seed_phrase>] [--wait-answer] [--v2]
```

`<depool_address>` - address of the DePool contract.

`<beneficiary_address>` - address of the beneficiary wallet .

`<donor_address>` - address of the donor wallet.

`<key_file or seed_phrase>` - either the keyfile for the beneficiary wallet, or the seed phrase in quotes.

`--wait-answer` - optional flag, which forces EVER-CLI to wait for DePool answer when performing the action and print out the resulting message. Without it only the results of the multisig transaction to DePool will be displayed.

`--v2` - optional flag, force to interpret wallet as multisig v2.

Example:

```bash
ever-cli depool --addr 0:3187b4d738d69776948ca8543cb7d250c042d7aad1e0aa244d247531590b9147 donor vesting --wallet 0:255a3ad9dfa8aa4f3481856aafc7d79f47d50205190bd56147138740e9b177f3 --donor 0:279afdbd7b2cbf9e65a5d204635a8630aec2baec60916ffdc9c79a09d2d2893d --sign "deal hazard oak major glory meat robust teach crush plastic point edge"
```

Not the whole stake is available to the beneficiary at once. Instead, it is split into parts and the next part of stake
becomes available to the beneficiary (is transformed into beneficiary's ordinary stake) at the end of the round that
coincides with the end of the next withdrawal period. Rewards from vesting stake are always added to the beneficiary's
ordinary stake. To withdraw these funds, beneficiary should use one of the [withdrawal functions](#75-withdraw-stakes).

Please note, that the vesting stake is split into two equal parts by the DePool, to be used in both odd and even rounds,
so to ensure DePool can participate in elections with just one vesting stake where validator wallet is beneficiary, the
stake should exceed `validatorAssurance` *2. Similarly, to ensure any vesting stake is accepted, make sure it exceeds
`minStake` *2.

**Donor uses the following command to make a vesting stake:**

```bash
ever-cli depool [--addr <depool_address>] stake vesting [--wallet <msig_address>] --value <number> --total <days> --withdrawal <days> --beneficiary <address> [--sign <key_file or seed_phrase>] [--wait-answer] [--v2]
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

`--wait-answer` - optional flag, which forces EVER-CLI to wait for DePool answer when performing the action and print out the resulting message. Without it only the results of the multisig transaction to DePool will be displayed.

`--v2` - optional flag, force to interpret wallet as multisig v2.

Example:

```bash
ever-cli depool --addr 0:37fbcb6e3279cbf5f783d61c213ed20fee16e0b1b94a48372d20a2596b700ace stake vesting --wallet 0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e --value 1000 --total 360 --withdrawal 30 --beneficiary 0:f22e02a1240dd4b5201f8740c38f2baf5afac3cedf8f97f3bd7cbaf23c7261e3 --sign "dizzy modify exotic daring gloom rival pipe disagree again film neck fuel"
```

> Note: Each participant can concurrently be the beneficiary of only one vesting stake. Once the current vesting stake expires, another can be made for the participant.

### 7.2.3. Lock stake

A wallet can make a lock stake, in which it locks its funds in DePool for a defined period, but rewards from this stake
will be paid to another target participant (beneficiary). As with vesting, the beneficiary has to indicate the donor as
its lock donor address before receiving a lock stake. This condition prevents unauthorized lock stakes from blocking the
beneficiary from receiving an expected lock stake from a known address.

**To receive a lock stake beneficiary must**:

- already have an ordinary stake of any amount in the DePool
- set the donor address with the following command:

```bash
ever-cli depool [--addr <depool_address>] donor lock [--wallet <beneficiary_address>] --donor <donor_address> [--sign <key_file or seed_phrase>] [--wait-answer] [--v2]
```

Where

`<depool_address>` - address of the DePool contract.

`<beneficiary_address>` - address of the beneficiary wallet .

`<donor_address>` - address of the donor wallet.

`<key_file or seed_phrase>` - either the keyfile for the beneficiary wallet, or the seed phrase in quotes.

`--wait-answer` - optional flag, which forces EVER-CLI to wait for DePool answer when performing the action and print out the resulting message. Without it only the results of the multisig transaction to DePool will be displayed.

`--v2` - optional flag, force to interpret wallet as multisig v2.

Example:

```bash
ever-cli depool --addr 0:3187b4d738d69776948ca8543cb7d250c042d7aad1e0aa244d247531590b9147 donor lock --wallet 0:255a3ad9dfa8aa4f3481856aafc7d79f47d50205190bd56147138740e9b177f3 --donor 0:279afdbd7b2cbf9e65a5d204635a8630aec2baec60916ffdc9c79a09d2d2893d --sign "deal hazard oak major glory meat robust teach crush plastic point edge"
```

Like vesting stake, lock stake can be configured to be unlocked in parts at the end of each round that coincides with
the end of the next withdrawal period. At the end of each period the Lock Stake is returned to the wallet which locked
it. The rewards of a lock stake are always added to the ordinary stake of the beneficiary. To withdraw these funds,
beneficiary should use one of the [withdrawal functions](#75-withdraw-stakes).

Please note that the lock stake is split into two equal parts by the DePool, to be used in both odd and even rounds, so
to ensure DePool can participate in elections with just one lock stake where validator wallet is beneficiary, the stake
should equal `validatorAssurance` *2. Similarly, to ensure any vesting stake is accepted, make sure it exceeds
`minStake` *2.

**Donor uses the following command to make a lock stake:**

```bash
ever-cli depool [--addr <depool_address>] stake lock [--wallet <msig_address>] --value <number> --total <days> --withdrawal <days> --beneficiary <address> [--sign <key_file or seed_phrase>] [--wait-answer] [--v2]
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

`--wait-answer` - optional flag, which forces EVER-CLI to wait for DePool answer when performing the action and print out the resulting message. Without it only the results of the multisig transaction to DePool will be displayed.

`--v2` - optional flag, force to interpret wallet as multisig v2.

Example:

```bash
ever-cli depool --addr 0:37fbcb6e3279cbf5f783d61c213ed20fee16e0b1b94a48372d20a2596b700ace stake lock --wallet 0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e --value 1000 --total 360 --withdrawal 30 --beneficiary 0:f22e02a1240dd4b5201f8740c38f2baf5afac3cedf8f97f3bd7cbaf23c7261e3 --sign "dizzy modify exotic daring gloom rival pipe disagree again film neck fuel"
```

> Note: Each participant can concurrently be the beneficiary of only one lock stake. Once the current lock stake expires, another can be made for the participant.

## 7.3. Remove stakes

This command removes an ordinary stake from a pooling round (while it has not been staked in the Elector yet):

```bash
ever-cli depool [--addr <depool_address>] stake remove [--wallet <msig_address>] --value <number> [--sign <key_file or seed_phrase>] [--wait-answer] [--v2]
```

Where

`<depool_address>` - address of the DePool contract.

`<msig_address>` - address of the wallet that made the stake.

all `--value` parameters must be defined in tons, like this: `--value 10.5`, which means the value is 10,5 tons.

`<key_file or seed_phrase>` - either the keyfile for the wallet making the stake, or the seed phrase in quotes.

`--wait-answer` - optional flag, which forces EVER-CLI to wait for DePool answer when performing the action and print out the resulting message. Without it only the results of the multisig transaction to DePool will be displayed.

`--v2` - optional flag, force to interpret wallet as multisig v2.

Example:

```bash
ever-cli depool --addr 0:37fbcb6e3279cbf5f783d61c213ed20fee16e0b1b94a48372d20a2596b700ace stake remove --wallet 0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e --value 100 --sign "dizzy modify exotic daring gloom rival pipe disagree again film neck fuel"
```

## 7.4. Transfer stakes

The following command assigns an existing ordinary stake or its part to another participant wallet. If the entirety of
the stake is transferred, the transferring wallet is removed from the list of participants in the DePool. If the
receiving wallet isn't listed among the participants, it will become a participant as the result of the command.

```bash
ever-cli depool [--addr <depool_address>] stake transfer [--wallet <msig_address>] --value <number> --dest <address> [--sign <key_file or seed_phrase>] [--wait-answer] [--v2]
```

Where

`<depool_address>` - address of the DePool contract.

`<msig_address>` - address of the wallet that made the stake.

all `--value` parameters must be defined in tons, like this: `--value 10.5`, which means the value is 10,5 tons.

`dest <address>` - address of the new owner of the stake.

`<key_file or seed_phrase>` - either the keyfile for the wallet making the stake, or the seed phrase in quotes.

`--wait-answer` - optional flag, which forces EVER-CLI to wait for DePool answer when performing the action and print out the resulting message. Without it only the results of the multisig transaction to DePool will be displayed.

`--v2` - optional flag, force to interpret wallet as multisig v2.

Example:

```bash
ever-cli depool --addr 0:37fbcb6e3279cbf5f783d61c213ed20fee16e0b1b94a48372d20a2596b700ace stake transfer --wallet 0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e --value 1000 --dest 0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e --sign "dizzy modify exotic daring gloom rival pipe disagree again film neck fuel"
```

> Note: Stakes cannot be transferred from or to DePool's validator wallet, and between any wallets during round completion step.

## 7.5. Withdraw Stakes

### 7.5.1. Withdraw entire stake

The following command allows to withdraw an ordinary stake to the wallet that owns it, as soon as the stake becomes
available. Use `withdraw on` to receive the stake, once it's unlocked. If you then make another stake, and want to keep
reinvesting it every round, run the command with `withdraw off`.

```bash
ever-cli depool [--addr <depool_address>] withdraw on | off [--wallet <msig_address>] [--sign <key_file or seed_phrase>] [--wait-answer] [--v2]
```

Where

`<depool_address>` - address of the DePool contract.

`<msig_address>` - address of the wallet that made the stake.

`<key_file or seed_phrase>` - either the keyfile for the wallet that made the stake, or the seed phrase in quotes.

`--wait-answer` - optional flag, which forces EVER-CLI to wait for DePool answer when performing the action and print out the resulting message. Without it only the results of the multisig transaction to DePool will be displayed.

`--v2` - optional flag, force to interpret wallet as multisig v2.

Example:

```bash
ever-cli depool --addr 0:37fbcb6e3279cbf5f783d61c213ed20fee16e0b1b94a48372d20a2596b700ace withdraw on --wallet 0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e --sign "dizzy modify exotic daring gloom rival pipe disagree again film neck fuel"
```

### 7.5.2. Withdraw part of the stake

The following command allows to withdraw part of an ordinary stake to the wallet that owns it, as soon as the stake
becomes available. If, as result of this withdrawal, participant's ordinary stake becomes less than `minStake`, then
participant's whole stake is sent to participant.

```bash
ever-cli depool [--addr <depool_address>] stake withdrawPart [--wallet <msig_address>] --value <number> [--sign <key_file or seed_phrase>] [--wait-answer] [--v2]
```

Where

`<depool_address>` - address of the DePool contract.

`<msig_address>` - address of the wallet that made the stake.

all `--value` parameters must be defined in tons, like this: `--value 10.5`, which means the value is 10,5 tons.

`<key_file or seed_phrase>` - either the keyfile for the wallet that made the stake, or the seed phrase in quotes.

`--wait-answer` - optional flag, which forces EVER-CLI to wait for DePool answer when performing the action and print out the resulting message. Without it only the results of the multisig transaction to DePool will be displayed.

`--v2` - optional flag, force to interpret wallet as multisig v2.

Example:

```bash
ever-cli depool --addr 0:37fbcb6e3279cbf5f783d61c213ed20fee16e0b1b94a48372d20a2596b700ace stake withdrawPart --wallet 0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e --value 1000 --sign "dizzy modify exotic daring gloom rival pipe disagree again film neck fuel"
```

## 7.6. Reinvest Stakes

[Ordinary](#721-ordinary-stake) stake reinvestment is controlled by the DePool reinvest flag. By default, this flag is
set to `yes`, and the participant's available ordinary stake will be reinvested every round, no additional action
required. It gets set to `no` when [withdrawing the entire stake](#751-withdraw-entire-stake). After stake withdrawal it
remains set to `no`. To re-enable ordinary stake reinvesting after withdrawing a stake, run the `withdraw` command with
option `off`:

```bash
ever-cli depool [--addr <depool_address>] withdraw off [--wallet <msig_address>] [--sign <key_file or seed_phrase>] [--wait-answer] [--v2]
```

`<depool_address>` - address of the DePool contract.

`<msig_address>` - address of the wallet that made the stake.

`<key_file or seed_phrase>` - either the keyfile for the wallet that made the stake, or the seed phrase in quotes.

`--wait-answer` - optional flag, which forces EVER-CLI to wait for DePool answer when performing the action and print out the resulting message. Without it only the results of the multisig transaction to DePool will be displayed.

`--v2` - optional flag, force to interpret wallet as multisig v2.

Example:

```bash
ever-cli depool --addr 0:37fbcb6e3279cbf5f783d61c213ed20fee16e0b1b94a48372d20a2596b700ace withdraw off --wallet 0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e --sign "dizzy modify exotic daring gloom rival pipe disagree again film neck fuel"
```

**Note:**

[Withdrawing a part of the stake](#752-withdraw-part-of-the-stake) does not affect the `reinvest` flag.

[Lock](#723-lock-stake) and [vesting](#722-vesting-stake) stakes are reinvested according to their initial settings for
the full duration of the staking period. There is no way to change these settings once lock and vesting stakes are made.

## 7.7. Read DePool answers

Every time anything happens with the participant stake in the DePool, e.g. a round completes and rewards are
distributed, DePool sends the participant a message with the relevant details. Use the following command to read these
messages:

```bash
ever-cli depool --addr <depool_address> answers --wallet <msig_address> [--since <unixtime>]
```

`<depool_address>` - address of the DePool contract.

`<msig_address>` - address of the wallet that made the stake.

`<unixtime>` - unixtime, since which you want to view DePool answers. If `--since` is omitted, all DePool answers are printed.

Example:

```bash
$ ever-cli depool --addr 0:127ae93241278304fff6b7e5b7b182fd382b6e95b200551061a7354e032e50bf answers --wallet 0:255a3ad9dfa8aa4f3481856aafc7d79f47d50205190bd56147138740e9b177f3
Config: /home/user/ever-cli.conf.json
Connecting to net.evercloud.dev
34 answers found
Answer:
Id: 7cacf43d2e748a5c9209e93c41c0aeccc71a5b05782dbfb3c8ac538948b67c49
Value: 0.000000001
Created at: 1619803878 (2021-04-30 17:31:18.000)
Decoded body:
onRoundComplete {"roundId":"104","reward":"2907725565","ordinaryStake":"211269425171","vestingStake":"0","lockStake":"0","reinvest":true,"reason":"5"}
```

## 7.8. View DePool events

Various events occurring in the DePool are broadcasted to the blockchain and can be monitored. use the following command
to view them:

```bash
ever-cli depool [--addr <depool_address>] events [--since <unixtime>]
```

`<depool_address>` - address of the DePool contract.

`<unixtime>` - unixtime, since which you want to view DePool events. If `--since` is omitted, all DePool events are printed.

Example:

```bash
$ ever-cli depool --addr 0:127ae93241278304fff6b7e5b7b182fd382b6e95b200551061a7354e032e50bf events --since 1619803870
Config: /home/user/ever-cli.conf.json
Input arguments:
  depool: 0:127ae93241278304fff6b7e5b7b182fd382b6e95b200551061a7354e032e50bf
   since: 1619803870
Connecting to net.evercloud.dev
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
ever-cli depool [--addr <depool_address>] events --wait-one
```

EVER-CLI waits until new event will be emitted and then prints it to terminal.

## 7.9. Replenish DePool balance

To operate correctly, DePool needs to maintain a balance over 20 tokens. Normally, this happens automatically, but in
some cases, when normal operation is interrupted, DePool balance may drop lower. Use the following command to replenish
DePool balance (this is not counted towards any stake):

```bash
ever-cli depool [--addr <depool_address>] replenish --value *number* [--wallet <msig_address>] [--sign <key_file_or_seed_phrase>] [--v2]
```

`<depool_address>` - address of the DePool contract.

all `--value` parameters must be defined in tons, like this: `--value 150.5`, which means the value is 150,5 tons.

`<msig_address>` - address of the wallet that made the stake.

`<key_file_or_seed_phrase>` - either the keyfile for the wallet, or the seed phrase in quotes.

`--v2` - optional flag, force to interpret wallet as multisig v2.

Example:

```bash
$ ever-cli depool --addr 0:127ae93241278304fff6b7e5b7b182fd382b6e95b200551061a7354e032e50bf replenish --value 5 --wallet 0:255a3ad9dfa8aa4f3481856aafc7d79f47d50205190bd56147138740e9b177f3 --sign key.json
Config: /home/user/ever-cli.conf.json
Input arguments:
  depool: 0:127ae93241278304fff6b7e5b7b182fd382b6e95b200551061a7354e032e50bf
  wallet: 0:255a3ad9dfa8aa4f3481856aafc7d79f47d50205190bd56147138740e9b177f3
   stake: 5
    keys: key.json
Connecting to net.evercloud.dev
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

To operate correctly, DePool needs to receive regular ticktock (state update) calls. One way to set them up, is through
a EVER-CLI with the use of a multisig wallet. Use the following command to send a ticktock call (you may set up a
script to run this command regularly):

```bash
ever-cli depool [--addr <depool_address>] ticktock [--wallet <msig_address>] [--sign <path_to_keys_or_seed_phrase>] [--v2]
```

- `--addr <depool_address>` - the address of the DePool
- `--wallet <msig_address>` - the address of the multisig wallet used to call DePool
- `--sign <path_to_keys_or_seed_phrase>` - either the keyfile for the wallet, or the seed phrase in quotes
- `--v2` - optional flag, force to interpret wallet as multisig v2.

1 token is always attached to this call. Change will be returned.

Example:

```bash
$ ever-cli depool --addr 0:127ae93241278304fff6b7e5b7b182fd382b6e95b200551061a7354e032e50bf ticktock --wallet 0:255a3ad9dfa8aa4f3481856aafc7d79f47d50205190bd56147138740e9b177f3 --sign key.json
Config: /home/user/ever-cli.conf.json
Input arguments:
  depool: 0:127ae93241278304fff6b7e5b7b182fd382b6e95b200551061a7354e032e50bf
  wallet: 0:255a3ad9dfa8aa4f3481856aafc7d79f47d50205190bd56147138740e9b177f3
    keys: key.json
Connecting to https://net.evercloud.dev
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

The following commands are used when voting for various FreeTON proposals at
[https://gov.freeton.org/](https://gov.freeton.org/)

## 8.1. Create proposal and cast the first vote

Use the following command:

```bash
ever-cli proposal create <msig_address> <proposal_address> "<comment>" <path_to_keyfile_or_seed_phrase>
```

`<msig_address>` -  address of judge wallet.

`<proposal_address>` - address of proposal contract.

`"<comment>"` - proposal description (max symbols: 382). Should be enclosed in double quotes.

`<path_to_keyfile_or_seed_phrase>` - path to key file or seed phrase for the judge wallet. Seed phrase should be enclosed in double quotes.

The utility generates the proposal transaction ID and casts the first vote for the proposal.

The proposal transaction ID can be used to vote for the proposal by all other wallet custodians and should be
communicated to them.

## 8.2. Vote for proposal

Receive proposal transaction ID and use the following command to cast a vote:

```bash
ever-cli proposal vote <msig_address> <proposal_id> <path_to_keyfile_or_seed_phrase>
```

`<msig_address>` - address of judge wallet.

`<proposal_id>` - proposal transaction ID.

`"<seed_phrase>"` - path to key file or seed phrase for the judge wallet. Seed phrase should be enclosed in double quotes.

Once the proposal transaction receives the required amount of votes (depends on judge wallet configuration), the
transaction is executed and the proposal is considered approved.

## 8.3. Decode proposal comment

Use the following command to read the proposal comment added when the proposal transaction was created:

```bash
ever-cli proposal decode <msig_address> <proposal_id>
```

`<msig_address>` - address of judge wallet.

`<proposal_id>` - proposal transaction ID.

# 9. Supplementary commands

## 9.1. Get global config

```bash
ever-cli getconfig [<index>]
```

Options:

`<index>` - number of the [global config parameter](https://docs.everos.dev/ever-sdk/reference/ever-os-api/field_descriptions#blockmasterconfig-type) (equals the numeric part of the config parameter field name). This option can be omitted and command will fetch all config parameters.

Example (requesting the maximum and minimum numbers of validators on the blockchain):

```bash
$ ever-cli getconfig 16
Config: /home/user/ever-cli.conf.json
Input arguments:
   index: 16
Connecting to net.evercloud.dev
Config p16: {
  "max_validators": 1000,
  "max_main_validators": 100,
  "min_validators": 13
}
```

## 9.2. NodeID

The following command calculates node ID from validator public key:

```bash
ever-cli nodeid --pubkey <validator_public_key> | --keypair <path_to_key_or_seed_phrase>
```

`<validator_public_key>` - public key of the validator wallet.

`<path_to_key_or_seed_phrase>` - path to validator wallet keyfile or the corresponding seed phrase in quotes.

Example:

```bash
$ ever-cli nodeid ---keypair "dizzy modify exotic daring gloom rival pipe disagree again film neck fuel"
Config: /home/user/ever-cli.conf.json
Input arguments:
     key: None
 keypair: dizzy modify exotic daring gloom rival pipe disagree again film neck fuel
50232655f2ad44f026b03ec1834ae8316bfa1f3533732da1e19b3b31c0f04143
```

## 9.3. Dump blockchain config

```bash
ever-cli dump config <path>
```

`<path>` - path where to save the blockchain config dump.

Example:

```bash
$ ever-cli dump config config.boc
Config: /home/user/ever-cli/ever-cli.conf.json
Input arguments:
    path: config.boc
Connecting to main.evercloud.dev
Config successfully saved to config.boc
```

## 9.4. Dump several account states

Dumps the list of accounts. Files will have address without workchain id as a name.

```bash
ever-cli dump account <list_of_addresses> [--path <dir_path>]
```

`<list_of_addresses>` - list of account addresses. Addresses should be specified separately with space delimiter.
Example: `0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13 0:14014af4a374bdd13dae2379063ea2597634c2c2fc8e99ca9eab431a7ab6f566  f89d946b5b4b8a06f01dc20dceef30caff844d5285abea8a21ad3730c0f3dd12 3333333333333333333333333333333333333333333333333333333333333333`.

`<dir_path>` - path to the directory where to save dumps. Defaults to current directory.

Example:

```bash
$ ever-cli dump account 0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13 0:14014af4a374bdd13dae2379063ea2597634c2c2fc8e99ca9eab431a7ab6f566  f89d946b5b4b8a06f01dc20dceef30caff844d5285abea8a21ad3730c0f3dd12 3333333333333333333333333333333333333333333333333333333333333333
Config: /home/user/ever-cli/ever-cli.conf.json
Input arguments:
addresses: 0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13, 0:14014af4a374bdd13dae2379063ea2597634c2c2fc8e99ca9eab431a7ab6f566, 0:f89d946b5b4b8a06f01dc20dceef30caff844d5285abea8a21ad3730c0f3dd12, 0:3333333333333333333333333333333333333333333333333333333333333333
    path: None
Connecting to net.evercloud.dev
Processing...
./2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13.boc successfully dumped.
./14014af4a374bdd13dae2379063ea2597634c2c2fc8e99ca9eab431a7ab6f566.boc successfully dumped.
0:f89d946b5b4b8a06f01dc20dceef30caff844d5285abea8a21ad3730c0f3dd12 was not found.
0:3333333333333333333333333333333333333333333333333333333333333333 was not found.
Succeeded.
```

## 9.5. Update global config parameter

Use the following command to update one parameter of the blockchain global config, that is stored in a .json file:

```bash
ever-cli update_config <seqno> <config_master_key_file> <new_param_file>
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
$ ever-cli update_config 9 config-master example.json
Config: /home/user/ever-cli/ever-cli.conf.json
Input arguments:
   seqno: 9
config_master: config-master
new_param: example.json
Message: b5ee9c720101020100850001e589feaaaaaaaaaaaaa...

```

## 9.6. Wait for an account change

The command `account-wait` waits for the change of the `last_trans_lt` account field. It exits with zero exit code upon success (the field has changed before timeout). Otherwise, it exits with non-zero code.

```bash
ever-cli account-wait <address> [--timeout <timeout_in_secs>]
```

`<address>` - address of account to wait for.

`<timeout_in_secs>` - timeout in seconds (the default is 30).

Example:

```bash
$ ever-cli account-wait --timeout 10 0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13
...
Succeeded.
$ echo $?
0
```

## 9.7. Make a raw GraphQL query

The command `query-raw` executes a raw network query by directly calling the `ever_client::net::query_collection` SDK
interface.

```bash
ever-cli query-raw <collection> <result> [--filter <filter>] [--limit <limit>] [--order <order>]
```

See relevant SDK documentation to learn about the command's parameters.

Examples:

```bash
$ ever-cli --json query-raw accounts "id bits cells" --filter '{ "id": { "eq": "0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13" } }'
[
  {
    "id": "0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13",
    "bits": "0x20bc",
    "cells": "0x25"
  }
]

$ ever-cli --json query-raw accounts "id bits cells" --order '[ { "path": "balance", "direction": "DESC" } ]' --limit 3
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

## 9.8. Fee commands

This commands allow user to learn how much funds smart contract can consume.

### 9.8.1. Call fee command

This command executes smart contract call locally, calculates fees and prints table of all fees in nanotons.
Command has the same option as [ever-cli call](#441-call-contract-on-the-blockchain) command:

```bash
ever-cli fee call  [--abi <contract.abi.json>] [--sign <seed_or_keyfile>] [--saved_config <config_contract_path>] <address> <method> <params>
```

`<contract.abi.json>` - contract interface file.

`<config_contract_path>` - path to the file with saved config contract state. Is used for debug on fail.

`<seed_or_keyfile>` - can either be the seed phrase or the corresponding key pair file. If seed phrase is used, enclose it in double quotes.

`<address>` - contract [address](#41-generate-contract-address).

`<method>` - the method being called.

`<params>` - parameters of the called method. Can be specified by a path to the file, which contains parameters in json
format.

Example:

```bash
ever-cli --json fee call --abi tests/samples/giver_v2.abi.json 0:ece57bcc6c530283becbbd8a3b24d3c5987cdddc3c8b7b33be6e4a6312490415 --sign tests/samples/giver_v2.key sendTransaction '{"dest":"0:ece57bcc6c530283becbbd8a3b24d3c5987cdddc3c8b7b33be6e4a6312490415","value":1000000,"bounce":false}'
Not set rand_seed_block
{
  "in_msg_fwd_fee": "2237000",
  "storage_fee": "13",
  "gas_fee": "9690000",
  "out_msgs_fwd_fee": "1000000",
  "total_account_fees": "12927013",
  "total_output": "1000000"
}
```

### 9.8.2. Deploy fee command

This command executes smart contract deploy locally, calculates fees and prints table of all fees in nanotons.
Command has the same option as [ever-cli deploy](#42-deploy-contract) command:

```bash
ever-cli fee deploy [--sign <deploy_seed_or_keyfile>] [--wc <int8>] [--abi <contract.abi.json>] <contract.tvc> <params>
```

`<deploy_seed_or_keyfile>` - can either be the seed phrase used to generate the deployment key pair file or the key pair file itself. If seed phrase is used, enclose it in double quotes.


`--wc <int8>` ID of the workchain the wallet will be deployed to (`-1` for masterchain, `0` for basechain). By default, this value is set to 0.

`--abi <contract.abi.json>` - contract interface file.

`<contract.tvc>` - compiled smart contract file.

`<params>` - deploy command parameters, depend on the contract.

Example:

```bash
ever-cli --json fee deploy tests/samples/SafeMultisigWallet.tvc '{"owners":["0xc8bd66f90d61f7e1e1a6151a0dbe9d8640666920d8c0cf399cbfb72e089d2e41"],"reqConfirms":1}' --abi tests/samples/SafeMultisigWallet.abi.json --sign tests/deploy_test.key
Not set rand_seed_block
{
  "in_msg_fwd_fee": "42421000",
  "storage_fee": "0",
  "gas_fee": "9004000",
  "out_msgs_fwd_fee": "0",
  "total_account_fees": "51425000",
  "total_output": "0"
}
```

### 9.8.3. Storage fee command

This command allows user to calculate storage fees for a deployed contract using its address.

```bash
ever-cli fee storage [--period <period>] <address>
```

`<period>` - Time period in seconds (default value is 1 year).

`<address>` - Contract address.

Example:

```bash
ever-cli --json fee storage --period 1000000 0:ece57bcc6c530283becbbd8a3b24d3c5987cdddc3c8b7b33be6e4a6312490415
{
  "storage_fee": "332978",
  "period": "1000000"
}
```

## 10. Fetch and replay

These two commands are commonly used in pairs to recover a state of the account at the specific point before a given
transaction.

Example:

1) Dump blockchain config history to the file.

```bash
$ ever-cli fetch -- -1:5555555555555555555555555555555555555555555555555555555555555555 config.txns
```

2) Dump account transactions from the network to the file.

```bash
$ ever-cli fetch 0:570ddeb8f632e5f9fde198dd4a799192f149f01c8fd360132b38b04bb7761c5d 570ddeb8.txns
```
where `0:570ddeb8f632e5f9fde198dd4a799192f149f01c8fd360132b38b04bb7761c5d` is an example of account address,
`570ddeb8.txns` - name of the output file.

```bash
$ ever-cli replay [-e] [-c config.txns] 570ddeb8.txns 197ee1fe7876d4e2987b5dd24fb6701e76d76f9d08a5eeceb7fe8ca73d9b8270
```

Transaction can be replayed with config using option `-c` or with the current network config (option `-e`).

where `197ee1fe7876d4e2987b5dd24fb6701e76d76f9d08a5eeceb7fe8ca73d9b8270` is a txn id before which account state should
be restored.

Note 1: last command generates 3 files. The file with the longest name in the form of `<addr>-<txn_id>.boc` is a
replayed and serialized Account state.

Note 2: to get StateInit (tvc) from Account state use `ever-cli decode account boc` command with `--dumptvc` option.

### 10.1. How to unfreeze account

- 1) Dump Account state before transaction in which account changed state from Active to Frozen.

- 2) Extract tvc from the generated Account state.

1) Use contract deployer (address in mainnet: `0:51616debd4296a4598530d57c10a630db6dc677ecbe1500acaefcfdb9c596c64`) to
deploy the extracted tvc to the frozen account. Send 1 ton to its address and then run its `deploy` method.

Example:

`ever-cli --url main.evercloud.dev call 0:51616debd4296a4598530d57c10a630db6dc677ecbe1500acaefcfdb9c596c64 deploy --abi deployer.abi.json "{\"stateInit\":\"$(cat state.tvc | base64 -w 0)\",\"value\":500000000,\"dest\":\"-1:618272d6b15fd8f1eaa3cdb61ab9d77ae47ebbfcf7f28d495c727d0e98d523eb\"}"`

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

### 10.2. Fetch block command

This command allow user to fetch block and save it to the output file.

```bash
ever-cli fetch-block <BLOCKID> <OUTPUT>
```

Options:

`<BLOCKID>` - Block ID.
`<OUTPUT>` - Output file name

## 11. Debug commands

Debug commands allow user to replay transaction locally or execute a function call locally and obtain TVM trace.
More about debug flow is written in [Debug.md](https://github.com/everx-labs/ever-cli/blob/master/Debug.md).

### 11.1. Debug transaction

```bash
ever-cli debug transaction [FLAGS] [OPTIONS] <tx_id>
```

FLAGS:

`--dump_config`           Dump the replayed config contract account state.

`--dump_contract`         Dump the replayed target contract account state.

`-e, --empty_config`      Replay transaction without full dump of the config contract.

`--full_trace`             Flag that changes trace to full version.

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
$ ever-cli debug transaction -o tvm_trace.log 74acbd354e605519d799c7e1e90e52030e8f9e781453e48ecad18bb035fe1586 --empty-config
Config: /home/user/sol2tvm/scripts/ever-cli.conf.json
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

### 11.2. Debug call

```bash
ever-cli debug call [FLAGS] [OPTIONS] [--addr <address>] [-m <method>] <params>
```

FLAGS:

`--boc`          Flag that changes behavior of the command to work with the saved account state (account BOC).

`--full_trace`             Flag that changes trace to full version.

`--tvc`          Flag that changes behavior of the command to work with the saved contract state (stateInit TVC).

`-u --update`    Update contract BOC after execution

OPTIONS:

`--abi <ABI>`                             Path to the contract ABI file. Can be specified in the config file.

`--tvc_address <ACCOUNT_ADDRESS>`         Account address for account constructed from TVC.

`--addr <ADDRESS>`                        Contract address or path the file with saved contract state if corresponding flag is used. Can be specified in th config file.

`-c, --config <CONFIG_PATH>`              Path to the file with saved config contract state.

`-d, --dbg_info <DBG_INFO>`               Path to the file with debug info.

`--decode_abi <DECODE_ABI>`               Path to the ABI file used to decode output messages.

`-o, --output <LOG_PATH>`                 Path where to store the trace. Default path is "./trace.log". Note: old file will be removed.

`--now <NOW>`                             Now timestamp (in milliseconds) for execution. If not set it is equal to the current timestamp.

`--keys <SIGN>`                           Seed phrase or path to the file with keypair used to sign the message. Can be specified in the config.

`-m, --method <METHOD>`                   Name of the function being called. Can be specified in the config file.

ARGUMENTS:

`<params>`     Function arguments. Must be specified in [alternative manner](#410-alternative-syntax-for-call-deploy-and-run-commands) or can be passed as a file path.

This command allows user locally emulate contract call and obtain TVM trace.
Command can work with contract in the network by querying its boc and running message on it or with saved account state
in format of account BOC or pure StateInit TVC. If contract is passed via TVC file, contract address can be specified
with `--address <tvc_address>` option. Also, execution timestamp can be specified with option `--now <timestamp>`.

```bash
$ ever-cli debug call --addr 0:2eb2365dba1bff21d786d7ceeb9b9641149709790c7b83337ef9e2fb528c69cb --abi ../samples/2_StorageClient.abi.json --keys keys/key0  -o call.log -m store -- --storageAddress 0:e59d5eee37b399eea0121eac2571d3762779ba88f1c575863f0ed1595caed0e8 --value 257
Input arguments:
   input: 0:2eb2365dba1bff21d786d7ceeb9b9641149709790c7b83337ef9e2fb528c69cb
  method: store
  params: {"storageAddress":"0:e59d5eee37b399eea0121eac2571d3762779ba88f1c575863f0ed1595caed0e8","value":"257"}
    sign: keys/key0
 opt_abi: ../samples/2_StorageClient.abi.json
  output: call.log
Connecting to:
        Url: net.evercloud.dev
        Endpoints: ["https://eri01.net.everos.dev", "https://rbx01.net.everos.dev", "https://gra01.net.everos.dev"]

Execution finished.
Log saved to call.log
```

### 11.3. Debug run

```bash
ever-cli debug run [FLAGS] [OPTIONS] [--addr <address>] [-m <method>] <params>
```

FLAGS:

`--boc`          Flag that changes behavior of the command to work with the saved account state (account BOC).

`--min_trace`    Flag that changes trace to minimal version.

`--tvc`          Flag that changes behavior of the command to work with the saved contract state (stateInit TVC).

OPTIONS:

`--abi <ABI>`                             Path to the contract ABI file. Can be specified in the config file.

`--tvc_address <ACCOUNT_ADDRESS>`         Account address for account constructed from TVC.

`--addr <ADDRESS>`                        Contract address or path the file with saved contract state if corresponding flag is used. Can be specified in th config file.

`-c, --config <CONFIG_PATH>`              Path to the file with saved config contract state.

`-d, --dbg_info <DBG_INFO>`               Path to the file with debug info.

`--decode_abi <DECODE_ABI>`               Path to the ABI file used to decode output messages.

`-o, --output <LOG_PATH>`                 Path where to store the trace. Default path is "./trace.log". Note: old file will be removed.

`--now <NOW>`                             Now timestamp (in milliseconds) for execution. If not set it is equal to the current timestamp.

`-m, --method <METHOD>`                   Name of the function being called. Can be specified in the config file.

ARGUMENTS:

`<params>`     Function arguments. Must be specified in [alternative manner](#410-alternative-syntax-for-call-deploy-and-run-commands) or can be passed as a file path.

This command is similar to `ever-cli debug call` but allows user to debug get methods.

```bash
$ ever-cli debug run --addr 0:2eb2365dba1bff21d786d7ceeb9b9641149709790c7b83337ef9e2fb528c69cb --abi ../samples/2_UintStorage.abi.json -o run.log -m value
Input arguments:
   input: 0:2eb2365dba1bff21d786d7ceeb9b9641149709790c7b83337ef9e2fb528c69cb
  method: value
  params: {}
    sign: None
 opt_abi: ../samples/2_UintStorage.abi.json
  output: run.log
Connecting to:
        Url: net.evercloud.dev
        Endpoints: ["https://eri01.net.everos.dev", "https://rbx01.net.everos.dev", "https://gra01.net.everos.dev"]

Execution finished.
Log saved to run.log
```

### 11.4. Debug replay transaction on the saved account state

```bash
    ever-cli debug replay [FLAGS] [OPTIONS] <TX_ID> <INPUT>
```

FLAGS:

`--full_trace`             Flag that changes trace to full version.

`-u --update`    Update contract BOC after execution

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
$ ever-cli debug replay --min_trace --update -d 2_StorageClient.dbg.json2 --decode_abi 2_UintStorage.abi.json -o trace2.log 82733d3ddf7cae1d3fa07ec5ce288b7febf3bffd9d229a8e538f62fac10eec3e contract.boc
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

### 11.5. Debug deploy

```bash
ever-cli debug deploy [FLAGS] [OPTIONS] <tvc> <params>
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

`--keys <SIGN>`                     Seed phrase or path to the file with keypair used to sign the message. Can be specified in the config.

`--wc <WC>`                         Workchain ID

ARGUMENTS:

`<tvc>`       Path to the tvc file with contract StateInit.

`<params>`     Function arguments. Must be specified in [alternative manner](#410-alternative-syntax-for-call-deploy-and-run-commands) or can be passed as a file path.

This command allows user locally emulate contract deploy.
Command can work with prepared network account or create a dummy one with big balance (if --init_balance flag is
specified).

### 11.6. Debug message

```bash
$ ever-cli debug message [--boc] [--addr <address_or_path>] [-u] [-o <log_path>] <message_in_base64_or_path_to_file>
```

FLAGS:

`--boc`               Flag that changes behavior of the command to work with the saved account state (account BOC).

`--full_trace`        Flag that changes trace to full version.

`-u, --update`        Update contract BOC after execution

OPTIONS:

`--addr <ADDRESS>`                        Contract address or path the file with saved contract state if corresponding flag is used. Can be specified in th config file.

`-c, --config <CONFIG_PATH>`        Path to the file with saved config contract state.

`-d, --dbg_info <DBG_INFO>`         Path to the file with debug info.

`--decode_abi <DECODE_ABI>`         Path to the ABI file used to decode output messages. Can be specified in the config file.

`-o, --output <LOG_PATH>`           Path where to store the trace. Default path is "./trace.log". Note: old file will be removed.

`--now <NOW>`                       Now timestamp (in milliseconds) for execution. If not set it is equal to the current timestamp.

ARGUMENTS:

`<message_in_base64_or_path_to_file>`     Message in Base64 or path to fil with message.

This command allows to play message on the contract state locally with trace.
It can be useful when user wants to play contract interaction locally. User can call one contract locally with
`ever-cli debug call` and find output messages in trace log:

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

`Message_base64` then can be passed to `ever-cli debug message` to play it on another account.

### 11.7. Debug account

Allows to debug transaction of the specified account

```bash
$ ever-cli debug account [--addr <address_or_path>] [-o <log_path>] [FLAGS] [OPTIONS]
```

FLAGS:

`--dump_config`           Dump the replayed config contract account state.

`--dump_contract`         Dump the replayed target contract account state.

`-e, --empty_config`      Replay transaction without full dump of the config contract.

`--full_trace`             Flag that changes trace to full version.

OPTIONS:

`--addr <ADDRESS>`                        Contract address or path the file with saved contract state if corresponding flag is used. Can be specified in th config file.

`-c, --config <CONFIG_PATH>`        Path to the file with saved config contract transactions. If not set transactions will be fetched to file "config.txns".

`-t, --contract <CONTRACT_PATH>`    Path to the file with saved target contract transactions. If not set transactions will be fetched to file "contract.txns".

`-d, --dbg_info <DBG_INFO>`         Path to the file with debug info.

`--decode_abi <DECODE_ABI>`         Path to the ABI file used to decode output messages.

`-o, --output <LOG_PATH>`           Path where to store the trace. Default path is "./trace.log". Note: old file will be removed.

Example:

```bash
$ ever-cli debug account -e --addr 0:2eb2365dba1bff21d786d7ceeb9b9641149709790c7b83337ef9e2fb528c69cb
Input arguments:
 address: 0:2eb2365dba1bff21d786d7ceeb9b9641149709790c7b83337ef9e2fb528c69cb
trace_path: ./trace.log
config_path: None
contract_path: None
Connecting to:
        Url: net.evercloud.dev
        Endpoints: ["https://eri01.net.everos.dev", "https://rbx01.net.everos.dev", "https://gra01.net.everos.dev"]



Choose transaction you want to debug:
1)      transaction_id: "7f88b986e91e08265a5c1a5d1fa0d890d7a96fc5202c4117460a0cd144e6a8e1"
        timestamp     : "2022-09-13 14:37:41.000"
        message_type  : "ExtIn"
        source_address: ""

2)      transaction_id: "fd9d0977a957e8fdfb451db8fc15a13cb2cd6dfec9861f03bc52d4b94f5dfaac"
        timestamp     : "2022-09-13 14:37:29.000"
        message_type  : "Internal"
        source_address: "0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13"



Enter number of the chosen transaction (from 1 to 2):
1
Fetching config contract transactions...
Fetching contract transactions...
account 0:2eb2365dba1bff21d786d7ceeb9b9641149709790c7b83337ef9e2fb528c69cb: zerostate not found, writing out default initial state
Replaying the last transactions...
Connecting to:
        Url: net.evercloud.dev
        Endpoints: ["https://devnet.evercloud.dev"]

Log saved to ./trace.log.
```

### 11.8. Render UML sequence diagram

```bash
    ever-cli debug sequence-diagram <address_list>
```

`<address_list>`    File containing a list of account addresses, one address per line. Blank lines and lines starting with # character are ignored.

This command generates a `.plantuml` text file which describes a sequence diagram of messages and transactions
for a provided list of accounts. See PlantUML documentation for a complete guide on rendering an image out of .plantuml.
To render an SVG the following command can be used:

```bash
    java -jar plantuml.jar accounts.plantuml -tsvg
```

### Caveat

Sequence diagrams are well suited for describing synchronous interactions. However, transactions (and messages which
spawn them) of the blockchain are inherently asynchronous. In particular, sequence diagram arrows can only be
horizontal, and there is no way to make them curve down towards the destination, skipping other transactions and thus
depicting asynchronicity.

Practically, this means that one should look cautiously at the point of transaction spawn, being aware that the spawning
message can be located somewhere above.

## 12. Alias functionality

Aliases can facilitate manual work with several contracts. When user deploys a contract an alias can be passed to the
`deploy` command. This option saves the deployed contract address, abi and key to the map in config file. Alias can be
used by commands `callx` and `runx` instead of address (`--addr`) for more convenient contract invocation.

Example workflow:

```bash
$ ever-cli deployx --abi ../samples/1_Accumulator.abi.json --keys keys/key0 --alias accum ../samples/1_Accumulator.tvc
{}
$ ever-cli config alias print
{
  "accum": {
    "abi_path": "../samples/1_Accumulator.abi.json",
    "address": "0:5b08b0c6beefb4645ef078acb6ca09129599a88f85da627354c42215b58af221",
    "key_path": "keys/key0"
  }
}
$ ever-cli runx --addr accum -m sum
{
  "sum": "0x0000000000000000000000000000000000000000000000000000000000000000"
}
$ ever-cli callx --addr accum -m add --value 255
{}
$ ever-cli runx --addr accum -m sum
{
  "sum": "0x00000000000000000000000000000000000000000000000000000000000000ff"
}
```

## 13. Evercloud authentication

Starting from version 0.28.1 ever-cli can perform [Evercloud](https://dashboard.evercloud.dev/)
(url: `https://dashboard.evercloud.dev/`) authentication. To use it user can specify config parameters:

```
--access_key <ACCESS_KEY>                     Project secret or JWT in Evercloud (dashboard.evercloud.dev).
--project_id <PROJECT_ID>                     Project Id in Evercloud (dashboard.evercloud.dev).

$ ever-cli config --project_id 1233316546 --access_key 8465465413246
Succeeded.
{
  "url": "sdk2.dev.everx.dev",
  "wc": 0,
  "addr": null,
  "method": null,
  "parameters": null,
  "wallet": null,
  "pubkey": null,
  "abi_path": null,
  "keys_path": null,
  "retries": 5,
  "timeout": 40000,
  "message_processing_timeout": 40000,
  "out_of_sync_threshold": 15,
  "is_json": false,
  "depool_fee": 0.5,
  "lifetime": 60,
  "no_answer": true,
  "balance_in_tons": false,
  "local_run": false,
  "async_call": false,
  "debug_fail": "None",
  "project_id": "1233316546",
  "access_key": "8465465413246",
  "endpoints": []
}
```

<PROJECT_ID> will be used to modify network endpoints in the manner: `<endpoint>/PROJECT_ID` and <ACCESS_KEY> will be
used while establishing the network connection.

Currently `mainnet.evercloud.dev` and `devnet.evercloud.dev` networks require a mandatory authentication and fail the
unauthenticated connection with such error:

```bash
$ ever-cli account -1:3333333333333333333333333333333333333333333333333333333333333333
Input arguments:
addresses: -1:3333333333333333333333333333333333333333333333333333333333333333
Connecting to:
        Url: main.evercloud.dev
        Endpoints: ["https://mainnet.evercloud.dev"]

Processing...
Error: failed to query account info: Query failed: Can not send http request: Server responded with code 401
Error: 1
```

This can be fixed by using valid `project_id` and `access_key`. They can be obtained for free after registration.
[Guide](https://docs.everos.dev/evernode-platform/products/evercloud/get-started).

Example (Note: this authentication credentials are placed just for demonstration and can't be used for real networks):

```bash
$ ever-cli config --project_id b2ad82504ff54fccb5bc6db8cbb3df1e --access_key 27377ff9027d4de792f100eb869e18e8
Succeeded.
{
  "url": "main.evercloud.dev",
  "wc": 0,
  "addr": null,
  "method": null,
  "parameters": null,
  "wallet": null,
  "pubkey": null,
  "abi_path": null,
  "keys_path": null,
  "retries": 5,
  "timeout": 40000,
  "message_processing_timeout": 40000,
  "out_of_sync_threshold": 15,
  "is_json": false,
  "depool_fee": 0.5,
  "lifetime": 99,
  "no_answer": true,
  "balance_in_tons": false,
  "local_run": false,
  "async_call": false,
  "debug_fail": "None",
  "project_id": "b2ad82504ff54fccb5bc6db8cbb3df1e",
  "access_key": "27377ff9027d4de792f100eb869e18e8",
  "endpoints": [
    "https://mainnet.evercloud.dev"
  ]
}
user@user-ZenBook:~/ever-cli/target$ ever-cli account -1:3333333333333333333333333333333333333333333333333333333333333333
Input arguments:
addresses: -1:3333333333333333333333333333333333333333333333333333333333333333
Connecting to:
        Url: main.evercloud.dev
        Endpoints: ["https://mainnet.evercloud.dev/b2ad82504ff54fccb5bc6db8cbb3df1e"]

Processing...
Succeeded.
address:       -1:3333333333333333333333333333333333333333333333333333333333333333
acc_type:      Active
balance:       288960739602255584 nanoton
last_paid:     0
last_trans_lt: 0x1c1992d20ec3
data_boc:      b5ee9c72020203dd00010000a098000002516817b004a6c6615d1c4acfe7fd3a11c51675e07831a0e6a07df357ea3989bac2b5325e8c8267f0f1c30348000102058f633101a800020169a15d1cc6665d1c000100008dddf9854d9349b56eb2fd01bee18b618c8873f3aea762581ea44990e24247c1801e7a19bcb2fa21b040000302012000cf0004020120007400050201200035000602012000280007020120001d0008020120001a00090201200017000a020120000e000b020120000d000c009fbdff79db55b156589fa41a8efae630cb826de4637bf20dba3d59c28399a071faf5683e8619867a23bdf23ffdec8fea3c5ca1c84406fdef15ce9a91eca0d62bf7000ed07c51ea82b0381c37f6a201fcf2009fbdca4cea0ce60a75e33197d9005ab5325e4984766387907a2938f81a419fce5e77725d850b5bb06d829d8d6034f36c563ad24f2e2f9135707b773b5c23e62381800e62acd6787837b81b66cb9bdba09a0201200010000f009fbdce5ab490612b2d61681ae46778481757b96b3c9259fab01416d2024b7eaab97b7a99e4f6effa96e96ce3b88a568c143be205014049443a593c17b5a3c7b405800d856572fe42cb3819c14d1d0ebb1a0201200014001102012000130012009fbd65e3a9b6ddc3f497b119ac2108c50131fa5a558651a04305f4eb478ee09742873ca262bdbd8533640ef71493e71a490b5b858cbb17e4fdbe6700db644ec1b6001a56d46909e000e0322bceb6e23508009fbd543f8378102093e51112a52b45a5cdd199d6e24b0f48e7408ab4ad45a79787a5a39abbcdd7ca6e6112da16db54f98f83bfd568d9919daabdd5756cf1c4d80400310f7160806a2ee05d7359402c000802012000160015009fbd505455619da90b756bbf55737af4bc1974e20ee1bba0da19ee566e7e6346f85bb04116d5c75645014deda9bf9166fb184f0fb05d8299e0eea52a5f416c31e8002900753dec6528e04e19c1465aa0a8009fbd5f365b509f793b828df2eed0bcf34893abb5e0f3dfe44950260ea970b5c38624ef93e8408d9bf4399a54ce693b57b40185b28011aa41f31da9df44cff63594003b41f147aa0ac0e070dfda8807f3c802012000190018009fbe1e9015f3eaef198206765a0d345ddb8705366f1430f9a3401a6fdc7131ca8150922e824baaad3a7436f559a9db05aadcc8602b9e9a2916c7c33f51c7f7af3c0005badfc8fd14fc5c0aea1dbac8992d009fbe0f6f8e31cc467d8eec643ee97b144411874f5d735434355df6e23347edb5dff5b39b620787f6520671941c2a45326c79ca3456f8efb2f293eaaa0eac1962568007683e28f541581c0e1bfb5100fe79020120001c001b00a0be6aba4341f47afbe00815510cd26938ea0979fae394a86069711236f69121cd8e887f2d3b02c06911fa34068d78429bf050f035bb89da7fedc35682ba0fcd1f4001c0456f3e42422e0355df13183b0600a0be63935307b7ce428f03f5234413744e289909fe9d882ca1524eedc22f8c0e2a8e49397b04afdd1ce99bfa10c2e2e0c70bf87854e7742700179ce19955b204172003a0cabaf025294e06e92bfe5a547c0201200023001e0201200022001f02012000210020009fbe0e3606312113bc644d7f4f9ecc3e329be0771ee5b8716779f310dc5abdc99179910fe156bd7903883b35a622ee8d2fd700b6dbe9e07409e751da06f1a31386c002a8f0ee2dc25adc05111033052d79009fbe2f081461ee9099da725da7e280d38c6021e69122fc3988d8571638ef7b5d5a41abc8be4def0ffc6f2b4b833a39d1908ae28a1654ecf2faa51b6a957300036d8002f27309b341dc9c059d151cef908100a0be7c841280fc03ab1363d6cb5bf741e3a1d33846b6543e04dbfb32e2c8318918b5dfd31c70ab27615ceaab2106b91bf366a712f0797152efad01428b4c4d6d2ac0015a53763a996e2e0293af672a3df00201480027002402015800260025009fbda0c74f4405a8a72e56263dc64ff23a0be378461451e0ef9d7d9dd657ff367c40d9f46d4411bf57064ff29c94ed1e3a828435dc9f4658d97d1fbe08f16bb5ed000c067c7db248257016e7ee38e1f604009fbda493d2fde6e74234b7a1779cc75e0033ae6d7143d119f2718939fd656586300448ae86b8633cc9aaa3b0ee0f285c9faf2a00398ee1ece8b2e08a4ac77a0d4b000e910e7012b87a701bbf248eab92e4009fbe0a6155a32dfe17faa972f06ddb244c8dd30b0e300a7e2df953beabd7e93469882f69f060c53be09b4f091430d4d0a7125215c09b7452743d88a697c8f66a4c0007683e28f541581c0e1bfb5100fe79020120002e0029020120002d002a020120002c002b00a0be587e8361c55dceb5c2004095f1f05ca32a74d91febd924b993189788e7e5f85cfc1206caf8734599c77d747e105485bb0cbda6e7445184bb184b543fedbf33c00161a8e62288ba6e02a1a7903e63e600a0be452558fed8b784f100fe78f2c55ab43fb3536ae50acd11274a578347547ed7969a837ed37b1a20c19554ddacdb3a594870df4910f3113beda09d0734df0f58e003a6c3d913e0454e06f48cabfed4ce00a1bea7852593a68e82dba2facbca21b043e0aec314e7a03edbd160fab0d7b8538abb8cd0ba7fbba5139e470f1479e6ab728e1ee4609e17c8052f091b84106c4b0d70016fb6a23569ae1702bc6c90370e16400201200030002f00a1be984525a10982c0f97eabc25fd865d8dda93ed81c5884d9c57d48e1876725fdcdf09d8d4bf6877e253276cac738f1ddc74b3ffde127a15c66d90064261c444fa000af30c282cc4c57014db45a4eb8f04002015800320031009fbe015584df4e36290cc94ee956b1ae1db961f1e3a407d1f8543778c40332ea8bcafc17c7fb806de30acd37703b83ad98efa2304d06f7bfcf6fb279f3cdefa18040042f8ae37f24695c07f915a93b837d02027100340033009fbd27790279ac14c3c2f9f929d487e698630c271087e3a84e29a32d694b5315c2d58bc7030783554c96c342fe74ca8b0b1bd4f7c6330e485c7223d5bb4dfada50002d926969567649c056ce4b57802190009fbd292602bc598c43b9c019c3f237b3453e48506b58077571ebc13db1cb972c157489f52c326e01e76cebb51ceed7848c318c6d800b502188f7e8165bc41aa8780068896526aac529c0c71f5d7354d3900201200049003602012000420037020120003b0038020148003a0039009fbe2ad38d27531c583d9886be62adb4c89c4d9d975911563c25f37ff5c9dcddedd4488d4f27cbafa37280c8919f712b17b15273684c7cd72373750a054a4061c54003c97fd80fbb151c0736b63b6eb361009fbe2b1cf90b7af71fc3b35ddf917353381b4d55db8f8bf64c9853d2590a87c55d39056cdba9dfece6eebb3e8983aec7647aeb9cdda8e3f5c783527c8e653f8080c007683e28f541581c0e1bfb5100fe79020120003d003c00a0be761b8223295587a5bcb32eac67092ec547fcc16b8f9ca8177be52962a2e0781ae1dbf0fa7d816d5de9bacd4b1f2d359374f9bb8a9733a9b5aeddc4552fb76280020c5b5a2abe224e03e6cccd16822e0201480041003e0202750040003f009fbcda3a7996a37b05df28ac475eb6c6686bee33c860cca4b0549c445485002ffba2945a08f9d39b82e99bd2b9247101dfddde47c4131e4b60d474c47945213b8000ed07c51ea82b0381c37f6a201fcf20009fbcfca427f292f7d083e22f6e242f40e89d9f7064327981850815eb1b68c9da120cb0210c62f12cdea868716f7aa6214b6dc6d842323fcb4b04bc9350ad64bb280069323f173706d380c860fea3c1e5a0009fbdc22629a945568389d42cb6d5a6df008cfad8c36c0baff162d2d02a38e3da7e6dc84928a56b23b0e87f6b80facf1c758de8a2c771002baa46661c4d6359f9a98006d580ec22ac27380d0478affc190a0201200046004302027200450044009fbd92baf2a745cd9f855e3a8cd93f7d47421673f136fb6f8847b89a47473e31a86ca3d8c8e4d0ea514d4c02dba714e65c21e4fac08893e38b12faf5ebb441a191000b158ed842ec3570151d020b16f6a4009fbd802d4c08bb88cffd0c9d699048c81178eb2fa735fe08c5d3ebed379cd0eec3044282c757ba16667133a5d8e783013202966b64f8b4d62aa4ba600268c54e6000132af7922df087702482d6a702dc5402015800480047009fbe22477f3f7c5fe68a3a498ef7d8796b052894afef3636bd629aff9e0986bcd6d0382b659387054b36caf0328e19a0f75f0bee60c673dc5147c68351fc9248a60003d5f7fc8763499c074e76a7f3e749009fbe37f25cfa72a7ca709cc03a5b0386697c3ad318490a33650ca45d1cc3b4fcb1efbcb117517b67f1b236b731d8128425ed4caf71ca6a9c612f7dde9a78be88c1800514203e36e6ee1c09ac7e15d8fefd020120005d004a0201200056004b020120004d004c00a0be56d3c6c180f043c7f44f33cd995158477d0d76926ab4db659ac67bb842e25cb2f5b2fcdcd209e478ec2d5e492d64894df33c69785b9098d220bdda52b1b87ea0021b8b503bb4d58e0403baa098a0000201580053004e0201200052004f02014800510050009fbd04af58c4ad50bdf36b2586d1fc3f6058ffcec1d2727f8d50cf8233b891721e6d0235ea522d1f54212280ad7d1c0b86cedae97f3932977c8f893e4aa57efb08002be03e0a63b1fdc0539348c060bbd0009fbd0e6828d3d4fd69059f215716cc5dc0257c09ac9af48152430235c65b2c6bff0d158dc58fb1b0fabd2666d934e96b98e5af50f40b9d2c56f935a328d2ad1e180072e44f1006d40dc0dad8d9e8a25bd0009fbd8a586e7a429dd64ca81b01441413ba665719aad5ed198745670a2465fed33307c8436e110eba5b09b28c8d9aae6c7cc35daf59567ffa72a715208cbc303c51000c2b5b56b0bd8270172e296e44b04402012000550054009fbda3d4b08dc558524a0142751a380677275312d5153ad5f30bfa03e2c88806278a78d1aa831b0fc933485546a264ec29d0db55f86c5eb9cdf5b7a945ea268019001316618cd2dc3170245ba043f20e54009fbda0f57d3400e11c7388445c5cb153fabbe115bd2ef108ab803ff3b0b0b309054212ebc61a81dd5382458dec030a6add3904047bc1a3bbf70bff95830b86abad0013f30a6f12039f7025fff0d6d7e004020120005a005702015800590058009fbdd5f1a149829fc739ddb17b46eeb9c39028effeca0202d2019e52745fc07a98006cc45c3cd91726a741b988d8666ae8868278b94b17f32fcb91aa3573964b908006365d0e9fd482380bd556ff30db5a009fbdc479d10048af4e87c81a3345300b01e22879aa60c846e0171aa7bb9a308a2b59f7e80ebb51ddf50dc1f6603c2f70da971605b8c732e275e0278bbf1d34d626800e02afea5dc5d2b81aaff4de3a0812020148005c005b009fbde5a7a203d6252987f8bf9242b50447a7ac215e3f69e69dc09d9bda117e546dcecc0445bb6b6bcdcff5a425df2539e4c7edeacef49ea9f75d2bf38cfb5a8aa8800574c19becb6e9380a648e0f6832aa009fbdfc41b00f0ef4db37a8c1bc1f51d616290e41dd3f1c335a4283ab0c5aec9567d50313ff6442b4294213f5969de24cc322f01089390d8112112dca535557e84d000ed07c51ea82b0381c37f6a201fcf2020120006b005e0201200062005f02012000610060009fbe0aa4ad07ec973f2702635a6079b666f9ca58de274359095ee3bbfe31b9082284f30a27f49bd557dfcddb247fae752746c8e4cf9b1a3e446c276b8385a42c57c00702fbc7ce07ec9c0d5b1a1dd0b745009fbe36e44f0cc06862bf6abd8f89daa39019d8578a4aa7c4a0cce9238037d7a8902ecce9645c79df9dfff7b554e38e292f221accecc597072f270a449cddb16d268007683e28f541581c0e1bfb5100fe790201200066006302012000650064009fbdfe2d49bd6494afe4dfdfd9c6d745dccc506e4c35d104638c0ac3b402460f7c56832966f8c7ef8ca65bb0df9a351fe85d4b484bd841f16c242d9a99b3403841800631e5deb69e10380bccd5715f5dc2009fbdc4f5e351d482c34a940eb8cdb31d8d2b08ab310aba8823c010b1b8be7de4632183c4f5c3628ce60158fd57d04e6a5bd3590839ec40cb828343927d476fe02800089602f802684238105acb374ee75a020120006a006702012000690068009fbdbb11046509c8e2430ec6ca82e0159d2086b4dbe61ff245dbd9ae177285cd6faab3b903badefbf8e9ae51aa159a59e72cb40de94f8a4dea2cd13d8716711d17001da0f8a3d5056070386fed4403f9e4009fbd825a0415bb943f1da98a951ed60cc62eff9772dd16120e64d4e5d6f47433bfa4fecd4f494a2cfcef959a07f1e5f93e90b85251fb315019de7dc729fc43cb1f0017c4e39f661112702d468e4e75c7e4009fbdf2b143677ccbc0f6e93245265e52ba2dc7617f826360f4748ffda1e7d25f94a5a39e8b85eff1045c917aae3e703da8e821ce31bc8385d16f8acce0c03fc5fd800da8eb53416764b81a04f7377ec082020120006f006c020166006e006d009fbd8a2a2965720111ffdb0da9d82a3677b40aa15e6523112debf020ebe4859add67105f88cb2e37614b27a37abaf12df7a29aa9c6d474f3c032134d9d0e27a0e6000ab5696ec53f8a701465de2d5e94b4009fbdbde126e443431f175d23131c5ec29df8bff59239fa12f764029df6fdc10ec8b241c5d1f950d9c467fb1be7ba4dbafed6a10a91e237d342fea598889a4fa52a000d128179692c6b7018e674e5f8ee84020148007300700203786000720071009fbca013be55d51153e8b771c5ab55cc9580128342a81c1374dad2780e0dbe2c187b558d47def20ddcf08e2f846a8aa357969d9c94935cf194eadb448b6869af200102b45fbd2b7a1701ecc873019ed040009fbcab608b45580b8b19bc8e48cb17a3528b64e529f3d5620cafb063d45cca66803ec6756506ed4ed31b8a9c259ccc0b8143ea21c3f859087ff30d842fe74a44c001cf908617133aa70373009316a92340009fbdc63bf784617d69b355f2d0f1e6be448ccb6826e1b75c2739a1692c0262cffa5802049a937bfaf6b12ec3d9490ee0096182b95886fb19614a54d294c518333c0009f985378901cfb812fff86b6bf00202012000ac007502012000870076020120007c00770201580079007800a0be59d1dbd33619abd485d47b32ad9220624fb0cb32bcbc25bd44050e2569fcd7c966a4e215a216a27ac4feb7424e097265ff9690995f0b756d469535ac7b28cde001962de5f161acae0305b1ba7bf9d0020166007b007a009fbd9b5e72acdad267465854c44355300da6f516e10835ecc99c8e0e70d9e7af924c238e4428c7ecc83621a977fecf22fba8fa7f7570dad7a671b09d8bf71af0e9000b3d72ec7a4c52701568fe1c43c6d4009fbdb6f137887dc86027bc0321fc7b789914d6e2291168f80c88c3e8b90027f41ede64b26b6ea0db5dbee12ec9a15c0a03b8544a83648c2e7c61327415dce653bb001da0f8a3d5056070386fed4403f9e40201200082007d020120007f007e00a0be5676d3a83fc962b2354e9d8c0b90d52f06bbc3f041d1449ab15358b91235e090513c13d9bf6d48fc770fcbe5d05f40f5763bc37798826035afd840b2b63d59400140ef2182152cce02635182720bb802027600810080009fbd7be48be655da1e209053643d2b07ab027f2890849bcebf61c86b2e27a547fad9c3b05ebcc8ec35137ac5fa75fc62356e4778a906e9b46bf5d646bd37090f2c002cb610f40027e0e0552a93f68eb4e8009fbd6034719fd659b9d4ec0803c887eb382039b7f19369e83329dd8aa34f7a70443a29f9e64b914a48b4c8114fb57f39becb217257c32e37fa54f2a703faed3fa60015866d2951b05ae02900504dbb14c80201200086008302015800850084009fbde4ca726d7249979aea18f22d833d50bcf9450b38868dbb6a918b989834cc78317dc7b100df75b2737de9a0ca8e923e152eefab8c36d399408636424b85490a800568cbb10eec89380a4dc5b0ac2dc2009fbdc4c1ba27a648e9212200752163f56987350cc8f7354f63bfb7cca43805be9484d9a4ecfcb0909bfa01fd3fc9786d698662ea746ef0b7af9c71e9c720d0110080087cbcfbcbc82638102aa7201def8200a0be5ba9c920f928ded5706ef6f45d235c3c5b0ee1b130db7d8c9cfe2287aea960cbcb47130701ab4b2caf1d8f4d3e7433f61796258d882b099be2df15cf6b2b044003b41f147aa0ac0e070dfda8807f3c02012000950088020120008c0089020120008b008a00a0be7b804e851ea518e6b672dc04e78b7b3642831a92ef96a67cf392ef303e6fa79487ccb0d44fcd83e948b59da16ada5fe0e708c3ce06eefea1095794c66167b84003b41f147aa0ac0e070dfda8807f3c00a0be4577a28b22530baedfbd0d87aa3f203bbd2516afc76111809b712878ff7b4018dd201cb57bc3a37d21b9e995a756bb8bf75b90de30fcd99d179ec5fc2fec53c003acfdefd4632cee0700691b9979e60201200092008d020120008f008e009fbe1e96e3db493b1f44bdee144d5fb4552b202375b0f743ba05f4726e7058a891176da5c1cc2e8a10bd106304042cbb186ffcd35e431c0c6813265eba1863ec5700073f81960eea9b5c0dce62de29ea8d02012000910090009fbdfac2dc4c67374c6f01370e00496cc38ef8617f09f69a71fb8a8de9aee17791783395b15858c250aadac6d544060b7c97ed4e4b0c868ab316873b0606c0f2600007442fdb5fa5a4b80dd74d5879512a009fbdf8e001588b46a2186e63ffc619ffd2a1b3c2ef449ad3aaff137c292e2c5f699098054af429233f241a20d699c581dc960c55eeb484192880718898b72f0b380005547681a1c565380a270acd80fd9a02012000940093009fbe30d291b446706f7d8a143d7a67a2c3dbc1e8179c0aea670dcede1dc5e327e09ff3bec03428b352cebbaad6498649a1568a00d170a430b4c2fd78551ce941ab80048638c516b88e5c089e31297dd23d009fbe12bf1e84647fc656aab4a97511734cf2a3e21f54f59a7946b934cf644a6a44b1657e18055eaf228518b34662e2772512a69dd59bcf6ce0baae0a9e9fdb046f00043b99f773ac6edc08100df503f251020120009f0096020120009a009702012000990098009fbe14190c2b0a6575c9d9d997e73cedf6fccd7c9713a1cf32c1769e2a48c3b49316b8a49f6fba2dcc0a51d8899038c0583e332caafa78aafa4f1fe850b805972e80068511f8d5d3f7dc0c6b42d86ae231009fbe072f9b187cd7429886db5ef3a387d8499c5563ce4998728401097a5fdfc16afd9f662d58c72c818e1c3887367926810b82cf6a1aeb6ed35d1bc7f34a3359224005fe3785ba51205c0b6a6425fddbf9020120009e009b020158009d009c009fbd87f8b0e49b899dffb72bff39d946784e4c7f0c1d9e6abe626371d0cbd2877ce3da41d0e04e8e87fd9892f44c1f2244a7631e381bf269e5944d885a1b49905e00130a3ec9de82e17024448279d3eee4009fbd89f6e2c31ed15b83f16bc61b6d8ceb72716b8693c5dbbe4d8459cee4943d0ba93bb10415b8438a57795cd9549d693d2ae32f05b688a73bcc16429a1abd5558000a29bc8f196df570135bd023258674009fbe3a19f7e14034ff188c54ce174da3502d5db9bbb795aae2421b77ec2ec15d63749d72cad4ca427d5a6bccb62e02766f8eb85fff0faa393a202f8e5c16afa3054007683e28f541581c0e1bfb5100fe7902012000a700a002012000a200a1009fbe3d6c47b544612565c388079f419c5ce68a83db6f550973a00a6d8c7bfb61ccb3be9841684a1357f1d124df1b5b4cc8316fd784fa68a4188d8f226ade12370b40068b7647eff8e2dc0c776fb3df2c7d02012000a400a3009fbdfbec1f9a358adc7e0b5204320ef69a173e180bdb3b1722fabd308c63b4c64f868663060805df2f03ac9b61ae8dedcbca84e9cf2326f0feb0e9d8940ab5ab2e000578df70711087b80a6c6567f3f47a02012000a600a5009fbd833da2fe324e7099e595255dfd4691ec256208b08c13a6173d87e9567df3a6c6fbc91cce96d2db3ecf727f3c4556bc90df17648666c15e42ef5ddfb19b50d7001d074f3d263b5570374b3b00a05c84009fbd8f03d4a850151fef469855897c921a516fe8dfbdd76632bac2e93ed421cfa0516159183cd8723134aaf66ae81001d72b30c421f32103f70f154604514a1299001c05631b05812c70355feff93bf4e402012000ab00a802012000aa00a9009fbdf768babbf6fe7ad1d5ab81a8e7e0f6f79a2c0945b0ccd476b6e06a0ef3e4fc99ea4102878c236faafd63044f642cdc7f7b417068d6a76329371e58d36638ae000e8164b4f67ec2b81ba14ec872018a009fbdc9524138ee44dfcfa42e3bb800d81eecea9583bd8f54c2c7f84d07911c9a5fb62e1d4b691f7f24a871ab791b18f7d193960b1cef873ae2826d036d2c2bb7f10004f02970a380e5380967fce0ab5452009fbe34df20bfe941e5538c6056269465dc70675107b29d29f893c20b9ff065c3ac0039bec4ef36b8511599fe41f2f56a2065f2988174d9aa13e2cf441a0799f90bc005ecc955267c7a9c0b493086ac887102012000c200ad02012000bb00ae02012000b800af02012000b500b002012000b400b102012000b300b2009fbdec53facefb35f3d7b5ab410d3d3ce4486ba9d40cd2eebb13362df3e7fa753d72df2a314ace400df4e7e64abb9387fa8a5cb6f6e235051700c5e1194ed0c016800ce47206d419fe38188eb86f405f82009fbdf9e8c870024f0678a00272d355f47e30bc1fe2ad7691e9897129443a0d57ce390800196633a64bd1f5fa3bad2bf7b1e57a9ca4587f739ee53147d50d3f02e8800650f64098e5ff380c080137c9f6f2009fbe08effb3c3764b3f263e6683b9ded0ae95cd0566ad6d32bc862128e4cac42a30269698f8c273888f29409824f46ab13afdd059b6ebae9f252e680ee168ed9bd4007683e28f541581c0e1bfb5100fe7902016a00b700b6009fbdbd33005795649816467b5335a360ee6ff66a1de8e904beaa14d1eb5369d60cc3733613772f75a2939a1073a9a52e7ca00cd965378ff13d13e9d4bf8cb11b0b00151b2403887d23702833f46c8e1ff4009fbdbcbb969ca6a69b9505bcaf71605dbedd512e4a99e045a672546a0b5bc2ea5397331a562c2de1d03798fc55d2b3fb6377e144bcfec22b13a9e0fc39948661c8000a6cabfabe70e97013db4fda3b385402037e6600ba00b9009fbccf944fcbe71b6837b917c90c68bd4b8924baa4674116180cdb15378f0bf473baf74f2caf7612f850a3d8287b3b629bc0ace221e07b64a747966305f78bf08800a210e024cdff538134b469a3d44620009fbcf704adf319846ad7168b3b1b67ccc0542cd0f0887f29dbcc9c095d4abc08d09c99bf22a8d9a40e4c5e5b7ce4e5df5be74fff99ff82ab7aba43d94dc051794800cfea3ea87f1d03818c09de0bcb27a002015800c100bc02012000c000bd02012000bf00be009fbdef0ddbff9366da61bc3feef58acdde2fa75aa1165eb551bf97afec32b87285ea29291e74d80a8fae6cdbaa1fd131809d986f6bf4314782c50fcf4f0c8f3337800e83d5f1aba98d381ba5f5bda253e2009fbdd88fe9ae72ba1b29612a232cccd0d1501918110ae6b2ef13997cbcd79fa2d621d1f48755ea3a972ff29871d2bd30a4d01419ffaae865d7e2bcd25172d71f6a8005986b0fe94609b80aa87bee538002009fbe0c75ac60a9ac2bd80989bffa13b421eba9c843006d4bc22371091e39d2a19de2b9dcbb73799c300dcf518874d214c6d26df985b6c1dfa928178265a37776318002d135800030b89c055dc4100daced00a0be7eff6ffd9a52a0b35d663b513775a835cd415010940d53c3eb4a67452348a01ea9ddcd6404f1d468bfd41b6fb45d45388a07d42a0d150245be60e36f6a389000015587bf16bb084e028a8cd6bbadfc02012000c600c302012000c500c400a1be8d4ac7cd4617b87e5ff8488707db7c3890dd7f0fd90efe86ea83b5ca0894ebab478ae3a9f09260ca33dbd8520d7a2e6914ccf904c593b8079023912eeb71daf000f01ebcbf07755701c9620e854ca94000a1be9e907140c5091b4593b0e6e3529c98adc7560ea742b189cda5f240936543b6ccef78ee7907318c68337eddadf8812a1b5e21ca9693d99aaac9b00c6cdceb089000acb9b1ed1489370149024b606e494002012000ca00c702012000c900c800a0be7f11863a9442cb2689b68e125a78129be7dcb71268fa3f2429f9d3855bf07213df0c9effe568c77e485c6ddd3b589a811f9c78c9535ae133c805a766391e5ae0014e2f32a1f19cee027c8ec060bbbe00a0be5e29994da140b7bff623cd34c79300e585e2cadbf383000417c1c137ac60e0e812749d066377fa381c9c427b8110074246397e60b38204811c23a633f83d076003b41f147aa0ac0e070dfda8807f3c02012000cc00cb00a0be4c18bc991c6fa4d47dfe42ae79740ebaec6ce0f5a90bf6a776a0c5a2f6a2a205e39a7d7657c6606fe56e1be3891df321c3eb665fdf01ed9cb8c7376e906215a00179735d8797450e02cef8be2f316202016a00ce00cd009fbd90ebfbe5815c5715062c895058e8c74ea508d496d6f37f4493ae10058349cb08c719ba537a62c4987f8e1bc0742560227e15f322dcefb8c17e6a5c806d3c760009fc8807f0bf6e701305b4ac7b72f4009fbd99563afb2407862aee73c4769f1044079a00689d88d50292321be6d7ff255b04ec222c539beb7d8cc25dca46ffaaa547646e9e6d36ca3e069d93d13ca6999a00147e87eab6ca06702709a49bdcc864020120013300d002012000fe00d102012000e900d202012000e000d302015800dd00d402012000d600d5009fbe23e38ea28275eb8e1aa16e6348d9f67c6ed2f2020dc08febd261e1771ab660e478cefb87942f15270865d248cade999bf6f4ddc9187043bee39efe7fafbe870004443588a5a50e1c082073554d589502012000dc00d702012000d900d8009fbd9e5d42a88b22ab86632079c8739a2adaa27ae9245133d8609d370b804f5212e01a4670d618263efb9a7198b7a5ad4a9f8b9a1c3a3bd2c6bdd41a79d2bc82d9000ed7e84f49b882701c4619c070bba402014800db00da009fbd0042421c2c75228994122032277a00478e389b48765a04e1ead4caf8c8f39ce9cdeab23d6d5eb70e01ecfd58c30a817f2f91a3a2a690af2c9e63290213680c0036db69e4b12131c0687e07d2091910009fbd0c96a2a9af850022d0d1d88ae451931ff83f39bc9c91f60f79bb5448ea212e84f2c1b3c72764254e0f9920a3c216731f66ac7e26a349528d6b1e6cb9a69638007683e28f541581c0e1bfb5100fe790009fbdd6749c546b2aa9e77b6fd5d80475eff024151e7c96c5ae7440d6773ed8658f8e721af8ffee451b281970e1f3fa6cb54487509677270a15e673086c9154e9e7000ed07c51ea82b0381c37f6a201fcf202012000df00de009fbe340e1297d5547e4b06a94f6ff1bfad9766ac5f7d9ade22f4d7f3cb767bf98a87c9de8420ce0151a1f33349b95a067b8c4192c3440ce17bb5f5f9f5eddb9ed4c003c8dcc44edace9c07357f99e81e09009fbe1ae13de8e34675798ccab16c39120eaa90979e441d75768ff4d308632b650db33c9c8c61e737ca899bf04c2b5f12f14b9fde20db37692c5980bf34a09e4bde0007683e28f541581c0e1bfb5100fe7902012000e800e102012000e500e202015800e400e3009fbdc08ee5f73b75fb0d34ee31412b366ee136d4fd6cb01d4b43a3437b155c141c7be03321fd2a64261fe878a17332b9a4bcc3542f9b8e274fb00184bd5e1ce29a8005744fa3f2157c380a63b4f8b8195a009fbdd18855a152562e44d3e2ba84c3d20671bc8456e1909d83a8cf05d957ac0d0c29aad66f7eb0ffcfcd58b3250666ba80cece4c16690c10da70dd2f036767c8ea00066e06e54f34d6380c3f5e3a4882ea02012000e700e6009fbe306145b937d45c1896235f5d20970f206b8a8f535acb21e36b09488ba0565f696c5b7d9807644ac5cba9e7ff10e9dc2c9437c60e80c0e98130b72b9213b8b78007683e28f541581c0e1bfb5100fe79009fbe0c30e56a33eae24348040870b217957800dc6f936e2280acb3b79dd528526341bbb270e9bc48b0a5a5f49eba61f92e447536b724df88f1ea962efc79212c83c002b4a6ec7532dc5c05275ece547be100a1bea4d71d47ecd52921b2f81b711dd848ae3ed6545ef8dea23c2e8d569d0c5335b188c27e0fdf741cffcee57b44f31e2ebd233ef74fce68c04d996c23c984e8112001da0f8a3d5056070386fed4403f9e4002012000f300ea02012000f000eb02012000ed00ec00a0be6b9a794d704ee63f73789dd5e2645052229c2231824411cc8626b43a51e350f689062dc2543961bf14cc07f7e606ba4027b95254c25854f4a3613b930ff2a3e0025851ac2ac09bce04777e5738870002012000ef00ee009fbe15cabfd7c0a6f4f9fca504f254865808c818c8a34fc75147ef2a2f871fea6999f60b43be0dc84f4c67fed0dec52f328b7b87c50d565cd3af11ae4a9d6db5e60003140f033d58561c05dd1a0c314e21009fbe35f3fff55357c6e65df90229da599c41913dde124252f01373fd9721f9e4201d1071f04167e17232930b9afb535545bfe2a4b937e39746329641de52fa8135c002bcdfbbdc13a6dc053708119c8d5d02012000f200f100a0be7f2206e0c73953c5ffbc7aa42a111d90b7664bba2d34a3de048b02b62110b5e6ecbb2f44c93e69a4d97413a604cd825d3daf6f327a498fb8b4429fef3ceb0340015ffd3954d0812e029e78ec8717fe00a0be60c07e65c983b6703677aa0d4c4e3136f975a08e2167f1cb2c6dff83faa5cc81e51102e6066257625bc900ca1f32b9a612c2c8087c33e452d209c2aad7b58f20015ecdac6dff080e029c36b7dba71a02012000fd00f402012000f800f502012000f700f6009fbe0f03ed2aeeba33e13d28bf0fb21f8730abfd40686f9c6e99cb5647ca5697ab2c8be81cb65ea742cb44a280d6c96f014a88f7d76091bd294f57d87fd76350648007683e28f541581c0e1bfb5100fe79009fbe225e6c098d0931f2dd1ee7693c6e13072eae0e3062cae71866e2fb2932497478b4082f8690a8ecd8feda744de368821f0ebf672ed09dfd307ae57aa1945fa20002f4e9fe3eee235c05a1c6f6750bed02012000fc00f902015800fb00fa009fbda67e673b5e39392fec4c98fa75b51e8ef8efc060a10d18b655933b5343d17e5b346ca2f44266086906f3ddf89ee7211e577cffbe971acdfcd696f6676a84a80013ae1526eb1ec870257c96b25c4434009fbd8b98bda1b0d555b7162714e4468a0a96db50e39e40d097652bf42981f326505a7d599bee83ca24728bf0ce4f7ba55f6360a68898f292e6f95301371e9890f0000bd3b44483c7d470168733454700d4009fbe2fd20490085b4e56fd0b3cae4c27f0c5d584a1b452c751c51df8b958ff74565ac1843843ad95afc54befba79f45e0b3694209c9ff3ce20e36fbf66666df3f18002b4a6ec7532dc5c05275ece547be100a1bebefe9446e73a9e5baf4f122552a8d743bb9d63cb8d57bfc3591145564159277041f6883a4336b9dbb06cdfb9a06d38b6bef5a0e60ea6596a2fbadf0cd8f0d84000a04d05760e3247013157b761e17640020120011400ff02012001050100020120010401010201200103010200a0be7f5c74f914e780b4c04511a767b3e2b94e8b028251a4aa2db8c0304c3d152ad75947ee6f09371f1e570ff23ab0de07e28293002251ac9571521ee7470d5eb50003b41f147aa0ac0e070dfda8807f3c00a0be717de4b01d5d605d45a3bfc0d8e848c9a560912784be03c7c382e3949cb2b3f3a6fef69dcd559aa05a3b6ce5bcb963f2dba8edd352d85c793d1d2095f1ef23e0015e95bec202acee029bcc2f6af79400a1be988620106f8a21118ac5db2abf2f55d2d9b435c6f5cb374cf2aff3f31389c5cc32a17fadb7a76839e022e692d7ce002c49d60c23fd287e18062ba21269363fa001aceabe7fc9d6f7033101550d76a140020120010901060201200108010700a0be7f6a5461a990a493c20a66ab3c6bb09dfca79dc825c446a2c4d60b0a14e12045b53d4f6b8f0ca56457bb24ae0e6d5d424519a5e9ca2b1f508751acbd665f426003b41f147aa0ac0e070dfda8807f3c00a0be524145c18ef577669a41a26200510d078d6473130eeeb9bd46852cab063fddc0f206529701e30c0b20cf4ab2e6ea009bcaaa337167dce0dd92535fc917e2b000021ae524afab9cce04027e1adab7220201200111010a020120010e010b02037eea010d010c009ebc796d329f2bd45fbfefc784cb283f2450e2f51493299b2566e5aa036b8e9cba506421c181eecced893c8970b691ba705fa0d148f6d7b5f5465f82d5b3d9198003b15fbcbba8a5ce0708c1ec91fdf0009ebc62ebaa2bc73bf7f77a4e8b76fb2837b53a1f4a7fe04b46e36616f608710312ba52cf71a390e1bbf763f42e2582643caab309f8682612466db37aa0fd07834003b41f147aa0ac0e070dfda8807f3c0201200110010f009fbde1b1143b9132cbea3665a205b5ac3db075d57f798d5acc31d8f59ea10eaa59fe340ccf0111c57fc9d7dbc2e3f5a4b57bb61ae5fdb568eb921dc82c12601597000e7d41a4323cdf381b996d76fa990a009fbdf31e3879b4a30c622766779574d6fbb387d03d7e759b7eeaa50316651d6063876eef6d7c8553241804404b8f250b2e4893cf878b26f02a83c5bfd38a6c119900059fe46619d64a380ab6b879257cc202015801130112009fbdcc9050ccb18783ea39364be70bf3b21324fb223d76d929864fe8eaf4230fa4e44317114f8baccc32c1c9336af4b1a46d4298e74b86983675840052ff7f80cd8007cb25db52a2bc380ed8608538989a009fbdc422b003225a4629ef96fd480315b24bf9b3657f81a474434c36c01c23c2ff15954421b3b5b4b6c9bf592f293a0d44b6086ec6eca4aa52185cc020ecb8dbe9000ed07c51ea82b0381c37f6a201fcf2020120012601150201200125011602012001200117020120011b0118020120011a0119009fbdd276bec9c5071d995bdd07faf18c98faedf102974fda294033bb7dd483faebe3aae5c7cd4ae7a31dc5b6017dc4f4ac6d1771df4d20e8d0fadce771b792b323800674245f1527f3380c4b0429325d22009fbdcf55fcdfccf1217a93a4033da16cb4bb744c80dcf296715b54591803b4edf437a1267d10dbf9b93b7e7c823ab9e03503e371c3b8e9ba12c66a72ff7587b9ac000ed07c51ea82b0381c37f6a201fcf2020120011f011c02016a011e011d009fbd32d3e6293de6fb0907d4cfb245a008a218d7041278329eb53d2ac0d880595a3b11611a1052cdf5610c232b48e9fbc5b25e549396e69227ec5c6bcf083c6e0400337d1e0754b0d5c06213852573b290009fbd1720907b33eee550d3ea551de3259c4d4c033f76c6b381fa8cc91b33908dc7d47546c7307697c0b8e4f25499d0446412005f498b8c668a929436377c8f2df0003c42c3a09fd9b5c072c90ccfd8b590009fbde9539b4a1fb70d8fe73280d0e4eeccbc90ea347e0479b30b34400ee5f136739e879d5074a5a03c2a7422643990a5764ec13936e2a2cdffc6d89222584d5bc380058b41b1ca92dbb80a8f69ed40699a02015801220121009fbdf418d1813cd58aa90be67f09f7342a31c86cc45f8cd0a9a8633afa6c85e0cf63cc074f5358b04cca3d5b2ff02483e9ad3f186209b3712b5e8c7e04b28269fe8005450a3a3d4c34b80a09aa153527ea02012001240123009fbd88c0e021dba799b0efedccf95a4c8354a040eb6d4dda295f6b3c03792da5b9c23f0c9a57aff0c4691e59b38c48e1b0f86606cd8f22f5e3b6fb633b24f8022a001da0f8a3d5056070386fed4403f9e4009fbd8e625b9e5009e678e44437d3dc0575257bbc1e29edf3c4f9226fbbfd92611ac2a1f5c221f64a70fa25424fb82c3efd5a3301d4d0ecbe87b03b9c1f8492126c001d2edcca5b9628703796924058c3f400a1bea4f7511f34941cbe8c76f2464c843602ec171e31507965a7f611b97149bcdccc0e135614417ead123d19aad936d378877b27d2ec21125923e089549408e906100150d46a2ea678a7028198b891796d4002012001300127020120012d0128020120012c0129020276012b012a009fbd13ccf0f91aeea51b81f43ccdee40b0605bb93c64d2cd44ffe8723e717cfffca4f22602a95eb32264c0ff1d13f3bf0df3c7e03a65b7dda146221c8ddf8a5680007683e28f541581c0e1bfb5100fe790009fbd3723eedc13092b749aa09477924f095ca557c7d8267cc023324547b3adc6b66d6ed5c14c3eeee9e3f07570430b6652ab8fb3db20f3c82527a8f2ce6ac24970007683e28f541581c0e1bfb5100fe790009fbe2e6b1e7bbfb6458788782d37df2f9d4bb8dc1148d6ae5976486b7337a10f020d72141d37b1481ef9a09032003d15e195319d64d5d18027a2d0c9fe140bdfa40005d45b122d4d009c0b1aa7590e39e1020120012f012e009fbe1fba78c1f0ccc40cb5dc9cb82383194b3aa17a18eec8df2995c9ab72448237f22c9d895ab11767a5489a046b060e62f62596e2f174139ae199a4d70589ce60800714081d1573639c0d7b93572d3635009fbe2f4594f58a4a520cebd89497de61ad20a2bd61147adc93927e46a3651b259789d5810bf6481b9118cfa3ab06d751106b6e470e2048117d18be733e137b1ea2c007683e28f541581c0e1bfb5100fe7902016201320131009fbde68392c29fd3680ba157ad17c7e2b0abb4d9a1d63fb03cdd172ebe7caa99ec075a1a62f2b7f4251b84540b69f25bd69059a775c2c916dafa1797d978a4b6ce0007f56b3e69d13eb80f28e53416938a009fbde3709607a3df0c36692c4d6aa54ec9fe0128b65d0654d990202049ef20473aef41b45839506ba7ac1b2f0b78fbf00d53e34c88771f61a9d7a42d0ad0f245de0005735c50680735b80a61e57b30d9aa0201200169013402012001540135020120014101360201200138013700a1be830407df5778e7669f4d12ac252a26231336f91880e478b2f74c000533a7dba81cf0327a87b0f219dda382ed658b10fcae015effb6de60918752025ee55887f000c948390db3f547017f6775736f3a40020120013e0139020120013b013a009fbe15c318a38e8f4abf792bac66de48b41787d9281c7f99d904486486429870a10aa25dddfe3812df560ee284a87585b50d988538d4938827f2e5482b8a2417928007683e28f541581c0e1bfb5100fe79020148013d013c009fbd9befe7a071ac88617de872ad5a42cd6de9a736ad880b1ee8eeaf6721de533589ca25134210f7c2b7e6568692f9a0b90b92a186dade09968c188291ca7bfd580013f30a6f12039f7025fff0d6d7e004009fbd9700a35999e34600758bc25a4a50251f241002ba748ed3ef3c352990b5aa560d19ad161fb5adc3e7e46ff55021fc0d0c7d6599fcda9606af78f0f907af8fec000aa1c799b3429f70144078ec4fcb440201200140013f009fbe0099409e23bf0c57e9f21a2a1dec68c2a0e5738fd896244340a26340cce6fc75cba2118218a77cd4e776c0faaaed89657db2ed27084305615a968ee1f97832c002b90d679990e85c052fc089b7a74d009fbe0eca0ab92c81e0f1528eb9043f1f95e4ac39d855f865e0934db6400c2bdb5f62b266bb9ae43d7e8bdc932d932fdd5438c12497b0db57f31a917bc64964171b000508761c3a76459c09964612192041020120014901420201200148014302012001450144009fbe0cb709d031b6b64cd1710c6f234f8a4b1cf369d250846b143d3ce8fcea1bb1d8ec0f669b2d9582c151dcdb24d511aef55a21a3b67197da471f89511f2df22440048a0b15a5d2a39c08a578aa56161d02037d0801470146009fbcaf041d26640ff46bfce8de066e64527d01eeae9f2533b4559f18b03753c0f90bd0b03f79f95d0502644eb280590884dee56ce3c21bb7a199532c08a235f70001b1178cea3ec8770338f534723d5540009fbc8f3df17a2a6bd6624a4db11005ad26315d203fbd673f2d6f9a70ac53a49b66828939acbf1045a3ac7cc2ded111d1ff9209466212dedece856ea2cd38769c0000a9802dd25ad1e70142ddd7ef1ffd4000a0be718429acc69c020c9d7b47f3ca80d727c2410fe45a64cd7a44da76a2574c4061b25a4b789d1098ac941739e7e38fb7a61666d6e33ba092f56ee83fca36e276200161460e08e2c22e02a0eb48bf89be020120014f014a020120014c014b009fbe2578aec84b1620273509effa7dcda5a71219902e7f7c0adfc50dd448f6a984414fe71289523e66ff3ca1c2f54c898c96268a1e2f1097866d69c03b363deeaac004c259682ab36edc0910b934d6dfdd020148014e014d009fbd9535ff5649d5f1d0b752ff988b71bd5f52196d6ac5ef4b8366d874f284921b626d92a11c68caea5d9d4f017c8bef6649e4e25c6e7952785dba83e2a3c93881001da0f8a3d5056070386fed4403f9e4009fbdb7673118883c6a15919c1449d2922ab64404745ece5358da8076dd5a239b20b35240ac9d6c542bb9e08c969bf85d33ee7601a2eb633d831f9fc67fcc12106f000a2e7fb3b507e6701364e25f697cf402012001510150009fbe0eab550debfa3bc49c2af5653deee85d9056ba74241ba1d8e0c8df31cc5af5375cd147b45f251c04828f736b271f133ecb2b861004f536721beebe2400368bc002a5f8c66e8f1edc050b683f88d85d02027401530152009fbd385a283b7650c50bf5ff1e19a8f781f5107068a92d18f90c2c6868987a764117da577ec1ea46cadbeb788bdf6b5bcc4abca51507e9f049abed81cb65725efc006a6386e731f72dc0caa67ef85ad950009fbd066a69643e42b2e6b48903d07bd1a208150bf1fc0a90c5e59acdc48c35764968d09eaa285c59f7beae434440e52d458f5aaeca51dae9b18c622852d718417c002977667ee20b75c04efc514f0a445002012001620155020120015d0156020120015a015702014801590158009fbdd1a27c338523e157e0c20cb3dcc8bc00fb13a253c2efd32b2de5fd8a2be035f328379a8986cfc84b81d9654c92da48b4a7a7d716fa8f9bcb687b0992a7826a0006d0bb40add859380cfb61a3988d32009fbdf7c4dc89c4fd61e24308e5e38942a0d198ccbd597c8b4fad9957d2cd9f733c68694d8c06bb2ac58152ee9374f29e81e1fd7249708bb1315dee63b6e04d79cc8005832589fcaa04b80a7ff73ee32f3a020120015c015b009fbe0132f356e48166933aa59e60550649c634f33f2957b2183515e89f9ff6112eee8af937a4cae9d6f300832ac4c2b6c95884443cd80f03d3ddcafb77b2ed80aa40027ce779d5d4a49c04bd2e69f30fd5009fbe0385bb0df9889e2024d591cbfa5d7d5fb4f099350ac681181cf813bda120d9d76fe30d113478f2f26f79a9438a703dafa83c1fe2b759d8168936032b0d50488002a87887226b3edc05102adafc86ad0201200161015e0202720160015f009fbd47a8bfbe9f5915a50cd6c1d67adf8dff45901eb3497a315d9b13f12f009f87e5303e1279b62ccab6ddf142876c4530a08831922901338e058fe924ebcb77400016c3bce574ff22e02b5cbb4ce2b208009fbd64dccac2575fa76dec3345ef9096af56dbf4da47e2d87c17a07db4aa97ec4c490586f107336b079ce1c7023bb727c09d001d4c09061b5d2eccb77ae75539c8001520be5851b4ece0283ea08d511f6800a0be62537abc73655e9b524b9f75de99890b9393ab6695f83c30263fe2de68d430f8987481f8d09900d91653703ce1c0386edf8cef8790dbba842eb9c3ff745246e0019251d8320ec3ae02fe57ac97113a0201200164016300a1be819d7b29e8a77d8046ed971b727c0460252be67bbb6986fc711a0f11ab9d6d33490efb3c45e576cfbe7fa9e22bfe3572818dd62494cda61ef02b27de360ab47000e641da6a7cce0701b698a20b90b3400201200168016502037ede01670166009fbc8c380ba0a56e591f3199a3ab2be42f7ef5c303dc6bd38bee4d8c186053ad5354a0e221b32b69c0355248a1cf1e9a075fa4776ef0a9817e715b6cb56d387fd000d81e6cb061f467019baa4f6bd96b40009fbc965c86dc9d04db03a9042ddeaa33fbf692e1ee00416bc6aa05d34a070f25495a15d054242ac95423da8c68de14ff83cf4112391ed4102d0a16dc7c01fdcc8001da0f8a3d5056070386fed4403f9e4000a0be768edde3e37e3c9df9b0dafa732f00fcd1a5bb117357d3e4c85ca1e13e4e54e371014056529427eca793475a4fcc039e373703cf62c9cdbe1e9c1ab3b621bee003a262ddbe319bce06ec356a3692c20201200185016a0201200176016b0201200173016c0201200170016d02016a016f016e009fbdae269b26d82872fcdce0d04b03f516303dcb23dc1500984f868df8eb4a83fdf81dfb39e07e2a76284a8e14c806e2e6dd84fe7e0f1be4ca40254cf1df999e4f000e5af475989aa5701b5816fbea1204009fbdb9c4ad93ab1ea7da63a049bd88f61800735292e3d93c3c207bebea44c6d04c7305fba6a80e9683d1370499bda04250ffbbe4aeb4ebba3fb161ec5dbc33d358000b02e16b280da97014f96e5619675402012001720171009fbe216a864f4680a14d2834e21f6ad84291c9e96120689540e93fe7f84e8a1968cf647e32256741888d51936c0bfb3c6ba4b22f595296045c225dba5c16fad7424007683e28f541581c0e1bfb5100fe79009fbe285dec70797a9918071b0a35e53396c02cac687bbb9a6c2c926ee3d0cbc6b42942b4dc240f918dedf70ff78d1f9fa7462b6e5bdf67a310a194b36beae17957c007683e28f541581c0e1bfb5100fe790201200175017400a0be488df23cf1aa84092ef54b81f256206b337e6a927e74cb7fe804803ade02d10775f7d41b416a2bf1839bdef7751bb9084b46569d87df7d4b7242971b85ab25800380b11dc4ff7b8e06ac0700b86b3a00a0be4f3c99572178164baeb422f3a4c8b0c61fa4d5deee4fb2b14fea3246bd5648a45d0fe49ae36e0820eda83b70a09ddc769c3e886e058e25d62b6f212813e51e0002bb05246fca1d4e0533800ff38000020120017e0177020120017d0178020120017a0179009fbe0f2bf42fdcd82d169308263e2eaf61c557b25625a712abea80fa7e92c1c0826f19a31efea8522e2304d7d098385fad38879388a036c014a49134d6cbf5a09cc00299fd5af49cc81c04f49565aa87fd020158017c017b009fbd97c1857328db1fc12d3435ed8b8988a4dbeed6cbb1ddc41bd75779891bfc869eefe72434ae8859ec32661cc3d797315bc9d217896238672648350f81962ae3001da0f8a3d5056070386fed4403f9e4009fbda2dd985dd3c437cbc689f68c28f84cb639935fe248c6377b0911455f36b3207f8009d170f6d6609778aa451477ee993947d6787b8e1c81dc3fdc1bba576026000ce013ab0721d8701886662c9d1e5400a0be40db27d29427abe80e5b37d8c63044517cee9c09bb5374bcb01d2c546f7c713b54c6657ce840f4e69e286fbbf0750486ba7455e03272c0b1d0ad6197c65e2b8002b8d3715028184e052f52219ad69e0202710184017f02012001810180009fbd66c75fab95b38ff4cbfabfd009706ea2ae814219795565db5515875408c476aceb3025043a20514bb07f3ab0b82160a8358280ff6368d00573338c7f8f80d8003b41f147aa0ac0e070dfda8807f3c802012001830182009fbd240795a00e45de5156a764e3ea5d87cfcee309ee7250dc7dcd059dfcba93101798d07c4890423d59ac2fc81e630843fbb5801af89f096480585f86f17da9e4007683e28f541581c0e1bfb5100fe790009fbd06531295fb5da1376bfd8260b5936dc4b8d3f608a22be5ade0a7ae55376785e2eadb8562eed74b095a6327e2bf088a8f740427e0936cf9784807430533d6b4002cb15fe1cf2021c05521a426bc0f10009fbdb247293e2e0c7df735b55fe59c6d565a7cd0f83b52790be26442245c278d7fa99e2a46b8850670feb77005db28fc14a81addc0304f071b06e9f8d7822962e5001d08dc1b15397070374e2ef55b1a940201200193018602012001900187020120018d0188020120018c0189020120018b018a009fbdd971f95158f6169c21dbc053042b97ec19087d178ab97f607823ddc4082f1da45d6d43009d87c2e8627d14678d0e5ba710f6507a0f158fca6a56c8a9dd6a6f0005342dc7d89c023809e98c12b0d032009fbdf90da2f070a0816f5309d7f51e865b2b8bc1a8b94521eadba4adb55a4ed352dc2030770d39e6705712b1e58515969b3047c0d2508863c95f74e6bbd190903b0008ef5e054e2c7b381104ffd63e732a009fbe2304e99a3620702f32bab31e0dbb706af33b85975b5119942c05535196a0c0120c43c6a2c90ee10023f5e1fc9342c66b03db5efb0441ff76caaca2e8578f458004ee1b7a0e294f1c096413048d1105020120018f018e009fbe377b2f0cbbe6a2e49f8e1850bc171e8e0a13f41b0c8be0e1da3aedf5736381368fe430d41d92ef3326890b816b1b30936c2123b550f3c0fbd8b648e53a942fc0033a651a67844e5c06262005bba6b9009fbe059e6da590cedc4dff8547531e5deccc254bbde5db1ab8ea9393d42c7b302b4353e0f3e7b8587efff880fe417ca52453c73691e8598f6b343e74b99a6d22fa8003aef9cc67f03b5c0704307cd0f5ad02015801920191009fbe30b108804be895adea2496f496f8fa9056d6546bdb2aabb876f4e75393eedaa530c363306c66521654bedf3248da52ff4a68f08ad75d7c573c846d3453784080054c05c761f3cb9c0a16f707298001009fbe185f3706c3f28670dfd7b16fdc0ab844d06251393f4a8a3b5f7f8f2496c2d92c55570c324cd4991266bc9a80a1019a32153ff1b3388720810f698252172c788002a3665c52f8aa9c0506821808f0f9020120019701940201200196019500a0be7f32fe4108bc8797edd58279e0cb77444611a934f3ba01269e8d007fa34f9be4f7cd27ebb95f217146f3b2858bdf4f48a09a9c93364d87eb73e273d13c494fa001fcddaea7d3e36e03c94af45a36da00a0be7100314d88c5618f1943ea398d93c88314ad7270ab9011db8c576acaa791b48d1659cee4193aadbed5b9d12cea3a900d15b7eeac2563880d049e07c4076c29a00376fb333f0ee4ae069987ce42464002012001a10198020120019c0199020120019b019a009fbdf41e041bf26d40ffd290634d41f7a4e0898b35157c13997baeaef07da0825cbf03ad07eb5cf2f5ef2a1d3d0b323c4b65a8703f6433cc2b569e92d060af799e800b81b69dbdf9673815eb05fa992b5a009fbdfe2501dcfdf960152399bb1e04fdba69e10879c96ccb6ab2f6a6b0d6684a0466717c17ae45d451b3c95adaf0beff7b63e5181067174ea07d9cfd85268a3a05800ed07c51ea82b0381c37f6a201fcf202012001a0019d020158019f019e009fbd6108f66bd28b1d8898bf8147fe21f1e28a76f9a1c48e8bda23b638920442a965fbc24bc6aa388857ed46c967721d4b68aaebc4209ccf33da106905c11d5a7e001cffb1f6293340e0373cba000917e8009fbd500d70f436f3c07e1b1f43e8b53f2ace48499fc718a73e2d35a2be39f550a4f075d359df225210491f1718404cf8aebf2dec49ebec648c10639c8a25eaefcc002734a02146249ce04aaddc93fc5768009fbde5687d01c0f5432c631b335b36cbadba008e174d10ff51c0fd325d174445416e3f8f56e63cc9e4c46f399269d616daed4189b6f9372ad826c77244e5d7a7ca800ed07c51ea82b0381c37f6a201fcf202012001a701a202015801a401a3009fbd80549bdc039a824b5f12eed4bfdc56a9a5f8a2ddfe0d91dfbe09c50f2e258ba4a8a27a25414e1f8d6e1d33f355b7054a0443796f2f2f3eab60a05ef36996ac000b30d61fd28c13701550f7dca7000402016601a601a5009fbcef1484ac0d25e615de5207fa117fd82ca4a5b8a3a2c528d7734c7aea69778b90107f25d0a90d9db3930bfb9d8617f752c8493893b63d32e5a841e379c0b7e800ed07c51ea82b0381c37f6a201fcf20009fbcfb681292338e86e87838a7cb852a90dd7d969c193ead3e7fc1046c637490b3735b18d9fe3c6b1d4f65160a0e79f6e45c77b3be3defbca3c6be11831d15f6b000d02adc0c1351ab818c84f23e884c20009fbe17a54176df91c34af7a52d8b955124df6ea6dba12ccfce313987f34bae94ed7aaf5a52bfbbd87755eaab85600710e8b664bb32a836248e188a86c6b28f4edd4002a624e4a8531c1c050bbc48e297d90175a15d1cc6645d1c000100004acfe7fd3a11c51675e07831a0e6a07df357ea3989bac2b5325e8c8267f0f1c3801c7c048de14350c6377cc47ff4c84001a9020120027301aa020120021201ab02012001db01ac02012001c801ad02012001b901ae02012001b601af02012001b501b002012001b401b102012001b301b2009fbde12b1f0be98fbe5feb431496d6a96621ad48295fdacf4c5ab87abedd389296cb43ab7a1756cd06faa79ecfee7388ea0577c1c64cf2b9b319b1ecd1a64d0a89000e2740dcedce3b38193265c0a1c24a009fbdfe4579c73300e82bb428d00ea601847b2b417a63275fd85e43f8dea7b1a045a0264dcd4f16f5b8d603a9de3af438d30e8a88e955ed443db8ac819b0f20d924000fdc17ced3d443b81c3c17e61e6c72009fbe3967331749226ebd24439956f28a445b07a7d10a22ee62e9dfec65b14edadc389f2e53aca9fc68e36c04b218ac2a2e1be7f7f2f17340428ecc234b5bcb283a000361ad85a0355a9c060525a10bbd6100a0be7a8713a9aa4027cccf5062fe352fcb0be50cad8253401ff4d43e15cde58856ee2b8a078a3dc1d2eecfc118c7a5458d807bba5a4f86cb3dd99cf8583374b220600163a1c5f477366e02791fa563f94402016a01b801b7009fbdfe3541e8f7df91857b3f3b0bcf967839525e521f1c6a99b5075e0306253ff84bd6cbf373482791e3b0b57924b5922537ccf1a5e16e42634882f7694ac6e1fa800905249e77614338100eea82628002009fbdd9ce07c6f6689982a607b4d01e1793079854cab425fc4980de7f13835457f3749d964bbbce79c4fc77758f16150c448835e85e847f4df85fa5117f9d0cbf15000fdc17ced3d443b81c3c17e61e6c7202012001c301ba02012001bc01bb00a0be6185cc4dbb9e50d6188397e421d6b8df199500d00ed54ea6fdb874cbb6a959b2739db501bed7573cde7fe9b09afd77ce9b4d6921f52d9ee5c9a17528d14990600168da02dbdfc58e02826a854d49f802012001c001bd02015801bf01be009fbdb02ffec1a6c94de25b5119ba0ced8a7acbdd4e082cdb7d887188e9a4dacb1d6f122ccf139e066a96dabf202e7d9a88649ba49723da6c2894dd0ffb0d623b01001fb82f9da7a8877038782fcc3cd8e4009fbd8947a50b1a470e6bf03f002aa4e95d392767de7bc196fa7659e66a2e2931c862c88b50e1a0321833c6aeea3c6914f4c84029bbcc3c32d0fb3d7cc40a3ae4e6000d538c9f504ffc7017b98163ae487402016201c201c1009fbd502096933349c96290aae59f0832c76edd7afc4eaba1e0d7cf23d6e1e1c8550f2c0f56a8d8f2d7ccc57be5dc75742c0dd29785c1ffdab7d3ec63728994d2ee00164576a290048ee027a6304ac340c8009fbd799700c2aa4c0763146e468b48c830186a9a8d428784435a92fb8e6024fa049ac91a828a01824dc06a197ebced481cc223942419ceca247a4efcff05a7cbe80016b391ff9d0558e0286a35a1025d8802012001c501c400a0be7c378ef992647ebb7156a1a064efac590a89566ae1a9890468efbd027afd3580f039e3730d4782f68c99536cf24e013f61ddc880638700d3969803766bcff420015257513c51b04e025a57532d33b402015801c701c6009fbdfaaeb262192a4e3c29090d1440421a5478932bdc0e36077ff4de42404c41bc24b65d41ea79b3b10ae65c218c146e576b45737eeb9dba3b9822e161d9be277e8005e7f116f74a13b80a83ad22fe2702009fbdca5cca0c6f96fbed0ad2057e7db8190bc3954701f50c9014dd57074ce893d4b925ca9b4b5dcd9ebb6dd00b23cc6d41b09dd4620e53cfce18ea2077a0a44ba58005de980c2a2d6cb80a7308db5a584202012001d201c902012001cd01ca02012001cc01cb00a0be662430cb4cd0ff729cebe2c5ae74c8a6d153dab42c8b0050e922980265a9cf4a37cb99f113ac7686ee35f436d06b4232c6f90dfe93f14371746b087fdd71e82002fa6bfe220d0a8e054d529ab0e40600a0be67c8aa7a5d72113d40e4cb22b2e89bdfaca97438dbe2d1e349642d1de17433752362571099e9a0710acc22f993c42e416de32e6f65a180577f2839edb977ee0003f705f3b4f510ee070f05f9879b1c02012001cf01ce00a0be57a6c1491f6019cd77edabb0a0dc795d7a4c99e6ce16132c693399ea631d105857bb001d5e34361f193e874fa8eda0a1fc09566e75bf3502daa2cf13c6c90e4001acf0c23312000e02fba217d23c8e02012001d101d0009fbe199e3b8013479f72c0ae65e12642657f54387f5eb6381a82304f72359715c6e1d99b8aea59ff46498085de28e0da493d35e7808a5e530a0100e2ee2f953dca400396fff1c82092dc0664132c453e7d009fbe0fd367d019d7b3e14db1f2be7c91d193dde40cbadc4c85c9f2a9276a160a5cb594263303f06f546c56cc3d655d753145d5f791d1d4cd54534014ef5a0b9c504006017410d2255c1c0ab11816907ec102012001da01d302012001d701d402037e6601d601d5009fbc8dc77b3d816f3f058e08831596c20b564543a54cd15033eedc4686e0da6bfa3b10b641d78320ae3dda2b9d750395e1577aed140c068723a2e584d619059fa001fb82f9da7a8877038782fcc3cd8e40009fbcbd0590e5116cb73977e0e6922ae969dae1ba3d027bddbe3b819dfb5ecfc300cbcb3250c24a12035c843cf75ed058ed8122786053a2845510ae8f29ffcafa5001fb82f9da7a8877038782fcc3cd8e4002015801d901d8009fbddcdcdff7f94ec8647d880f399881998569f43ba2e074a741949079c0215689d78a5a6e134e49f9c3fae3ce730306fbdd20f2175384763768d1b44b0be559aa800fdc17ced3d443b81c3c17e61e6c72009fbde23a168094d423c032401e849909f18310be162a7b110cb15dd16d3b6bd22d5baf88a22c0b3a861d58a73205e200baff05378d22cff6cd44d0a64131bb6c03000f7d68312beeb9b81b9386a63f3d7200a1bea8008e7b0b46eadcb4144a890e05dcfdcac8a15ae0e0558121f685293bd06c39f1d53b402862bdbc2a81cda7b742f0d5ce7c538c6ec886f070abb386303eb9a001fb82f9da7a8877038782fcc3cd8e4002012001f301dc02012001ee01dd02012001ed01de02012001e801df02012001e501e002012001e201e1009fbdda962f98daa636e54b9e2c9f1dd45cd61d47667e67a752bfecce79e52c846d44423880628c2a4bd2c806f8951eaaa0950ed2e1d7f46b88ff89a6dd61f99e7e800b1a6ffec326153813c4540add383202012001e401e3009fbd8740f2e96ba18ff4eb01db853bd97ac5409075756e5f70e9d14240a156b9d6f1dc3270bd5115d298792908e67778e2b67663e33b5da8d1b8c6541887158f6c000e467f32f35a4670196a05032ac514009fbdab1193835823bd19f9915e51a34b3606a03939727b40eafde64480961e499a09bfb49c21631786801dca20240f4c3744afe9a25a4738bceaef4fe6d7db69b1001fb82f9da7a8877038782fcc3cd8e402037a6001e701e6009fbcf5eb042c51db38df2a89b1033c8bd52937597ddeae2d9c4875dd8d41b20635dedb36c5166d83f15602531c9cab3aeed5dd541c10dfb0be519aef13fe0acb980065481e8aa449cb80b44f4133771aa0009fbccfe827a7393f994e195c8dd76da305d0e6c9cb2af57833bcf680808c1256ca47c24934d4bba0ed9bd94cb3796d74b0d5397ef99b4c2c2ebb9e1bcd6cbe0bf800aa5f05e5b37a9b812f4edb3e33c82002012001ec01e902012001eb01ea009fbdcda3d74dd925a7261ee777ccd0fe85420b1ada289bbcb2e80974e2f9da7c43e789177248e4d28c9eddc6e2ac7bf5d57e00e4de52bc9b8730d8ac4f724d3d34800fdc17ced3d443b81c3c17e61e6c72009fbdc10363d44b292eb663824e56742548ee9770ebfb3d7140e1ec045adc86317000baa5d650b8d91fe533c5d292c1e6fc315b17fcad96dae13169baac30fa58658007f5c3927efecf380e2bc9301457ca009fbe1a1a37e082ff86cbaee6d73c9c26bc003248c0fe6c0e0de8253b78aa846e5cf83148decdb83a3f08f0ee406dc1193809f6eddebdb5efb638298ae66e26ad9a400548c6e303bd4adc09685193b35f5900a1be99ccdaf196d49187cff49380bf19ea708eb4a23df5b530afccf7f8017ffc0582c174fca8c9d8a1f6a3f6023ea881770c9e4d831ed2ced3ccfc62a2638c7ce5e001fb82f9da7a8877038782fcc3cd8e4002012001f001ef00a1bebda62539b286bc705220a6a6d481434b38ff32d064d09cc08f215abdcf7168135f96f66663ea4d4831e43360027b61831970cb7fb6d0516dd88065de4ca9b4e000bff4c6ad3141e70155bc3b5631bd4002012001f201f100a0be420ae4e617d38384eda3b58f01949df80f61f1dfa9730d91cac928acafc088170c08b2aab8dab95012f51d93bceb184a00311510ba15d968837b66175b5cada00193b5575325ef6e02ceb6891f7e7800a0be5271bc5f2ad155052baa3c189f319fe27ae2be72682d9b5d9bd755656adbf0752b92e263949c4f4c1b03196b08b1da0e6fe3185f276d9a70644516cf767c01e00204ac9cade8100e0397d2deff5c6c020120020301f402012001fe01f502012001f701f600a0be5e7620e36b2302f5f718ce6600ca060672385b0d133a4b7720dd1f1912fdf13d05fa8764c36a043d9e985931b43d79abb0c6d8eae520b0397888b03d19d3c98003de58db3d5c11ae06e317ca32998c02012001fb01f802012001fa01f9009fbdefbff09fb1abba9cebbbd77f4827013c9e7130d6c9c37e5e8726b399d1466bf1f10ad4f3aee1fc3cb32ff82334d8379b0ade03481a2190cac353fe5377af13800faae3a3ce4f1fb81be47f3885fd72009fbdde056526b1bf829be877593a339b35d71a1f4c6102feaf0011dbaab7741a3f99fa0c77b57f997a8016b3576e227223cbf707aad97a00dd7f2c6c822e74dff1000fdc17ced3d443b81c3c17e61e6c7202012001fd01fc009fbddfbd93a9d1767fadd14dc1201169d1c720e5728af15f0de9bf23272e203be7eaff7a1b9f05cf8290c6697484ee3f709fd17ff8ed8180af34a0736f92b488068005dc6f4279062ab80a6f30bdabde5a009fbdcd54bf5753315b0d20d145860d8864d5f5e8503fddc1d7bdf98419e2003880e63cf4006354961e000342c0d3cd17841f1fe800e68d1f1ccf2bccfa12d8f2688009b994fd61b69138115025c2b84fc2020120020201ff02014802010200009fbddd13b01cb8908c648434c401481d4151b3ca8b67730f4e07052f8093f98a8bc4f1ff49a1f765f78a4ea1b93b670759a686adc9e1592904728bb82eb0d95586000fdc17ced3d443b81c3c17e61e6c72009fbde8249b3da9e781902334dcf8da8231fe4f3aecd540236c24470bdf2f9b7a8ac1c7ff86e02672f72ffdebdfdd808307aeac461b6fe406ca596b325bebfc90670008bb1a184f4b30380f8b1a30f9720200a0be62554621255e4d2b6bb8cfa96b63f3b729eefddaf0ce6de1a1ae5bf79540868008b32f454812c445c201e61cb41e479fe0aea471748de6d241b6950336efcc80021d32a09545b16e03c37b7b6d789c020120020f0204020120020802050205825a7002070206009dbbeef82c10ff381df199fa0147eaea6fcf0a6f1b89b71f8e091daa821eea7930e6f4815d675dfdd21aed9759bf177d2a251f0b8d8f8dcda9c6b282dcfebda780061c58b4aa924f380ae0f8a8e8ad32009dbbcd509acd0419abf0beebf19c38ac82640a86ce10dd8a754bd2519e1d4565a0ef6fa7fc4c8bd4bcdaef933bfd551eb729fab6f67a58db5b8850c71c01a8800007117f10d8e3ae380c9567f94b971a020120020a0209009fbe0fb23b99b3eda6758b98dc1c0f3213d99285b497ff9610f4292c6aefd6ff8ad1e192af203035b3eae3d818d7d05e291612d2a6dc6054403430953581e17d13c007ee0be769ea21dc0e1e0bf30f3639020120020c020b009fbdf29f029ec3b7c093b547d01b9b0f33a47e73f41816b5359411d7600c64499f638f87b66f614a50057922f6e1513b6d62f53ec684e642c10737d1e7e724af8d000617655640751cb80ad828642a85ba02037ba0020e020d009fbc98d44c8069b1461f19f1b42952922909ed79caa06b93dad317d22d879f3022cec59f3e3c4b40f4bf17bf59c44d03c527e0c346091506a7a88f7884932e52b001fb82f9da7a8877038782fcc3cd8e40009fbcb630ba5d51da06b33b1cc2c01bd5d252c62d971999e18cb345a6642488ce84c5f3a7d6b09c7466c3dd6b8cd5ff7d4c9a21a02e4d6770b6f3a7c262385a36e000b5d87ecab0f5270143bc5f7bfc4a4002016602110210009fbde3604b78fef71df0394ba56538b1c7403279687dd54cc6cc7149a2d4b2e336cc9c96f50e281aff6ed8dd9edcf3889c6c14347ba30636d1ee513782903cc75e8007bbac2e892e94b80dc45dfb56815a009fbdc8317e978d30e0cbd8602c05caba0f95b55f9d600b521c45952bbebec2413bf9b944fbd0c371b0d783b693ad7b49609f993583e1e3b9ddff6fcfc179e22aea8005ba9683dac212b80a32ef16af4a6202012002500213020120023b0214020120022c021502012002230216020120021c0217020120021b0218020120021a0219009fbdec941d1d580bcfd853ffc72a161d9015b8938079c2d06dfcdd6e1ca947d373f6553860e5d7f1ed6beaa254aead37f918965736c314975b0750088554d51bd3800fdc17ced3d443b81c3c17e61e6c72009fbddf9cafd59d7ad0deb8ba654ccb570b976705a4083ffebcd621c126e40de63250853f2fa20af6a4c966659969c07e5ee0ad575e524d0e816cb5588110dda4c4000a8c8e2ffcc29db812c7bd229cf172009fbe14df8613b7eca32b153ffac982bd08d04d876377f487858bab5acafc1a660450967375886e12c79a12089560e60b508a20f2e198390b61b9959de7771c5f234005b1f8790aed3fdc0a2397b87bde790201580222021d0201480221021e0201200220021f009fbd03694272828e6043f96ce89f0a75f6397580028bc0b63cf2d1ae54e00ae92c47613dd403052526f9211a531c0a6d1b352766140502cac8fa8998d7b24bd4dc003f19fd1f499965c0705696567ffc50009fbd2d049ea902fedc70a88a9f6fe762e3ebf5e0add12acab87439f9641e6d89e3b9b90cf4d59f8d8fd2950d7a7085aad63a388cd006dc33a087bd18799b716d18003e506aaf66f8d1c06eefbbaa703a90009fbd553d1cc6ae44aa9243c99153b0c086dc21c1e91f466cfa54c5e37ff5a6c0049db000f5327017ba7a5ac0430f36fefacd7ad86588e0cedf3a52a915e4cb06520030217769d74e20e055afa209d77548009fbddfb601a16bb991238555925715f58d087b74e90454d1bd616ceab512a6cf578c52f956f5abe9fc4510ce544c7b1c2193285f0755f1b5b5db91e00b66aa60ed8006d6125194ac1c380c2b9d0edfc2620201200229022402012002260225009fbe2aa7b708795752e0fec77d6b8768d4051f6481615c1a4e2364802983fdc55171b095e1977684bd195d9d38c7cc352894a7a713b5223ca92f3fcbd4cdae2be08004c4fd03bf5a95dc087db2caf6465d02012002280227009fbdc39f0d099efc77ea828848f59587d4f07208aa556da467cf32c2b07081238927311fa7468978e4e263d6b9e7f852c0d2dd2c755c44ae743161a05e34a5906a0006bb7156eea352380bfc34f1124c0a009fbdd6586b13978944decb05fc7757fa49c096cda19585f61f1e9ab7dfefe7dbd8daafc22b016dcfb6bc613aba186f284ca0ab32c21875bb18c80d5dd99af5a574000f00896f60a3fbb81ab538f132999a020120022b022a009fbe0bfc4096bccbf07affeef21c05fea3257098750a24fe7af9b63a12cbc3f3d2f50b9def1fdca320b7db41b1c879381ee241979c42ffa4a03e5a42af6da8c8c08005c43599f76e361c0a44101109ecc5009fbe29d74a0c178135b86d1d0e0c8f717661087c24fd3c8486d83bd06a9700dcb12e72d25645e3bc23420bacc78c67aa30a1e8276383abf437b04d8c2bffae6bbdc007ee0be769ea21dc0e1e0bf30f36390201200230022d020120022f022e00a0be63e20035c05f8ab63154632d30736d925dc81b7b6eb6d6e817f7142e2608edf97fb1a30928e7544cd00d621886107e143ccee6318d280f00ded05132803a60e003f705f3b4f510ee070f05f9879b1c00a0be5b5576c8feab2eed9837968361aa7da5010ed1fdd41c4ba86bdcbf2eea312702254760f41b4f365eec680b9cf3b4e618049bd62d5ae51a9dbe1bf9650347ba0001707e2a6491f9ee02900504dbb14c020120023602310201480235023202012002340233009fbd85fa25fb69bd770d7be15bdb98e1c7658470e76df40d9ab7c72290166e3cf675f4cd98872670e023d22447a5e2e6bdc94fb0c587ca50db0117f3a87bcd2b170011dc01e3216499701fcb7171796d44009fbd9c9bc9d9fd020fa49e0fd6d83e5363e5b16c725e9c28e9ca05ac9b13ffd64819dfa4c41b80296f4ef6d7151cdfedb3d4a412e0259ba39cefcdf601f21f67a2000f895af0f7f9d7701ba8cc129961a4009fbdeeb8040d7e9e09700227b1b50c7780e652ac9e4b932bd3240fa9bb523438fbbc0fc2a523b0ca0b720d549151b6f04019b7c8bbc26b4eb72f4a556b3827707e000f8dda933be01db81bb0ce4d018f8202012002380237009fbe24c5a72cdee5363a51ed9f18d33ae622596621b400e1aee6e31e9a5c53a90b252dc61bdf330fc7be8b205ffa83654304c85b8e68aab14bc85229d23ff184d30002d594b362e85c5c050bbc48e297d9020120023a0239009fbdfd649da760227b218dae55ec94094217aeec134eb2db9ae8cc4b3a6d543a92fdda82609faf9060858302634727a5e9d63e66f1ab899ef1a25b4602ef26eadc0007412df661a720b80cea4ba4005f0a009fbdf14c0f02b08bcfb2a0bbcc441f9382a01042191a9df0219a11d34ea788bec8f1f628f3002d74ce80416fae1eac3bb541dad40145f08413969930fbf679d0e08007c3f34da85084b80dd31a9b595d1a0201200241023c020120023e023d00a1be9b7a8b6eb7963a38c3951a41fb5c57a4198b0c93d9919b13191b1ffdafb0f1bd2d1cd5de6ebe53730896d0b6daa7cc7c1dfeab46cc8ced55eeabab678e26c02001a3efe42e676fd702eb9aca016000400201580240023f009fbe2389c2e541b9f0882eb43098d6b06ac994953a0d68d420b63c25d87dee94b5eab73efd8ece8d443eacfbf3ebc224303eace27fcfdb3620748892c3b96b523800038d0ebb1e32771c06525ffc5fc181009fbe059871f44fae74051fa2f51abe315bd95b988ef030e05a1e85d2295cd27fdf7c10bfe39fe0d298cf53f5b661487042c8c90eb80dc95d6eaf0e20a2e294e68f40046583a60450a4dc07d3ba5f2b96450201200247024202016602440243009fbde08c0eaff4d417e314795ee0172d275a7b13587543579023a6802a5eb7a8bf352d2a6d23d7249bf91ca267aa10b0566c3e7d07edb887c7d3b0f33120c679400007c2dd7b818373b80dd12c0229b56a02014802460245009fbd57df8faa5704d99abc73fe86d8301c1eb6ea1e3577ed1ddf8d789e5a5c9fcc7c161d099f62d204ed2c40134d60cd1c67367b955273f6696c6b6c3fa9e0a0000016d6bab960f2c2e028a8cd6bbadfc8009fbd4ffd4393e00ee10824439c210ce7330d153c228ac84df2948ac9315600f8b56d0c1e890e8e6a2d79032b0b70dcc43f143e98621352628d58ee8ee131543dce003d60c1be2934c8e06d451262bf1988020120024b0248020277024a0249009fbd4364aba76f6c2f8d5b5f11cef82f19fcc65664cd3102a3cd5e83d255364606a0674eb4432c24cbbc8263a599f3a2d02ae7e4c8b8e5cf825f1957e6cd08f4240018e79e94c69798e02c565fa81bf908009fbd6554b354ed805ea98ad20225f3ed5b9a8c10a30d50e6f829dfd23be16f0700a2863cf058daafe2d5f2035b26ebb36d570ad363a8fbcae9c60944e0bb9195fc0027604f59a1cb26e04619c07910bb28020158024d024c009fbdf2c3b634e0f920a1478a6c88904cbd775840b25e02dba2c86786afde183ebc895784d1c2b99e269c1fc0f8ad3fdbc07a15bd0fef8ed4ec648b523d4d6b2b0380073fd89f6c92ff380ce7ebf61067e2020274024f024e009fbcca8238aec2c7a773930183e3959321ed87f0ec160031e6b7ce482550e8b968772806dccc4894a7deb12a1c4554e5786f19e5263f2e9d71a85580619c9a8140007273f40eedd6a380cbc20e517a25a0009fbcf0e1e3938e0cdc599719637f454ad8caf2530ff0232e462101fafc288321890879f2aa5de5350031f854798a5d362de30e33b168ec917160a29be4a825a95800a9e3b2e0e98e9b812e734e0f52f92002012002640251020120025b0252020120025602530201200255025400a0be7d21e5c6e2bd092df501c72b88c6c12f0d774d04c02b35b57a1f90a32c69508ed22eaedda9b84e0f4bb0ad57281784a94dbb20e495f7f23709828a16b451a72001e430814076f58e035dfde3ec306a00a0be5c51fd391c64b956e7d3fa9bd94878fa97a8d857af7300e5b5606107031e998d2c3cd0cc17195bb020928659839fa161ada4bd80364c8268f36555a13d3a3ea002ccd6b538b4e58e04fc2c0929f1ca0201200258025700a0be4cf053f850b03c3a96c7810a75fd01523a80b7f585bfb7d8754ed6af0e590a83c7df5122c58b1032e0c12546202ea3230aa90ff0e5a1ec5489014d5769d5fa600172b35e2f33e48e0293f33c708fee020120025a0259009fbe2548910d72a1429a7adb40eb5b93801f6a8bea724600233e9a257cf250853d1084baf186a07754e091637b00c29ab74e41011ef068eefdc2ffe560c2e1aaeb4005561214fe5b0b9c097ffc35b5f801009fbe08b6dfd9995d16043cc73bddf0dc5e4974da9d1ea6299a71c2b8d3e183e1002d96b84b46b8c073fec864b098de7d3f474690f705c1444137fcf7f88fa791e9c004b564340b85e4dc0861ee7ac15e4d0201200261025c020148025e025d009fbe0510b96dd6ab1d1a1b9628df73cbd1bed989266617bdb154b473d887363ef687360cbd53b17c35fe24c82462c264e370ee5f27abf9fd03016f38b4716bb6000002f4edf69c560c5c05438b7b3c61a90205826bb00260025f009dbbbfbbda45f556f1a9b0ff2f7df627ec38253712397c8bf4bad34e4c5a9c87c9b4d063f26f143bd1eda8fe45935c605d1c7299aee900f0250ba692574819e1001527997ee156117025a945891ed744009dbbb595a96b5a5ca98b070d801c7243a7a8248ac918c75142f38bca43ea00dc71a9e213b6d0d5bcafa1b118955d5ecba06a26bae10a5372aef4e62ed1a442cd001ad9366fb33f3e702fcc3a4dbd27040201200263026200a0be6238d32bfd24517a1ab97b2209c92e4d8b94748463159669583a0134be2351713944a268421ef856fccad0d25f341721725430db5bc132d1831052394f7fab0002ab090a7f2d85ce04bffe1adafc0000a0be6d46039e4883e4cbe72afb60970e1423fec7289b3f2edfb39a571b4933ef72914080c8108e125fe77d087f12560fa4d835acd9178c0ec4870a6d59da6b77ac80016ab6ff1666764e0285bbafde3ffa0201200266026500a1bedfdd19a61bbe49ab8817c530ae677392238798f8fb52de6fa9cff9ab569d4a4b092da18f0b87ab023221245a2491979ecdb28689f6c8ceaab7dc6fe2b4383008007811e9bfe6c2ab80d5c2061095d220020120026a026702027602690268009fbdac5deb945f686544b7929f47487a5b4b1d8b28b287df5f7d3c40b8602d4e082d1b4a9fcf91fc429de3f04cbce7956f3df3eb5b4b18803fe6303a937d2fbb57000aeff8bef21879701378ba32ea6b84009fbdab2c631171abc7a169cc7d9f6e888ee22e05dd1aedcc73b12caf812d134724342d15965ee32dccdc6f6fc7a459b73c0392e6da9653c70534b1cd324282fede000c36bae7ac1a9e7015be72b7eda274020120026e026b02016a026d026c009fbd85d6806f3b152ff7ed35685bdf07586fc41a3defb4c381ddf7c209cc69dffb8349c23beea397d40d7b40aca470b0a53c4d560f3fdf83012a8701e48c24bada0013683dd9ad8b3870228cd9a26e07a4009fbdb97df990cff50f130baeef5434d2e6b9a1f854a47f30a108519dc5a130cf0b662651a19154d4bcde1853a3b472b35574a8b859e21cf18fa46598d061de7520001fb82f9da7a8877038782fcc3cd8e40201200272026f02014802710270009fbd8647f614666fabe6ed0b45cce5b625b01612717b7c32bd9042efc6af7adc65b766373a0f531bb2141d09be02020de58c1bc787c003675879848b3c9aebaab90018b196349afeda702bf62e2f852144009fbd8e661c3fb13e5ac5de65d4b9e213ef292ef89da2c249c763fdd98193ba9f492457c450ec7ac85bb4d1b58ea0047b71a2aac4a09498184a03c134c4f99699df00151070239e7dd670258009a24edd74009fbe16f291de16e27dd0c7f07a87bf976861aa59d4a938b9bcc69f588a3db52979b8e04b4ed77f33554a5e41cfd1d74681b92479226cc9384eaf9998fc2211bd028007ee0be769ea21dc0e1e0bf30f363902012002ef027402012002aa02750201200295027602012002840277020120027d0278020158027a0279009fbe37d9839dab17d48bba358ad434894c93a6df623ee624b85904a4becd01fe5bbb02aa005b67df4b7f032eb8ac5f89b5c7a009ac439ad4be52dac8efc8a50f104005688243c6d8761c09a0cf723ef12d020120027c027b009fbde9e1135808ee6a4e0cf9166bbdc121c8503c7c5db6eecd403d9f15ec474ed6587d5f331966bcae6dc77ced85f1df7cbb34567cf70a8fc075ae9805df57caa5000ed7d22bc5ef0a381a6cbca30110d2009fbdefa73fcefcd375eec63d33662985e0b452038009469b99faf53efc6728f7eb11f37232cec7b2ee78fadb834ee1e9bad6d169b0ce4c0ba3a8264ff60810b5108005a6c5bd2b2a87380a0fa8235447da020120027f027e00a0be4e5734ad8273ba00d751f692e07ee8e8d37b12c97cf5255965a7fd10bf8109278f91017f9aac8ed367c02b6b65cbbc4c39aafe35cf67489967e104a5cb53c40003f705f3b4f510ee070f05f9879b1c02012002810280009fbe2bdb86b892b7ff3627e8516b718042bffd50c3f6f664224fc170146cf8c2a2cfb0319b20159e15ded2312c51bb96f7f8c7670e707771da0f25c4d3bb3e9da1c00356d739b9b3839c05f1da9c5673b102012002830282009fbdfe084586145f280fd3171f64430cd6d7aa2a67766febf611c65b5d66e068682cd8ebc9c6fec2c0132cea4a2e9652a7bc8bc5ba473f3f41b90af7055cbf48c3000fdc17ced3d443b81c3c17e61e6c72009fbdd88c0e4ff46801097f2c6fe5a1dfd4d6a7fc729724f6f4199f85530bf38ccdd925a68ae1dbde01a072e622b3d1cc415f9a8edbbc2540f24cd57338904090b60006c82344809d25b80c12ceb75ffaaa020120028c02850201480289028602027502880287009fbd2ebe6185c901e85403eace2ed92d3e0117308a8adeffbe76eeb2d7987bd00c7e765fdef0372914a47daeac9f07057d4933b03b77c107e5c115b893569a7a08004818b97d54b745c0805a18be5ede50009fbd234b516188fc6cef66242fe9c6b2a01a32af0bf27b01e4209dbd06978881765909aa808d8eca6bb798e2739c46a3b318b3b7ca6c34898d57ffaffa6483f1ec00569ae5764acfa9c09a2e5aa8096190020120028b028a009fbdf6dcc1f06e8ffab02c924a76c7f60aa675a27e23411d355133d60b80536e4c82f47ef5a12813e521e1e192243ec8782591a44e5747a42569f712ec1895ae2c800fdc17ced3d443b81c3c17e61e6c72009fbde8cf0ab68bb41fea77e84a1fae4b668d0ee2c07b2a747b4404339c919642aaafdb39a3fd0670fa5a5d6fcdc526ca74617017b04c49fbbc659a8dc9782b768c800fdc17ced3d443b81c3c17e61e6c720201200290028d020148028f028e009fbdd13e1ad354ca61fc0be46e1e7468e3df301aa035f4e0f269db0a547be973b212177645229033df246da23353e28213ee765d66d4069eb6f1503ffac7f495818005de6c1c672191380a72baa3365a5a009fbdef26d7b1a0c3db4123f4884a2bb1b832d0706050287a428faa8ce4a1d534d1880689e578291c99fc4dde038f96ae2d062f4669027c2d7d1fadd651f5f9715700058a8e95593102b809dd6cea283aa202012002920291009fbe3193d07a8bc14b17986a810ca928c253b8ec9bf33fc34c9fc78eb777fec505aad733eb56cdecf55137945e971a64f3c38c144340b4e595e1f53844b18cdf5bc0036c417b5706df1c0617fa8cbebf2102037e5e02940293009ebc486cef1c5638c91eb4762f30b8411b5d4d9b00263e88d5dd1b90368bd9bb9536bdd9b5a356e752522140c2fad6c3908c94d4219dae077259b24fa3c6349120017757a7d2b8a9ee029c36b7dba718009ebc634b8d3316dfa485c928562b1559681270acc0912f3c252aa07340077ed94addffbdb15c1ff9191fd9ab4be4092df8e9c6b33801a8a7a90fcfe76284861cc0017f277df9c169ce02aa1efb94e00002012002a30296020120029c02970201200299029800a0be7640def3b9f798f525982fff0bbb0c7e8a10d75a9ea430eceedc61f7d7b4160f842f8f60b1c272df5a42a606adbf268b2757ebd69bc2b50ba18829ac3437ba8001f2a154cdb3b90e0377b344a723d2020120029b029a009fbe25d9251c7c471930a6357e1ac77327537a807ceb0cbcbc7087a2c7b00be7e0d407835d76c1f6407af77af061f1720e83ffdb28f6e3ea6c3e9f168e3fef8bd1000595f6f7be2d065c09f1bc0292c361009fbe2ed0f06a4023e3f1de5e510cf1b3ec59c2d72bf31184ca259faae138caae3d6cb493923aff837aa53725709ff90f0d8198b8d99aa0e2822ba45eb84acb49cb800466b79da145f35c07d5dea394ecc902012002a0029d020158029f029e009fbdf8f978c17c3a49e2e65a9d2ed2fb9a590460704ebcf32cf703a3db2612c97e631d5d26830f1654282a962334cf0c4dd434a6af524b2b70b88483db9ddec2c2000fad572c291ac5b81be8dc6738403a009fbddce9fce901fb2b5193ef8f8a85165d5005119fa3c13e150b214169d10a2abc75001c30e2f4f0a80d8573184d99eeef000c7a993f25d982bf6bf638612c55b7000c511aa1b9e3bd3815ed66aae97aaa02014802a202a1009fbdd13411961f254278ff5aef0c4f7d08ae57367c3aeef7b10742084deaaecddc6a299b775b43fe25be5ee6b798169849960fd845972499deb7a98f252d38b94f00055b24b668c0c6380989042204c692009fbdf1d3db12d4cace42e7c85fa4dd740fe34569fd54b5c2e10f9191369f1204dc2ad1c01e79191e2f47b892cc87f14deceb197a06c44fd003906e6e2aae0105960006b776cdd9eaf5b80bf51faa65beb202012002a902a402016a02a802a502012002a702a6009fbd96bd297470a6baf11dcd00d26b2403449d1e91d8e646030a5ebf69d050b583b645425e6334f19dc5eb681ea8815bb570ba43c22934e0ef2bd6c07637e8686b001fb82f9da7a8877038782fcc3cd8e4009fbda993651e210acaffe3793be7341f006d76c5aa0d4601c13d5b47acf8455d39515f995c5356874d74c924f7f0b6cc3930838f015c05494d470eedba2b2bd518000ca18b7f67b80170167c9bb07537e4009fbdce0c121df14dd6a9409ac147efa627b25f7f9e7212df4afd5517a84428669cd0718b84a0daf7dc629a5ed9191e33985752254333cb8116b15615fed2545ab40006a58f6f8509ac380bd53fff1d880a00a1bea6cd7a6c35aa235629be058da0363a5a21c3813626461772c24e4858d50ec8b69f7153c87df69cdd2e4f637fba4c0e7a8f3ff6faafd2fe0e43585bc7d4a95d6000ba73f3d7de4847014bf0226de9d34002012002d202ab02012002bf02ac02012002b602ad02015802b302ae02012002b202af02012002b102b0009fbda8b2aa235e4cded3ccf941c9f9bf200502da5d1307f093c82f559542e613c60166d0181a19f87af9397040a68671e1b239f12152824f7d987fd6897d6a9587001dfe1ed82f46407035652ff2db92f4009fbdb0334e477b5f019f8fafa5cde0cacdb12499fdf837a465e1c9f9b49414340446332617167b1af80437ffe335c67bfab5faab55c71468545d2bbc7ad9c27de4000bf46d542785607015486909af03c4009fbdf904aae43e9a94f035907da520426f34c1b89a9f35da21c3dada56d844ab27e575e5e1484e911066886633bd40ad8d8bed0675703ddfa8bf393c46d263bd6f00071485e8bb511e380c9acb68fd7e5202012002b502b4009fbdfdb5244bea368e1caa1ab9870b39b95bc0e23798c23ddb896bd074fc4a0c6c1684744f07e2762dbcb0281b9927f0fc746f91c42ee72628c62f047e3ce690de80069b22aa01916fb80bc2b0d785180a009fbde7bb150269a0d2cada5dcf4116efb01fafee7be1668a680245d10c76ca61496edfc6e2845b5a161cc9a7e0d5bbccd986f1753d260e3c35788db78a7552b07e0005dff72f388127b80a7579fa670c4a02012002b802b700a0be71235fe79f6a738a2e8af141cd0f6753bb85e3654dbf473d734dc28df48b1908cac299d68e4f6ac422ebb460f46174a7aca574bd153fd560b0c0b8fc6d4454e00223d7d9e1cfaa2e03cf502213b72802012002ba02b9009fbe198e102ae2be8c7ab3f2ecbc94b9a456f1e2910546a4d9a51ebcc9dbd5adda19ee62b62581c53c00240f18937135814ad005932223504f4d4e35687d7a4e8b4006f40c0d8f5466dc0c60fa6fcf7d4902012002be02bb02014802bd02bc009fbd6780ab0c58f24f02c6a0b15927eb1ffc97de3c7fb1dedfe3c21e189df26e1b00e635a05b2bf8eb1359692ca586a78a2c2ef423f3c3992787de3eb6c6cb0326001658f935dc2ce2e027c8ec060bbbc8009fbd64d3bdf015aa543372717af0f8c7274c0c0007874755c766f15c3e59148c50e0d9c715e69e8975c39e9a7138cf19d77275046c33ab63d0abd746749bf58d6400160454881e829ae027323bc1a59588009fbdc7d8474e42030e20e558f1a2f9602838afc5379e95978b9bb978545b77748009db5127284ace3311d76530d8d06a17a9f5e5cbc5e6915743012a835d2129e880098e3485986cbfb81102ecb91ef3d202012002cb02c002012002c802c102012002c502c202027202c402c3009fbd22bddd1cdc07ce52476df0cf4e835585901973cd637c3a672ccabe359c1d580c107b3a6200187639a0fd18b5b72a3b8c1a19829fbff0804001a8259de624a8007ee0be769ea21dc0e1e0bf30f36390009fbd2678606b5227cd1c121a7fdedc1d639ddedebf35217ae43a3ff724701e3be60a31adb8d37929964a1dba24291f529f4d5366d9546099bebd6298bad375c33c00694d065a81c3b9c0bb76fe08a3691002012002c702c6009fbdf01b3fa7bab62727e348e2088d098b6c70eccdc57afad83d74d6e5d74bcd43ddb254e1c89fc2e6fa2e8de949331ce3ebd054375c87ac52b8dffb27995575e30005508104dd66ec3809761333a9aca2009fbdfd013268cb38f97708b2e6a8090ac58806581087d71028d69324d74baff42b0be1144bc6c4b23c92aec18b04f3f68d3406708423945b6df7c9674743fe198c8005a154f3575037b80a05f89678eef202015802ca02c9009fbdcb1c5d3707ccf7f6133a62cae79fb97e013d0a4f6d2e3be4e2053904441d9dfe54172d9f3a667ab25487d65de15e1702b4a4a4ef1e5fcb1a42bc230209a176000fdc17ced3d443b81c3c17e61e6c72009fbde65e7fdf492162036ae431c9f26ed23e0cc4ce9b8878f79f04b46ef9ceb3b914cdd62ed0b2751bc57f8fb9df10737b06078a5578ac89ec464f9db2d4c30239800c848aa9391c80b81648f96ae6594a02012002cf02cc02012002ce02cd009fbe1ac813e889b4a1e6bf1c1f19c84f758400ee7c649448928cfbfaacf38b36b87223b9b8ce8dae6c3a1eb0bf65590addf8e23cf2d89beedc19fb26b9e9ae6494c002f138e10e0e435c053cf1d90e2ff9009fbe1aedbdc13b0ae4eb8b4f0073eccef8a4911ac98148f294338af80cfdbfd13ef57ddfb03679e3976cab209997b766d2b56f0acce76095f66717b9b0ed75f6b240030001d2957e7c5c0557441940655d02015802d102d0009fbdc28ec7f47c560f6ee2cf163d9ce67d6a83291023492f58b01914122c5b83683203e8541a1219faf3d68ee445894dd5a5358a404fc8fb021da70f95ed121a808007ccf32971b99d380de3201d70f25a009fbdc4c2c12da18425f1f95f2bd6263ebf9219c6b4c131cdab278299195b7a5d4035b42edd4ba2cf3f75ddac8cbdfa704057d262af300cd0f044041ac8d2e5fd19800f832734003812381b9dc1752b5fea02012002e602d302012002d502d400a1be9431e75b76073b9fc15ee51d120012e4b627d85dff3fb8d1c183de26d07880d54c012689b9c8f5e8a7851cd4824d3c70318f4c17c0d3d5689c184d9ddeb011e0011ad1f8a981e49701f77f9ef02b574002012002d702d600a0be66c965a6a90e403bf60a0ac128e41eda142514901c7253aee669083062b03df66942ca0f4d1a317079085ea7da77bc93c1c361322f40463775a0330f5b18e52003e260a9a166f48e06ea44b106e51002012002d902d8009fbe1428bc327b8d4fe43ccdfd5cdb82311a81813934be633656bdd035d6ba68c81ef88cf1256dce2f859b77754383295af12050b30bffc6a2e1a3e74d82640e470007ee0be769ea21dc0e1e0bf30f363902012002df02da02012002dc02db009fbdab2d10abdcceb4084ce414087bda1257c4906a1f89727a82c36f4855df34b49971e28e9f7d5000984b137a33bdeed9a41fef15efe63cc1547cd39940654103000b47ea4ebb02787014154a97ac94f402016602de02dd009fbcc8be74db6461ce824ac78b983ee68c1a8278568f31ecf2db964ee7251fd97e4b0405b5e6089df784855a6b5bc782eb92a614bd0d698ceb4879f9e21bd630c0005dc7e4087a94c380a6f4b6e7ab1ba0009fbcf59765ef8bc5f9d47fac3d3d0da54c59e1fc74ad05f325a1563e9ee342f09f7b6a9d95bad33476cd47aa39735eefbb21e84ae9a9ce825d653b5f17ee120770006b1ac65fe72a6380bead0925ad412002012002e302e002014802e202e1009fbd219ab9816251f87bdeb603c01c0d0aaab9a172d4932c5c3e6d20ae2bd79a9847c36d9f997c02e1d73c3684aa444e2bbbe72c1ea6f23531a3c201a100c6ccb00073e8442ab4e83dc0ce58e058e74dd0009fbd3fcc55558ba67cdcb84f0a3f3f3d214a6b23fc04a171bc8e8a176e2d1b4bd3dce1a40f9a1e737f448135392ef3ea31b689cc7e280606dd6991826766c980d8003b519384b859e9c0699a8b41ee1c1002012002e502e4009fbd63e73a76dfc28e254f87b253077072d873e0b89278496fba77a5a3b1a8bf3a2ca339f73059b3a0db43e6d66cf157186b2baad92f1243ca17ebfb4d05099352003f705f3b4f510ee070f05f9879b1c8009fbd4a9b3568d91fcad9bcb73297dee4322944100ece6f840a697c3ef52f1dbac2449545d124f21cdf3d507b280a2b99fcb51914872f2bf81c1c545137b1fb006200180c145f0388f2e02acf881c68a56802012002ec02e702016a02e902e8009fbdddadb565548a35b34d787f6a749aaf96e00391fa6084b4c4a0ce5d3b86d4c75d97d2d90c961c90786ba5b145d1f066c219cec0eca67c2b7eb93b61ed0e91a2000abd3525bafacf38131e5a6c737fba02012002eb02ea009fbd8798202b6c721a3c72374a1d807c76358c88b80e157ad73717fc8f6b1e27b0e14ac044812e9b4c6eb927a3a3446e405031c8c600c87a04811e1eeb116f9fd9000bd46e9db486a970150f734b446614009fbd851aff2932c6dc7a0dc225e32d48748d656dadae4351ecf8e00ed5909677091cd61e1c4212cd76671689c05e0893f08c4851c340068a64a1e44e1e3e031660000d1b5289b4fe8b70175567ebb5e95402012002ee02ed00a0be420de9ee573da0851cf52f00267486b091e52802c4fc15684281f8ccbb9b8188268ca552d6eb8b3e618aafd406784fb7c80e0bec0b03c932bf1abf520097fba001726a734831210e0293716c2b0b7000a0be4c247032e19c85775fc560ab458105b96a6a5806160d25e6299f2ccb2e8680f91043d6214cdd9ebee118a1a8e18497fdbdab8f7233ea9368b45da337e05d2bc001a9619760f0976e02f54bf6534b1a020120032302f0020120030802f102012002fd02f202012002fa02f302012002f502f400a0be636b942dfafd06a7507cd52dc2ed5c9027dd950977bf66cd649ef0c3266791e8747d21d57a8ea5cbfca61c74af4c293405067feaba1975f8af34945cb5c7daa0017f277df9c169ce02aa1efb94e00002012002f702f6009fbe209f655fb6ef00526be8c47ac3652886447e22acab60514b23e4b3857242d1aa3e912aefa3bf33c1bcdd1ea246e65b102bf4b71cbffcbcc69da709a4c045e14007ee0be769ea21dc0e1e0bf30f363902012002f902f8009fbdc6547f59f7c5bf9a77644818765fb0f4147ce25737d46012e57faa74b2defdb1ffc574b74d304faee01311d19e4048b6c68e1a850461699f063d319fddacf1000f9f2f049a7c0e381bcfa866c4861a009fbdc0c060c257c4e0445cd8fae411805d4cfc5f9c4a200e504fd734cbe3f28d04c2faca6e3f4e567455e8c6f34ccf874ffe9d1b6309a794408ee20227cb801d11000638b7acfbc0afb80b137ac38e85f202015802fc02fb009fbe01c96df6597fbc2ed53dba5a6b845df66ac4ca80bad5e5d131210221e5ac4ec12f6dfab87edbfd8bc555a687355c4c786ded43d700794da93d9c8a603c2829c002f7030af0f18f9c05474082c5bdad009fbe113398b81951838cd0c0cc4bd91dc5f4c20d6b914eed1a36ad4f0f24b2562c284c463677ceca6b785912011ab9fb56656cccc0a97e8662cbef2c8f54d1e3bfc00352dc044642041c05eac422cc1331020120030102fe020120030002ff00a0be6c1a5c2275ebaf08cf511b84b4a9edb8a17bfc37fc4e1c25a28c1555cef6e476008126a4defebdac4bb0f65243b8025860ae5621bec658529534a531460ccf0002ab090a7f2d85ce04bffe1adafc0000a0be7e236409a26f4a544aad4de1465d3ef97ddb3d1b956f0fd6b9c03fc3e9cc4a7518b1010cc786cd6359db016a7786a3fbd883a13bd7d4763f3465faf4f05c0120021a1422a6b5c54e03bdedf1d8bbc8020120030703020201200306030302014803050304009fbd8fbf3954e2f4d436c21145f33940bce216a1ef0617f0d6b3a1c002f40470b618e3b866be904de0c38ddf1c989bfe6dc4bfa3e5e2d4d8fc394067d112953e58001db92f75db631c7034ea76b3ef9aa4009fbd95fadbf0746198d5680dd7f0481049fa349aabafecf13ad0dbfd810c09acd56527164d691243d1f5b38d3c25688efd2ae54066a031a28ce7624fd3ab8fb06500108cd76c43b57f701d76c138f59214009fbe365c008bc16d49d2c00b1f070e4b7f9cec8e84fbec7957102421e64916e13255fd9003218c1d4ea9096a9b5ddc3f1a66c454390839424a1af3435c0cc4e8ed0002fc5729150f181c0550bd04f2d11900a0be7bc10ca7bc17cfba4ec29c26655f582c53a39625d92736663403700ce33b8159e60fbd1d815b93109aab75b6cf5fe29f235e21a8d5d42a5828099e2f75be786003f705f3b4f510ee070f05f9879b1c02012003140309020120030f030a020120030c030b00a0be4cd54681d01606631daa2b0b01c5e63b206800b5c2d89a35fa5b9e3a9d85c7b333ffc9ef99b204c710dd81cf4d77246c49ecb5b587a10e410ea06d8b1b5304c003f4d96e93ff1f0e070b2736e7c50a02016a030e030d009fbdb39428e54d54ae7f63c45d7e2db12c43030d4352860558d1a5cc83fbaeb25122c5c2844b09f840a944e0e79c02443c2d7e448cc3f5a7c2e414adf1da4c62df000bb9ca899223c27014e005a8d6ec74009fbd9fe648c633a93dccd22056ad9d5a307ed361a0d92493066b7d7f312d5ac2b690159f0447464201ba7256ddbb5bd8e7cd548e39b643bbd7bef4919de7ec9ad40011ecf4a64fd68e701fe99da54657640201200313031002027303120311009fbd41b74448fff3c1c42a57febe84290e14187034fac28b471a3be465d30fb67329861b1983633290b2a5f6f99246d297fa53478456baebe2b9e42369a29bc204002d57051801fb04e050b7b8394c0008009fbd728299fb2c522d9935f435ad64cd21e64a779634b51b80e35ce3dd48ad31053fac5def4c0389ca1dbc996f9330f2bd96e27bf6058b8b75fb8e82a5e2635cb4001867c6a740463ae02b72c6da9c778800a0be76c97d17082d8178c332d434ce9eac319f5abadfee3f08c6903ec50544936a2f05d436575bff98001acae4fecb2717adb7981083373676d16a63f22440388c00015c43e1b89940ae026c023c3998ac020120031c0315020120031b031602014803180317009fbdf83d55516e3448f35897185901ab18bfe1f3c9c84d2a3ac98073b9c4086672825884265b16be775b18804024ee7b9db452b3aa21d28cacde9e1d2674b65881800f0081b1ea34b6b81ab52b29a4cd02020120031a0319009fbd9300d55c8b07cc295b0f6a1aa1223bc585032cbc51efd734b95eda73edad1ccc372318572e216ce2140a6e3e39b5292e53a6ceced3ba538f6630ebfa0ac13b000f975786c6d551701bc1b282479124009fbdbd7db4e91836ee10d4c21fdea3070e8c93ba4c535c8ec5d622f1f0fe25e23499d8be570dc60565adef9e5826ed2343e8d5cd1b19a742fec0e4de4cbb24d2ab000d08de9d83699a7017348e06b8080400a0be4d4eab228bc4e7d6cb55e09998fd9b6b0a05d937233073d136a28dce07fbeea4860cbc255333198bd49fc409fc6e315680f4652c9b05055abc905ae2acb91520015bf75ddc7a218e026b7a0464b0ce020148031e031d009fbe0ac627a8f9af79c4f6f6f22de7057b7a15031194b662d6af983684d2dad02f69fe3767dd4a5d5cc138827832495640299d7c5e42932d057f6fc14cb68cfe5f000455516379c1349c07b6e4e09939d50201200320031f009fbddb41d6c458b362212208c277521889aec0d7d4bfd0f1894edef4f2b506c3175772355790f8fb44d970c722c3e83bafd07c99d3f22898dc63fd8796170b6e13000fdc17ced3d443b81c3c17e61e6c7202012003220321009fbd984302ed31daa9f97054858de30a0485267baa750ff6a1b86f06d35b6b32ad742959cd619c3c120fc7e7600f79f3772b18e68768e486ccbe47923aef9b16c4000aaf5ce5adefea701305b4ac7b72f4009fbd80427365e24e5715f443ea0e2e2aa68a35b35f9537a76eaac488e9fdfb72f90f4f1c4f7df4a0517be22d8ad622502e1bb665c6ec1260473da6a830dd2b59c1000b946f95eb8fa070149d84f2f121e4020120032d0324020120032803250201200327032600a1beafa0beccdc075c09dc91a5aad9ab1d3c66f242de4ca68bb68694a1593b3973808975fc03b9e43386efca24feb1702b4e78a137c76aa8068beb0caa3f947025a000be92092e2f8e97015344b2165d6e4000a1be8a56137525ba943a6fdbc2e55a84cfa52809ff0e1fdcb1ec7c42d30054e593339b045d44f666519afa74368c3734fc275beadbd1530a0d947a46a73db03de68001fb82f9da7a8877038782fcc3cd8e40020120032c0329020120032b032a00a0be47e30b5e8336e76561eea719e4cd732fa88c906f0ce55dc186455c0143ca33bb4b30a071d39033289af6f4002a7dfed16d75f84ce06eee376a358fba3359afc003b86e555ed1c8ce069f977b94d0d800a0be74bd00c27f2433e777e13656242114d0b91be6c9218cffe65db8fb9761a0b6244bd2fa77910b4d280f3210f37347f6e99ce251e392eccd581dff29d996cb396001649566c114ee2e027ad15f2d979e00a1be89006809572798f3a9b3c265bd7e7965ec31354503c122acdd93e7431dad7fd4ebfa1951e571bab8b9206135a5cf83dc69c29623c3064161fbfbff8ad6d1a69000d61d369cd21d67017d2ebeeba4a840020120033b032e0201200338032f0201200333033002016e03320331009fbdb004f42304da979bab439872ce9ee7fbd1cd93a94ef88325f5f99cf3aa67e72a76cfebcff3b24e9d45d088955a85793cfca45ab947a8bebd73b881692f3f57001bf9797816bcaf7031cd69f5f930d4009fbd8ed5ad6b0373fc99ef2cb61c18cdabb962ef2fd9361b92f0750dedc4c03609fafcbc84dcf717af74b490cb242f286ec8917b510d409b3637b9ac1aeb1caf4d00119e9ebbe65e2b701f5e281ae8f4b402012003350334009fbe3bd448d1bf5b4c8a87683f359f8099a5a3df8d91c3842d09b2f115a0366469dbec73bfe10d256929ac71a98b02db1f15c081bc83b5bef1e7656548a749fa9fc0031df0c9d4a02d9c058c8e4e7036b902015803370336009fbda3a3bba868ab5904740e6a278f8a1783c0eb53801fbb9d559b46afdb8e67f1b3aea821a56276e925f3e9c71829e97db475a4a3cd64f48b988d88e815167008000e1797235dbfbd70191683541239e4009fbd85e6c4fcb5e787633c1dd3f8c39a491a18b4a696f4c93468c80af6ee22d7d9a009984f5bd90ebce37c2b6aa5ec9d86567442f90329935db8c5f9c3e3596a9e000aa5d32214a0f37012f4b9a7cc3f64020148033a0339009fbe04cd4cef4336ad2072bf6e0488fa2852e701f7e63bcb6a1c7064777736686bfca821cd8e7fa6faf203eb4658eede64a3538655d818f6aca7a2529e1eb308a6c003559987c05668dc05efa506645689009fbe109177a61a5c18e6fc768ca70069ca6a4827bf05d322381eecf0ec14b14edf55559866bb1e9507b30b10d6836987f0984e7ceb8556848b245f6cc2659ab4a68002e51be57ae3e81c0527613cbc48790201200341033c0201200340033d020120033f033e009fbe033fd458970f76e1f8aace42c7088c573754863f80c3a0d85dcf34d90b761bd7fc6350a03e12c91b02121415be06f16a8cab055d26c5be858d9d041c7c627dc00780422c0f72e6dc0d5a97f084ee29009fbe0a22e754873e46ed9423fca1a94083e0efe8443eed64c00199cc6e344cce711d4f8a6344add2525bfb351572097787e8112bf88d840655da7dba4785ece3400007ee0be769ea21dc0e1e0bf30f363900a0be6359bc69cccff7b31a6185d785d2cbbbcf0ef6b2fb43681a503969b90add93e235d4887e3b3c958cdbb2880f6607b3fcb8710e3fc6606818ade795c64bdf5a80017b814674a110ae02a39fd133b59e0201200345034202015803440343009fbdc2564a336d4dbd7f5e0d560e33a494fe508d1d3275b3c7278cb698e01baaa64b5f834cfeaa82d82d569f3999adeb8b022a081e3667b9cc64723ad055752527000e29ddc62aec003819370c999ab5ea009fbddd5adc254b4a2255adf9bbd8ddb2c2956b897dcaaa2a18519c9d0e38b66bf0f276a2cb1fae500ecf3034ac104defb0f5d2ed62611c627551f6640a951fdbaa000a8e913e4213a33812cb5213d7734202012003470346009fbe207da92d4c114fea615e7504306af0ce79c1a7cdffa5fb6cb1def1d1e7347ef863ef934f799ab1c152e08c032739db02416080aa7c698d13cbf7f12d3f29420007ad4402bfb1fd9c0daab80348c201009fbe2d19a8c3c6f31888ada9fd0d845496c69ce5be24991e29e40f38009f7f8467bb9629f8a600f751edb0f3aa06f0c021d67fff6a367ecb8adcef666cf385d7cc0007ee0be769ea21dc0e1e0bf30f363902012003980349020120036f034a0201200356034b020120034f034c020120034e034d004fbecfc5f04a66b990d2cb6219f8035f05ddf0afe6d5849c471a9a7bb2f5a5e354f3443ddaf7f38240004fbeefc66cc6867997afae9aa8e59578576e1fe5f69dff7e733f7baee7d79839c6731bce196bd6c14002012003530350020120035203510050beb3512dcdc6528b732f8384f55253dd777150b016766658c64436d44d4082dd0701e8a74b316a51004ebead07d0c330cf4477be47ffbd91fd478b94390880dfbde2b9d3523d941ac57ee6376571ede7cd02027403550354004fbdc881d116130b5ae327d05d344a2939a43af4dadea7f43f15f35557a35cc6a6380a69976997adac004dbdcad9b753bfd9cddd767d13075d8ec8f5d739bb51c7eb8f06a4f91cca7f0101b1231d29a7d08c02012003600357020120035b0358020158035a0359004fbe4c964d6dd41b6bb7dc25d9342b8140770a89506c9185cf8c264e82bb9cca776e067536bdaa5877004dbe4fe3d5b98f3279311bce649a7585b6bb50626dbe4dcab609b1dc913975e9f2ac38c19b7f7585020120035f035c020120035e035d004dbe59cdb103c3fb290338ca0e152299363ce51a2b7c77d97949f55507560cb12b4c6eca97a995ff004fbe68a6965b3c2edf9c57c42d9a769aff9446be864ae55805e31ae5bc38696d9eee03c6092951164f004ebea75cb2b532909f569af32d8b809d9be3ae17ffc3ea8e4e880be39705abe8c1561fe2015f5b2d02012003680361020120036703620201620366036302016e03650364004dbd09c8c61e737ca899bf04c2b5f12f14b9fde20db37692c5980bf34a09e4bde18dc0e7d54f4960004dbd0be0bd722ea28d9e4ad6d785f7fbdb1f28c08338ba7503ece7ec293451d02d8959f433afea60004fbdf412c0a19e8b8f23083bb423aa9d8e631616fba0084d62da892a8b652c971eb80d54e38e2e158c0050bea9eb496d0a0857811a320c2ce567bde893964aa7e70069b0ffbafa6a0483a21701ee45979bb822020120036a0369004ebe852c2b80a95cbec732e94f9ca28c70b176fb82eac42f7d04721d760219ebec26379c32d7ad82020120036e036b02047f0f036d036c004bbc61934af5fe188d23cb3671891c361f0cc0d9b1e45ebc76df6c67649f00454c6eca808772ad004dbc416e2ad27d9112a71ab93dbbda191eea9e885a9447826ac1cd105f17d6c5ce02834353f55125004dbe538ef323d388fe3a0ee131b77bd734c646a528f364cd235b69bd8366d686f4cc12309ce5400102012003870370020120037a03710201200375037202016603740373004dbe0ce9645c79df9dfff7b554e38e292f221accecc597072f270a449cddb16d2698dd928626f38e004dbe328fdcde126e3e3cae1fe47561bc0fc505260044a3592ae2a43dce8e1abd6a189745835e381a02012003770376004ebe968106cc0214842ae21a703d124c7dfdead2b9af3adf4a5d0905a37c5c058e160eb8b6cd8c2b02012003790378004fbe45f40e5b2f53a165a251406b64b780a5447bebb048de94a7abec3febb1a8324e08ebd713a49f1f004fbe71244ee62c58362056e4865c396d1878ade8090551bf9585cb8d9ce095a9892e0280fe0e1519bb020120037e037b020272037d037c004dbdd9dc81dd6f7dfc74d728d50acd2cf3965a06f4a7c526f516689ec38b388e8bb1bb2fe4e44bc4004dbdc313ff6442b4294213f5969de24cc322f01089390d8112112dca535557e84d31bb34353e826c0201200386037f02012003810380004fbe4c7375dd61085ca970d2b98e268686bffd6ed9ba07eb8dcd0c1791859e3d8dee0d8be050780ac10201200385038202012003840383004dbdd8b6fb300ec8958b9753cffe21d3b859286f8c1d0181d302616e572427716f30ea31af510a7c004fbdc569b8481f231bdbee1fef1a3f3f4e8c56dcb7becf4621432966d7d5c2f2afb80ae2e169792804004dbe0f9961a89f9b07d2916b3b42d5b4bfc1ce11879c0dddfd4212af298cc2cf709a82b3562e348a004ebe93cb06cf1c9d9095383e64828f0859cc7d9ab1f89a8d254a35ac79b2e69a58e624fdea9c11c90201200395038802012003900389020120038b038a0050beaefe72434ae8859ec32661cc3d797315bc9d217896238672648350f81962ae3701e7d15a746ca1020120038f038c020275038e038d004fbd4d601f7cdc91fcb6f7c7def9e253617ef7d92f094ed078a749252f498a3b62e0288119b785ea50004fbd776ae0a61f7774f1f83ab82185b32955c7d9ed9079e41293d47967356124b8e035dad6b6fc83b0004fbe6e6c47522d00f8830d9cc5cf84ec8061a6eeb47e2917b281d043a5ad739e098e03c5815e5749b702015803920391004fbe7e4fbc44742232400ff2d08cf2d0fa4984180cc6319b40bbe496a43494d9386e06573bc7d2034102016e03940393004dbd92967b8d1c870ddfbb1fa1712c1321e555984fc3413092336d9bd507e83c1a6284b14104dec8004fbda15d054242ac95423da8c68de14ff83cf4112391ed4102d0a16dc7c01fdcc8701e6c3460ef2f18020148039703960050beb5151007cdff94a183a7dad2a2572853ec873f527ead4163fda6e4e17dd7a267014067dbc039690050be904815e26d2598c7e0e639a42427d0ad632462dd65cdb74cbc0326b28d1f7e67019a5042fb302802012003b4039902012003a7039a02012003a0039b020148039f039c020120039e039d004fbe5ea638ad2f40cfd21e08fc2afbefa668d509555315b4de835c35053db045ea2e03be500bd61ea3004fbe4a516823e74e6e0ba66f4ae491c4077f77791f104c792d8351d311e51484ee0e025ae667fd56b1004ebea020fe4ba1521b3b672617f73b0c2feea5909271276c7a65cb5083c6f3816fd63769a530b72f02012003a603a102012003a303a2004ebeb424cfa21b7f37276fcf9047573c06a07c6e38771d374258cd4e5feeb0f7358606d43f75b8330205824fd003a503a4004dbc39aedcde1bb891cf6ed806629ddd07e08f2d930362fb98738a57609df390dc053adcf4d6f84a004dbc1267b8bae7766715604f4223f288e4528caebb25549626841f6071b43a3d9c07894d96af57a60051bed36c9508e3465752ecea780be45f7b324f2712e373ca93c2edd41f151e49c40b80c05e8870da14c002012003ab03a802012003aa03a9004fbef2d1c4c1c06ad2cb2bc763d34f9d0cfd85e58963620ac266f8b7c573dacac1131248c5e201a5c0004fbef3acc09410e881452ec1fceac2e08582a0d60a03fd8da34015ccce31fe3e03631bb2179cb6b84002012003b103ac02012003ae03ad0050be8f4c831f9260bb67dee4efa41600c75df4306a603e5ba3a46f77e44da6080c87038dec1e030c4802016603b003af004fbdf681783cda5f0d558351ccbd2bcb37ceef40c707bbfaff29c291903452ec7b3809a90a7b4afc44004dbdd7c458c306ee27377829c03fddf49c3c18fd9e2ceb6856183367eafe5db884b731ff7d60b0f402012003b303b20050be81618921cdc482f35836b0d904191196fe822ce97c1830eb1acf57ec40f4e827013522dd3eb329004ebe893a4e8331bbfd1c0e4e213dc08803a1231cbf3059c102408e11d319fc1e83b621529265932602012003ce03b502012003bb03b602012003ba03b702012003b903b8004ebe991f8c8959d062235464db02fecf1ae92c8bd654a5811708976e9705beb5d0961edcf18155000050be8621a843d396882a267e8651ebfe2273f7c6affdd3750be9a17412ab02bbbba701f059c3a023e40051bec20ac48de2ad13b0dd833775b0d384fd09ad20185750d3b0d740928fc9a6b81b80ae9435cf24964002012003c703bc02012003c003bd02012003bf03be004fbe7f46cf1324b7f8004ed8cb05f035f38ab639fac1bfb927bf976e15469b501c0e028198caad98ff004dbe753d4f6b8f0ca56457bb24ae0e6d5d424519a5e9ca2b1f508751acbd665f426c39723dbb43c702012003c403c102012003c303c2004dbe0aa210d9dada5b64dfac97949d06a25b043763765255290c2e6010765c6df49860fab98c7d1e004dbe225dddfe3812df560ee284a87585b50d988538d4938827f2e5482b8a24179298dd926426f05602012003c603c5004fbe0f22602a95eb32264c0ff1d13f3bf0df3c7e03a65b7dda146221c8ddf8a5681c0b7bea0f3f4c22004fbe1b74716b7bd3dc970ec370638acd21d580ef3fbd3cb9f6f3d22c18da841be11c06ab5917ae102202012003cb03c802016e03ca03c9004dbdfeb83d506253f3269913178497ab4131eb2ecf3e109c73700d34a35c00950db048c273950004004dbdeb0217ec903722319f47560daea220d6dc8e1c409022fa317ce67c26f63d45b0aa259c2c4f9c02027103cd03cc004fbd90052f8cfbf00c5a4dd0b9349aca5c593afb157236841ab6c206e13ae600977039fab22ab432d8004dbdbda7c18314ef826d3c2450c353429c494857026dd149d0f6229a5f23d9a9306444422381491802012003d803cf02012003d503d002012003d203d1004ebe8e435f1ffdc8a365032e1c3e7f4d96a890ea12cee4e142bcce610d922a9d3ce625713ac945bf02016203d403d3004fbdc613f07efba0e7fe772bda2798f175e919f7ba7e7346026ccb611e4c27408938292498a4f10334004dbdf60ce3e56e20f5fc20c8ce61393035656cd264dcd15a82fdc58d6adbdf5e6ab444802de1cdec02012003d703d60050beb0b4080f998bdff52eee1f5ad1b7a3e49bdc848c68c3f79a7a1b005a1626abd7019a197cead1c6004ebea77c9f42046cdfa1ccd2a67349dabda00c2d94008d520f98ed4efa267fb1aca621069b6ee08802012003dc03d902012003db03da004ebeb9a17376c35b96c12135d53fa9d113b29b02c145ab6386b7aabfaec99824c2a609184e72a000004ebe9a5a63e309ce223ca5026093d1aac4ebf74166dbaeba7c94b9a03b85a3b66f56376a106cb0c0004fbeef31a0f89120847ab3585f903cc61087f76b0035f13e12c900b0bf0de2fb53cb1bb26f448b15c0
code_hash:     cb6eb312df8187a61e386563f4643c14cad2b98360ce0d9e78da49467188a66f
```

As you can see in the command output, setting up the project_id changed the network endpoint:

```bash
...
"project_id": "b2ad82504ff54fccb5bc6db8cbb3df1e",
...
Connecting to:
        Url: main.evercloud.dev
        Endpoints: ["https://mainnet.evercloud.dev/b2ad82504ff54fccb5bc6db8cbb3df1e"
...
```
