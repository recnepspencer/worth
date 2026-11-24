//! Only the sealed, payload-checked Native inverse is outside application authoring.
use super::*;
use crate::mvcc::publication::application_publication::RelationalAuthoredRestoration;
use crate::mvcc::validation::validated_proposal::ValidatedRelationalProposal;
use crate::transactions::data::{
    intents::mutation_intent::MutationIntent,
    outcomes::transaction_commit_error::TransactionCommitError, primitives::RecordRef,
};
type Error = ValidatedMutationTouchProjectionError;
fn plan(proposal: &ValidatedRelationalProposal) -> Option<&RelationalAuthoredRestoration> {
    proposal
        .state
        .required_output_publication
        .as_ref()?
        .publication
        .authored_restoration()
}
pub(super) fn validate(
    proposal: &ValidatedRelationalProposal,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<(), Error> {
    let Some(restoration) = plan(proposal) else {
        return Ok(());
    };
    let validate = || validate_candidate(proposal, execution, restoration);
    match (
        crate::execution::request_custody::request_lease(execution),
        proposal.state.commit_work_budget.as_ref(),
    ) {
        (Some(_), Some(budget)) => budget
            .run_stage(*execution, validate)
            .map_err(|cause| collection::execution(cause.into()))?,
        _ => validate(),
    }
}
fn validate_candidate(
    proposal: &ValidatedRelationalProposal,
    execution: &ExecutionRequest<'_, '_>,
    restoration: &RelationalAuthoredRestoration,
) -> Result<(), Error> {
    let result =
        execution.in_scope(|lease| {
            worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling())
                .run_with_checkpoint(lease, None, |context| {
                    crate::history::data::validate_authored_restoration_candidate(
                        proposal.state.validation_input.basis(),
                        &proposal.state.proposed_mutation.state,
                        &proposal.state.proposed_mutation.outcome.created_entities,
                        &proposal.state.proposed_mutation.outcome.created_relations,
                        restoration,
                        lease,
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
                            .authored_source_retirement()
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
    let outcome = &proposal.state.proposed_mutation.outcome;
    super::inverse_projection::owns_create(
        super::inverse_projection::InverseProjection {
            created_entities: &outcome.created_entities,
            created_relations: &outcome.created_relations,
            non_anchor_entities: || {
                // Every anchor remains authored by its own inverse Create.
                let anchors = restoration
                    .retirement
                    .authored_source_retirement()
                    .expect("Native restoration has its retirement")
                    .co_anchors
                    .len()
                    + 1;
                restoration.entities.get(anchors..).unwrap_or_default()
            },
            relations: &restoration.relations,
        },
        intent,
        checkpoint,
    )
}
