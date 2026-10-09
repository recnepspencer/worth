//! Exercise the direct refresh door after genuine exact-publication admission.
use super::*;
impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Certification-only entry into the direct door. Public program recovery
    /// uses selected execution; this admits a genuine Recovery demand and
    /// calls the other door itself, skipping the progression that leads there.
    #[doc(hidden)]
    pub fn attempt_recovery_refresh_for_test<Family>(
        &self,
        retained: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        current: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        redisclosed: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<FamilySourceQuery<Schema, Family>, FamilySourceValue<Schema, Family>>,
        limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    ) -> Result<(), WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceValue<Schema, Family>: 'static,
        FamilySourceQuery<Schema, Family>: 'static,
    {
        let mut demand = self
            .admit_recovered_output_demand::<Family>(retained, current, limits, receipt)
            .expect("certification admits Recovery before exercising its refresh door");
        let (value, source) = redisclosed.into_single_source().ok_or_else(|| {
            denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                Family::IDENTITY,
            )
        })?;
        let interest = demand
            .interest
            .as_ref()
            .expect("Recovery retains its admitted interest");
        let crate::domain_computation::primary_graph::application_output_demand::OutputRowStage::Ready(completion) = self.output_demands.row_stage(interest) else {
            panic!("the direct-door probe must join the exact Ready before refreshing");
        };
        let mut admission = self.demand_request_admission();
        self.refresh_output_demand(
            &mut demand,
            value,
            source,
            Some(&completion.authority),
            &mut admission,
        )
    }
}
