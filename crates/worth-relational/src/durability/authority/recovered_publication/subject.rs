//! Match original source and recorded component axes without predicting IDs.
use super::RelationalRecoveredPublicationAdmissionCause as Cause;
use crate::branch::{
    RecordedRelationalComponentReference as Recorded, RecordedRelationalComponentTarget as Target,
};
use crate::durability::authority::{
    ReadmittedRelationalAssociatedEffect, RecoveredRelationalRuntimeAuthority,
    RelationalAssociatedEffectSettlement as Settlement,
};
use crate::runtime::RelationalRuntime;

pub(super) fn validate(
    recovered: &RecoveredRelationalRuntimeAuthority,
    runtime: &RelationalRuntime,
    effect: &ReadmittedRelationalAssociatedEffect,
    predecessor: &Recorded,
    successor: &Recorded,
) -> Result<(), Cause> {
    if recovered.runtime_instance_id != runtime.runtime_instance_id()
        || !effect.subject().belongs_to_runtime(runtime)
    {
        return Err(Cause::ForeignRecoveryRuntime);
    }
    match effect.subject().current_settlement(runtime) {
        Some(Settlement::PerformedPending) => return Err(Cause::PendingSettlement),
        Some(Settlement::PerformedSettled | Settlement::CheckpointReadmitted) => {}
        None => return Err(Cause::MissingSettlementEvidence),
    }
    let source = effect.source_envelope();
    let original = source
        .branch_cell_checkpoint
        .as_ref()
        .ok_or(Cause::MissingOriginalPredecessor)?;
    let original = original
        .in_canonical_frame()
        .map_err(|_| Cause::SubjectMismatch)?;
    if predecessor.descriptor().branch_id() != &original.branch_id
        || predecessor.descriptor().reference() != &original.observation
        || predecessor.descriptor().truth_version() != original.truth_version
        || successor.descriptor().branch_id() != &source.branch_context
    {
        return Err(Cause::SubjectMismatch);
    }
    let Target::Canonical(target) = successor.target() else {
        return Err(Cause::SubjectMismatch);
    };
    if target.receipt() != effect.subject().receipt()
        || target.position() != effect.subject().position()
        || target.artifact_digest() != effect.subject().artifact_digest()
        || target.roots() != effect.subject().roots()
    {
        return Err(Cause::SubjectMismatch);
    }
    Ok(())
}
