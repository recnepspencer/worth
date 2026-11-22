//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryRecoveryHandleBinding {
    #[doc(hidden)]
    pub fn application_binding_generation_for_test(&self) -> &u64 {
        &self.application_binding_generation
    }
}

impl WorthQueryRecoveryHandleBinding {
    #[doc(hidden)]
    pub fn commit_for_test(&self) -> &RelationalCommitReceipt {
        &self.commit
    }
}

impl WorthQueryRecoveryHandleBinding {
    #[doc(hidden)]
    pub fn committed_dispatch_outbox_for_test(
        &self,
    ) -> &Option<crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxBinding>
    {
        &self.committed_dispatch_outbox
    }
}

impl WorthQueryRecoveryHandleBinding {
    #[doc(hidden)]
    pub fn expires_at_unix_ms_for_test(&self) -> &Option<u64> {
        &self.expires_at_unix_ms
    }
}

impl WorthQueryRecoveryHandleBinding {
    #[doc(hidden)]
    pub fn idempotency_for_test(&self) -> &WorthQueryApplicationIdempotencyBinding {
        &self.idempotency
    }
}

impl WorthQueryRecoveryHandleBinding {
    #[doc(hidden)]
    pub fn installed_aftermath_for_test(&self) -> &WorthQueryInstalledAftermathContract {
        &self.installed_aftermath
    }
}

impl WorthQueryRecoveryHandleBinding {
    #[doc(hidden)]
    pub fn installed_operation_for_test(&self) -> &[u8; 32] {
        &self.installed_operation
    }
}

impl WorthQueryRecoveryHandleBinding {
    #[doc(hidden)]
    pub fn principal_scope_for_test(&self) -> &WorthQueryOperationScopeBinding {
        &self.principal_scope
    }
}

impl WorthQueryRecoveryHandleBinding {
    #[doc(hidden)]
    pub fn retained_governed_input_for_test(&self) -> &Option<WorthQueryRetainedGovernedInput> {
        match &self.input_custody {
            RecoveryInputCustody::Performed {
                retained_governed_input,
                ..
            } => retained_governed_input,
            RecoveryInputCustody::Recorded { .. } => &None,
        }
    }
}

impl WorthQueryRecoveryHandleBinding {
    #[doc(hidden)]
    pub fn runtime_instance_id_for_test(&self) -> &u64 {
        &self.runtime_instance_id
    }
}

impl WorthQueryRecoveryHandleBinding {
    #[doc(hidden)]
    pub fn schema_identity_for_test(&self) -> &[u8; 32] {
        &self.schema_identity
    }
}

impl WorthQueryRecoveryHandleBinding {
    #[doc(hidden)]
    pub fn application_binding_generation_mut_for_test(&mut self) -> &mut u64 {
        &mut self.application_binding_generation
    }
}

impl WorthQueryRecoveryHandleBinding {
    #[doc(hidden)]
    pub fn committed_dispatch_outbox_mut_for_test(
        &mut self,
    ) -> &mut Option<
        crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxBinding,
    > {
        &mut self.committed_dispatch_outbox
    }
}

impl WorthQueryRecoveryHandleBinding {
    #[doc(hidden)]
    pub fn expires_at_unix_ms_mut_for_test(&mut self) -> &mut Option<u64> {
        &mut self.expires_at_unix_ms
    }
}

impl WorthQueryRecoveryHandleBinding {
    #[doc(hidden)]
    pub fn idempotency_mut_for_test(&mut self) -> &mut WorthQueryApplicationIdempotencyBinding {
        &mut self.idempotency
    }
}

impl WorthQueryRecoveryHandleBinding {
    #[doc(hidden)]
    pub fn installed_aftermath_mut_for_test(
        &mut self,
    ) -> &mut WorthQueryInstalledAftermathContract {
        &mut self.installed_aftermath
    }
}

impl WorthQueryRecoveryHandleBinding {
    #[doc(hidden)]
    pub fn installed_operation_mut_for_test(&mut self) -> &mut [u8; 32] {
        &mut self.installed_operation
    }
}

impl WorthQueryRecoveryHandleBinding {
    #[doc(hidden)]
    pub fn principal_scope_mut_for_test(&mut self) -> &mut WorthQueryOperationScopeBinding {
        &mut self.principal_scope
    }
}

impl WorthQueryRecoveryHandleBinding {
    #[doc(hidden)]
    pub fn retained_governed_input_mut_for_test(
        &mut self,
    ) -> &mut Option<WorthQueryRetainedGovernedInput> {
        match &mut self.input_custody {
            RecoveryInputCustody::Performed {
                retained_governed_input,
                ..
            } => retained_governed_input,
            RecoveryInputCustody::Recorded { .. } => {
                panic!("recorded source has no mutable performed input")
            }
        }
    }
}

impl WorthQueryRecoveryHandleBinding {
    #[doc(hidden)]
    pub fn runtime_instance_id_mut_for_test(&mut self) -> &mut u64 {
        &mut self.runtime_instance_id
    }
}

impl WorthQueryRecoveryHandleBinding {
    #[doc(hidden)]
    pub fn schema_identity_mut_for_test(&mut self) -> &mut [u8; 32] {
        &mut self.schema_identity
    }
}

impl WorthQueryRecoveryHandleBinding {
    #[doc(hidden)]
    pub fn from_receipt_for_test(
        receipt: &WorthQueryApplicationCommitReceipt,
        expires_at_unix_ms: Option<u64>,
    ) -> Result<Self, WorthQueryRecoveryHandleDenial> {
        Self::from_receipt(receipt, expires_at_unix_ms)
    }
}

impl WorthQueryRecoveryHandleBinding {
    #[doc(hidden)]
    pub fn commit_mut_for_test(&mut self) -> &mut RelationalCommitReceipt {
        &mut self.commit
    }
}
