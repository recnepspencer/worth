//! Admit and apply one authenticated surviving writer's own successor commands.
use super::*;
use crate::execution::request_stage::checkpoint;
use crate::{
    authority::commit::phases::prepare::prepare_authoritative_working_state_scope_for_base,
    publication::data::AuthoredRetirementRefusal as Cause,
    transactions::data::{AuthoritativeApplyPlan, RecordRef, WorkerIntentBatch},
};

pub(super) fn reject(
    cause: Cause,
    subject: crate::identity::data::EntityId,
) -> TransactionCommitError {
    TransactionCommitError::publication(
        crate::publication::data::PublicationError::authored_retirement(
            cause,
            RecordRef::Entity(subject),
        ),
    )
}

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
        self.parts.push(CoupledPublicationPart {
            ordinary_ids,
            input: applied.read,
            publication,
            commands: applied.plan,
            records: applied.records,
        });
        let latest = self.parts.last().expect("the actual part just appended");
        self.advance_pin(
            execution,
            &applied.delta_state,
            &latest.records,
            &latest.input,
        )
    }
}
