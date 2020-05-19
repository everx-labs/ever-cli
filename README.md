# TON OS command line tool

`tonos-cli` is a command line interface utility designed to work with TON blockchain.

## How to build

    cargo build [--release]

## How to test

    cargo test

## How to run

#### Default

cargo run [subcommand args]

#### Linux
`> cd ./target/release`

`> ./tonos-cli [subcommand args]`

## How to use

By default, tonos-cli connects to `https://net.ton.dev` network.

### Crypto commands:

### 1) Generate seed phrase 

    tonos-cli genphrase

### 2) Generate pubkey from seed phrase

    tonos-cli genpubkey

### 3) Generate keyfile from seed phrase

    tonos-cli getkeypair <keyfile.json> "<seed_phrase>"

### Query commands:

### 1) Get global config

    tonos-cli getconfig <index>
    
### Smart contract commands:

### 1) Generate Contract Address

    tonos-cli genaddr [--genkey|--setkey <keyfile.json>] <tvc> <abi>

Example: `tonos-cli genaddr --genkey wallet_keys.json wallet.tvc wallet.abi.json`

`wallet_keys.json` file will be created with new keypair.

### 2) Deploy Smart Contract

    tonos-cli deploy [--sign <keyfile>] [--wc <int8>] [--abi <abifile>] <tvc> <params> 

Example: `tonos-cli deploy --abi wallet.abi.json --sign wallet_keys.json wallet.tvc {param1:0}`

If `--abi` or `--sign` option is omitted in parameters it must present in config file. See below.

### 3) Call Method

    tonos-cli call [--abi <abi_file>] [--sign <keyfile>] <address> <method> <params>

If `--abi` or `--sign` option is omitted in parameters, it must be specified in the config file. See below for more details.

Alternative command:

    tonos-cli callex <method> [<address>] [<abi>] [<keys>] params...

`params...` - one or more function arguments in the form of  `--name value`.

`address`, `abi`, and `keys` parameters can be omitted. In this case default values will be used from config file.

Example:

    tonos-cli callex submitTransaction 0:1b91c010f35b1f5b42a05ad98eb2df80c302c37df69651e1f5ac9c69b7e90d4e SafeMultisigWallet.abi.json msig.keys.json --dest 0:c63a050fe333fac24750e90e4c6056c477a2526f6217b5b519853c30495882c9 --value 1.5T --bounce false --allBalance false --payload ""

Integer and address types can be supplied without quotes. 

`--value 1.5T` - suffix `T` converts integer to nanotokens -> `1500000000`. The same as `--value 1500000000`.

Arrays can be used without `[]` brackets.


Run get-method:

    tonos-cli run [--abi <abi_file>] <address> <method> <params>

If `--abi` or `--sign` option is omitted in parameters, it must be specified in the config file. See below for more details.

### 4) Generate signed message

    tonos-cli message [--abi <abi_file>] [--sign <keyfile>] <address> <method> <params> [--lifetime <seconds>]

### 5) Send prepared message

    tonos-cli send [--abi <abi_file>] <message>


### 6) Store Parameter Values in the Configuration File

tonos-cli can remember some parameter values and use it automatically in `deploy`, `call` and `run` subcommands.

    tonos-cli config [--url <url>] [--abi <abifile>] [--keys <keysfile>]

Example: `tonos-cli config --abi wallet.abi.json --keys wallet_keys.json`

After that you can omit `--abi` and `--sign` parameters in `deploy`, `call` and `run` subcommands. 

### 7) Get Account Info

    tonos-cli account <address>

Example: `tonos-cli account 0:c63a050fe333fac24750e90e4c6056c477a2526f6217b5b519853c30495882c9`

### Sample Test Sequence
Task scope: deploy a contract to TON Labs testnet at net.ton.dev.

#### 1) compile contract and get `.tvc` file and `.abi.json`. Lets name it `contract.tvc`.

#### 2) generate contract address.

    tonos-cli genaddr contract.tvc --genkey contract_keys.json

Save `Raw address` printed to stdout.

#### 3) Ask the testnet giver for Grams.

Note: You have to get giver address, abi and keys. 

Let's request 10 Grams to our account.

    tonos-cli call --abi giver.abi.json --sign giver_keys.json <giver_address> sendTransaction {"dest":"<our_address>","value":10000000000,"bounce":false}

#### 4) Get our contract state, check that it is created in blockchain and has the `Uninit` state.

    tonos-cli account <raw_address>

#### 5) Deploy our contract to the testnet.

    tonos-cli deploy --abi contract.abi.json --sign contract_keys.json contract.tvc {<constructor_arguments>}

#### 6) Check the contract state.

    tonos-cli account <raw_address>

The contract should be in the `Active` state.

#### 7) Use the `call` subcommand to execute contract methods in blockchain.

    tonos-cli call --abi contract.abi.json --sign contract_keys.json <raw_address> methodName {<method_args>}
