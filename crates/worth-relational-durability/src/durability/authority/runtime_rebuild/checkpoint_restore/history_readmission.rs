//! Admit checkpoint canonical entries before installing routes and live roots.
use super::{branch_root_images, root_readmission};
use crate::durability::data::CheckpointRestoreWork;
use crate::durability::data::DurableCheckpoint;
use branch::history::data::VersionNode;
use branch::runtime::state::subsystems::history::HistorySubsystem;
use contracts::durability::data::recovery_errors::DurabilityError;
use std::sync::Arc;
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;
pub struct PreparedCheckpointHistory {
    pub(super) history: HistorySubsystem,
    pub(super) roots: root_readmission::ReadmittedRootImages,
}

pub fn prepare_history(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    history: &HistorySubsystem,
    registry: &contracts::schema::data::RelationalSchemaRegistry,
    checkpoint: &DurableCheckpoint,
    mut branch_roots: branch_root_images::RestoredBranchRootImages,
    symbols: &contracts::symbols::data::string_interner::StringInterner,
    work: &mut CheckpointRestoreWork,
) -> Result<
    PreparedCheckpointHistory,
    contracts::durability::data::recovery_stop::RelationalRecoveryStop,
> {
    let mut history = history.detached_owner_snapshot().map_err(|denial| {
        DurabilityError::new(
            contracts::durability::data::recovery_errors::RecoveryFailureClass::CorruptCheckpoint,
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
        let mut catalog = branch::history::commit::catalog::RelationalCommitCatalog::default();
        for commit in &checkpoint.envelopes {
            let original = branch::history::commit::original_payload::RelationalOriginalRootCommitEvidence::prepare(
                execution,
                Arc::clone(commit.canonical_arc()),
            )
            .map_err(super::super::original_inputs::root_stop)?;
            let artifact = match checkpoint.commit_descriptors.get(&commit.commit.commit_id) {
                Some(descriptor) => {
                    branch::history::commit::artifact::RelationalCommitArtifact::from_preencoded_with_descriptor(
                        original,
                        descriptor.clone(),
                    )
                }
                None => branch::history::commit::artifact::RelationalCommitArtifact::from_preencoded(original),
            }
            .map_err(|cause| {
                crate::durability::checkpoints::region_directory::corrupt(format!(
                    "checkpoint canonical admission: {cause:?}"
                ))
            })?;
            let artifact = artifact.prepare_catalog_owner(execution).map_err(
                contracts::durability::data::recovery_stop::RelationalRecoveryStop::Request,
            )?;
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
                        contracts::durability::data::recovery_errors::RecoveryFailureClass::CorruptCheckpoint,
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
                contracts::durability::data::recovery_errors::RecoveryFailureClass::CorruptCheckpoint,
                detail,
            )
        })?;
    work.branch_cells_readmitted += checkpoint.branch_cells.len();
    Ok(PreparedCheckpointHistory { history, roots })
}
