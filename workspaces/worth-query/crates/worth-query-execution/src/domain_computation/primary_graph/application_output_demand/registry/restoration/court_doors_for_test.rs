//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryRestoredAcceptedOutput {
    #[doc(hidden)]
    pub fn checkpoint_for_test(&self) -> &super::super::WorthQueryAcceptedOutputCheckpointIdentity {
        &self.checkpoint
    }
}

impl WorthQueryRestoredAcceptedOutput {
    #[doc(hidden)]
    pub fn source_scope_for_test(&self) -> &crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding {
        &self.source_scope
    }
}
