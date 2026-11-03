//! Only the sealed, payload-checked Native inverse is outside application authoring.
use super::*;
use crate::{
    mvcc::{RelationalAuthoredRestoration, RequiredOutputPublication},
    transactions::data::*,
};
type Error = ValidatedMutationTouchProjectionError;
fn plan(proposal: &ValidatedRelationalProposal) -> Option<&RelationalAuthoredRestoration> {
    match &proposal.required_output_publication.as_ref()?.publication {
        RequiredOutputPublication::AuthoredCascadeRestoration { restoration, .. } => {
            Some(restoration)
        }
        _ => None,
    }
}
pub(super) fn validate(
    proposal: &ValidatedRelationalProposal,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<(), Error> {
    let Some(restoration) = plan(proposal) else {
        return Ok(());
    };
    let result =
        execution.in_scope(|lease| {
            worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling())
                .run_with_checkpoint(lease, None, |context| {
                    crate::history::data::validate_authored_restoration_candidate(
                        proposal.validation_input.basis(),
                        &proposal.proposed_mutation.state,
                        &proposal.proposed_mutation.outcome.created_entities,
                        &proposal.proposed_mutation.outcome.created_relations,
                        restoration,
                        context,
                    )
                })
        });
    result
        .map_err(|cause| collection::execution(cause.into()))?
        .map_err(|cause| collection::execution(cause.into()))?
        .0
        .map_err(|error| match error {
            TransactionCommitError::Publication { error, .. } => Error::AuthoredRestoration(error),
            TransactionCommitError::Execution { denial, .. } => Error::Execution(denial),
            _ => Error::AuthoredRestoration(
                crate::publication::data::PublicationError::authored_retirement(
                    crate::publication::data::AuthoredRetirementRefusal::IncompleteRestoration,
                    RecordRef::Entity(
                        restoration
                            .retirement
                            .authored_cascade_retirement
                            .as_ref()
                            .expect("Native retirement")
                            .anchor,
                    ),
                ),
            ),
        })
}
pub(super) fn owns_create(
    proposal: &ValidatedRelationalProposal,
    intent: &MutationIntent,
    checkpoint: &mut dyn FnMut(u64) -> Result<(), Error>,
) -> Result<bool, Error> {
    let Some(restoration) = plan(proposal) else {
        return Ok(false);
    };
    match intent {
        MutationIntent::Create(CreateIntent::EntityAspects(spec)) => {
            let created = &proposal.proposed_mutation.outcome.created_entities;
            let target = CreatedEntityRef {
                partition_id: spec.partition_id,
                kind_id: spec.kind_id,
                client_key: spec.client_key.clone(),
            };
            checkpoint(
                created
                    .resolve_work_bound(&target)
                    .ok_or_else(collection::capacity)?,
            )?;
            let id = created.resolve(&target);
            // The distinguished source is still authored by the application.
            for (_, target) in restoration.entities.iter().skip(1) {
                checkpoint(
                    created
                        .resolve_work_bound(target)
                        .ok_or_else(collection::capacity)?,
                )?;
                if id.is_some() && id == created.resolve(target) {
                    return Ok(true);
                }
            }
        }
        MutationIntent::Create(CreateIntent::RelationAspects(spec)) => {
            let created = &proposal.proposed_mutation.outcome.created_relations;
            let target = CreatedRelationRef {
                partition_id: spec.partition_id,
                kind_id: spec.kind_id,
                client_key: spec.client_key.clone(),
                source: spec.source.clone(),
                target: spec.target.clone(),
            };
            checkpoint(
                created
                    .resolve_work_bound(&target)
                    .ok_or_else(collection::capacity)?,
            )?;
            let id = created.resolve(&target);
            for (_, target) in &restoration.relations {
                checkpoint(
                    created
                        .resolve_work_bound(target)
                        .ok_or_else(collection::capacity)?,
                )?;
                if id.is_some() && id == created.resolve(target) {
                    return Ok(true);
                }
            }
        }
        _ => {}
    }
    Ok(false)
}
