//! Judge one continued round by what it would actually apply, before it can publish.
use worth_relational_publication::internal_port::mvcc::publication::application_publication::coupled_assembly::producer_posture::RelationalCoupledProducerPostureOwnerPort as _;
use worth_relational_publication::internal_port::mvcc::publication::application_publication::custody::RelationalRequiredOutputPublicationFieldsPort as _;
use worth_relational_publication::internal_port::visibility::materialization::read_records::reader::pinned_read::RelationalPinnedReadOwnerPort as _;
use super::super::{RelationalCoupledAssemblyFailure, RequiredOutputPublication};
use super::{
    application, postponed, ExecutionRequest, RelationalAuthoredCoupledCandidate,
    RelationalCoupledProducerContinuation,
};
use crate::transactions::data::primitives::WorkerIntentBatch;
use std::sync::Arc;
use worth_relational_publication::internal_port::mvcc::publication::application_publication::coupled_assembly::continuation::round;

impl RelationalAuthoredCoupledCandidate<'_> {
    /// Consume next-round custody and judge the continued round's body on a
    /// discarded image of D. An unchanged application leaves this candidate
    /// exactly as it was. Any effect refuses before publication: it would be
    /// this route's second producing round under one single-input origin.
    pub fn verify_producer_round(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        token: RelationalCoupledProducerContinuation,
        content: &crate::content::RelationalContentAttachments,
        batches: Vec<WorkerIntentBatch>,
    ) -> Result<(), RelationalCoupledAssemblyFailure> {
        let index = self
            .continuation
            .consume(token, self.identity, self.applied_parts())?;
        let source = self.parts[index]
            .publication
            .producing_meaning()
            .expect("admitted producer route")
            .source_scope;
        if !self.round_preserves_application(execution, index, content, batches)? {
            return Err(postponed(
                crate::publication::data::CoupledProvenancePostponement::ContinuedRoundEffect,
                source,
            )
            .into());
        }
        Ok(())
    }

    /// O(round commands + records they touch). Nothing here writes this
    /// candidate: commands apply to a fresh working state over the current
    /// pin and reserve symbols only in a discarded copy of its dictionary.
    fn round_preserves_application(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        index: usize,
        content: &crate::content::RelationalContentAttachments,
        batches: Vec<WorkerIntentBatch>,
    ) -> Result<bool, RelationalCoupledAssemblyFailure> {
        if !content.is_empty() {
            return Ok(false);
        }
        if !application::commands_are_in_place(execution, &batches)? {
            return Ok(false);
        }
        if batches.iter().all(|batch| batch.intents.is_empty()) {
            return Ok(true);
        }
        let predecessor = match self.parts[index].publication.owned() {
            Some(RequiredOutputPublication::Preserve { predecessor, .. }) => {
                Some(Arc::clone(predecessor.origin()))
            }
            _ => None,
        };
        let read = self.pin_source(execution)?;
        let root = read.resident_basis().root();
        let runtime = self
            .runtime
            .with_candidate_symbols(self.mutation.candidate_symbols());
        let intents = crate::mvcc::transaction::normalized_plan::OwnedNormalizedPlan::copy_batches(
            self.transaction.transaction_id(),
            &batches,
            execution,
        )?;
        let plan = self.transaction.build_merged_plan_for_state(
            &runtime,
            root.as_ref(),
            intents,
            execution,
        )?;
        round::verify(
            execution,
            root,
            plan.plan(),
            &self.binding.owners,
            predecessor.as_deref(),
            runtime.config.as_ref(),
            (&runtime).into(),
            runtime.commit_work_budget.as_ref(),
            &self.prepared.schema_authority,
            self.identity.proposed_version_id(),
            &mut self.mutation.identities,
        )
    }
}
