use serde_json::Value;
use super::{ApproveKind, DebotManifest, ChainLink};
use std::sync::Arc;
use std::slice::Iter;
use ton_client::debot::DebotActivity;

#[derive(Debug)]
pub enum ProcessorError {
    InterfaceCallNeeded,
    NoMoreInputsInChain,
    UnexpectedChainLinkKind,
    UnexpectedInterface,
    UnexpectedMethod,
    UnexpectedApproveKind,
}

pub(crate) struct ChainProcessor<'a> {
    manifest: Arc<DebotManifest>,
    chain_iter: Iter<'a, ChainLink>,
}

impl ChainProcessor<'_> {
    pub fn new(manifest: Arc<DebotManifest> ) -> Self {
        Self { manifest, chain_iter: manifest.chain.iter() }
    }

    pub fn next_input(
        &mut self,
        in_interface: &str,
        in_method: &str,
        in_params: &Value
    ) -> Result<Option<Value>, ProcessorError> {
        let chlink = self.chain_iter.next().ok_or(
            if self.manifest.interactive {
                ProcessorError::InterfaceCallNeeded
            } else {
                ProcessorError::NoMoreInputsInChain
            }
        )?;
        
        match chlink {
            ChainLink::Input {interface, method, params, mandatory} => {
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

    pub fn next_approve(&mut self, activity: DebotActivity) -> Result<bool, ProcessorError> {
        
        let app_kind = match activity {
            DebotActivity::Transaction {..} => ApproveKind::ApproveOnchainCall,
            _ => panic!("not implemented")
        };
        let auto_approve = if let Some(approve_vec) = self.manifest.auto_approve {
            approve_vec.iter().find(move |x| x.clone() == app_kind).is_some()
        } else { false };

        let chlink = self.chain_iter.next();
        if chlink.is_none() && auto_approve {
            return Ok(true);
        }

        match chlink {
            ChainLink::OnchainCall { approve, iflq, ifeq } => {
                if let DebotActivity::Transaction {msg: _, dst: _, out, fee, setcode: _, signkey: _, signing_box_handle: _} = activity {
                    // TODO: check iflq ifeq
                    Ok(approve)
                } else {
                    Err(ProcessorError::UnexpectedApproveKind)
                }
            },
            _ => Ok(auto_approve),
        }
    }
}

   