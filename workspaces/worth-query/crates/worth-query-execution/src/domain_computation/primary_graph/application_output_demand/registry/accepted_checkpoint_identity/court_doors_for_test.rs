//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryAcceptedOutputCheckpointIdentity {
    #[doc(hidden)]
    pub fn idempotency_key_for_test(&self) -> &[u8; 32] {
        &self.idempotency_key
    }
}

impl WorthQueryAcceptedOutputCheckpointIdentity {
    #[doc(hidden)]
    pub fn idempotency_key_mut_for_test(&mut self) -> &mut [u8; 32] {
        &mut self.idempotency_key
    }
}

impl WorthQueryAcceptedOutputCheckpointIdentity {
    #[doc(hidden)]
    pub fn scope_for_test(&self) -> &crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding {
        &self.scope
    }
}

impl WorthQueryAcceptedOutputCheckpointIdentity {
    #[doc(hidden)]
    pub fn source_partition_for_test(&self) -> &[u8; 32] {
        &self.source_partition
    }
}
