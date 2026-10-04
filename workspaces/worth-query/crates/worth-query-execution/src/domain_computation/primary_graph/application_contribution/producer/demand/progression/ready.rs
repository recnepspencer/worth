//! Both committed and restored output delivery require current producer facts.
use super::*;
use crate::domain_computation::primary_graph::application_output_demand::{
    WorthQueryAcceptedOutputAuthority, WorthQueryCompletedOutputDemand,
    WorthQueryOutputDemandSettlement,
};

impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(super) fn advance_ready_output<Family>(
        &self,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        completion: WorthQueryCompletedOutputDemand,
        source: FamilySourceValue<Schema, Family>,
        observed_source: crate::domain_computation::primary_graph::WorthQueryObservedSource<
            FamilySourceQuery<Schema, Family>,
        >,
        delivery_branch: crate::basis::WorthQueryProductBranch,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceValue<Schema, Family>: 'static,
        FamilySourceQuery<Schema, Family>: 'static,
    {
        let settlement = match &completion.authority {
            WorthQueryAcceptedOutputAuthority::Committed(receipt) => {
                WorthQueryOutputDemandSettlement::from_commit(
                    self,
                    receipt,
                    &completion.readiness,
                    &demand.selected.identity,
                    Family::IDENTITY,
                    demand.producer_contacts_in_this_demand,
                )
            }
            WorthQueryAcceptedOutputAuthority::Restored(restored) => {
                WorthQueryOutputDemandSettlement::from_restoration(self, restored, Family::IDENTITY)
            }
        };
        let current_settlement = settlement.and_then(|settlement| {
            let current = self.on_branch(delivery_branch).select().map_err(|denial| {
                WorthQueryOutputDemandDenial::product_selection(
                    denial,
                    "ready output currentness basis could not be selected",
                )
            })?;
            current.require_current_output_settlements(
                [settlement.as_ref()],
                demand.currentness_work_limit(),
            )?;
            Ok(settlement)
        });
        match current_settlement {
            Ok(settlement) => Ok(WorthQueryOutputDemandAdvance::Settled(settlement)),
            Err(denial) if denial.kind() == WorthQueryOutputDemandDenialKind::Superseded => {
                self.refresh_output_demand(demand, source, observed_source, &completion.authority)
            }
            Err(denial) => Err(denial),
        }
    }
}
