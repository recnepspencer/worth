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

/// Describe the actual rejected route without changing its admission or cause.
pub(super) fn reject_extra_owner(
    owners: &super::super::RelationalAuthoredCoupledOwners,
    subject: crate::identity::data::EntityId,
    detail: String,
) -> TransactionCommitError {
    let source_kind = match &owners.source {
        super::super::CoupledSource::Edit(_) => "edit",
        super::super::CoupledSource::Retirement { .. } => "retirement",
        super::super::CoupledSource::Restoration { .. } => "restoration",
        super::super::CoupledSource::MemberEditRetirement { .. } => "member-edit-retirement",
        super::super::CoupledSource::MemberEditRestoration { .. } => "member-edit-restoration",
    };
    let mut error = crate::publication::data::PublicationError::authored_retirement(
        Cause::ExtraCoupledOwner,
        RecordRef::Entity(subject),
    );
    error.detail.push_str(&format!(
        "; {detail}; source_kind={source_kind}; source_root={:?}",
        owners.source.subject(),
    ));
    TransactionCommitError::publication(error)
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
        let first_batch = self.transaction.batches().len();
        let first_fragment = self.mutation.outcome.effect.publication.patch_fragments.len();
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
        let chain = super::succession::Chain {
            batches: first_batch..self.transaction.batches().len(),
            fragments: (first_fragment..publication_effect.patch_fragments.len()).collect(),
            superseded: Vec::new(),
            local_keys: applied.local_keys,
            created: applied.created,
            records_end: publication_effect.changed_records.len(),
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
            &latest.records,
            &latest.input,
        )
    }
}
