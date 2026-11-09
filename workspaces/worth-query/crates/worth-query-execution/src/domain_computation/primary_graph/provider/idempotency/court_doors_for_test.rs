//! Read-only access to the existing idempotency judge for retry courts.
use super::*;
impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn resolve_idempotency_binding_at_product_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: WorthQueryApplicationIdempotencyBinding,
        product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
    ) -> Result<
        WorthQueryProviderIdempotencyResolution,
        WorthQueryProviderIdempotencyResolutionDenial,
    > {
        self.resolve_idempotency_binding_at_product(execution, binding, product)
    }
}
