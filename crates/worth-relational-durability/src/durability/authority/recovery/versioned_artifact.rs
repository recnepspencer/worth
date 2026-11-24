//! Reconstructed artifact installation coordinates a prepared ledger and the recovered head.
use std::sync::Arc;
use worth_relational_branch::internal_port as branch;

use branch::history::data::positioned_canonical_commit::PositionedCanonicalCommit;
use branch::runtime::state::subsystems::history::history_publication::PreparedRecoveredVersionedArtifactPublication;
use branch::runtime::state::subsystems::history::HistorySubsystem;

pub fn install_recovered_versioned_artifact(
    history: &HistorySubsystem,
    prepared: PreparedRecoveredVersionedArtifactPublication,
    positioned: &PositionedCanonicalCommit,
) -> Result<(), String> {
    let (mut next_cell, root, accelerators) = prepared.into_installation();
    next_cell.install_root(Arc::clone(&root));
    let branch_id = accelerators.branch_id().clone();
    let current_cell = history.branch_cell(&branch_id).ok_or_else(|| {
        format!(
            "prepared recovery branch `{}` is not registered",
            branch_id.0
        )
    })?;
    let previous_root = current_cell.root().ok_or_else(|| {
        format!(
            "prepared recovery branch `{}` has no retained head",
            branch_id.0
        )
    })?;
    let head_retirement = history
        .reserve_branch_head_retirement(
            current_cell.identity(),
            &previous_root,
            current_cell.head_retention(),
        )
        .map_err(|denial| format!("recovery head replacement denied: {denial:?}"))?;
    let next_root = root;
    accelerators.install(history, positioned.position());
    // Recovery truth still installs a complete reconstructed owner cell.
    current_cell.replace_state(next_cell.state_snapshot());
    head_retirement.transfer_head(&next_root).replace_head();
    Ok(())
}
