# CHANGELOG

## v1.0.0

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
