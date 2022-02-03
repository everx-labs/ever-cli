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

use ton_executor::{
    BlockchainConfig, CalcMsgFwdFees, ExecutorError, TransactionExecutor, ExecuteParams, VMSetup
};

use std::sync::{atomic::Ordering, Arc};
use ton_block::{AddSub, Grams, Serializable, Account, AccStatusChange, CommonMsgInfo, Message, Transaction, TransactionDescrOrdinary, TransactionDescr, TrComputePhase, TrBouncePhase, CurrencyCollection, GlobalCapabilities, ComputeSkipReason, TrComputePhaseVm, AccountStatus, GasLimitsPrices, TrActionPhase, OutActions, Deserializable, OutAction, SENDMSG_ALL_BALANCE, MASTERCHAIN_ID, BASE_WORKCHAIN_ID, MsgAddressInt, SENDMSG_REMAINING_MSG_BALANCE, SENDMSG_VALID_FLAGS, SENDMSG_DELETE_IF_EMPTY, SENDMSG_IGNORE_ERROR, SENDMSG_PAY_FEE_SEPARATELY, RESERVE_VALID_MODES, RESERVE_PLUS_ORIG, RESERVE_REVERSE, RESERVE_IGNORE_ERROR, RESERVE_ALL_BUT, WorkchainFormat, AnycastInfo};
use ton_types::{error, fail, Result, HashmapE, Cell, ExceptionCode, UInt256};
use ton_vm::{boolean, int, stack::{Stack, StackItem, integer::IntegerData}, SmartContractInfo};
use ton_vm::error::tvm_exception;
use ton_vm::executor::gas::gas_state::Gas;
use ton_block::GetRepresentationHash;
use ton_labs_assembler::DbgInfo;
use std::fs::File;
use ton_block::MsgAddressInt::{AddrStd, AddrVar};
use ton_vm::executor::{EngineTraceInfo, EngineTraceInfoType};
// use crate::decode::msg_printer::serialize_msg;

const RESULT_CODE_ACTIONLIST_INVALID:            i32 = 32;
const RESULT_CODE_TOO_MANY_ACTIONS:              i32 = 33;
const RESULT_CODE_UNKNOWN_OR_INVALID_ACTION:     i32 = 34;
const RESULT_CODE_INCORRECT_SRC_ADDRESS:         i32 = 35;
const RESULT_CODE_INCORRECT_DST_ADDRESS:         i32 = 36;
const RESULT_CODE_NOT_ENOUGH_GRAMS:              i32 = 37;
const RESULT_CODE_NOT_ENOUGH_EXTRA:              i32 = 38;
const RESULT_CODE_INVALID_BALANCE:               i32 = 40;
const RESULT_CODE_BAD_ACCOUNT_STATE:             i32 = 41;
const RESULT_CODE_UNSUPPORTED:                   i32 = -1;

const MAX_ACTIONS: usize = 255;

const MAX_MSG_BITS: usize = 1 << 21;
const MAX_MSG_CELLS: usize = 1 << 13;

#[derive(PartialEq, Clone)]
pub enum TraceLevel {
    Full,
    Minimal,
    None
}

pub struct DebugTransactionExecutor {
    config: BlockchainConfig,
    dbg_info: Option<String>,
    trace_level: TraceLevel,
    is_getter: bool,
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

        let mut acc_balance = account.balance().cloned().unwrap_or_default();
        let mut msg_balance = in_msg.get_value().cloned().unwrap_or_default();
        let ihr_delivered = false;  // ihr is disabled because it does not work
        if !ihr_delivered {
            msg_balance.grams.0 += in_msg.int_header().map_or(0, |h| h.ihr_fee.0);
        }
        log::debug!(target: "executor", "acc_balance: {}, msg_balance: {}, credit_first: {}",
            acc_balance.grams, msg_balance.grams, !bounce);

        let is_special = self.config.is_special_account(account_address)?;
        let lt = std::cmp::max(
            account.last_tr_time().unwrap_or(0),
            std::cmp::max(params.last_tr_lt.load(Ordering::Relaxed), in_msg.lt().unwrap_or(0) + 1)
        );
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
            description.credit_ph = match self.credit_phase(account, &mut tr, &mut msg_balance, &mut acc_balance) {
                Ok(credit_ph) => Some(credit_ph),
                Err(e) => fail!(
                    ExecutorError::TrExecutorError(
                        format!("cannot create credit phase of a new transaction for smart contract for reason: {}", e)
                    )
                )
            };
        }
        description.storage_ph = match self.storage_phase(
            account,
            &mut acc_balance,
            &mut tr,
            is_masterchain,
            is_special
        ) {
            Ok(storage_ph) => Some(storage_ph),
            Err(e) => fail!(
                ExecutorError::TrExecutorError(
                    format!("cannot create storage phase of a new transaction for smart contract for reason {}", e)
                )
            )
        };

        if description.credit_first && msg_balance.grams.0 > acc_balance.grams.0 {
            msg_balance.grams.0 = acc_balance.grams.0;
        }

        log::debug!(target: "executor",
            "storage_phase: {}", if description.storage_ph.is_some() {"present"} else {"none"});
        let mut original_acc_balance = account.balance().cloned().unwrap_or_default();
        original_acc_balance.sub(tr.total_fees())?;

        if !description.credit_first && !is_ext_msg {
            description.credit_ph = match self.credit_phase(account, &mut tr, &mut msg_balance, &mut acc_balance) {
                Ok(credit_ph) => Some(credit_ph),
                Err(e) => fail!(
                    ExecutorError::TrExecutorError(
                        format!("cannot create credit phase of a new transaction for smart contract for reason: {}", e)
                    )
                )
            };
        }
        log::debug!(target: "executor",
            "credit_phase: {}", if description.credit_ph.is_some() {"present"} else {"none"});

        let last_paid = if !is_special {params.block_unixtime} else {0};
        account.set_last_paid(last_paid);

        let smci = self.build_contract_info(&acc_balance, account_address, params.block_unixtime, params.block_lt, lt, params.seed_block);
        let mut stack = Stack::new();
        stack
            .push(int!(acc_balance.grams.0))
            .push(int!(msg_balance.grams.0))
            .push(StackItem::Cell(in_msg_cell))
            .push(StackItem::Slice(in_msg.body().unwrap_or_default()))
            .push(boolean!(is_ext_msg));
        log::debug!(target: "executor", "compute_phase");
        let (compute_ph, actions, new_data) = match self.debug_compute_phase(
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
        ) {
            Ok((compute_ph, actions, new_data)) => (compute_ph, actions, new_data),
            Err(e) =>
                if let Some(e) = e.downcast_ref::<ExecutorError>() {
                    match e {
                        ExecutorError::NoAcceptError(num, stack) => fail!(
                            ExecutorError::NoAcceptError(*num, stack.clone())
                        ),
                        _ => fail!("Unknown error")
                    }
                } else {
                    fail!(ExecutorError::TrExecutorError(e.to_string()))
                }
        };
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
                    match self.debug_action_phase(
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
                        Ok((action_ph, msgs)) => {
                            out_msgs = msgs;
                            Some(action_ph)
                        }
                        Err(e) => fail!(
                            ExecutorError::TrExecutorError(
                                format!("cannot create action phase of a new transaction for smart contract for reason {}", e)
                            )
                        )
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
                    AccStatusChange::Deleted => {
                        *account = Account::default();
                        description.destroyed = true;
                    },
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
        if description.aborted && !is_ext_msg && bounce {
            if !action_phase_processed {
                log::debug!(target: "executor", "bounce_phase");
                let my_addr = account.get_addr().unwrap_or(&in_msg.dst().ok_or_else(|| ExecutorError::TrExecutorError(
                    format!("Or account address or in_msg dst address should be present")
                ))?).clone();
                description.bounce = match self.bounce_phase(
                    msg_balance.clone(),
                    &mut acc_balance,
                    &compute_phase_gas_fees,
                    in_msg,
                    &mut tr,
                    &my_addr
                ) {
                    Ok((bounce_ph, Some(bounce_msg))) => {
                        out_msgs.push(bounce_msg);
                        Some(bounce_ph)
                    }
                    Ok((bounce_ph, None)) => Some(bounce_ph),
                    Err(e) => fail!(
                        ExecutorError::TrExecutorError(
                            format!("cannot create bounce phase of a new transaction for smart contract for reason {}", e)
                        )
                    )
                };
            }
            // if money can be returned to sender
            // restore account balance - storage fee
            if let Some(TrBouncePhase::Ok(_)) = description.bounce {
                log::debug!(target: "executor", "restore balance {} => {}", acc_balance.grams, original_acc_balance.grams);
                acc_balance = original_acc_balance;
                if (account.status() == AccountStatus::AccStateUninit) && acc_balance.is_zero()? {
                    *account = Account::default();
                }
            } else {
                if account.is_none() && !acc_balance.is_zero()? {
                    *account = Account::uninit(
                        in_msg.dst().ok_or(
                            ExecutorError::TrExecutorError(
                                "cannot create bounce phase of a new transaction for smart contract".to_string()
                            )
                        )?.clone(),
                        0,
                        last_paid,
                        acc_balance.clone()
                    );
                }
            }
        }
        if (account.status() == AccountStatus::AccStateUninit) && acc_balance.is_zero()? {
            *account = Account::default();
        }
        tr.set_end_status(account.status());
        log::debug!(target: "executor", "set balance {}", acc_balance.grams);
        account.set_balance(acc_balance);
        log::debug!(target: "executor", "add messages");
        params.last_tr_lt.store(lt, Ordering::Relaxed);
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
    pub fn new(
        config: BlockchainConfig,
        dbg_info: Option<String>,
        trace_level: TraceLevel,
        is_getter: bool,
    ) -> Self {
        Self {
            config,
            dbg_info,
            trace_level,
            is_getter,
        }
    }

    /// Implementation of transaction's computing phase.
    /// Evaluates new account state and invokes TVM if account has contract code.
    fn debug_compute_phase(
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
        let init_code_hash = self.config().has_capability(GlobalCapabilities::CapInitCodeHash);
        let is_external = if let Some(msg) = msg {
            if let Some(header) = msg.int_header() {
                log::debug!(target: "executor", "msg internal, bounce: {}", header.bounce);
                if result_acc.is_none() {
                    if let Some(new_acc) = account_from_message(msg, true, init_code_hash) {
                        result_acc = new_acc;
                        result_acc.set_last_paid(if !is_special {smc_info.unix_time()} else {0});

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
        if acc_balance.grams.is_zero() {
            log::debug!(target: "executor", "skip computing phase no gas");
            return Ok((TrComputePhase::skipped(ComputeSkipReason::NoGas), None, None))
        }
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

        vm_phase.gas_credit = match gas.get_gas_credit() as u32 {
            0 => None,
            value => Some(value.into())
        };
        vm_phase.gas_limit = (gas.get_gas_limit() as u64).into();

        if result_acc.get_code().is_none() {
            vm_phase.exit_code = -13;
            if is_external && !self.is_getter {
                fail!(ExecutorError::NoAcceptError(vm_phase.exit_code, None))
            } else {
                vm_phase.exit_arg = None;
                vm_phase.success = false;
                vm_phase.gas_fees = Grams::from(if is_special { 0 } else { gas_config.calc_gas_fee(0) });
                if !acc_balance.grams.sub(&vm_phase.gas_fees)? {
                    log::debug!(target: "executor", "can't sub funds: {} from acc_balance: {}", vm_phase.gas_fees, acc_balance.grams);
                    fail!("can't sub funds: from acc_balance")
                }
                *acc = result_acc;
                return Ok((TrComputePhase::Vm(vm_phase), None, None));
            }
        }
        let code = result_acc.get_code().unwrap_or_default();
        let data = result_acc.get_data().unwrap_or_default();
        libs.push(result_acc.libraries().inner());
        libs.push(state_libs);

        let mut vm = VMSetup::new(code.into())
            .set_contract_info(smc_info, self.config().raw_config().has_capability(ton_block::GlobalCapabilities::CapInitCodeHash))?
            .set_stack(stack)
            .set_data(data)?
            .set_libraries(libs)
            .set_gas(gas)
            .set_debug(debug)
            .create();

        match self.trace_level {
            TraceLevel::None => {},
            _ => {
                let dbg_info = match self.dbg_info.clone() {
                    Some(dbg_info) => {
                        match File::open(dbg_info) {
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
                        }
                    },
                    _ => None
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
                    let mut error_code = exception.exception_code().unwrap_or(ExceptionCode::UnknownError) as i32;
                    let out_of_gas_code = ExceptionCode::OutOfGas as i32;
                    if error_code == out_of_gas_code {
                        error_code = !(out_of_gas_code as i32); // correct error code according cpp code
                    }
                    error_code
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
            if is_external && !self.is_getter {
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
            log::error!(target: "executor", "This situation is unreachable: can't sub funds: {} from acc_balance: {}", vm_phase.gas_fees, acc_balance.grams);
            fail!("can't sub funds: from acc_balance")
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
    
    /// Implementation of transaction's action phase.
    /// If computing phase is successful then action phase is started.
    /// If TVM invoked in computing phase returned some output actions, 
    /// then they will be added to transaction's output message list.
    /// Total value from all outbound internal messages will be collected and
    /// substracted from account balance. If account has enough funds this 
    /// will be succeded, otherwise action phase is failed, transaction will be
    /// marked as aborted, account changes will be rollbacked.
    fn debug_action_phase(
        &self,
        tr: &mut Transaction,
        acc: &mut Account,
        original_acc_balance: &CurrencyCollection,
        acc_balance: &mut CurrencyCollection,
        msg_remaining_balance: &mut CurrencyCollection,
        compute_phase_fees: &Grams,
        actions_cell: Cell,
        new_data: Option<Cell>,
        is_special: bool,
    ) -> Result<(TrActionPhase, Vec<Message>)> {
        let mut acc_copy = acc.clone();
        let mut acc_remaining_balance = acc_balance.clone();
        let mut phase = TrActionPhase::default();
        let mut total_reserved_value = CurrencyCollection::default();
        let mut actions = match OutActions::construct_from_cell(actions_cell) {
            Err(err) => {
                log::debug!(
                    target: "executor", 
                    "cannot parse action list: format is invalid, err: {}", 
                    err
                );
                // Here you can select only one of 2 error codes: RESULT_CODE_UNKNOWN_OR_INVALID_ACTION or RESULT_CODE_ACTIONLIST_INVALID
                phase.result_code = RESULT_CODE_UNKNOWN_OR_INVALID_ACTION;
                return Ok((phase, vec![]))
            }
            Ok(actions) => actions
        };

        if actions.len() > MAX_ACTIONS {
            log::debug!(target: "executor", "too many actions: {}", actions.len());
            phase.result_code = RESULT_CODE_TOO_MANY_ACTIONS;
            return Ok((phase, vec![]))
        }
        phase.action_list_hash = actions.hash()?;
        phase.tot_actions = actions.len() as i16;

        let process_err_code = |mut err_code: i32, i: usize, phase: &mut TrActionPhase| {
            if err_code == -1 {
                err_code = RESULT_CODE_UNKNOWN_OR_INVALID_ACTION;
            }
            if err_code != 0 {
                log::debug!(target: "executor", "action failed: error_code={}", err_code);
                phase.valid = true;
                phase.result_code = err_code;
                if i != 0 {
                    phase.result_arg = Some(i as i32);
                }
                if err_code == RESULT_CODE_NOT_ENOUGH_GRAMS || err_code == RESULT_CODE_NOT_ENOUGH_EXTRA {
                    phase.no_funds = true;
                }
                true
            } else {
                false
            }
        };

        let mut account_deleted = false;

        let mut out_msgs0 = vec![];
        let my_addr = acc_copy.get_addr().ok_or(error!("Not found account address"))?.clone();
        let workchains = match self.config().raw_config().workchains() {
            Ok(workchains) => workchains,
            Err(e) => {
                log::error!(target: "executor", "get workchains error {}", e);
                fail!("get workchains error {}", e)
            }
        };

        for (i, action) in actions.iter_mut().enumerate() {
            log::debug!(target: "executor", "\nAction #{}", i);
            log::debug!(target: "executor", "Type: {}", action_type(action));
            log::debug!(target: "executor", "Initial balance: {}", balance_to_string(&acc_remaining_balance));
            let mut init_balance = acc_remaining_balance.clone();
            let err_code = match std::mem::replace(action, OutAction::None) {
                OutAction::SendMsg{ mode, mut out_msg } => {
                    if let Some(header) = out_msg.int_header() {
                        // let src_workchain_id = my_addr.workchain_id();
                        match header.dst.workchain_id() {
                            // allow to send only to -1 from 0 or -1
                            MASTERCHAIN_ID => {
                                if my_addr.workchain_id() != MASTERCHAIN_ID && my_addr.workchain_id() != BASE_WORKCHAIN_ID {
                                    log::error!(target: "executor", "masterchain cannot accept from {} workchain", my_addr.workchain_id());
                                    fail!("masterchain cannot accept from {} workchain", my_addr.workchain_id())
                                }
                            }
                            // allow to send to self or from master to any which is possible
                            workchain_id => {
                                if my_addr.workchain_id() == workchain_id || my_addr.workchain_id() == MASTERCHAIN_ID {
                                    match workchains.get(&workchain_id) {
                                        Ok(None) => {
                                            log::error!(target: "executor", "workchain {} is not deployed", workchain_id);
                                            fail!("workchain {} is not deployed", workchain_id)
                                        }
                                        Err(e) => {
                                            log::error!(target: "executor", "workchain {} cannot be get {}", workchain_id, e);
                                            fail!("workchain {} cannot be get {}", workchain_id, e)
                                        }
                                        Ok(Some(descr)) => {
                                            if !descr.accept_msgs {
                                                log::error!(target: "executor", "cannot send message from {} to {} it doesn't accept", header.src, header.dst);
                                                fail!("cannot send message from {} to {} it doesn't accept", header.src, header.dst)
                                            }
                                        }
                                    }
                                } else {
                                    log::error!(target: "executor", "cannot send message from {} to {} it doesn't allow yet", header.src, header.dst);
                                    fail!("cannot send message from {} to {} it doesn't allow yet", header.src, header.dst)
                                }
                            }
                        }
                    }
                    if (mode & SENDMSG_ALL_BALANCE) != 0 {
                        out_msgs0.push((i, mode, out_msg));
                        log::debug!(target: "executor", "Message with flag `SEND_ALL_BALANCE` it will be sent last. Skip it for now.");
                        continue
                    }
                    let result = outmsg_action_handler(
                        &mut phase,
                        mode,
                        &mut out_msg,
                        &mut acc_remaining_balance,
                        msg_remaining_balance,
                        compute_phase_fees,
                        self.config(),
                        is_special,
                        &my_addr,
                        &total_reserved_value,
                        &mut account_deleted
                    );
                    match result {
                        Ok(_) => {
                            phase.msgs_created += 1;
                            out_msgs0.push((i, mode, out_msg));
                            0
                        }
                        Err(code) => code
                    }
                }
                OutAction::ReserveCurrency{ mode, value } => {
                    match reserve_action_handler(mode, &value, original_acc_balance, &mut acc_remaining_balance) {
                        Ok(reserved_value) => {
                            phase.spec_actions += 1;
                            match total_reserved_value.add(&reserved_value) {
                                Ok(_) => 0,
                                Err(_) => RESULT_CODE_INVALID_BALANCE
                            }
                        }
                        Err(code) => code
                    }
                }
                OutAction::SetCode{ new_code: code } => {
                    match setcode_action_handler(&mut acc_copy, code) {
                        None => {
                            phase.spec_actions += 1;
                            0
                        }
                        Some(code) => code
                    }
                }
                OutAction::ChangeLibrary{ mode, code, hash} => {
                    match change_library_action_handler(&mut acc_copy, mode, code, hash) {
                        None => {
                            phase.spec_actions += 1;
                            0
                        }
                        Some(code) => code
                    }
                }
                OutAction::None => RESULT_CODE_UNKNOWN_OR_INVALID_ACTION
            };
            log::debug!(target: "executor", "Final balance:   {}", balance_to_string(&acc_remaining_balance));
            init_balance.sub(&acc_remaining_balance)?;
            log::debug!(target: "executor", "Delta:           {}", balance_to_string(&(init_balance)));
            if process_err_code(err_code, i, &mut phase) {
                return Ok((phase, vec![]))
            }
        }

        let mut out_msgs = vec![];
        for (i, mode, mut out_msg) in out_msgs0.into_iter() {
            if (mode & SENDMSG_ALL_BALANCE) == 0 {
                out_msgs.push(out_msg);
                continue
            }
            log::debug!(target: "executor", "\nSend message with all balance:");
            log::debug!(target: "executor", "Initial balance: {}", balance_to_string(&acc_remaining_balance));
            let result = outmsg_action_handler(
                &mut phase,
                mode,
                &mut out_msg,
                &mut acc_remaining_balance,
                msg_remaining_balance,
                compute_phase_fees,
                self.config(),
                is_special,
                &my_addr,
                &total_reserved_value,
                &mut account_deleted
            );
            log::debug!(target: "executor", "Final balance:   {}", balance_to_string(&acc_remaining_balance));
            let err_code = match result {
                Ok(_) => {
                    phase.msgs_created += 1;
                    out_msgs.push(out_msg);
                    0
                }
                Err(code) => code
            };
            if process_err_code(err_code, i, &mut phase) {
                return Ok((phase, vec![]));
            }
        }

        //calc new account balance
        log::debug!(target: "executor", "\nReturn reserved balance:");
        log::debug!(target: "executor", "Initial:  {}", balance_to_string(&acc_remaining_balance));
        log::debug!(target: "executor", "Reserved: {}", balance_to_string(&total_reserved_value));
        if let Err(err) = acc_remaining_balance.add(&total_reserved_value) {
            log::debug!(target: "executor", "failed to add account balance with reserved value {}", err);
            fail!("failed to add account balance with reserved value {}", err)
        }

        log::debug!(target: "executor", "Final:    {}", balance_to_string(&acc_remaining_balance));

        if let Some(fee) = phase.total_action_fees.as_ref() {
            log::debug!(target: "executor", "\nTotal action fees: {}", fee);
            tr.add_fee_grams(fee)?;
        }

        if account_deleted {
            log::debug!(target: "executor", "\nAccount deleted");
            phase.status_change = AccStatusChange::Deleted;
        }
        phase.valid = true;
        phase.success = true;
        *acc_balance = acc_remaining_balance;
        *acc = acc_copy;
        if let Some(new_data) = new_data {
            acc.set_data(new_data);
        }
        Ok((phase, out_msgs))
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


fn outmsg_action_handler(
    phase: &mut TrActionPhase,
    mut mode: u8,
    msg: &mut Message,
    acc_balance: &mut CurrencyCollection,
    msg_balance: &mut CurrencyCollection,
    compute_phase_fees: &Grams,
    config: &BlockchainConfig,
    is_special: bool,
    my_addr: &MsgAddressInt,
    reserved_value: &CurrencyCollection,
    account_deleted: &mut bool
) -> std::result::Result<CurrencyCollection, i32> {
    // we cannot send all balance from account and from message simultaneously ?
    let invalid_flags = SENDMSG_REMAINING_MSG_BALANCE | SENDMSG_ALL_BALANCE;
    if  (mode & !SENDMSG_VALID_FLAGS) != 0 ||
        (mode & invalid_flags) == invalid_flags ||
        ((mode & SENDMSG_DELETE_IF_EMPTY) != 0 && (mode & SENDMSG_ALL_BALANCE) == 0)
    {
        log::error!(target: "executor", "outmsg mode has unsupported flags");
        return Err(RESULT_CODE_UNSUPPORTED);
    }
    let skip = if (mode & SENDMSG_IGNORE_ERROR) != 0 {
        None
    } else {
        Some(())
    };
    let (fwd_mine_fee, total_fwd_fees);
    let mut result_value; // to sub from acc_balance

    if let Some(new_src) = check_replace_src_addr(&msg.src(), my_addr) {
        msg.set_src_address(new_src.clone());
    } else {
        log::warn!(target: "executor", "Incorrect source address {:?}", msg.src());
        return Err(RESULT_CODE_INCORRECT_SRC_ADDRESS);
    }

    let fwd_prices = config.get_fwd_prices(msg.is_masterchain());
    let compute_fwd_fee = if is_special {
        Grams::default()
    } else {
        msg.serialize()
            .map(|cell| fwd_prices.fwd_fee(&cell))
            .map_err(|err| {
                log::error!(target: "executor", "cannot serialize message in action phase : {}", err);
                RESULT_CODE_ACTIONLIST_INVALID
            })?
    };

    if let Some(int_header) = msg.int_header_mut() {
        if let Some(new_dst) = check_rewrite_dest_addr(my_addr, &int_header.dst, config) {
            int_header.dst = new_dst;
        } else {
            log::warn!(target: "executor", "Incorrect destination address {}", int_header.dst);
            return Err(skip.map(|_| RESULT_CODE_INCORRECT_DST_ADDRESS).unwrap_or(0))
        }

        int_header.bounced = false;
        result_value = int_header.value.clone();

        if cfg!(feature = "ihr_disabled") {
            int_header.ihr_disabled = true;
        }
        if !int_header.ihr_disabled {
            let compute_ihr_fee = fwd_prices.ihr_fee(&compute_fwd_fee);
            if int_header.ihr_fee < compute_ihr_fee {
                int_header.ihr_fee = compute_ihr_fee
            }
        } else {
            int_header.ihr_fee = 0.into();
        }
        let fwd_fee = std::cmp::max(&int_header.fwd_fee, &compute_fwd_fee).clone();
        fwd_mine_fee = fwd_prices.mine_fee(&fwd_fee);
        total_fwd_fees = Grams::from(fwd_fee.0 + int_header.ihr_fee.0);

        let fwd_remain_fee = fwd_fee.0 - fwd_mine_fee.0;
        if (mode & SENDMSG_ALL_BALANCE) != 0 {
            //send all remaining account balance
            result_value = acc_balance.clone();
            int_header.value = acc_balance.clone();

            mode &= !SENDMSG_PAY_FEE_SEPARATELY;
        }
        if (mode & SENDMSG_REMAINING_MSG_BALANCE) != 0 {
            //send all remainig balance of inbound message
            result_value.add(msg_balance).ok();
            if (mode & SENDMSG_PAY_FEE_SEPARATELY) == 0 {
                if result_value.grams.0 < compute_phase_fees.0 {
                    return Err(skip.map(|_| RESULT_CODE_NOT_ENOUGH_GRAMS).unwrap_or(0))
                }
                result_value.grams.sub(&compute_phase_fees).map_err(|err| {
                    log::error!(target: "executor", "cannot subtract msg balance : {}", err);
                    RESULT_CODE_ACTIONLIST_INVALID
                })?;
            }
            int_header.value = result_value.clone();
        }
        if (mode & SENDMSG_PAY_FEE_SEPARATELY) != 0 {
            //we must pay the fees, sum them with msg value
            result_value.grams.0 += total_fwd_fees.0;
        } else if int_header.value.grams.0 < total_fwd_fees.0 {
            //msg value is too small, reciever cannot pay the fees
            log::warn!(
                target: "executor",
                "msg balance {} is too small, cannot pay fwd+ihr fees: {}",
                int_header.value.grams, total_fwd_fees
            );
            return Err(skip.map(|_| RESULT_CODE_NOT_ENOUGH_GRAMS).unwrap_or(0))
        } else {
            //reciever will pay the fees
            int_header.value.grams.0 -= total_fwd_fees.0;
        }

        //set evaluated fees and value back to msg
        int_header.fwd_fee = fwd_remain_fee.into();
    } else if msg.ext_out_header().is_some() {
        fwd_mine_fee = compute_fwd_fee.clone();
        total_fwd_fees = compute_fwd_fee.clone();
        result_value = CurrencyCollection::from_grams(compute_fwd_fee);
    } else {
        return Err(-1)
    }

    // log::debug!(target: "executor", "sub funds {} from {}", result_value, acc_balance.grams);
    if acc_balance.grams.0 < result_value.grams.0 {
        log::warn!(
            target: "executor",
            "account balance {} is too small, cannot send {}", acc_balance.grams, result_value.grams
        );
        return Err(skip.map(|_| RESULT_CODE_NOT_ENOUGH_GRAMS).unwrap_or(0))
    }
    match acc_balance.sub(&result_value) {
        Ok(false) | Err(_) => {
            log::warn!(
                target: "executor",
                "account balance {} is too small, cannot send {}", acc_balance, result_value
            );
            return Err(skip.map(|_| RESULT_CODE_NOT_ENOUGH_EXTRA).unwrap_or(0))
        }
        _ => ()
    }

    if (mode & SENDMSG_DELETE_IF_EMPTY) != 0
        && (mode & SENDMSG_ALL_BALANCE) != 0
        && acc_balance.grams.0 + reserved_value.grams.0 == 0 {
        *account_deleted = true;
    }

    // total fwd fees is sum of messages full fwd and ihr fees
    if total_fwd_fees.0 != 0 {
        phase.total_fwd_fees.get_or_insert(Grams::default()).0 += total_fwd_fees.0;
    }

    // total action fees is sum of messages fwd mine fees
    if fwd_mine_fee.0 != 0 {
        phase.total_action_fees.get_or_insert(Grams::default()).0 += fwd_mine_fee.0;
    }

    let msg_cell = msg.serialize().map_err(|err| {
        log::error!(target: "executor", "cannot serialize message in action phase : {}", err);
        RESULT_CODE_ACTIONLIST_INVALID
    })?;
    phase.tot_msg_size.append(&msg_cell);

    if phase.tot_msg_size.bits() as usize > MAX_MSG_BITS || phase.tot_msg_size.cells() as usize > MAX_MSG_CELLS {
        log::warn!(target: "executor", "message too large : bits: {}, cells: {}", phase.tot_msg_size.bits(), phase.tot_msg_size.cells());
        return Err(RESULT_CODE_INVALID_BALANCE);
    }

    if (mode & (SENDMSG_ALL_BALANCE | SENDMSG_REMAINING_MSG_BALANCE)) != 0 {
        *msg_balance = CurrencyCollection::default();
    }
    // log::debug!(target: "executor", "sub funds {} from {}", result_value, acc_balance.grams);
    // log::info!(target: "executor", "msg with flags: {} exports value {}", mode, result_value.grams.0);
    log::debug!(target: "executor", "Message details:");
    log::debug!(target: "executor", "\tFlag: {}", mode);
    log::debug!(target: "executor", "\tValue: {}", balance_to_string(&result_value));
    log::debug!(target: "executor", "\tSource: {}", msg.src()
        .map_or("None".to_string(), |addr| addr.to_string()));
    log::debug!(target: "executor", "\tDestination: {}", msg.dst()
        .map_or("None".to_string(), |addr| addr.to_string()));
    log::debug!(target: "executor", "\tBody: {}", msg.body()
        .map_or("None".to_string(), |data| data.to_string()));
    log::debug!(target: "executor", "\tStateInit: {}", msg.state_init()
        .map_or("None".to_string(), |_| "Present".to_string()));

    Ok(result_value)
}


/// Reserves some grams from accout balance.
/// Returns calculated reserved value. its calculation depends on mode.
/// Reduces balance by the amount of the reserved value.
fn reserve_action_handler(
    mode: u8,
    val: &CurrencyCollection,
    original_acc_balance: &CurrencyCollection,
    acc_remaining_balance: &mut CurrencyCollection,
) -> std::result::Result<CurrencyCollection, i32> {
    if mode & !RESERVE_VALID_MODES != 0 {
        return Err(RESULT_CODE_UNKNOWN_OR_INVALID_ACTION);
    }
    log::debug!(target: "executor", "Reserve with mode = {} value = {}", mode, balance_to_string(val));
    let mut reserved;
    if mode & RESERVE_PLUS_ORIG != 0 {
        // Append all currencies
        if mode & RESERVE_REVERSE != 0 {
            reserved = original_acc_balance.clone();
            let result = reserved.sub(val);
            match result {
                Err(_) => return Err(RESULT_CODE_INVALID_BALANCE),
                Ok(false) => return Err(RESULT_CODE_UNSUPPORTED),
                Ok(true) => ()
            }
        } else {
            reserved = val.clone();
            reserved.add(original_acc_balance).or(Err(RESULT_CODE_INVALID_BALANCE))?;
        }
    } else {
        if mode & RESERVE_REVERSE != 0 { // flag 8 without flag 4 unacceptable
            return Err(RESULT_CODE_UNKNOWN_OR_INVALID_ACTION);
        }
        reserved = val.clone();
    }
    if mode & RESERVE_IGNORE_ERROR != 0 {
        // Only grams
        reserved.grams.0 = std::cmp::min(reserved.grams.0, acc_remaining_balance.grams.0);
    }

    let mut remaining = acc_remaining_balance.clone();
    let result = remaining.sub(&reserved);
    match result {
        Err(_) => return Err(RESULT_CODE_INVALID_BALANCE),
        Ok(false) => return Err(RESULT_CODE_NOT_ENOUGH_GRAMS),
        Ok(true) => ()
    }
    std::mem::swap(&mut remaining, acc_remaining_balance);

    if mode & RESERVE_ALL_BUT != 0 {
        // swap all currencies
        std::mem::swap(&mut reserved, acc_remaining_balance);
    }

    Ok(reserved)
}

fn setcode_action_handler(acc: &mut Account, code: Cell) -> Option<i32> {
    log::debug!(target: "executor", "OutAction::SetCode {}", code);
    log::debug!(target: "executor", "Previous code hash: {}", acc.get_code().unwrap_or_default().repr_hash().to_hex_string());
    log::debug!(target: "executor", "New code hash:      {}", code.repr_hash().to_hex_string());
    match acc.set_code(code) {
        true => None,
        false => Some(RESULT_CODE_BAD_ACCOUNT_STATE)
    }
}

fn change_library_action_handler(acc: &mut Account, mode: u8, code: Option<Cell>, hash: Option<UInt256>) -> Option<i32> {
    let result = match (code, hash) {
        (Some(code), None) => {
            log::debug!(target: "executor", "OutAction::ChangeLibrary mode: {}, code: {}", mode, code);
            if mode == 0 { // TODO: Wrong codes. Look ton_block/out_actions::SET_LIB_CODE_REMOVE
                acc.delete_library(&code.repr_hash())
            } else {
                acc.set_library(code, (mode & 2) == 2)
            }
        }
        (None, Some(hash)) => {
            log::debug!(target: "executor", "OutAction::ChangeLibrary mode: {}, hash: {:x}", mode, hash);
            if mode == 0 {
                acc.delete_library(&hash)
            } else {
                acc.set_library_flag(&hash, (mode & 2) == 2)
            }
        }
        _ => false
    };
    match result {
        true => None,
        false => Some(RESULT_CODE_BAD_ACCOUNT_STATE)
    }
}

fn check_replace_src_addr<'a>(src: &'a Option<MsgAddressInt>, acc_addr: &'a MsgAddressInt) -> Option<&'a MsgAddressInt> {
    match src {
        None => Some(acc_addr),
        Some(src) => match src {
            AddrStd(_) => {
                if src != acc_addr {
                    None
                } else {
                    Some(src)
                }
            }
            AddrVar(_) => None
        }
    }
}


fn check_rewrite_dest_addr(src: &MsgAddressInt, dst: &MsgAddressInt, config: &BlockchainConfig) -> Option<MsgAddressInt> {
    let (mut anycast_opt, addr_len, workchain_id, address, mut repack);
    match dst {
        MsgAddressInt::AddrVar(dst) => {
            repack = dst.addr_len.0 == 256 && dst.workchain_id >= -128 && dst.workchain_id < 128;
            anycast_opt = dst.anycast.clone();
            addr_len = dst.addr_len.0 as u16;
            workchain_id = dst.workchain_id;
            address = dst.address.clone();
        }
        MsgAddressInt::AddrStd(dst) => {
            repack = false;
            anycast_opt = dst.anycast.clone();
            addr_len = 256;
            workchain_id = dst.workchain_id as i32;
            address = dst.address.clone();
        }
    }

    let is_masterchain = workchain_id == MASTERCHAIN_ID;
    if !is_masterchain {
        let workchains = config.raw_config().workchains().unwrap_or_default();
        if let Ok(Some(wc)) = workchains.get(&workchain_id) {
            if !wc.accept_msgs {
                log::debug!(target: "executor", "destination address belongs to workchain {} not accepting new messages", workchain_id);
                return None;
            }
            let (min_addr_len, max_addr_len, addr_len_step) = match wc.format {
                WorkchainFormat::Extended(wf) => (wf.min_addr_len(), wf.max_addr_len(), wf.addr_len_step()),
                WorkchainFormat::Basic(_) => (256, 256, 0)
            };
            if !is_valid_addr_len(addr_len, min_addr_len, max_addr_len, addr_len_step) {
                log::debug!(target: "executor", "destination address has length {} invalid for destination workchain {}", addr_len, workchain_id);
                return None
            }
        } else {
            log::debug!(target: "executor", "destination address contains unknown workchain_id {}", workchain_id);
            return None
        }
    }

    if let Some(anycast) = &anycast_opt {
        if is_masterchain {
            log::debug!(target: "executor", "masterchain address cannot be anycast");
            return None
        }
        match src.address().get_slice(0, anycast.depth.0 as usize) {
            Ok(pfx) => {
                if pfx != anycast.rewrite_pfx {
                    match AnycastInfo::with_rewrite_pfx(pfx) {
                        Ok(anycast) => {
                            repack = true;
                            anycast_opt = Some(anycast)
                        }
                        Err(err) => {
                            log::debug!(target: "executor", "Incorrect anycast prefix {}", err);
                            return None
                        }
                    }
                }
            }
            Err(err) => {
                log::debug!(target: "executor", "Incorrect src address {}", err);
                return None
            }
        }
    }

    if !repack {
        Some(dst.clone())
    } else if addr_len == 256 && workchain_id >= -128 && workchain_id < 128 {
        // repack as an addr_std
        MsgAddressInt::with_standart(anycast_opt, workchain_id as i8, address).ok()
    } else {
        // repack as an addr_var
        MsgAddressInt::with_variant(anycast_opt, workchain_id, address).ok()
    }
}

fn is_valid_addr_len(addr_len: u16, min_addr_len: u16, max_addr_len: u16, addr_len_step: u16) -> bool {
    (addr_len >= min_addr_len) && (addr_len <= max_addr_len) &&
        ((addr_len == min_addr_len) || (addr_len == max_addr_len) ||
            ((addr_len_step != 0) && ((addr_len - min_addr_len) % addr_len_step == 0)))
}

fn balance_to_string(balance: &CurrencyCollection) -> String {
    let value = balance.grams.0;
    format!("{}.{:03} {:03} {:03}      ({})",
            value / 1e9 as u128,
            (value % 1e9 as u128) / 1e6 as u128,
            (value % 1e6 as u128) / 1e3 as u128,
            value % 1e3 as u128,
            value,
    )
}

fn action_type(action: &OutAction) -> String {
    match action {
        OutAction::SendMsg {mode:_, out_msg:_} => "SendMsg".to_string(),
        OutAction::SetCode {new_code:_} => "SetCode".to_string(),
        OutAction::ReserveCurrency {mode:_, value:_} => "ReserveCurrency".to_string(),
        OutAction::ChangeLibrary {mode:_, code:_, hash:_} => "ChangeLibrary".to_string(),
        _ => "Unknown".to_string()
    }
}