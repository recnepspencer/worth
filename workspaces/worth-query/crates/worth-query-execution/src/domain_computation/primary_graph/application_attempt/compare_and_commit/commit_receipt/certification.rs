//! Existing owner-only receipt construction controls.
use super::*;
impl WorthQueryApplicationCommitReceipt {
    pub(in crate::domain_computation::primary_graph) fn without_settlement_description(
        mut self,
    ) -> Self {
        self.exact_output_settlement = None;
        self
    }

    pub(in crate::domain_computation::primary_graph) fn with_corrupted_original_disclosure(
        mut self,
        disclosure: std::sync::Arc<crate::domain_computation::primary_graph::source_disclosure::OriginalSourceInspectionDisclosure>,
    ) -> Self {
        self.authority_binding
            .corrupt_original_disclosure(disclosure);
        self
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn with_provider_runtime_instance_id_for_test(
        mut self,
        runtime: u64,
    ) -> Self {
        self.provider_runtime_instance_id = runtime;
        self
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn with_idempotency_binding_for_test(
        mut self,
        binding: super::super::super::WorthQueryApplicationIdempotencyBinding,
    ) -> Self {
        self.authority_binding
            .replace_idempotency_binding_for_test(binding);
        self
    }
}
