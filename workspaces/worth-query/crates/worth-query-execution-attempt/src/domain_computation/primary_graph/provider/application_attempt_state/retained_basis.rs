//! Narrow attempt-owned products used outside the store lock.

use super::super::WorthQueryPrimaryGraphApplicationDecisionFact;
use worth_query_execution_runtime::facade::ports::ProductPublicationBindingInternalPort as _;

pub struct WorthQueryObservedApplicationFactBasis
{
    fact: WorthQueryPrimaryGraphApplicationDecisionFact,
    product: crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
}

impl WorthQueryObservedApplicationFactBasis {
    pub const fn new(
        fact: WorthQueryPrimaryGraphApplicationDecisionFact,
        product: crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
    ) -> Self {
        Self { fact, product }
    }

    pub fn remains_equal_in(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
    ) -> Result<bool, crate::domain_computation::WorthQueryDecisionReadSetFailure> {
        let snapshot = crate::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(
            runtime,
            self.product.observation().basis().relational_basis(),
        )
        .map_err(|denial| super::super::decision_facts::snapshot_read_set_failure(denial.into()))?;
        let fresh = self.fact.remains_equal_in(execution, runtime, &snapshot);
        crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
        fresh.map_err(super::super::decision_facts::fact_read_set_failure)
    }
}

pub struct WorthQueryApplicationIdempotencyBasis {
    binding: crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding,
    product: crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
}

impl WorthQueryApplicationIdempotencyBasis {
    pub const fn new(
        binding: crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding,
        product: crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
    ) -> Self {
        Self { binding, product }
    }

    pub fn resolve(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        provider: &super::super::WorthQueryPrimaryGraphProvider,
    ) -> Result<
        super::super::WorthQueryProviderIdempotencyResolution,
        super::super::WorthQueryProviderIdempotencyResolutionDenial,
    > {
        provider.resolve_idempotency_binding_at_product(execution, self.binding, &self.product)
    }
}
