//! Admit checkpoint canonical entries before installing routes and live roots.
use super::{branch_root_images, root_readmission};
use crate::durability::data::{CheckpointRestoreWork, DurabilityError, DurableCheckpoint};
use crate::history::data::VersionNode;
use crate::runtime::{HistorySubsystem, RelationalRuntime};
use std::sync::Arc;

pub(super) fn prepare_history(
    restored: &mut RelationalRuntime,
    checkpoint: &DurableCheckpoint,
    mut branch_roots: branch_root_images::RestoredBranchRootImages,
    symbols: &crate::symbols::data::StringInterner,
    work: &mut CheckpointRestoreWork,
) -> Result<HistorySubsystem, DurabilityError> {
    let mut history = restored.history.detached_owner_snapshot();
    let mut catalog = crate::history::RelationalCommitCatalog::default();
    for commit in &checkpoint.envelopes {
        catalog
            .append_checkpoint_recovery(
                commit.canonical_arc().clone(),
                None,
                checkpoint.commit_descriptors.get(&commit.commit.commit_id),
            )
            .map_err(|denial| {
                DurabilityError::new(
                    crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
                    format!("checkpoint canonical admission: {denial:?}"),
                )
            })?;
    }
    history.install_commit_catalog(catalog);
    history.with_ledger_mut(|ledger| {
        ledger.patch_stream_index = checkpoint
            .envelopes
            .iter()
            .map(|commit| (commit.position(), commit.envelope().commit.commit_id))
            .collect();
    });
    for commit in &checkpoint.envelopes {
        history
            .install_recovered_canonical_route(Arc::new(commit.clone()))
            .map_err(|detail| {
                DurabilityError::new(
                    crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
                    detail,
                )
            })?;
        work.history_envelopes_routed += 1;
    }
    if let Some(position) = history.latest_recorded_patch_position() {
        history.advance_canonical_stream_floor(position);
    }
    let commit_graph = checkpoint
        .envelopes
        .iter()
        .map(|envelope| {
            (
                envelope.commit.commit_id,
                VersionNode {
                    commit: envelope.commit.clone(),
                },
            )
        })
        .collect();
    history.with_ledger_mut(|ledger| ledger.commit_graph = commit_graph);
    let cells = checkpoint
        .envelopes
        .iter()
        .map(|commit| {
            let id = commit.commit.commit_id;
            history
                .commit_artifact(id)
                .map(|artifact| (id, artifact.envelope_residency()))
                .ok_or_else(|| {
                    DurabilityError::new(
                        crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
                        "restored envelope has no admitted catalog entry",
                    )
                })
        })
        .collect::<Result<_, _>>()?;
    history.with_ledger_mut(|ledger| ledger.commit_envelopes = cells);
    let roots = root_readmission::readmit_selected_images(
        &history,
        checkpoint,
        &mut branch_roots,
        symbols,
    )?;
    history
        .restore_branch_cells(
            &checkpoint.branch_cells,
            &checkpoint.branch_lifecycle,
            &checkpoint.retired_branch_names,
            &roots,
            &restored.config.schema.registry,
        )
        .map_err(|detail| {
            DurabilityError::new(
                crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
                detail,
            )
        })?;
    work.branch_cells_readmitted += checkpoint.branch_cells.len();
    history.restore_branch_transition_coverage(checkpoint.coverage.up_to_branch_transition);
    Ok(history)
}
