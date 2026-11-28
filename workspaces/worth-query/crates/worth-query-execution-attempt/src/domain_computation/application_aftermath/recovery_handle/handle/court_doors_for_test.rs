//! Sealed test doors over this module's private members.
use super::*;
use worth_query_execution_lineage::facade::ports::{
    CommitAuthorityBindingInternalPort as _, RetainedGovernedInputInternalPort as _,
};

impl WorthQueryRecoveryHandle {
    #[doc(hidden)]
    pub fn authority_identity_for_test(&self) -> WorthQueryRecoveryHandleAuthorityIdentity {
        Self::authority_identity(self)
    }
}


impl WorthQueryRecoveryHandle {
    #[doc(hidden)]
    pub fn consume_for_test(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        terminal: RecoveryClaimDisposition,
    ) -> Result<WorthQueryRecoveryHandleBinding, super::super::WorthQueryRecoveryTransitionDenial>
    {
        Self::consume(self, phase, terminal)
    }
}

impl WorthQueryRecoveryHandle {
    #[doc(hidden)]
    pub fn registry_arc_for_test(&self) -> Arc<WorthQueryRecoveryHandleRegistry> {
        Self::registry_arc(self)
    }
}

impl WorthQueryRecoveryHandle {
    #[doc(hidden)]
    pub fn registry_slot_for_test(&self) -> WorthQueryRecoveryRegistrySlot {
        Self::registry_slot(self)
    }
}

impl WorthQueryRecoveryHandle {
    #[doc(hidden)]
    pub fn runtime_authority_for_test(
        &self,
    ) -> crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity {
        Self::runtime_authority(self)
    }
}
