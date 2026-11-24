//! Inspect the exact appended producer routes and their owned records.
use super::super::custody;
use super::producer_posture::resolve_applied_roles;
use super::retained_assembly::{BoundAuthoredCoupling, CoupledPublicationPart};
use contracts::execution::request_stage::checkpoint;
use contracts::identity::data::record_ids::EntityId;
use contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaning;
use contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal as Cause;
use contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_execution::{ExecutionArray, ExecutionRequest};
use worth_relational_contracts::internal_port as contracts;

pub fn prepared_index(
    execution: &ExecutionRequest<'_, '_>,
    parts: &[CoupledPublicationPart],
    family: &str,
    source: EntityId,
    partition: &[u8; 32],
) -> Result<Option<usize>, TransactionCommitError> {
    execution
        .run_native_with_checkpoint(|context| {
            for (index, part) in parts.iter().enumerate() {
                let meaning = part
                    .publication
                    .producing_meaning()
                    .expect("admitted producing part");
                checkpoint(context, (family.len() + meaning.family.len() + 65) as u64)
                    .map_err(custody::execution_stop)?;
                if meaning.family == family
                    && meaning.source_scope == source
                    && meaning.source_partition.bytes() == partition
                {
                    return Ok(Some(index));
                }
            }
            Ok(None)
        })
        .map_err(custody::execution_stop)?
        .0
}

pub fn output_entities(
    execution: &ExecutionRequest<'_, '_>,
    parts: &[CoupledPublicationPart],
    created_entities: &CommitCreatedEntityBindings,
    producer: &str,
    meaning: &RequiredOutputPublicationMeaning,
) -> Result<Option<ExecutionArray<EntityId>>, TransactionCommitError> {
    for part in parts {
        execution
            .run_native_with_checkpoint(|context| checkpoint(context, 1))
            .map_err(custody::execution_stop)?
            .0
            .map_err(custody::execution_stop)?;
        if part
            .publication
            .lineage_attribution()
            .is_some_and(|(actual, _)| actual == producer)
            && part.publication.producing_meaning() == Some(meaning)
        {
            return resolve_applied_roles(execution, part.publication.roles(), created_entities)
                .map(Some);
        }
    }
    Ok(None)
}

/// The exact refusal and subject shared by selection and continuation issuance.
pub fn continuation_refusal(
    parts: &[CoupledPublicationPart],
    binding: &BoundAuthoredCoupling,
    index: usize,
) -> Option<(Cause, EntityId)> {
    let Some(part) = parts.get(index) else {
        return Some((Cause::MissingCoupledOwner, binding.owners.source.subject()));
    };
    let source = part
        .publication
        .producing_meaning()
        .expect("admitted part")
        .source_scope;
    // Only an Edit may observe its already appended owned route again;
    // a retirement or restoration still requires its exact preserving
    // participants.
    if !binding.owners.source.is_edit() {
        return Some((Cause::CoupledContinuationRequiresEditSource, source));
    }
    // Only an owned publication of a producer that owns a source scope.
    let producer = part
        .publication
        .lineage_attribution()
        .expect("actual producer")
        .0;
    if part.publication.owned().is_none()
        || binding
            .installed
            .source_scope_ownership
            .scope_kind(producer)
            .is_none()
    {
        return Some((Cause::CoupledContinuationRequiresOwnedRoute, source));
    }
    None
}
