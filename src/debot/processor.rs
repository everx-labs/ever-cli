use serde_json::Value;
use super::{ApproveKind, DebotManifest, ChainLink};
use std::vec::IntoIter;
use ton_client::debot::DebotActivity;
use ton_client::abi::{CallSet};

#[derive(Debug)]
pub enum ProcessorError {
    InterfaceCallNeeded,
    NoMoreChainlinks,
    UnexpectedChainLinkKind,
    UnexpectedInterface,
    UnexpectedMethod,
    InteractiveApproveNeeded,
    // TODO: 
    // UnexpectedApproveKind,
}

pub struct ManifestProcessor {
    manifest: DebotManifest,
    chain_iter: IntoIter<ChainLink>,
}

impl ManifestProcessor {
    pub fn new(mut manifest: DebotManifest ) -> Self {
        let chain_vec = std::mem::take(&mut manifest.chain);
        Self { manifest, chain_iter: chain_vec.into_iter() }
    }

    pub fn print(&self, message: &str) {
        if self.manifest.interactive {
            println!("{}", message);
        }
    }

    pub fn initial_msg(&self) -> Option<String> {
        self.manifest.init_msg.clone()
    }

    pub fn initial_call_set(&self) -> Option<CallSet> {
        if self.manifest.init_msg.is_some() {
            return None;
        }
        if self.is_default_start() {
            return None;
        }
        match &self.manifest.init_args {
            Some(args) => CallSet::some_with_function_and_input(&self.manifest.init_method, args.clone()),
            None => CallSet::some_with_function(&self.manifest.init_method),
        }
        
    }

    pub fn is_default_start(&self) -> bool {
        self.manifest.init_method == "start"
    }

    pub fn next_input(
        &mut self,
        in_interface: &str,
        in_method: &str,
        _in_params: &Value
    ) -> Result<Option<Value>, ProcessorError> {
        let chlink = self.chain_iter.next().ok_or(
            if self.manifest.interactive {
                ProcessorError::InterfaceCallNeeded
            } else {
                ProcessorError::NoMoreChainlinks
            }
        )?;
        
        match chlink {
            ChainLink::Input {interface, method, params, mandatory: _} => {
                if interface != in_interface {
                    Err(ProcessorError::UnexpectedInterface)
                } else if method != in_method {
                    Err(ProcessorError::UnexpectedMethod)
                } else {
                    Ok(params.clone())
                }
            },
            _ => Err(ProcessorError::UnexpectedChainLinkKind),
        }
    }

    pub fn next_signing_box(&mut self) -> Result<u32, ProcessorError> {
        let chlink = self.chain_iter.next().ok_or(
            if self.manifest.interactive {
                ProcessorError::InterfaceCallNeeded
            } else {
                ProcessorError::NoMoreChainlinks
            }
        )?;

        match chlink {
            ChainLink::SigningBox {handle} => {
                Ok(handle)
            },
            _ => Err(ProcessorError::UnexpectedChainLinkKind),
        }
    }

    pub fn next_approve(&mut self, activity: &DebotActivity) -> Result<bool, ProcessorError> {
        
        let app_kind = match activity {
            DebotActivity::Transaction {..} => ApproveKind::ApproveOnchainCall,
        };
        let auto_approve = self.manifest.auto_approve.as_ref().and_then(|vec| {
            Some(vec.iter().find(|x| **x == app_kind).is_some())
        });

        let chlink = self.chain_iter.next();
        if chlink.is_none() {
            if auto_approve.is_some() {
                return Ok(auto_approve.unwrap());
            } else {
                if self.manifest.interactive {
                    return Err(ProcessorError::InteractiveApproveNeeded);
                } else {
                    return Ok(false);
                }
            }
        }

        // TODO: ?
        let chlink = chlink.unwrap();
        match chlink {
            ChainLink::OnchainCall { approve, iflq: _, ifeq: _ } => {
                match activity {
                    DebotActivity::Transaction {msg: _, dst: _, out: _, fee: _, setcode: _, signkey: _, signing_box_handle: _} => {
                        Ok(approve.clone())
                    }
                }
            },
            _ => Err(ProcessorError::UnexpectedChainLinkKind)
        }
    }
}

   