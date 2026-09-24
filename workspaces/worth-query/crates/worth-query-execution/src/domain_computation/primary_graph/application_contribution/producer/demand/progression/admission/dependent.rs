use super::*;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub(in crate::domain_computation) fn admit_dependent_output_demand<Family>(
        &self,
        source_result: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        maximum_work: usize,
        maximum_retained_bytes: usize,
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
        self.admit_output_demand_with_source::<Family>(
            source,
            observed_source,
            None,
            profile_kind,
            maximum_work,
            maximum_retained_bytes,
            None,
            crate::domain_computation::primary_graph::application_output_demand::DemandAdmissionKind::Required,
            None,
            None,
            Some(retained_basis),
        )
    }
}
