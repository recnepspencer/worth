//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryRecoveryHandle {
    #[doc(hidden)]
    pub fn authority_identity_for_test(&self) -> WorthQueryRecoveryHandleAuthorityIdentity {
        Self::authority_identity(self)
    }
}

impl WorthQueryRecoveryHandle {
    #[doc(hidden)]
    pub fn axis_probe_for_test(binding: WorthQueryRecoveryHandleBinding) -> Self {
        Self::axis_probe(binding)
    }
}

impl WorthQueryRecoveryHandle {
    #[doc(hidden)]
    pub fn axis_probe_in_registry_for_test(
        binding: WorthQueryRecoveryHandleBinding,
        registry: Arc<WorthQueryRecoveryHandleRegistry>,
    ) -> Self {
        Self::axis_probe_in_registry(binding, registry)
    }
}

impl WorthQueryRecoveryHandle {
    #[doc(hidden)]
    pub fn consume_for_test(
        self,
        terminal: WorthQueryRecoveryResourceTerminal,
    ) -> Result<WorthQueryRecoveryHandleBinding, WorthQueryRecoveryHandleDenial> {
        Self::consume(self, terminal)
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
    ) -> crate::domain_computation::vocabulary::execution_runtime::runtime_identity::WorthQueryRuntimeAuthorityIdentity{
        Self::runtime_authority(self)
    }
}
