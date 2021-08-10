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
- [1. Installation](#1-installation)
  - [Install compiled executable](#install-compiled-executable)
  - [Install through TONDEV](#install-through-tondev)
  - [Build from source](#build-from-source)
    - [Prerequisites](#prerequisites)
    - [Build from source on Linux and Mac OS](#build-from-source-on-linux-and-mac-os)
    - [Build from source on Windows](#build-from-source-on-windows)
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
    - [4.4.2. Alternative command to call contract in the blockchain](#442-alternative-command-to-call-contract-in-the-blockchain)
    - [4.4.3. Run contract method locally](#443-run-contract-method-locally)
    - [4.4.4. Run funC get-method](#444-run-func-get-method)
    - [4.4.5. Run contract method locally for saved account state](#445-run-contract-method-locally-for-saved-account-state)
  - [4.5. Generate encrypted message offline](#45-generate-encrypted-message-offline)
  - [4.6. Broadcast previously generated message](#46-broadcast-previously-generated-message)
  - [4.7. Broadcast previously generated message from a file](#47-broadcast-previously-generated-message-from-a-file)
  - [4.8. Decode commands](#48-decode-commands)
    - [4.8.1. Decode BOC file](#481-decode-boc-file)
    - [4.8.2. Decode message body](#482-decode-message-body)
    - [4.8.3. Decode account commands](#483-decode-account-commands)
      - [4.8.3.1. Decode account data fields](#4831-decode-account-data-fields)
      - [4.8.3.2. Decode data from the account BOC file](#4832-decode-data-from-the-account-boc-file)
  - [4.9. Generate payload for internal function call](#49-generate-payload-for-internal-function-call)
- [5. DeBot commands](#5-debot-commands)
- [6. Multisig commands](#6-multisig-commands)
  - [6.1. Send tokens](#61-send-tokens)
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
- [8. Proposal commads](#8-proposal-commads)
  - [8.1. Create proposal and cast the first vote](#81-create-proposal-and-cast-the-first-vote)
  - [8.2. Vote for proposal](#82-vote-for-proposal)
  - [8.3. Decode proposal comment](#83-decode-proposal-comment)
- [9. Supplementary commands](#9-supplementary-commands)
  - [9.1. Convert tokens to nanotokens](#91-convert-tokens-to-nanotokens)
  - [9.2. Get global config](#92-get-global-config)
  - [9.3. NodeID](#93-nodeid)
- [10. Fetch and replay commands](#10-fetch-and-replay)

# 1. Installation

## Install compiled executable

Create a folder. Download the `.zip` file from the latest release from here: [https://github.com/tonlabs/tonos-cli/releases](https://github.com/tonlabs/tonos-cli/releases) to this folder. Extract it.

## Install through TONDEV

You can use [TONDEV](https://github.com/tonlabs/tondev) to install the latest version of TONOS-CLI.

```bash
tondev tonos-cli install
```

The installer requires [NPM](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm) to be installed, so it can install packages globally without using sudo. In case of error, manually set environment variable `PATH=$PATH:$HOME./tondev/solidity`

This command updates TONOS-CLI installed through TONDEV to the latest version:

```bash
tondev tonos-cli update
```

This command specifies TONOS-CLI version to use and downloads it if needed:

```bash
tondev tonos-cli set --version 0.8.0
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

### Tails OS secure environment

For maximum security while working with offline TONOS-CLI features (such as cryptographic commands or encrypted message generation), you can use the [Tails OS](https://tails.boum.org/).

### Put TONOS-CLI into system environment

Optional, Linux/Mac OS. Use the following command to put the utility into system environment:

```bash
export PATH="<tonos_folder_path>:$PATH"
```

This step can be skipped, if TONOS-CLI was installed through TONDEV. Otherwise, if you skip this step, make sure you always run the utility from folder containing the utility:

```bash
./tonos-cli <command> <options>
```

## Check version

You can check the version of he current TONOS-CLI installation with the following command:

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

After that you can omit the corresponding parameters in sunsequent subcommands.

`tonos-cli.config.json` configuration file will be created in the current working directory. All subsequent calls of the utility will use this file.

Use the following command to create a configuration file:

```bash
tonos-cli config <--option> <option_value>
```

All other TONOS-CLI commands will indicate the configuration file currently used.

List of available options:

```bash
--abi <ABI>                            File with contract ABI.
--addr <ADDR>                          Contract address.
--async_call <ASYNC_CALL>              Disables wait for transaction to appear in the network after call
                                       command.
--balance_in_tons <BALANCE_IN_TONS>    Print balance for account command in tons. If false balance is printed in
                                       nanotons.
--depool_fee <DEPOOL_FEE>              Value added to message sent to depool to cover it's fees (change will be
                                       returned).
--keys <KEYS>                          File with keypair.
--lifetime <LIFETIME>                  Period of time in seconds while message is valid.
--local_run <LOCAL_RUN>                Enable preliminary local run before deploy and call commands.
--no-answer <NO_ANSWER>                FLag whether to wait for depool answer when calling a depool function.
--pubkey <PUBKEY>                      User public key. Used by DeBot Browser.
--retries <RETRIES>                    Number of attempts to call smart contract function if previous attempt
                                       was unsuccessful.
--timeout <TIMEOUT>                    Contract call timeout in ms.
--url <URL>                            Url to connect.
--wallet <WALLET>                      Multisig wallet address. Used in commands which send internal messages
                                       through multisig wallets.
--wc <WC>                              Workchain id.
```

Example:

```bash
$ tonos-cli config --url https://main.ton.dev --wc -1 --keys key.json --abi SafeMultisigWallet.abi.json --lifetime 3600 --local_run true --retries 3 --timeout 600 --delimiters true
Config: /home/user/tonos-cli.conf.json
Succeeded.
{
  "url": "https://main.ton.dev",
  "wc": -1,
  "addr": null,
  "wallet": null,
  "abi_path": "SafeMultisigWallet.abi.json",
  "keys_path": "key.json",
  "retries": 3,
  "timeout": 600,
  "is_json": false,
  "depool_fee": 0.5,
  "lifetime": 3600,
  "no_answer": false,
  "use_delimiters": true,
  "local_run": true,
  "async_call": false,
  "endpoints": [
    "https://main2.ton.dev",
    "https://main4.ton.dev",
    "https://main3.ton.dev"
  ]

}
```

Some of the frequently used networks:

`https://net.ton.dev` - developer sandbox for testing. TONOS-CLI connects to it by default.

`https://main.ton.dev` - main Free TON network.

`https://rustnet.ton.dev` - test network running on Rust nodes.

TONOS-CLI supports the use of multiple endpoints for networks: if several endpoints are [specified in the endpoint map](#24-configure-endpoints-map) for a network, TONOS-CLI will use them all when accessing it. Otherwise the network URL will be treated as the only endpoint.

`main.ton.dev` and `net.ton.dev` networks already have their current endpoints specified in the default endpoint map.
See [section 2.4 below](#24-configure-endpoints-map) on how to edit and add endpoints to the endpoint map.

> **Note**: This change was introduced in version 0.16.1 and is fully compatible with scripts written for previous versions, where main.ton.dev and net.ton.dev networks were specified with a single url. TONOS-CLI will simply use the default endpoint map to access these networks.


Network configuration can be [overridden](#25-override-network-settings) for any single subcommand.

To connect to a [DApp Server](https://github.com/tonlabs/TON-OS-DApp-Server) you are running, it should have domain name and a DNS record. Then its URL may be used to access it with TONOS-CLI:

```jsx
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
    "https://net1.ton.dev",
    "https://net5.ton.dev"
  ],
  "main.ton.dev": [
    "https://main2.ton.dev",
    "https://main3.ton.dev",
    "https://main4.ton.dev"
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
tonos-cli config endpoint add main.ton.dev "https://main2.ton.dev","https://main3.ton.dev","https://main4.ton.dev"
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

> Note: You cannot use the config subcommand to create or edit a configuration file located outside of the current working directory. It should either be called from the directory containing the file, or the file should be manually copied to the desired directory later.

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
tonos-cli getkeypair <keyfile.json> "<seed_phrase>"
```

`<keyfile.json>` - the file the key pair will be written to.

Example:

```bash
$ tonos-cli getkeypair key.json "rule script joy unveil chaos replace fox recipe hedgehog heavy surge online"
Config: /home/user/tonos-cli.conf.json
Input arguments:
key_file: key.json
  phrase: rule script joy unveil chaos replace fox recipe hedgehog heavy surge online
Succeeded.
```

# 4. Smart contract commands

When working with smart contracts, TONOS-CLI requires the following files:

- **ABI file** - a .json file that describes the contract interface, the methods and parameters used to interact with it.
- **TVC file** - the compiled smart contract file. Used only when generating contract address and deploying contract code to the blockchain.
- **Key pair file** - used in contracts with implemented authorization. It is the file containing [private and public keys](#3-cryptographic-commands) authorized to access the contract. In `--sign` parameter the corresponding seed phrase may be used instead of it.

By default the utility looks for these files in the current working directory.

## 4.1. Generate contract address

Contract address uniquely identifies the contract on the blockchain. Contract balance is attached to its address, the address is used for any interactions with the contract, such as calling contract functions, sending messages, etc.

Contract address is generated based on contract TVC file and selected keys. To get a different address for the same type of contract, use different keys.

> **Note**: If your contract has static variables, they can be initialized through [TVM linker](https://github.com/tonlabs/TVM-linker#5-initialize-static-variables-in-compiled-contract) before deployment.

Use the following command to generate the contract address:

```bash
tonos-cli genaddr [--genkey|--setkey <keyfile.json>] [--wc <int8>] <contract.tvc> <contract.abi.json>
```

- `--genkey <keyfile.json>` - generate new `keyfile.json` key pair file and use it to calculate the contract address.

> Note: if you use --genkey, the corresponding seed phrase will be displayed. Write it down, if you mean to keep using this key pair.

- `--setkey <keyfile.json>` - use already [existing](#33-generate-key-pair-file) `keyfile.json` key pair file to calculate the contract address. Seed phrase cannot be used instead of the file.
- `--wc <int8>`  ID of the workchain the contract will be deployed to (`-1` for masterchain, `0` for basechain). By default this value is set to 0.

`<contract.tvc>` - compiled smart contract file.

`<contract.abi.json>` - contract interface file.

As result the utility displays the new contract address (`Raw address`).

Example ([multisignature wallet](https://github.com/tonlabs/ton-labs-contracts/tree/master/solidity/safemultisig) address generation for the masterchain):

```bash
$ tonos-cli genaddr --genkey key.json --wc -1 SafeMultisigWallet.tvc SafeMultisigWallet.abi.json
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
- `--wc <int8>` ID of the workchain the wallet will be deployed to (`-1` for masterchain, `0` for basechain). By default this value is set to 0.

`<contract.abi.json>` - contract interface file.

`<contract.tvc>` - compiled smart contract file.

`<params>` - deploy command parameters, depend on the contract.

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
- `--wc <int8>` ID of the workchain the wallet will be deployed to (`-1` for masterchain, `0` for basechain). By default this value is set to 0.

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
tonos-cli account <address> [--dumptvc <tvc_path>]
```

`<address>` - contract [address](#41-generate-contract-address).
`--dumptvc <tvc_path>` - this flag can be specified to dump account StateInit to the <tvc_path> file. 

Example:

```bash
$ tonos-cli account 0:255a3ad9dfa8aa4f3481856aafc7d79f47d50205190bd56147138740e9b177f3
Config: /home/user/tonos-cli.conf.json
Input arguments:
 address: 0:255a3ad9dfa8aa4f3481856aafc7d79f47d50205190bd56147138740e9b177f3
Connecting to https://net.ton.dev
Processing...
Succeeded.
acc_type:      Active
balance:       99196914225
last_paid:     1620910749
last_trans_lt: 0x4833a56e482
data(boc): b5ee9c720101020100980001df8534c46f7a135058773fa1298cb3a299a5ddd40dafe41cb06c64f274da360bfb0000017965cef0e9c29a6237bd09a82c3b9fd094c659d14cd2eeea06d7f20e583632793a6d1b05fd80000000000000000000000000000000000000000000000000000000000000002020000000001018010045a010a6988def426a0b0ee7f4253196745334bbba81b5fc83960d8c9e4e9b46c17f6010
code_hash: 207dc560c5956de1a2c1479356f8f3ee70a59767db2bf4788b1d61ad42cdad82
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

### 4.4.2. Alternative command to call contract in the blockchain

```bash
tonos-cli callex <method> [<address>] [<contract.abi.json>] [<seed_or_keyfile>] params...
```

`<method>` - the method being called.

`<address>` - contract [address](#41-generate-contract-address).

`<contract.abi.json>` - contract interface file.

`<seed_or_keyfile>` - can either be the seed phrase or the corresponding key pair file. If seed phrase is used, enclose it in double quotes.

Example:

- `--sign "flip uncover dish sense hazard smile gun mom vehicle chapter order enact"`

or

- `--sign keyfile.json`

`params...` - one or more parameters of the called method in the form of `--name value`.

`address`, `abi`, and `keys` parameters can be omitted. In this case default values will be used from config file.

Integer and address types can be supplied without quotes.

- `--value 1.5T` - suffix `T` converts integer to nanotokens -> `1500000000`. The same as `--value 1500000000`.

Arrays can be used without `[]` brackets.

Example of transaction creation in a [multisignature wallet](https://github.com/tonlabs/ton-labs-contracts/tree/master/solidity/safemultisig) contract, equivalent to the example in section 4.4.1. above:

```bash
$ tonos-cli callex submitTransaction 0:a4629d617df931d8ad86ed24f4cac3d321788ba082574144f5820f2894493fbc SetcodeMultisigWallet.abi.json k1.keys.json --dest -1:0c5d5215317ec8eef1b84c43cbf08523c33f69677365de88fe3d96a0b31b59c6 --value 0.234T --bounce false --allBalance false --payload ""
Config: /home/user/tonos-cli.conf.json
Input arguments:
 address: 0:a4629d617df931d8ad86ed24f4cac3d321788ba082574144f5820f2894493fbc
  method: submitTransaction
  params: {"dest":"-1:0c5d5215317ec8eef1b84c43cbf08523c33f69677365de88fe3d96a0b31b59c6","value":"0234000000","bounce":"false","allBalance":"false","payload":""}
     abi: SetcodeMultisigWallet.abi.json
    keys: k1.keys.json
Connecting to net.ton.dev
Generating external inbound message...

MessageId: a38f37bfbe3c7427c869b3ee97c3b2d7f4421ca1427ace4e7a92f1a61d7ef234
Expire at: Sat, 08 May 2021 15:10:15 +0300
Processing... 
Succeeded.
Result: {
  "transId": "6959890394123980993"
}
```

### 4.4.3. Run contract method locally

```bash
tonos-cli run [--abi <contract.abi.json>] <address> <method> <params>
```

`<contract.abi.json>` - contract interface file.

`<address>` - contract [address](#41-generate-contract-address).

`<method>` - the method being called.

`<params>` - parameters of the called method.

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

### 4.4.4. Run funC get-method

```bash
tonos-cli runget <address> <method> [<params>...]
```

`<address>` - contract [address](#41-generate-contract-address).

`<method>` - the method being called.

`<params>` - parameters of the called method. Can have multiple values: one for each function parameter.

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
```

### 4.4.5. Run contract method locally for saved account state

```bash
tonos-cli run --boc [--abi <contract.abi.json>] <account> <method> <params>
```

`<contract.abi.json>` - contract interface file.

`<account>` - path to the file with account boc (It can be obtained from the TON Live). 

`<method>` - the method being called.

`<params>` - parameters of the called method.

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

Multisig commands allow you to work with any existing Multisig wallets <link to repo> in a more convenient way and without the need for ABI files.

## 6.1. Send tokens

Use the following command to send tokens to any recipient:

```bash
tonos-cli multisig send --addr <sender_address> --dest <recipient_address> --purpose <"text_in_quotes"> --sign <path_to_keys_or_seed_phrase> --value *number*
```

`<sender_address>` - address of the multisig wallet that tokens are sent from.

`<recipient_address>` - address of the account tokens are sent to.

`<"text_in_quotes">` - accompanying message. Only the recipient will be able to decrypt and read it.

`<path_to_keys_or_seed_phrase>` - path to sender wallet key file of the corresponding seed phrase in quotes.

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

# 8. Proposal commads

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

## 9.1. Convert tokens to nanotokens

Transaction amounts in `tonos-cli` are indicated in nanotokens. To convert tokens to nanotokens use the following command:

```bash
tonos-cli convert tokens <amount>
```

Example:

```bash
$ tonos-cli convert tokens 125.8
Config: /home/user/tonos-cli.conf.json
125800000000
```

## 9.2. Get global config

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

## 9.3. NodeID

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

## 10. Fetch and replay

These two commands are commonly used in pairs to recover a state of account at a specific point before a given transaction.

Example:

```bash
$ tonos-cli fetch -- -1:5555555555555555555555555555555555555555555555555555555555555555 config.txns
$ tonos-cli fetch 0:570ddeb8f632e5f9fde198dd4a799192f149f01c8fd360132b38b04bb7761c5d 570ddeb8.txns
$ tonos-cli replay config.txns 570ddeb8.txns 197ee1fe7876d4e2987b5dd24fb6701e76d76f9d08a5eeceb7fe8ca73d9b8270
```
