use super::*;
use crate::domain_computation::primary_graph::application_output_demand::ReadyCompletion;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub(super) fn advance_validated_ready<Family>(
        &self,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        disclosure: ValidatedOutputDisclosure<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        completion: ReadyCompletion,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        selected: bool,
        request_admission: &mut InvalidationEditAdmission,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceValue<Schema, Family>: 'static,
        FamilySourceQuery<Schema, Family>: 'static,
    {
        let disclosed_source = disclosure.source();
        let interest = demand
            .interest
            .as_ref()
            .expect("validated Ready retains its live Interest");
        if selected {
            // Exact Ready still needs the wave's registry and
            // actor/native proof before it can be Current.
            return Ok(WorthQueryOutputDemandAdvance::Pending);
        }
        if !demand.matches_observed_source(disclosed_source) {
            // A Ready Stable alias retains the exact predecessor needed
            // to admit a freshly selected source in this same interest.
            if matches!(
                        &completion.authority,
                        crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputAuthority::Stable(_)
                    ) {
                        let (disclosed_value, disclosed_source) = disclosure.into_parts();
                        return self.refresh_output_demand(
                            demand,
                            disclosed_value,
                            disclosed_source,
                            &completion.authority,
                            request_admission,
                        );
                    }
            return Err(self
                .output_demands
                .finish_superseded(interest, Family::IDENTITY));
        }
        return match &completion.authority {
                    crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputAuthority::Committed(receipt) => {
                        let current = self.on_branch(delivery_branch).select().map_err(|denial| {
                            WorthQueryOutputDemandDenial::product_selection(
                                denial,
                                "ready output currentness basis could not be selected",
                            )
                        })?;
                        match current.require_current_output_receipts(
                            [receipt],
                            demand.currentness_work_limit(),
                        ) {
                            Ok(()) => {}
                            Err(denial)
                                if denial.kind() == WorthQueryOutputDemandDenialKind::Superseded =>
                            {
                                let (disclosed_value, disclosed_source) = disclosure.into_parts();
                                return self.refresh_output_demand(
                                    demand,
                                    disclosed_value,
                                    disclosed_source,
                                    &completion.authority,
                                    request_admission,
                                );
                            }
                            Err(denial) => return Err(denial),
                        }
                        let settlement = crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandSettlement::from_commit(
                            self,
                            receipt,
                            &completion.readiness,
                            &demand.selected.identity,
                            Family::IDENTITY,
                            demand.producer_contacts_in_this_demand,
                        );
                        match settlement {
                            Ok(settlement) => {
                                self.output_demands
                                    .finish_settlement(interest, &completion.authority)?;
                                Ok(WorthQueryOutputDemandAdvance::Settled(settlement))
                            }
                            Err(denial)
                                if denial.kind() == WorthQueryOutputDemandDenialKind::Superseded =>
                            {
                                let (disclosed_value, disclosed_source) = disclosure.into_parts();
                                self.refresh_output_demand(
                                    demand,
                                    disclosed_value,
                                    disclosed_source,
                                    &completion.authority,
                                    request_admission,
                                )
                            }
                            Err(denial) => Err(denial),
                        }
                    }
                    crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputAuthority::Stable(stable) => {
                        let current = self.on_branch(delivery_branch).select().map_err(|denial| {
                            WorthQueryOutputDemandDenial::product_selection(
                                denial,
                                "stable output currentness basis could not be selected",
                            )
                        })?;
                        let settlement = crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandSettlement::from_stable(
                            self,
                            stable,
                            &current,
                            &demand.selected.identity,
                            Family::IDENTITY,
                        );
                        match current.require_current_output_settlements(
                            [settlement.as_ref()],
                            demand.currentness_work_limit(),
                        ) {
                            Ok(()) => {}
                            Err(denial)
                                if denial.kind() == WorthQueryOutputDemandDenialKind::Superseded =>
                            {
                                let (disclosed_value, disclosed_source) = disclosure.into_parts();
                                return self.refresh_output_demand(
                                    demand,
                                    disclosed_value,
                                    disclosed_source,
                                    &completion.authority,
                                    request_admission,
                                );
                            }
                            Err(denial) => return Err(denial),
                        }
                        self.output_demands
                            .finish_settlement(interest, &completion.authority)?;
                        Ok(WorthQueryOutputDemandAdvance::Settled(settlement))
                    }
                    crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputAuthority::Restored(restored) => {
                        crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandSettlement::from_restoration(
                            self,
                            restored,
                            Family::IDENTITY,
                        )
                        .and_then(|settlement| {
                            self.output_demands
                                .finish_settlement(interest, &completion.authority)?;
                            Ok(WorthQueryOutputDemandAdvance::Settled(settlement))
                        })
                    }
                };
    }
}
