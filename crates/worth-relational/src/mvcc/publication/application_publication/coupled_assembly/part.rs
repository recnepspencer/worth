//! Admit and apply one authenticated surviving writer's own successor commands.
use super::chain::Chain;
pub(super) use super::refusals::{reject, reject_extra_owner};
use super::*;
use crate::authority::commit::phases::prepare::prepare_authoritative_working_state_scope_for_base;
use crate::execution::request_stage::checkpoint;
use crate::publication::data::AuthoredRetirementRefusal as Cause;
use crate::transactions::data::outcomes::plan_artifacts::AuthoritativeApplyPlan;
use crate::transactions::data::primitives::RecordRef;
use crate::transactions::data::primitives::WorkerIntentBatch;

pub(super) fn require_unseen_route(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    parts: &[CoupledPublicationPart],
    meaning: &RequiredOutputPublicationMeaning,
) -> Result<(), TransactionCommitError> {
    for part in parts {
        let prior = part
            .publication
            .producing_meaning()
            .expect("admitted producer route");
        checkpoint(
            context,
            (meaning.family.len() + prior.family.len() + 65) as u64,
        )
        .map_err(custody::execution_stop)?;
        if prior.same_route(meaning) {
            return Err(reject(Cause::RepeatedCoupledOwner, meaning.source_scope));
        }
    }
    Ok(())
}

mod suffix;
pub(super) use suffix::SuffixWriter;

impl RelationalAuthoredCoupledCandidate<'_> {
    pub fn append_publication(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        mut publication: RelationalCoupledProducerPosture,
        content: &crate::content::RelationalContentAttachments,
        batches: Vec<WorkerIntentBatch>,
    ) -> Result<(), RelationalCoupledAssemblyFailure> {
        let predecessor_origin =
            super::participant_admission::admit(self, execution, &publication)?;
        let source = publication
            .producing_meaning()
            .expect("admitted writer")
            .source_scope;
        let index = self.parts.len();
        let first_record = self
            .mutation
            .outcome
            .effect
            .publication
            .changed_records
            .len();
        let first_batch = self.transaction.batches().len();
        let first_fragment = self
            .mutation
            .outcome
            .effect
            .publication
            .patch_fragments
            .len();
        let applied = self.apply_suffix(
            execution,
            SuffixWriter::Producer {
                index,
                source,
                roles: publication.roles_mut(),
                predecessor: predecessor_origin.as_deref(),
            },
            content,
            batches,
        )?;
        let ordinary_ids = publication
            .resolve_ordinary_roles(execution, &self.mutation.outcome.created_entities)?;
        let publication_effect = &self.mutation.outcome.effect.publication;
        let latest = super::succession::appended_rounds(&self.parts);
        let chain = Chain {
            head_input: applied.read.clone(),
            prior_touches: Vec::new(),
            batches: first_batch..self.transaction.batches().len(),
            fragments: (first_fragment..publication_effect.patch_fragments.len())
                .map(|position| (position, applied.read.clone()))
                .collect(),
            superseded: Vec::new(),
            local_keys: applied.local_keys,
            created: applied.created,
            rounds: vec![first_record..publication_effect.changed_records.len()],
            latest,
        };
        self.parts.push(CoupledPublicationPart {
            inputs: None,
            ordinary_ids,
            input: applied.read,
            publication,
            commands: applied.plan,
            records: applied.records,
            chain,
        });
        let latest = self.parts.last().expect("the actual part just appended");
        self.advance_pin(
            execution,
            &applied.delta_state,
            &applied.content,
            &latest.records,
            &latest.input,
        )
    }
}
