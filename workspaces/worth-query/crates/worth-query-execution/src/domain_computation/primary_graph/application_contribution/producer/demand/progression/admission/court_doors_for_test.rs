//! Sealed test doors over this module's private members.
use super::*;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> where
    Schema: ApplicationSchema + 'static, {
    #[doc(hidden)]
    pub fn admit_required_output_demand_for_test<Family>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source_result: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits,
    ) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>, {
        Self::admit_required_output_demand::<Family>(self, execution, source_result, limits)
    }
}
