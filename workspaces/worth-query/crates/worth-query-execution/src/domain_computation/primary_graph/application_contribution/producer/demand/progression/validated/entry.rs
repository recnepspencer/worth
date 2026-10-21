//! Typed entry points into the shared validated progression owner.

use super::*;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand::progression) fn advance_validated_output_demand<
        Family,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        disclosure: ValidatedOutputDisclosure<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        entry: &InstalledProducerProvider<Schema>,
        commit_authority: WorthQueryProducerCommitAuthority,
        performed: &std::cell::RefCell<super::super::required_wave::performed::PerformedMembers>,
    ) -> Result<
        CallerPass<FamilySourceQuery<Schema, Family>, FamilySourceValue<Schema, Family>>,
        WorthQueryOutputDemandDenial,
    >
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceValue<Schema, Family>: 'static,
        FamilySourceQuery<Schema, Family>: 'static,
    {
        use crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandAdvanceAdmission as Admission;
        let waiting = || {
            #[cfg(test)]
            own_effect_observation::record(Stage::CheckpointWait);
            CallerPass::Answer(WorthQueryOutputDemandAdvance::Pending)
        };
        let wave_authority = commit_authority.clone();
        let checkpoint_progress = match self.advance_ordinary_validated_output_demand(
            phase,
            demand,
            principal,
            request_scope,
            delivery_branch,
            disclosure,
            entry,
            commit_authority,
            performed,
        )? {
            OwnStages::Answer(advance) => return Ok(CallerPass::Answer(advance)),
            OwnStages::Refreshed(disclosure) => return Ok(CallerPass::Refreshed(disclosure)),
            OwnStages::Checkpoint(progress) => progress,
        };
        // The checkpoint this call published or moved is its own to finish.
        // A stage that did not move waits on a delivery outside this call.
        let completion = loop {
            let interest = demand
                .interest
                .as_ref()
                .expect("a published row retains its live Interest");
            match self.output_demands.begin_published(interest) {
                Admission::AdvanceCheckpoint { claim, checkpoint } => {
                    if !self.advance_output_checkpoint(
                        execution,
                        phase,
                        interest,
                        &demand.selected.identity,
                        claim,
                        checkpoint,
                    )? {
                        return Ok(waiting());
                    }
                }
                Admission::Ready(completion) => break completion,
                Admission::Failed(denial) => return Err(denial),
                _ => return Ok(waiting()),
            }
        };
        performed.borrow_mut().capture_ordinary(
            demand
                .interest
                .as_ref()
                .expect("Ready retains its Interest")
                .key(),
            &completion,
            self,
        )?;
        if matches!(checkpoint_progress, CheckpointProgress::Published(_)) {
            return self.settle_own_ready(
                execution,
                phase,
                demand,
                &completion,
                delivery_branch,
                checkpoint_progress,
            );
        }
        // The Ready this call reached is certified as any Ready is: by the
        // required wave when it has one, and otherwise on its own proof.
        if let Some(advance) = super::super::required_wave::advance_required_before_caller(
            execution,
            phase,
            self,
            demand,
            principal,
            request_scope,
            delivery_branch,
            &wave_authority,
            performed,
        )? {
            return Ok(CallerPass::Answer(advance));
        }
        self.settle_own_ready(
            execution,
            phase,
            demand,
            &completion,
            delivery_branch,
            checkpoint_progress,
        )
    }
}
