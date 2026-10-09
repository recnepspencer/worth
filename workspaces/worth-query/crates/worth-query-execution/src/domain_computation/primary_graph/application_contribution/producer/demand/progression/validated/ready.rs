use super::*;
use crate::domain_computation::primary_graph::application_output_demand::{
    ReadyCompletion, WorthQueryAcceptedOutputAuthority as Authority,
    WorthQueryOutputDemandSettlement as Settlement,
};

/// What a Ready row proves on the branch as it stands.
#[derive(Clone, Copy)]
enum ReadyUnavailable {
    Superseded,
    WorkExhausted,
}

enum ReadyVerdict {
    Settled(std::sync::Arc<Settlement>),
    /// A superseded output, or an exhausted source-currentness allowance,
    /// leaves no current proof. Movable demands refresh into fresh execution;
    /// exact-publication recovery refuses that refresh.
    Unavailable(ReadyUnavailable),
}

/// Selected waves supply their own currentness proof; ordinary Ready uses a branch.
pub(super) enum ReadyCertification {
    SelectedWave,
    OnBranch(crate::basis::WorthQueryProductBranch),
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub(super) fn advance_validated_ready<Family>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        disclosure: ValidatedOutputDisclosure<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        completion: ReadyCompletion,
        certification: ReadyCertification,
        request_admission: &mut InvalidationEditAdmission,
    ) -> Result<
        OwnStages<FamilySourceQuery<Schema, Family>, FamilySourceValue<Schema, Family>>,
        WorthQueryOutputDemandDenial,
    >
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceValue<Schema, Family>: 'static,
        FamilySourceQuery<Schema, Family>: 'static,
    {
        let delivery_branch = match certification {
            // Exact Ready still needs the wave's registry and
            // actor/native proof before it can be Current.
            ReadyCertification::SelectedWave => {
                return Ok(OwnStages::Answer(WorthQueryOutputDemandAdvance::Pending));
            }
            ReadyCertification::OnBranch(branch) => branch,
        };
        // The disclosed source is the one this demand names: a replaced
        // source left through `leave_replaced_source` before the row began.
        if let ReadyVerdict::Settled(settlement) =
            self.certify_ready(phase, demand, &completion, delivery_branch)?
        {
            return Ok(OwnStages::Answer(WorthQueryOutputDemandAdvance::Settled(
                settlement,
            )));
        }
        request_admission
            .charge_external_work(std::mem::size_of_val(disclosure.source()) as u64)
            .map_err(|_| denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, ""))?;
        self.refresh_output_demand(
            demand,
            disclosure.value(),
            disclosure.source().clone(),
            Some(&completion.authority),
            request_admission,
        )?;
        Ok(OwnStages::Refreshed(disclosure))
    }

    /// Settle the Ready this call's own stages reached. Its disclosure went
    /// into the execution. A born-stale publication requests one reobservation
    /// within this advance, using the same request admission.
    pub(super) fn settle_own_ready<Family>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        demand: &WorthQueryAdmittedOutputDemand<Schema, Family>,
        completion: &ReadyCompletion,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        progress: CheckpointProgress,
    ) -> Result<
        CallerPass<FamilySourceQuery<Schema, Family>, FamilySourceValue<Schema, Family>>,
        WorthQueryOutputDemandDenial,
    >
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        Ok(
            match self.certify_ready(phase, demand, completion, delivery_branch)? {
                ReadyVerdict::Settled(settlement) => {
                    CallerPass::Answer(WorthQueryOutputDemandAdvance::Settled(settlement))
                }
                ReadyVerdict::Unavailable(ReadyUnavailable::Superseded) => match progress {
                    CheckpointProgress::Published(publication) => {
                        let current = self.on_branch(delivery_branch).select().map_err(|stop| {
                            WorthQueryOutputDemandDenial::product_selection(
                                stop,
                                "own publication readmission",
                            )
                        })?;
                        if publication
                            .0
                            .is_selected_at(current.product().observation())
                            && matches!(&completion.authority, Authority::Committed(receipt) if receipt.committed_product_publication() == &publication.0)
                        {
                            CallerPass::PublishedStale(publication)
                        } else {
                            CallerPass::Answer(WorthQueryOutputDemandAdvance::Pending)
                        }
                    }
                    CheckpointProgress::Advanced => {
                        CallerPass::Answer(WorthQueryOutputDemandAdvance::Pending)
                    }
                },
                ReadyVerdict::Unavailable(ReadyUnavailable::WorkExhausted) => {
                    CallerPass::Answer(WorthQueryOutputDemandAdvance::Pending)
                }
            },
        )
    }

    fn certify_ready<Family>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        demand: &WorthQueryAdmittedOutputDemand<Schema, Family>,
        completion: &ReadyCompletion,
        delivery_branch: crate::basis::WorthQueryProductBranch,
    ) -> Result<ReadyVerdict, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        let interest = demand
            .interest
            .as_ref()
            .expect("validated Ready retains its live Interest");
        let current = self.on_branch(delivery_branch).select().map_err(|denial| {
            WorthQueryOutputDemandDenial::product_selection(
                denial,
                "ready output currentness basis could not be selected",
            )
        })?;
        let unavailable = |proof: Result<(), WorthQueryOutputDemandDenial>| match proof {
            Ok(()) => Ok(None),
            Err(denial) => match ready_unavailable(&denial) {
                Some(cause) => Ok(Some(cause)),
                None => Err(denial),
            },
        };
        // The source is proven by the disclosure. A stable alias or a
        // restored output and its facts are compared here as well, and for a
        // restored output that comparison is what gives it its marks.
        let alias = |settlement: std::sync::Arc<Settlement>| {
            let proof = current.require_current_output_settlements(
                phase,
                [settlement.as_ref()],
                demand.currentness_work_limit(),
            );
            unavailable(proof).map(|unavailable| match unavailable {
                None => Ok(settlement),
                Some(kind) => Err(kind),
            })
        };
        let settlement = match &completion.authority {
            Authority::Committed(receipt) => {
                let proof = current.require_current_output_receipts(
                    phase,
                    [receipt],
                    demand.currentness_work_limit(),
                );
                if let Some(kind) = unavailable(proof)? {
                    return Ok(ReadyVerdict::Unavailable(kind));
                }
                match Settlement::from_commit(
                    self,
                    receipt,
                    &completion.readiness,
                    &demand.selected.identity,
                    Family::IDENTITY,
                    demand.producer_contacts_in_this_demand,
                ) {
                    Ok(settlement) => Ok(settlement),
                    Err(denial)
                        if denial.kind() == WorthQueryOutputDemandDenialKind::Superseded =>
                    {
                        Err(ReadyUnavailable::Superseded)
                    }
                    Err(denial) => return Err(denial),
                }
            }
            Authority::Stable(stable) => alias(Settlement::from_stable(
                self,
                stable,
                &current,
                &demand.selected.identity,
                Family::IDENTITY,
                demand.producer_contacts_in_this_demand,
            ))?,
            Authority::Restored(restored) => alias(Settlement::from_restoration(
                self,
                restored,
                Family::IDENTITY,
                demand.producer_contacts_in_this_demand,
            )?)?,
        };
        let settlement = match settlement {
            Ok(settlement) => settlement,
            Err(kind) => return Ok(ReadyVerdict::Unavailable(kind)),
        };
        self.output_demands
            .finish_settlement(interest, &completion.authority)?;
        Ok(ReadyVerdict::Settled(settlement))
    }
}

fn ready_unavailable(denial: &WorthQueryOutputDemandDenial) -> Option<ReadyUnavailable> {
    match denial.kind() {
        WorthQueryOutputDemandDenialKind::Superseded => Some(ReadyUnavailable::Superseded),
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded => {
            Some(ReadyUnavailable::WorkExhausted)
        }
        _ => None,
    }
}
