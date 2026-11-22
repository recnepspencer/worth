//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationCommitReceipt {
    #[doc(hidden)]
    pub fn authority_binding_for_test(&self) -> &WorthQueryApplicationCommitAuthorityBinding {
        &self.authority_binding
    }
}

impl WorthQueryApplicationCommitReceipt {
    #[doc(hidden)]
    pub fn provider_runtime_instance_id_for_test(&self) -> &u64 {
        &self.provider_runtime_instance_id
    }
}

impl WorthQueryApplicationCommitReceipt {
    #[doc(hidden)]
    pub fn exact_output_settlement_mut_for_test(
        &mut self,
    ) -> &mut Option<
        std::sync::Arc<
            crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity,
        >,
    > {
        &mut self.exact_output_settlement
    }
}

impl WorthQueryApplicationCommitReceipt {
    #[doc(hidden)]
    pub fn provider_runtime_instance_id_mut_for_test(&mut self) -> &mut u64 {
        &mut self.provider_runtime_instance_id
    }
}

impl WorthQueryApplicationCommitReceipt {
    #[doc(hidden)]
    pub fn exact_output_settlement_for_test(
        &self,
    ) -> Option<
        &std::sync::Arc<
            crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity,
        >,
    > {
        Self::exact_output_settlement(self)
    }
}

impl WorthQueryApplicationCommitReceipt {
    #[doc(hidden)]
    pub fn authority_binding_mut_for_test(
        &mut self,
    ) -> &mut WorthQueryApplicationCommitAuthorityBinding {
        &mut self.authority_binding
    }
}

impl WorthQueryApplicationCommitReceipt {
    #[doc(hidden)]
    pub fn retained_preimage_for_test(&self) -> Option<&WorthQueryRetainedPreImage> {
        Self::retained_preimage(self)
    }
}

impl WorthQueryApplicationCommitReceipt {
    #[doc(hidden)]
    pub fn retain_output_correspondence_for_test(
        &self,
    ) -> std::sync::Arc<WorthQueryApplicationOutputCorrespondence> {
        Self::retain_output_correspondence(self)
    }
}

impl WorthQueryApplicationCommitReceipt {
    #[doc(hidden)]
    pub fn runtime_authority_for_test(
        &self,
    ) -> crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity {
        Self::runtime_authority(self)
    }
}
