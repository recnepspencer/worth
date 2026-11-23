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
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
    match effect.origin() {
        super::super::associated_effects::ReadmittedEffectOrigin::Original(source) => {
            let source = source.envelope();
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
            {
                return Err(Cause::SubjectMismatch);
            }
            validate_original_successor_axes(
                &original,
                &source.branch_context,
                successor.descriptor(),
            )?;
        }
        super::super::associated_effects::ReadmittedEffectOrigin::Checkpoint { commit, .. } => {
            commit
                .require_publication_axes(
                    predecessor.descriptor(),
                    successor.descriptor(),
                    execution,
                )
                .map_err(Cause::AdmissionImage)?;
        }
    }
    validate_successor_target(effect, successor)
}

pub(super) fn validate_pruned_predecessor(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    recovered: &RecoveredRelationalRuntimeAuthority,
    runtime: &RelationalRuntime,
    effect: &ReadmittedRelationalAssociatedEffect,
    expected_branch: &crate::history::data::BranchId,
    expected_predecessor_commit: crate::history::data::CommitId,
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
    if &effect.subject().receipt().branch_id != expected_branch
        || !effect
            .subject()
            .receipt()
            .parents
            .contains(&expected_predecessor_commit)
    {
        return Err(Cause::SubjectMismatch);
    }
    match effect.origin() {
        super::super::associated_effects::ReadmittedEffectOrigin::Original(source) => {
            let source = source.envelope();
            let original = source
                .branch_cell_checkpoint
                .as_ref()
                .ok_or(Cause::MissingOriginalPredecessor)?;
            let original = original
                .in_canonical_frame()
                .map_err(|_| Cause::SubjectMismatch)?;
            let worth_foundational::FoundationalBranchTarget::Basis(prior_target) =
                original.observation.target()
            else {
                return Err(Cause::SubjectMismatch);
            };
            if &original.branch_id != expected_branch
                || prior_target.selected_commit_id() != expected_predecessor_commit.0
            {
                return Err(Cause::SubjectMismatch);
            }
            validate_original_successor_axes(
                &original,
                &source.branch_context,
                successor.descriptor(),
            )?;
        }
        super::super::associated_effects::ReadmittedEffectOrigin::Checkpoint { commit, .. } => {
            commit
                .require_pruned_publication_axes(
                    expected_branch,
                    expected_predecessor_commit,
                    successor.descriptor(),
                    execution,
                )
                .map_err(Cause::AdmissionImage)?;
        }
    }
    validate_successor_target(effect, successor)
}

fn validate_successor_target(
    effect: &ReadmittedRelationalAssociatedEffect,
    successor: &Recorded,
) -> Result<(), Cause> {
    let Target::Canonical(target) = successor.target() else {
        return Err(Cause::SubjectMismatch);
    };
    let receipt = effect.subject().receipt();
    if target.receipt() != receipt
        || target.position() != effect.subject().position()
        || target.artifact_digest() != effect.subject().artifact_digest()
        || target.roots() != effect.subject().roots()
    {
        return Err(Cause::SubjectMismatch);
    }
    let worth_foundational::FoundationalBranchTarget::Basis(ref_target) =
        successor.descriptor().reference().target()
    else {
        return Err(Cause::SubjectMismatch);
    };
    if successor.descriptor().branch_id() != &receipt.branch_id
        || ref_target.selected_commit_id() != receipt.commit_id.0
        || ref_target.version_id() != receipt.version_id.0
        || ref_target.roots() != effect.subject().roots()
        || ref_target
            .parent_commit_ids()
            .iter()
            .copied()
            .ne(receipt.parents.iter().map(|id| id.0))
    {
        return Err(Cause::SubjectMismatch);
    }
    Ok(())
}

/// Bind Original successor axes with the exact same transition rule used to
/// project checkpoint admission rows (`admission_projection/row.rs`).
fn validate_original_successor_axes(
    original: &crate::branch::RelationalBranchCellCheckpoint,
    branch_context: &crate::history::data::BranchId,
    successor: &crate::branch::RelationalBranchBasisDescriptor,
) -> Result<(), Cause> {
    let expected_generation = original
        .observation
        .generation()
        .checked_advance()
        .map_err(|_| Cause::SubjectMismatch)?;
    let expected_truth = original
        .truth_version
        .checked_advance()
        .ok_or(Cause::SubjectMismatch)?;
    if successor.branch_id() != branch_context
        || successor.reference().generation() != expected_generation
        || successor.truth_version() != expected_truth
        || successor.posture() != crate::branch::RelationalBranchBasisPosture::Live
    {
        return Err(Cause::SubjectMismatch);
    }
    Ok(())
}
