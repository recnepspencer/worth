//! Existing owner-only receipt construction controls.
use super::*;
impl WorthQueryApplicationCommitReceipt {
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
