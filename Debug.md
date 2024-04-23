# How to debug contracts with ever-cli

ever-cli can help user investigate at what moment of contract execution has error happened.

# 1. Preliminary actions

User should generate debug info file to bind contract and source files on the stage of contract compilation with
[tvm-linker](https://github.com/everx-labs/TVM-linker#1-generating-a-ready-to-deploy-contract):

```bash
$ tvm_linker compile <contract>.code --abi-json <contract>.abi.json --debug-map <contract>.dbg.json -o <contract>.tvc --lib stdlib_sol.tvm
```

Besides **.tvc** file pay attention to the **.dbg.json** file, it contains debug information. To generate it, option
`--debug-map` must be specified.

# 2. Deploy your contract

```bash
$ ever-cli deploy --wc <wc> --abi <contract>.abi.json --sign <key> <contract>.tvc <params>
```

Here user obtains contract address, which will be denoted as `<address>` in further instruction.  
Before actual deploy or if deploy failed user can debug deploy with ever-cli command which has almost the same options:

```bash
$ ever-cli debug deploy --wc <wc> --abi <contract>.abi.json --sign <key> -d <contract>.dbg.json <contract>.tvc <params> [--init_balance] [-o <path_to_log>]
```

`--init_balance` option allows to debug deploy without preliminary initiating balance of the network address.  
After successful execution a log file will be created (path to this file can be set with `-o <path_to_log>` option,
default path is `./trace.log`)

# 3. Make a local test call or run of contract function

Before calling contract function onchain, user can run function call locally with trace.

```bash
ever-cli debug call --abi <contract>.abi.json --sign <key> -d <contract>.dbg.json [-o <path_to_log>] <address> <function> <params>
```

The same as call, run command can also be debugged:

```bash
ever-cli debug run --abi <contract>.abi.json -d <contract>.dbg.json [-o <path_to_log>] <address> <function> <params>
```

As in previous case log will be saved to a specified path.

# 4. Debug a real contract error

1) Contract call fails due to unknown error.
2) Explore the error message, look for this string:

```
Error: Failed: {
...
    "transaction_id": "69a8250000571041c011ef717228f6637b836248f8af46755c33bc9bcf0e9b88"
```

Or get transaction ID from [Live](https://ever.live/landing).

3) Run the ever-cli debug transaction command with the obtained value to get TVM trace:

```
ever-cli debug transaction 69a8250000571041c011ef717228f6637b836248f8af46755c33bc9bcf0e9b88 \
--dump_contract -e --min_trace -d <contract>.dbg.json -o trace_old_code.log
```

4) Explore the output for contract dump:

```
...
Contract account was dumped to 0:8be07ec3f8f25ebb35ce1a29d48b0cbbf1d41aa00249f34e89f136c561cae3fa-69a8250000571041c011ef717228f6637b836248f8af46755c33bc9bcf0e9b88.boc
...
```

5) Rewrite your contract to fix the error and compile a new version of the contract.
6) Replace code in the account dump using tvm_linker:

```
tvm_linker replace_code -a <new_contract>.abi.json --debug-map <new_contract>.dbg.json -o contract.boc <new_contract>.code "0:8be07ec3f8f25ebb35ce1a29d48b0cbbf1d41aa00249f34e89f136c561cae3fa-69a8250000571041c011ef717228f6637b836248f8af46755c33bc9bcf0e9b88.boc"
```

7.1) Run debug replay to replay the transaction on the modified account state:

```
ever-cli debug replay --update_state -d <new_contract>.dbg.json -o new_trace.log 69a8250000571041c011ef717228f6637b836248f8af46755c33bc9bcf0e9b88 contract.boc"
```

7.2) Run debug call locally on the new account to test new version of the contract on a new generated call message:

```
ever-cli debug call --boc --abi <new_contract>.abi.json -d <new_contract>.dbg.json -o new_trace.log --sign <key> contract.boc <function> <params>
```


