//! Admit checkpoint canonical entries before installing routes and live roots.
use super::{branch_root_images, root_readmission};
use crate::durability::data::{CheckpointRestoreWork, DurabilityError, DurableCheckpoint};
use crate::history::data::VersionNode;
use crate::runtime::state::subsystems::history::HistorySubsystem;
use std::sync::Arc;
pub(super) struct PreparedCheckpointHistory {
    pub(super) history: HistorySubsystem,
    pub(super) roots: root_readmission::ReadmittedRootImages,
}

pub(super) fn prepare_history(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    history: &HistorySubsystem,
    registry: &crate::schema::data::RelationalSchemaRegistry,
    checkpoint: &DurableCheckpoint,
    mut branch_roots: branch_root_images::RestoredBranchRootImages,
    symbols: &crate::symbols::data::StringInterner,
    work: &mut CheckpointRestoreWork,
) -> Result<PreparedCheckpointHistory, crate::durability::data::RelationalRecoveryStop> {
    let mut history = history.detached_owner_snapshot().map_err(|denial| {
        DurabilityError::new(
            crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
            format!("history snapshot denied: {denial:?}"),
        )
    })?;
    let catalog = if let Some(directory) = checkpoint
        .directory
        .as_ref()
        .or(checkpoint.canonical_directory.as_ref())
    {
        super::selected_catalog::admit(execution, directory, |entry| {
            let ordinal = usize::try_from(entry.position.0.checked_sub(1)?).ok()?;
            checkpoint
                .envelopes
                .get(ordinal)
                .filter(|commit| commit.position() == entry.position)
                .cloned()
        })?
    } else {
        let mut catalog = crate::history::commit::RelationalCommitCatalog::default();
        for commit in &checkpoint.envelopes {
            let original = crate::branch::root::RelationalOriginalRootCommitEvidence::prepare(
                execution,
                Arc::clone(commit.canonical_arc()),
            )
            .map_err(super::super::original_inputs::root_stop)?;
            let artifact = match checkpoint.commit_descriptors.get(&commit.commit.commit_id) {
                Some(descriptor) => {
                    crate::history::commit::RelationalCommitArtifact::from_preencoded_with_descriptor(
                        original,
                        descriptor.clone(),
                    )
                }
                None => crate::history::commit::RelationalCommitArtifact::from_preencoded(original),
            }
            .map_err(|cause| {
                crate::durability::checkpoints::region_directory::corrupt(format!(
                    "checkpoint canonical admission: {cause:?}"
                ))
            })?;
            let artifact = artifact
                .prepare_catalog_owner(execution)
                .map_err(crate::durability::data::RelationalRecoveryStop::Request)?;
            catalog
                .append_preencoded_recovery(artifact)
                .map_err(|cause| {
                    crate::durability::checkpoints::region_directory::corrupt(format!(
                        "checkpoint canonical insertion: {cause:?}"
                    ))
                })?;
        }
        catalog
    };
    history.install_commit_catalog(catalog);
    history.with_ledger_mut(|ledger| {
        ledger.patch_stream_index = checkpoint
            .envelopes
            .iter()
            .map(|commit| (commit.position(), commit.envelope().commit.commit_id))
            .collect();
    });
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
        execution,
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
            &roots.roots,
            registry,
        )
        .map_err(|detail| {
            DurabilityError::new(
                crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
                detail,
            )
        })?;
    work.branch_cells_readmitted += checkpoint.branch_cells.len();
    Ok(PreparedCheckpointHistory { history, roots })
}
