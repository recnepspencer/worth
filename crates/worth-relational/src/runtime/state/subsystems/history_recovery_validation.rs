use crate::branch::{RelationalBranchCellCheckpoint, RelationalBranchReferenceCell};
use crate::history::data::{BranchId, CanonicalCommitEnvelope, CommitId};
use worth_foundational::FoundationalBranchTarget;

use super::history_recovery_lineage::{
    validate_branch_target_lineage, validate_target_authoring_lineage,
};
use super::HistorySubsystem;

/// Replay may reacquire a retention lease while reconstructing a commit. That
/// lease is not branch currentness; the owner truth axes must still match the
/// carried checkpoint exactly before the envelope is admitted.
pub(super) fn branch_cell_truth_matches(
    existing: &RelationalBranchCellCheckpoint,
    incoming: &RelationalBranchCellCheckpoint,
) -> bool {
    existing.runtime_instance_id == incoming.runtime_instance_id
        && existing.branch_id == incoming.branch_id
        && existing.observation == incoming.observation
        && existing.truth_version == incoming.truth_version
        && existing.fork_provenance == incoming.fork_provenance
        && existing.fork_source_branch_id == incoming.fork_source_branch_id
}

/// A fresh runtime installs one empty owner cell before tail-only recovery can
/// readmit the carried pre-first-commit checkpoint. With no recovered history,
/// those cells carry the same empty truth even if their local truth counters
/// were initialized through different construction paths.
pub(super) fn empty_bootstrap_cells_are_equivalent(
    existing: &RelationalBranchCellCheckpoint,
    incoming: &RelationalBranchCellCheckpoint,
) -> bool {
    existing.runtime_instance_id == incoming.runtime_instance_id
        && existing.branch_id == incoming.branch_id
        && matches!(
            existing.observation.target(),
            FoundationalBranchTarget::Empty
        )
        && matches!(
            incoming.observation.target(),
            FoundationalBranchTarget::Empty
        )
        && existing.fork_provenance.is_none()
        && incoming.fork_provenance.is_none()
        && existing.fork_source_branch_id.is_none()
        && incoming.fork_source_branch_id.is_none()
}

/// Validate a branch-cell checkpoint admitted while replay is extending an
/// already restored checkpoint. Tail admission must use the same artifact and
/// fork-provenance court as the complete checkpoint restore; structural
/// deserialization alone is not an operational admission.
pub(super) fn validate_recovered_branch_cell(
    history: &HistorySubsystem,
    cell: &RelationalBranchReferenceCell,
) -> Result<(), String> {
    let branch_id = cell.identity().branch_id();
    let observation = cell.observation();
    let fork_source_branch_id = cell.fork_source_branch_id();
    let fork_provenance = cell.fork_provenance();
    validate_branch_target_artifact(history, branch_id, observation.target())?;
    validate_branch_target_lineage(
        history,
        branch_id,
        observation.target(),
        fork_source_branch_id.as_ref(),
        fork_provenance.as_ref(),
    )?;
    match (fork_source_branch_id, fork_provenance) {
        (Some(source_branch_id), Some(provenance)) => {
            let source_cell = history.branch_cell(&source_branch_id).ok_or_else(|| {
                format!(
                    "branch cell `{}` names missing fork source `{}`",
                    branch_id.0, source_branch_id.0
                )
            })?;
            if provenance.branch_id() != source_cell.observation().branch_id()
                || provenance.generation().get() > source_cell.observation().generation().get()
            {
                return Err(format!(
                    "branch cell `{}` fork provenance disagrees with source `{}`",
                    branch_id.0, source_branch_id.0
                ));
            }
            validate_branch_target_artifact(history, branch_id, provenance.target())?;
            validate_target_authoring_lineage(history, &source_branch_id, provenance.target())?;
            Ok(())
        }
        (None, None) => Ok(()),
        (Some(source_branch_id), None) => Err(format!(
            "branch cell `{}` names source `{}` without fork provenance",
            branch_id.0, source_branch_id.0
        )),
        (None, Some(_)) => Err(format!(
            "branch cell `{}` carries provenance without a fork source",
            branch_id.0
        )),
    }
}

pub(super) fn validate_branch_target_artifact(
    history: &HistorySubsystem,
    branch_id: &BranchId,
    target: &FoundationalBranchTarget<crate::branch::RelationalBranchTarget>,
) -> Result<(), String> {
    let FoundationalBranchTarget::Basis(target) = target else {
        return Ok(());
    };
    let selected_commit_id = CommitId(target.selected_commit_id());
    let artifact = history.commit_artifact(selected_commit_id).ok_or_else(|| {
        format!(
            "branch cell `{}` references missing commit artifact `{}`",
            branch_id.0, selected_commit_id.0
        )
    })?;
    if artifact.version_id().0 != target.version_id()
        || artifact
            .parentage()
            .as_slice()
            .iter()
            .map(|parent| parent.0)
            .collect::<Vec<_>>()
            != target.parent_commit_ids()
        || artifact.roots() != target.roots()
    {
        return Err(format!(
            "branch cell `{}` target does not match immutable commit artifact `{}`: target version/parents/roots = {}/{:?}/{:?}, artifact = {}/{:?}/{:?}",
            branch_id.0,
            selected_commit_id.0,
            target.version_id(),
            target.parent_commit_ids(),
            target.roots(),
            artifact.version_id().0,
            artifact.parentage().as_slice(),
            artifact.roots(),
        ));
    }
    Ok(())
}

pub(super) fn require_branch_target_artifact(
    history: &HistorySubsystem,
    branch_id: &BranchId,
    target: &FoundationalBranchTarget<crate::branch::RelationalBranchTarget>,
) -> Result<(), String> {
    let FoundationalBranchTarget::Basis(target) = target else {
        return Ok(());
    };
    let commit_id = CommitId(target.selected_commit_id());
    if history.commit_artifact(commit_id).is_none() {
        return Err(format!(
            "branch cell `{}` references missing commit artifact `{}`",
            branch_id.0, commit_id.0
        ));
    }
    Ok(())
}

pub(super) fn validate_branch_target_envelope(
    envelope: &CanonicalCommitEnvelope,
    branch_id: &BranchId,
    target: &FoundationalBranchTarget<crate::branch::RelationalBranchTarget>,
) -> Result<(), String> {
    let FoundationalBranchTarget::Basis(target) = target else {
        return Ok(());
    };
    let commit_id = CommitId(target.selected_commit_id());
    if envelope.commit.commit_id != commit_id
        || envelope.commit.version_id.0 != target.version_id()
        || envelope
            .commit
            .parents
            .iter()
            .map(|parent| parent.0)
            .collect::<Vec<_>>()
            != target.parent_commit_ids()
    {
        return Err(format!(
            "branch cell `{}` target does not match commit envelope `{}`",
            branch_id.0, commit_id.0
        ));
    }
    Ok(())
}
