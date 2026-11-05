use super::*;

pub(in crate::domain_computation) trait DependentScopedOps<Schema>: crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema> where
    Schema: ApplicationSchema + 'static, {
fn admit_dependent_output_demand<Family>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source_result: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits,
        retained_basis: std::sync::Arc<
            crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
        >,
    ) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        let (source, observed_source) = source_result.into_single_source().ok_or_else(|| {
            denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "dependent output source query did not return one owner-paired occurrence",
            )
        })?;
        let profile_kind = Family::profile_kind(&source);
        self.admit_output_demand_with_source::<Family>(execution,
            observed_source,
            None,
            profile_kind,
            limits,
            None,
            crate::domain_computation::primary_graph::application_output_demand::DemandAdmissionKind::Required,
            None,
            None,
            Some(retained_basis),
        )
    }
}

impl<Schema, HostRuntime: crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema> + ?Sized> DependentScopedOps<Schema> for HostRuntime where
    Schema: ApplicationSchema + 'static, {}
