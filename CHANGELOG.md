# CHANGELOG

## v0.2.0

### Breaking changes
- `tonos-cli` now stores its configuration in `./tonos-cli.conf.json`. `tonlabs-cli.conf.json` is now obsolete and can be renamed or deleted.
- Commands `deploy`, `call`, `callex`, `run`, `message` and others now output errors in a different format, compatible with the corresponding changes in the SDK v1.

### Fixes

- Some fixes were made in SDK Debot module that affects running of debots in cli terminal debot browser. The following were fixed:
    - invoked debot terminated correctly after error occured during
    execution of one of its actions. Initial `prev_state` of invoked debot changed to STATE_EXIT;
    - fixed double jumping to current context in invoker debot after
    returning control to it from invoked debot;
    - fixed conversation of exception codes thrown by debots to their user-friendly description.
- Fixed bug in terminal debot browser. Error in invoked debot doesn't shutdown caller debot. 
    
This fixes affects all debots invoking other debots (e.g. depool debot, mludi debot, DoD debot).

### Miscellaneous
- `tonos-cli` switched to SDK v1. All code using sdk api was refactored.
- `tonos-cli` started to use Debot Engine from SDK Debot Module.

## v0.9.0

### New feature
- Depool commands now by default wait for message to be sent to msig, from msig to depool, wait for depool
  answer, decoed it and print. User can disable this feature by using --no-answer flag with depool command.
  
## v0.9.2

### Fixes
- Fixed retries for `call` subcommand.

### New feature
- Added config parameter `--lifetime` which sets lifetime for `call` messages.

## v0.10.1

### Fixes
- Fixed a bug in retries with negative error code

## v0.11.1

### Improvements
- Added ability to specify depool command `--no-answer` flag in the config and in the
  end of command.
  
## v0.11.2

### Improvements
- Added ability to print account balance with delimiters using config parameter `use-delimiters`

## v0.11.5

### Improvements
- `tonos-cli` now provides several different endpoints for main.ton.dev and net.ton.dev in order to improve reliability.

## v0.12.0

### New feature
- tonos-cli now can execute the transaction locally for deploy and call commands before executing
  it onchain. If local execution fails, onchain execution is not performed. Local run can be
  enabled by setting the flag `local_run` in the tonos-cli config.
- tonos-cli can calculate fees for call and deploy without executing it onchain and also calculate
  storage fee for an existing contract.
