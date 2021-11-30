/*
 * Copyright 2018-2021 TON DEV SOLUTIONS LTD.
 *
 * Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
 * this file except in compliance with the License.
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific TON DEV software governing permissions and
 * limitations under the License.
 */

use ton_executor::{BlockchainConfig, CalcMsgFwdFees, ExecutorError, TransactionExecutor, ExecuteParams, VMSetup};

use std::sync::{atomic::Ordering, Arc};
use ton_block::{
    AddSub,
    Grams,
    Serializable,
    Account,
    AccStatusChange,
    CommonMsgInfo,
    Message,
    Transaction,
    TransactionDescrOrdinary,
    TransactionDescr,
    TrComputePhase,
    TrBouncePhase,
    CurrencyCollection,
    GlobalCapabilities,
    ComputeSkipReason,
    TrComputePhaseVm,
    AccountStatus,
    GasLimitsPrices
};
use ton_types::{error, fail, Result, HashmapE, Cell, ExceptionCode};
use ton_vm::{boolean, int, stack::{Stack, StackItem, integer::IntegerData}, SmartContractInfo};
use ton_vm::error::tvm_exception;
use ton_vm::executor::gas::gas_state::Gas;
use ton_block::GetRepresentationHash;
use ton_labs_assembler::DbgInfo;
use std::fs::File;
use ton_vm::executor::{EngineTraceInfo, EngineTraceInfoType};

#[derive(PartialEq, Clone)]
pub enum TraceLevel {
    Full,
    Minimal,
    None
}

pub struct DebugTransactionExecutor {
    config: BlockchainConfig,
    dbg_info: String,
    trace_level: TraceLevel,
}

impl TransactionExecutor for DebugTransactionExecutor {
    ///
    /// Create end execute transaction from message for account
    fn execute_with_params(
        &self,
        in_msg: Option<&Message>,
        account: &mut Account,
        params: ExecuteParams,
    ) -> Result<Transaction> {

        let in_msg = in_msg.ok_or_else(|| error!("Ordinary transaction must have input message"))?;
        let in_msg_cell = in_msg.serialize()?; // TODO: get from outside
        let is_masterchain = in_msg.is_masterchain();
        log::debug!(target: "executor", "Ordinary transaction executing, in message id: {:x}", in_msg_cell.repr_hash());
        let (bounce, is_ext_msg) = match in_msg.header() {
            CommonMsgInfo::ExtOutMsgInfo(_) => fail!(ExecutorError::InvalidExtMessage),
            CommonMsgInfo::IntMsgInfo(ref hdr) => (hdr.bounce, false),
            CommonMsgInfo::ExtInMsgInfo(_) => (false, true)
        };

        let account_address = in_msg.dst_ref().ok_or_else(|| ExecutorError::TrExecutorError(
            format!("Input message {:x} has no dst address", in_msg_cell.repr_hash())
        ))?;
        let account_id = match account.get_id() {
            Some(account_id) => {
                log::debug!(target: "executor", "Account = {:x}", account_id);
                account_id
            }
            None => {
                log::debug!(target: "executor", "Account = None, address = {:x}", account_address.address());
                account_address.address()
            }
        };

        // TODO: add and process ihr_delivered parameter (if ihr_delivered ihr_fee is added to total fees)
        let mut acc_balance = account.balance().cloned().unwrap_or_default();
        let mut msg_balance = in_msg.get_value().cloned().unwrap_or_default();
        let ihr_delivered = false;  // ihr is disabled because it does not work
        if !ihr_delivered {
            msg_balance.grams.0 += in_msg.int_header().map_or(0, |h| h.ihr_fee.0);
        }
        log::debug!(target: "executor", "acc_balance: {}, msg_balance: {}, credit_first: {}",
            acc_balance.grams, msg_balance.grams, !bounce);

        let is_special = self.config.is_special_account(account_address)?;
        let lt = params.last_tr_lt.load(Ordering::Relaxed);
        let mut tr = Transaction::with_address_and_status(account_id, account.status());
        tr.set_logical_time(lt);
        tr.set_now(params.block_unixtime);
        tr.set_in_msg_cell(in_msg_cell.clone());

        let mut description = TransactionDescrOrdinary {
            credit_first: !bounce,
            ..TransactionDescrOrdinary::default()
        };

        // first check if contract can pay for importing external message
        if is_ext_msg && !is_special {
            // extranal message comes serialized
            let in_fwd_fee = self.config.get_fwd_prices(is_masterchain).fwd_fee(&in_msg_cell);
            log::debug!(target: "executor", "import message fee: {}, acc_balance: {}", in_fwd_fee, acc_balance.grams);
            if !acc_balance.grams.sub(&in_fwd_fee)? {
                fail!(ExecutorError::NoFundsToImportMsg)
            }
            tr.add_fee_grams(&in_fwd_fee)?;
        }

        if description.credit_first && !is_ext_msg {
            description.credit_ph = self.credit_phase(account, &mut tr, &mut msg_balance, &mut acc_balance);
        }
        description.storage_ph = self.storage_phase(
            account,
            &mut acc_balance,
            &mut tr,
            is_masterchain,
            is_special
        );

        if description.credit_first && msg_balance.grams.0 > acc_balance.grams.0 {
            msg_balance.grams.0 = acc_balance.grams.0;
        }

        log::debug!(target: "executor",
            "storage_phase: {}", if description.storage_ph.is_some() {"present"} else {"none"});
        let mut original_acc_balance = account.balance().cloned().unwrap_or_default();
        original_acc_balance.sub(tr.total_fees())?;

        if !description.credit_first && !is_ext_msg {
            description.credit_ph = self.credit_phase(account, &mut tr, &mut msg_balance, &mut acc_balance);
        }
        log::debug!(target: "executor",
            "credit_phase: {}", if description.credit_ph.is_some() {"present"} else {"none"});

        account.set_last_paid(params.block_unixtime);

        // TODO: check here
        // if bounce && (msg_balance.grams > acc_balance.grams) {
        //     msg_balance.grams = acc_balance.grams.clone();
        // }

        let smci = self.build_contract_info(&acc_balance, account_address, params.block_unixtime, params.block_lt, lt, params.seed_block);
        let mut stack = Stack::new();
        stack
            .push(int!(acc_balance.grams.0))
            .push(int!(msg_balance.grams.0))
            .push(StackItem::Cell(in_msg_cell))
            .push(StackItem::Slice(in_msg.body().unwrap_or_default()))
            .push(boolean!(is_ext_msg));
        log::debug!(target: "executor", "compute_phase");
        let (compute_ph, actions, new_data) = self.compute_phase2(
            Some(in_msg),
            account,
            &mut acc_balance,
            &msg_balance,
            params.state_libs,
            smci,
            stack,
            is_masterchain,
            is_special,
            params.debug
        )?;
        let mut out_msgs = vec![];
        let mut action_phase_processed = false;
        let mut compute_phase_gas_fees = Grams(0);
        description.compute_ph = compute_ph;
        description.action = match &description.compute_ph {
            TrComputePhase::Vm(phase) => {
                compute_phase_gas_fees = phase.gas_fees.clone();
                tr.add_fee_grams(&phase.gas_fees)?;
                if phase.success {
                    log::debug!(target: "executor", "compute_phase: success");
                    log::debug!(target: "executor", "action_phase: lt={}", lt);
                    action_phase_processed = true;
                    // since the balance is not used anywhere else if we have reached this point, then we can change it here
                    match self.action_phase(
                        &mut tr,
                        account,
                        &original_acc_balance,
                        &mut acc_balance,
                        &mut msg_balance,
                        &phase.gas_fees,
                        actions.unwrap_or_default(),
                        new_data,
                        is_special
                    ) {
                        Some((action_ph, msgs)) => {
                            out_msgs = msgs;
                            Some(action_ph)
                        }
                        None => None
                    }
                } else {
                    log::debug!(target: "executor", "compute_phase: failed");
                    None
                }
            }
            TrComputePhase::Skipped(skipped) => {
                log::debug!(target: "executor", "compute_phase: skipped reason {:?}", skipped.reason);
                if is_ext_msg {
                    fail!(ExecutorError::ExtMsgComputeSkipped(skipped.reason.clone()))
                }
                None
            }
        };

        description.aborted = match description.action.as_ref() {
            Some(phase) => {
                log::debug!(target: "executor",
                    "action_phase: present: success={}, err_code={}", phase.success, phase.result_code);
                match phase.status_change {
                    AccStatusChange::Deleted => *account = Account::default(),
                    _ => ()
                }
                !phase.success
            }
            None => {
                log::debug!(target: "executor", "action_phase: none");
                true
            }
        };

        log::debug!(target: "executor", "Desciption.aborted {}", description.aborted);
        tr.set_end_status(account.status());
        if description.aborted && !is_ext_msg && bounce {
            if !action_phase_processed {
                log::debug!(target: "executor", "bounce_phase");
                let my_addr = account.get_addr().unwrap_or(&in_msg.dst().ok_or_else(|| ExecutorError::TrExecutorError(
                    format!("Or account address or in_msg dst address should be present")
                ))?).clone();
                description.bounce = match self.bounce_phase(msg_balance.clone(), &compute_phase_gas_fees, in_msg, &mut tr, &my_addr) {
                    Some((bounce_ph, Some(bounce_msg))) => {
                        out_msgs.push(bounce_msg);
                        Some(bounce_ph)
                    }
                    Some((bounce_ph, None)) => Some(bounce_ph),
                    None => None
                };
            }
            // if money can be returned to sender
            // restore account balance - storage fee
            if let Some(TrBouncePhase::Ok(_)) = description.bounce {
                log::debug!(target: "executor", "restore balance {} => {}", acc_balance.grams, original_acc_balance.grams);
                acc_balance = original_acc_balance;
            } else {
                if account.is_none() {
                    // if bounced message was not created, then we must burn money because there is nowhere else to put them
                    let mut tr_fee = tr.total_fees().clone();
                    tr_fee.add(&msg_balance)?;
                    tr.set_total_fees(tr_fee);
                }
            }
        }
        log::debug!(target: "executor", "set balance {}", acc_balance.grams);
        account.set_balance(acc_balance);
        log::debug!(target: "executor", "add messages");
        let lt = self.add_messages(&mut tr, out_msgs, params.last_tr_lt)?;
        account.set_last_tr_time(lt);
        tr.write_description(&TransactionDescr::Ordinary(description))?;

        Ok(tr)
    }
    fn ordinary_transaction(&self) -> bool { true }
    fn config(&self) -> &BlockchainConfig { &self.config }
    fn build_stack(&self, in_msg: Option<&Message>, account: &Account) -> Stack {
        let mut stack = Stack::new();
        let in_msg = match in_msg {
            Some(in_msg) => in_msg,
            None => return stack
        };
        let acc_balance = int!(account.balance().map(|value| value.grams.0).unwrap_or_default());
        let msg_balance = int!(in_msg.get_value().map(|value| value.grams.0).unwrap_or_default());
        let function_selector = boolean!(in_msg.is_inbound_external());
        let body_slice = in_msg.body().unwrap_or_default();
        let in_msg_cell = in_msg.serialize().unwrap_or_default();
        stack
            .push(acc_balance)
            .push(msg_balance)
            .push(StackItem::Cell(in_msg_cell))
            .push(StackItem::Slice(body_slice))
            .push(function_selector);

        stack
    }
}

impl DebugTransactionExecutor {
    pub fn new(config: BlockchainConfig, dbg_info:  &str, trace_level: TraceLevel) -> Self {
        let dbg_info = dbg_info.to_string();
        Self {
            config,
            dbg_info,
            trace_level,
        }
    }

    /// Implementation of transaction's computing phase.
    /// Evaluates new accout state and invokes TVM if account has contract code.
    fn compute_phase2(
        &self,
        msg: Option<&Message>,
        acc: &mut Account,
        acc_balance: &mut CurrencyCollection,
        msg_balance: &CurrencyCollection,
        state_libs: HashmapE, // masterchain libraries
        smc_info: SmartContractInfo,
        stack: Stack,
        is_masterchain: bool,
        is_special: bool,
        debug: bool,
    ) -> Result<(TrComputePhase, Option<Cell>, Option<Cell>)> {
        let mut result_acc = acc.clone();
        let mut vm_phase = TrComputePhaseVm::default();
        let init_code_hash = self.config().raw_config().has_capability(GlobalCapabilities::CapInitCodeHash);
        let is_external = if let Some(msg) = msg {
            if let Some(header) = msg.int_header() {
                log::debug!(target: "executor", "msg internal, bounce: {}", header.bounce);
                if result_acc.is_none() {
                    if let Some(new_acc) = account_from_message(msg, true, init_code_hash) {
                        result_acc = new_acc;
                        result_acc.set_last_paid(smc_info.unix_time());

                        // if there was a balance in message (not bounce), then account state at least become uninit
                        *acc = result_acc.clone();
                        acc.uninit_account();
                    }
                }
                false
            } else {
                log::debug!(target: "executor", "msg external");
                true
            }
        } else {
            debug_assert!(!result_acc.is_none());
            false
        };
        log::debug!(target: "executor", "acc balance: {}", acc_balance.grams);
        log::debug!(target: "executor", "msg balance: {}", msg_balance.grams);
        let is_ordinary = self.ordinary_transaction();
        let gas_config = self.config().get_gas_config(is_masterchain);
        let gas = init_gas(acc_balance.grams.0, msg_balance.grams.0, is_external, is_special, is_ordinary, gas_config);
        if gas.get_gas_limit() == 0 && gas.get_gas_credit() == 0 {
            log::debug!(target: "executor", "skip computing phase no gas");
            return Ok((TrComputePhase::skipped(ComputeSkipReason::NoGas), None, None))
        }

        let mut libs = vec![];
        if let Some(msg) = msg {
            if let Some(state_init) = msg.state_init() {
                libs.push(state_init.libraries().inner());
            }
            if let Some(reason) = compute_new_state(&mut result_acc, &acc_balance, msg, init_code_hash) {
                if !init_code_hash {
                    *acc = result_acc;
                }
                return Ok((TrComputePhase::skipped(reason), None, None))
            }
        };
        //code must present but can be empty (i.g. for uninitialized account)
        let code = result_acc.get_code().unwrap_or_default();
        let data = result_acc.get_data().unwrap_or_default();
        libs.push(result_acc.libraries().inner());
        libs.push(state_libs);

        vm_phase.gas_credit = match gas.get_gas_credit() as u32 {
            0 => None,
            value => Some(value.into())
        };
        vm_phase.gas_limit = (gas.get_gas_limit() as u64).into();

        let mut vm = VMSetup::new(code.into())
            .set_contract_info(smc_info)
            .set_stack(stack)
            .set_data(data)
            .set_libraries(libs)
            .set_gas(gas)
            .set_debug(debug)
            .create();
        match self.trace_level {
            TraceLevel::None => {},
            _ => {
                let dbg_info = match File::open(self.dbg_info.clone()) {
                    Ok(file ) => match serde_json::from_reader(file) {
                        Ok(info) => Some(info),
                        Err(e) => {
                            println!("serde failed: {}", e);
                            None
                        },
                    },
                    Err(e) =>  {
                        println!("open failed: {}", e);
                        None
                    }
                };
                if self.trace_level == TraceLevel::Minimal {
                    vm.set_trace_callback(move |_, info| { trace_callback_minimal(info, &dbg_info); });
                } else {
                    vm.set_trace_callback(move |_, info| { trace_callback(info, &dbg_info); });
                }
            }
        };
        //TODO: set vm_init_state_hash

        let result = vm.execute();
        log::trace!(target: "executor", "execute result: {:?}", result);
        let mut raw_exit_arg = None;
        match result {
            Err(err) => {
                log::debug!(target: "executor", "VM terminated with exception: {}", err);
                let exception = tvm_exception(err)?;
                vm_phase.exit_code = if let Some(code) = exception.custom_code() {
                    code
                } else {
                    !(exception.exception_code().unwrap_or(ExceptionCode::UnknownError) as i32)
                };
                vm_phase.exit_arg = match exception.value.as_integer().and_then(|value| value.into(std::i32::MIN..=std::i32::MAX)) {
                    Err(_) | Ok(0) => None,
                    Ok(exit_arg) => Some(exit_arg)
                };
                raw_exit_arg = Some(exception.value);
            }
            Ok(exit_code) => vm_phase.exit_code = exit_code
        };
        vm_phase.success = vm.get_committed_state().is_committed();
        log::debug!(target: "executor", "VM terminated with exit code {}", vm_phase.exit_code);

        // calc gas fees
        let gas = vm.get_gas();
        let credit = gas.get_gas_credit() as u32;
        //for external messages gas will not be exacted if VM throws the exception and gas_credit != 0
        let used = gas.get_gas_used() as u64;
        vm_phase.gas_used = used.into();
        if credit != 0 {
            if is_external {
                fail!(ExecutorError::NoAcceptError(vm_phase.exit_code, raw_exit_arg))
            }
            vm_phase.gas_fees = Grams::zero();
        } else { // credit == 0 means contract accepted
            let gas_fees = if is_special { 0 } else { gas_config.calc_gas_fee(used) };
            vm_phase.gas_fees = gas_fees.into();
        };

        log::debug!(
            target: "executor",
            "gas after: gl: {}, gc: {}, gu: {}, fees: {}",
            gas.get_gas_limit() as u64, credit, used, vm_phase.gas_fees
        );

        //set mode
        vm_phase.mode = 0;
        vm_phase.vm_steps = vm.steps();
        //TODO: vm_final_state_hash
        log::debug!(target: "executor", "acc_balance: {}, gas fees: {}", acc_balance.grams, vm_phase.gas_fees);
        if !acc_balance.grams.sub(&vm_phase.gas_fees)? {
            log::debug!(target: "executor", "can't sub funds: {} from acc_balance: {}", vm_phase.gas_fees, acc_balance.grams);
        }

        let new_data = if let StackItem::Cell(cell) = vm.get_committed_state().get_root() {
            Some(cell)
        } else {
            log::debug!(target: "executor", "invalid contract, it must be cell in c4 register");
            vm_phase.success = false;
            None
        };

        let out_actions = if let StackItem::Cell(root_cell) = vm.get_committed_state().get_actions() {
            Some(root_cell)
        } else {
            log::debug!(target: "executor", "invalid contract, it must be cell in c5 register");
            vm_phase.success = false;
            None
        };

        *acc = result_acc;
        Ok((TrComputePhase::Vm(vm_phase), out_actions, new_data))
    }
}


fn init_gas(acc_balance: u128, msg_balance: u128, is_external: bool, is_special: bool, is_ordinary: bool, gas_info: &GasLimitsPrices) -> Gas {
    let gas_max = if is_special {
        gas_info.special_gas_limit
    } else {
        std::cmp::min(gas_info.gas_limit, gas_info.calc_gas(acc_balance))
    };
    let mut gas_credit = 0;
    let gas_limit = if !is_ordinary {
        gas_max
    } else {
        if is_external {
            gas_credit = std::cmp::min(gas_info.gas_credit, gas_max);
        }
        std::cmp::min(gas_max, gas_info.calc_gas(msg_balance))
    };
    log::debug!(
        target: "executor",
        "gas before: gm: {}, gl: {}, gc: {}, price: {}",
        gas_max, gas_limit, gas_credit, gas_info.get_real_gas_price()
    );
    Gas::new(gas_limit as i64, gas_credit as i64, gas_max as i64, gas_info.get_real_gas_price() as i64)
}

/// Calculate new account according to inbound message.
/// If message has no value, account will not created.
/// If hash of state_init is equal to account address (or flag check address is false), account will be active.
/// Otherwise, account will be nonexist or uninit according bounce flag: if bounce, account will be uninit that save money.
fn account_from_message(msg: &Message, check_address: bool, init_code_hash: bool) -> Option<Account> {
    let hdr = msg.int_header()?;
    if hdr.value().grams.is_zero() {
        log::trace!(target: "executor", "The message has no money");
        return None
    }
    if let Some(init) = msg.state_init() {
        if init.code().is_some() {
            if !check_address || (init.hash().ok()? == hdr.dst.address()) {
                return Account::active_by_init_code_hash(hdr.dst.clone(), hdr.value().clone(), 0, init.clone(), init_code_hash).ok();
            } else if check_address {
                log::trace!(
                    target: "executor",
                    "Cannot construct account from message with hash {:x} because the destination address does not math with hash message code",
                    msg.hash().unwrap()
                );
            }
        }
    }
    if hdr.bounce {
        log::trace!(target: "executor", "Account will not be created. Value of {:x} message will be returned", msg.hash().unwrap());
        None
    } else {
        Some(Account::uninit(hdr.dst.clone(), 0, 0, hdr.value().clone()))
    }
}


/// Calculate new account state according to inbound message and current account state.
/// If account does not exist - it can be created with uninitialized state.
/// If account is uninitialized - it can be created with active state.
/// If account exists - it can be frozen.
/// Returns computed initial phase.
fn compute_new_state(acc: &mut Account, acc_balance: &CurrencyCollection, in_msg: &Message, init_code_hash: bool) -> Option<ComputeSkipReason> {
    log::debug!(target: "executor", "compute_account_state");
    match acc.status() {
        AccountStatus::AccStateNonexist => {
            log::error!(target: "executor", "account must exist");
            Some(ComputeSkipReason::BadState)
        }
        //Account exists, but can be in different states.
        AccountStatus::AccStateActive => {
            //account is active, just return it
            log::debug!(target: "executor", "account state: AccountActive");
            None
        }
        AccountStatus::AccStateUninit => {
            log::debug!(target: "executor", "AccountUninit");
            if let Some(state_init) = in_msg.state_init() {
                // if msg is a constructor message then
                // borrow code and data from it and switch account state to 'active'.
                log::debug!(target: "executor", "message for uninitialized: activated");
                match acc.try_activate_by_init_code_hash(state_init, init_code_hash) {
                    Err(err) => {
                        log::debug!(target: "executor", "reason: {}", err);
                        Some(ComputeSkipReason::NoState)
                    }
                    Ok(_) => None
                }
            } else {
                log::debug!(target: "executor", "message for uninitialized: skip computing phase");
                Some(ComputeSkipReason::NoState)
            }
        }
        AccountStatus::AccStateFrozen => {
            log::debug!(target: "executor", "AccountFrozen");
            //account balance was credited and if it positive after that
            //and inbound message bear code and data then make some check and unfreeze account
            if !acc_balance.grams.is_zero() {
                if let Some(state_init) = in_msg.state_init() {
                    log::debug!(target: "executor", "message for frozen: activated");
                    return match acc.try_activate_by_init_code_hash(state_init, init_code_hash) {
                        Err(err) => {
                            log::debug!(target: "executor", "reason: {}", err);
                            Some(ComputeSkipReason::NoState)
                        }
                        Ok(_) => None
                    }
                }
            }
            //skip computing phase, because account is frozen (bad state)
            log::debug!(target: "executor", "account is frozen (bad state): skip computing phase");
            Some(ComputeSkipReason::NoState)
        }
    }
}

fn trace_callback(info: &EngineTraceInfo, debug_info: &Option<DbgInfo>) {
    if info.info_type == EngineTraceInfoType::Dump {
        log::info!(target: "tvm", "{}", info.cmd_str);
        return;
    }

    log::info!(target: "tvm", "{}: {}",
             info.step,
             info.cmd_str
    );
    log::info!(target: "tvm", "{} {}",
             info.cmd_code.remaining_bits(),
             info.cmd_code.to_hex_string()
    );

    log::info!(target: "tvm", "\nGas: {} ({})",
             info.gas_used,
             info.gas_cmd
    );

    let position = get_position(info, debug_info);
    if position.is_some() {
        log::info!(target: "tvm", "Position: {}", position.unwrap());
    } else {
        log::info!(target: "tvm", "Position: Undefined");
    }

    log::info!(target: "tvm", "\n--- Stack trace ------------------------");
    for item in info.stack.iter() {
        log::info!(target: "tvm", "{}", item);
    }
    log::info!(target: "tvm", "----------------------------------------\n");
}


fn trace_callback_minimal(info: &EngineTraceInfo, debug_info: &Option<DbgInfo>) {
    let position = match get_position(info, debug_info) {
        Some(position) => position,
        _ => "".to_string()
    };
    log::info!(target: "tvm", "{} {} {} {} {}", info.step, info.gas_used, info.gas_cmd, info.cmd_str, position);
}

fn get_position(info: &EngineTraceInfo, debug_info: &Option<DbgInfo>) -> Option<String> {
    if let Some(debug_info) = debug_info {
        let cell_hash = info.cmd_code.cell().repr_hash();
        let offset = info.cmd_code.pos();
        let position = match debug_info.get(&cell_hash) {
            Some(offset_map) => match offset_map.get(&offset) {
                Some(pos) => format!("{}:{}", pos.filename, pos.line),
                None => String::from("-:0 (offset not found)")
            },
            None => String::from("-:0 (cell hash not found)")
        };
        return Some(position)
    }
    None
}