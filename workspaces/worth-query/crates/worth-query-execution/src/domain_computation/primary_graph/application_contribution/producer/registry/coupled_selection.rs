//! A declared source uses its installed family Query and applicable Preserve.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::coupled_outputs::DeclaredCoupledOutputDemand;
impl<Schema: ApplicationSchema> WorthQueryInstalledApplicationProducerRegistry<Schema> {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn declared_coupled_source(
        &self,
        demand: &DeclaredCoupledOutputDemand,
    ) -> Result<&Arc<InstalledProducerProvider<Schema>>, WorthQueryOutputDemandDenial> {
        self.entries
            .values()
            .find(|entry| {
                entry.declaration.output_family_type == demand.family_type
                    && entry.declaration.source_type == demand.source_binding_type
            })
            .ok_or_else(|| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ProducerUnavailable,
                    demand.family,
                )
            })
    }
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn declared_coupled_preserve(
        &self,
        demand: &DeclaredCoupledOutputDemand,
        profile: &str,
    ) -> Result<&Arc<InstalledProducerProvider<Schema>>, WorthQueryOutputDemandDenial> {
        self.select_matching_entry(
            |entry| {
                entry.declaration.output_family_type == demand.family_type
                    && entry.declaration.source_type == demand.source_binding_type
                    && entry.declaration.applicability.iter().any(|applies| {
                        applies.lifecycle()
                            == super::super::WorthQueryProducerLifecyclePosture::Preserve
                            && applies.profile_kind() == profile
                    })
            },
            WorthQueryOutputDemandDenialKind::MissingApplicableProducer,
            demand.family,
        )
    }
}
