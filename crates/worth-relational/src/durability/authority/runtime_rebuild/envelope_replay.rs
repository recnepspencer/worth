use crate::durability::data::RelationalRecoveryStop;
use crate::durability::data::{DurabilityError, RecoveryFailureClass};
use crate::history::data::CanonicalCommitAuthorityKind;
use crate::runtime::RelationalRuntime;
use std::collections::BTreeSet;

use super::root_inventory::RecoveredRootInventory;

mod commit_stop;
mod effect;
mod owner_bindings;
mod preparation;

#[cfg(test)]
mod interruption_tests;

use effect::{
    complete_incomplete_metadata_commit, is_metadata_only_merge_commit, replay_envelope_effect,
    replay_merge_commit, replay_ordinary_commit, restore_authoritative_artifacts_when_required,
};
use preparation::prepare_replay_context;

pub(super) fn replay_readmitted_envelope(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    restored: &mut RelationalRuntime,
    readmitted: crate::durability::migration::ReadmittedCanonicalCommit,
    available_commit_ids: &BTreeSet<crate::history::data::CommitId>,
    restore_authoritative_envelope: bool,
    recovered_roots: &mut RecoveredRootInventory,
) -> Result<crate::history::data::PositionedCanonicalCommit, RelationalRecoveryStop> {
    let requires_completion = readmitted.needs_replay_completion();
    let envelope = readmitted.envelope();
    let commit_id = envelope.commit.commit_id;
    let had_checkpoint_artifact = prepare_replay_context(
        restored,
        envelope,
        readmitted.position(),
        available_commit_ids,
        recovered_roots,
        requires_completion,
    )?;
    if requires_completion {
        return replay_incomplete_envelope(
            execution,
            restored,
            readmitted,
            commit_id,
            restore_authoritative_envelope,
            had_checkpoint_artifact,
            recovered_roots,
        );
    }
    let positioned = readmitted.positioned().cloned().ok_or_else(|| {
        DurabilityError::new(
            RecoveryFailureClass::ReplayFailure,
            "exact recovery input lost its positioned canonical admission",
        )
    })?;
    let replays_mutation_pipeline = envelope.authority_kind()
        == CanonicalCommitAuthorityKind::VersionedTransaction
        && !is_metadata_only_merge_commit(envelope);
    if !replays_mutation_pipeline && !envelope.record_allocations().is_empty() {
        return Err(DurabilityError::new(
            RecoveryFailureClass::ReplayFailure,
            format!(
                "durable commit {} carries record allocation evidence without a mutation replay path",
                envelope.commit.commit_id.0
            ),
        ).into());
    }
    if replays_mutation_pipeline {
        restored
            .record_identity
            .stage_replay_allocations_with_leading_gaps(envelope.record_allocations().to_vec())
            .map_err(|detail| DurabilityError::new(RecoveryFailureClass::ReplayFailure, detail))?;
    }
    let replay_result = replay_envelope_effect(execution, restored, &positioned);
    let staged_was_not_consumed = restored.record_identity.clear_staged_replay_allocations();
    replay_result?;
    if staged_was_not_consumed && replays_mutation_pipeline {
        return Err(DurabilityError::new(
            RecoveryFailureClass::ReplayFailure,
            format!(
                "durable commit {} did not consume its canonical record allocation evidence",
                envelope.commit.commit_id.0
            ),
        )
        .into());
    }
    restore_authoritative_artifacts_when_required(
        restored,
        &positioned,
        restore_authoritative_envelope,
        !had_checkpoint_artifact,
    )?;
    recovered_roots.retain_current(restored, &positioned.envelope().branch_context)?;
    Ok(positioned)
}

fn replay_incomplete_envelope(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    restored: &mut RelationalRuntime,
    readmitted: crate::durability::migration::ReadmittedCanonicalCommit,
    commit_id: crate::history::data::CommitId,
    restore_authoritative_envelope: bool,
    had_checkpoint_artifact: bool,
    recovered_roots: &mut RecoveredRootInventory,
) -> Result<crate::history::data::PositionedCanonicalCommit, RelationalRecoveryStop> {
    let envelope = readmitted.envelope();
    let positioned =
        if envelope.authority_kind() == CanonicalCommitAuthorityKind::BranchReferenceMovement {
            complete_incomplete_metadata_commit(execution, restored, readmitted)?
        } else {
            if !envelope.record_allocations().is_empty() {
                return Err(DurabilityError::new(
                    RecoveryFailureClass::CorruptSegment,
                    "incomplete recovery input unexpectedly carried current record allocations",
                )
                .into());
            }
            if envelope.merge_parent_branches.is_empty() {
                replay_ordinary_commit(execution, restored, envelope, readmitted.position())?;
            } else {
                replay_merge_commit(execution, restored, envelope, readmitted.position())?;
            }
            let replayed = restored
                .history
                .canonical_envelope(commit_id)
                .ok_or_else(|| {
                    DurabilityError::new(
                        RecoveryFailureClass::ReplayFailure,
                        "incomplete recovery replay produced no current canonical evidence",
                    )
                })?;
            readmitted.complete(replayed.as_ref()).map_err(|detail| {
                DurabilityError::new(RecoveryFailureClass::CorruptSegment, detail)
            })?
        };
    restore_authoritative_artifacts_when_required(
        restored,
        &positioned,
        restore_authoritative_envelope,
        !had_checkpoint_artifact,
    )?;
    recovered_roots.retain_current(restored, &positioned.envelope().branch_context)?;
    Ok(positioned)
}
