use worth_foundational::FoundationalBranchTarget;

use crate::history::data::{BranchId, CommitId};

use super::HistorySubsystem;

pub(super) fn validate_branch_target_lineage(
    history: &HistorySubsystem,
    branch_id: &BranchId,
    target: &FoundationalBranchTarget<crate::branch::target::RelationalBranchTarget>,
    fork_source_branch_id: Option<&BranchId>,
    fork_provenance: Option<
        &crate::branch::reference::observation::RelationalBranchReferenceObservation,
    >,
) -> Result<(), String> {
    let FoundationalBranchTarget::Basis(target_basis) = target else {
        return Ok(());
    };
    let commit_id = CommitId(target_basis.selected_commit_id());
    let artifact = history.commit_artifact(commit_id).ok_or_else(|| {
        format!(
            "branch cell `{}` references missing commit artifact `{}`",
            branch_id.0, commit_id.0
        )
    })?;
    if artifact.identity().authoring_branch() == branch_id {
        return Ok(());
    }
    let inherited_fork_target =
        fork_provenance.is_some_and(|provenance| provenance.target() == target);
    let inherited_author = fork_source_branch_id.is_some_and(|source_branch_id| {
        branch_lineage_contains(
            history,
            source_branch_id,
            artifact.identity().authoring_branch(),
        )
    });
    if inherited_fork_target && inherited_author {
        return Ok(());
    }
    Err(format!(
        "branch cell `{}` target commit `{}` belongs to foreign branch stream `{}`",
        branch_id.0,
        commit_id.0,
        artifact.identity().authoring_branch().0
    ))
}

pub(super) fn validate_target_authoring_lineage(
    history: &HistorySubsystem,
    source_branch_id: &BranchId,
    target: &FoundationalBranchTarget<crate::branch::target::RelationalBranchTarget>,
) -> Result<(), String> {
    let FoundationalBranchTarget::Basis(target) = target else {
        return Ok(());
    };
    let commit_id = CommitId(target.selected_commit_id());
    let artifact = history.commit_artifact(commit_id).ok_or_else(|| {
        format!(
            "branch cell `{}` references missing commit artifact `{}`",
            source_branch_id.0, commit_id.0
        )
    })?;
    if branch_lineage_contains(
        history,
        source_branch_id,
        artifact.identity().authoring_branch(),
    ) {
        return Ok(());
    }
    Err(format!(
        "fork provenance for `{}` selects foreign branch stream `{}`",
        source_branch_id.0,
        artifact.identity().authoring_branch().0
    ))
}

fn branch_lineage_contains(
    history: &HistorySubsystem,
    branch_id: &BranchId,
    candidate: &BranchId,
) -> bool {
    let mut cursor = Some(branch_id.clone());
    let mut visited = std::collections::BTreeSet::new();
    while let Some(current) = cursor {
        if &current == candidate {
            return true;
        }
        if !visited.insert(current.clone()) {
            return false;
        }
        cursor = history
            .branch_cell(&current)
            .and_then(|cell| cell.fork_source_branch_id());
    }
    false
}
