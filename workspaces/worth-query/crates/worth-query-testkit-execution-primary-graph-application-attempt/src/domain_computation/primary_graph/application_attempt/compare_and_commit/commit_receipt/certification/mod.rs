//! Existing owner-only receipt construction controls.
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitReceipt;
/// Test doors on a library type, owned by the moved courts.
pub trait CertificationWorthQueryApplicationCommitReceiptForTest: Sized {
    fn without_settlement_description(self) -> Self;
    fn with_corrupted_original_disclosure(
        self,
        disclosure: std::sync::Arc<worth_query_execution::domain_computation::primary_graph::source_disclosure::inspection::OriginalSourceInspectionDisclosure>,
    ) -> Self;
    #[cfg(all())]
    fn with_provider_runtime_instance_id_for_test(self, runtime: u64) -> Self;
    #[cfg(all())]
    fn with_idempotency_binding_for_test(
        self,
        binding: worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyBinding,
    ) -> Self;
}

impl CertificationWorthQueryApplicationCommitReceiptForTest for WorthQueryApplicationCommitReceipt {
    fn without_settlement_description(mut self) -> Self {
        self.exact_output_settlement = None;
        self
    }

    fn with_corrupted_original_disclosure(
        mut self,
        disclosure: std::sync::Arc<worth_query_execution::domain_computation::primary_graph::source_disclosure::inspection::OriginalSourceInspectionDisclosure>,
    ) -> Self {
        self.authority_binding
            .corrupt_original_disclosure(disclosure);
        self
    }

    #[cfg(all())]
    fn with_provider_runtime_instance_id_for_test(mut self, runtime: u64) -> Self {
        self.provider_runtime_instance_id = runtime;
        self
    }

    #[cfg(all())]
    fn with_idempotency_binding_for_test(
        mut self,
        binding: worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyBinding,
    ) -> Self {
        self.authority_binding
            .replace_idempotency_binding_for_test(binding);
        self
    }
}
