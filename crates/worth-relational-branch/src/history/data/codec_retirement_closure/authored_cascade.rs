//! Catalog-bounded closure from installed source ownership, never submitted reachability.
use worth_relational_contracts::internal_port::publication::data::publication_error::PublicationErrorOwnerPort as _;
use worth_relational_contracts::internal_port::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use super::{charge, CodecRetirementClosureStop as Stop};
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::history::data::CommitId;
use crate::identity::data::EntityId;
use crate::publication::data::AuthoredRetirementRefusal as Cause;
use crate::publication::data::PublicationError;
use crate::storage::overlay::PartitionAccess;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::RecordRef;
use worth_execution::{ExecutionRequest, MapKernelContext};

pub fn reject(cause: Cause, id: EntityId) -> TransactionCommitError {
    TransactionCommitError::publication(PublicationError::authored_retirement(
        cause,
        RecordRef::Entity(id),
    ))
}
pub fn stopped(stop: Stop) -> TransactionCommitError {
    match stop {
        Stop::Payload(detail) => crate::transactions::data::outcomes::transaction_commit_error::origin_denial(detail),
        Stop::Request(cause) => {
            TransactionCommitError::execution(crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
                kind: crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(cause),
                partition_identity: None,
            })
        }
    }
}

pub fn authored_birth(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &crate::history::data::codec_retirement_closure::history_request::RuntimeHistoryRequest<'_, '_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    anchor: EntityId,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<CommitId, TransactionCommitError> {
    if anchor.generation_value() == 0 {
        return Err(reject(Cause::InvalidAuthoredAnchor, anchor));
    }

    basis
        .inner
        .root
        .demand_full_partition(execution, anchor.partition_id)
        .map_err(|cause| stopped(super::request(cause)))?;
    let partition = basis
        .inner
        .root
        .partition_state(anchor.partition_id)
        .map_err(|cause| stopped(Stop::from(cause)))?
        .ok_or_else(|| reject(Cause::InvalidAuthoredAnchor, anchor))?;
    partition
        .entity_arena
        .get(&anchor)
        .filter(|row| row.is_live())
        .ok_or_else(|| reject(Cause::InvalidAuthoredAnchor, anchor))?;
    let created = partition
        .entity_arena
        .created_at_for_slot(anchor.slot_index())
        .ok_or_else(|| reject(Cause::InvalidAuthoredAnchor, anchor))?;
    let envelope = runtime
        .version(created, context)
        .map_err(stopped)?
        .ok_or_else(|| reject(Cause::InvalidAuthoredAnchor, anchor))?;
    if envelope.authored_source_restoration().is_some() {
        super::source_scope_closure::validate_historical_prefix(
            execution, runtime, basis, &envelope, anchor, context,
        )?;
    }
    require_authored_birth(&envelope, anchor, context).map_err(|failure| match failure {
        Stop::Request(_) => stopped(failure),
        Stop::Payload(_) => reject(Cause::InvalidAuthoredAnchor, anchor),
    })?;
    Ok(envelope.commit.commit_id)
}

pub(super) fn require_authored_birth(
    envelope: &crate::history::data::CanonicalCommitEnvelope,
    anchor: EntityId,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), Stop> {
    use crate::publication::patch::data::RecordStructuralChange as Change;
    charge(
        context,
        envelope.record_allocations.len() + envelope.patch.authoritative_record_patches.len() + 1,
    )?;
    let mut owned = false;
    for origin in envelope.output_origins() {
        charge(context, origin.outputs.len())?;
        owned |= origin.outputs.contains(&anchor);
    }
    if owned
        || envelope
            .record_allocations
            .iter()
            .filter(|allocation| allocation.record() == &RecordRef::Entity(anchor))
            .count()
            != 1
        || !envelope
            .patch
            .authoritative_record_patches
            .iter()
            .any(|patch| {
                patch.target == RecordRef::Entity(anchor)
                    && patch.structural_change == Change::Created
            })
    {
        return Err("authored birth is absent or belongs to a producer publication".into());
    }
    Ok(())
}
