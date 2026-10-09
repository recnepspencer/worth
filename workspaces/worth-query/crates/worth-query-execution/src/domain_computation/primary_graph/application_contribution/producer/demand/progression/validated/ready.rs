use super::*;
use crate::domain_computation::primary_graph::application_output_demand::{
    ReadyCompletion, WorthQueryAcceptedOutputAuthority as Authority,
    WorthQueryOutputDemandSettlement as Settlement,
};

/// What a Ready row proves on the branch as it stands.
enum ReadyVerdict {
    Settled(std::sync::Arc<Settlement>),
    /// A superseded output, or an exhausted source-currentness allowance,
    /// leaves no current proof. The demand refreshes into fresh execution
    /// instead of stopping terminally.
    Unavailable,
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
    ) -> Result<OwnStages, WorthQueryOutputDemandDenial>
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
        let (disclosed_value, disclosed_source) = disclosure.into_parts();
        self.refresh_output_demand(
            demand,
            disclosed_value,
            disclosed_source,
            Some(&completion.authority),
            request_admission,
        )?;
        Ok(OwnStages::Refreshed)
    }

    /// Settle the Ready this call's own stages reached. Its disclosure went
    /// into the execution, so an output superseded before it could settle
    /// waits for the next advance to refresh it from a new one.
    pub(super) fn settle_own_ready<Family>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        demand: &WorthQueryAdmittedOutputDemand<Schema, Family>,
        completion: &ReadyCompletion,
        delivery_branch: crate::basis::WorthQueryProductBranch,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        Ok(
            match self.certify_ready(phase, demand, completion, delivery_branch)? {
                ReadyVerdict::Settled(settlement) => {
                    WorthQueryOutputDemandAdvance::Settled(settlement)
                }
                ReadyVerdict::Unavailable => WorthQueryOutputDemandAdvance::Pending,
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
            Ok(()) => Ok(false),
            Err(denial) if current_proof_unavailable(&denial) => Ok(true),
            Err(denial) => Err(denial),
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
            unavailable(proof).map(|unavailable| (!unavailable).then_some(settlement))
        };
        let settlement = match &completion.authority {
            Authority::Committed(receipt) => {
                let proof = current.require_current_output_receipts(
                    phase,
                    [receipt],
                    demand.currentness_work_limit(),
                );
                if unavailable(proof)? {
                    return Ok(ReadyVerdict::Unavailable);
                }
                match Settlement::from_commit(
                    self,
                    receipt,
                    &completion.readiness,
                    &demand.selected.identity,
                    Family::IDENTITY,
                    demand.producer_contacts_in_this_demand,
                ) {
                    Ok(settlement) => Some(settlement),
                    Err(denial)
                        if denial.kind() == WorthQueryOutputDemandDenialKind::Superseded =>
                    {
                        None
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
            ))?,
            Authority::Restored(restored) => alias(Settlement::from_restoration(
                self,
                restored,
                Family::IDENTITY,
            )?)?,
        };
        let Some(settlement) = settlement else {
            return Ok(ReadyVerdict::Unavailable);
        };
        self.output_demands
            .finish_settlement(interest, &completion.authority)?;
        Ok(ReadyVerdict::Settled(settlement))
    }
}

fn current_proof_unavailable(denial: &WorthQueryOutputDemandDenial) -> bool {
    matches!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::Superseded
            | WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
    )
}
