//! Readmit the selected original roots before running the index verifier.
use crate::durability::checkpoints::region_directory::{
    ReadmittedRegionDirectory, ReadmittedSchemas,
};
use crate::durability::data::recovery_stop::RelationalRecoveryStop;
use crate::durability::data::RelationalCheckpointDirectory;
use crate::indexes::authority::CheckpointIndexProjections;
use crate::indexes::data::IndexBasisCheckpoint;
use crate::runtime::state::subsystems::history::HistorySubsystem;
use crate::runtime::state::subsystems::indexing::IndexingState;
use crate::storage::substrate::maintenance::ReconstructionLane;
use std::sync::Arc;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};
pub(crate) fn verify_with_images(
    projections: &CheckpointIndexProjections,
    lane: &ReconstructionLane<'_>,
    execution: &ExecutionRequest<'_, '_>,
    indexes: &IndexingState,
    bases: &[IndexBasisCheckpoint],
    directory: Arc<RelationalCheckpointDirectory>,
    history: &HistorySubsystem,
    schemas: ReadmittedSchemas,
) -> Result<(), RelationalRecoveryStop> {
    let mut regions = ReadmittedRegionDirectory::from_readmitted_schemas(
        directory.clone(),
        projections.context().symbols(),
        schemas,
        execution,
    )?;
    let mut roots = ExecutionOrderedMap::default();
    for basis in bases {
        crate::durability::data::recovery_stop::account_work(execution, 1)?;
        let current = history
            .branch_cell(&basis.branch)
            .and_then(|cell| cell.root())
            .filter(|root| root.commit_id() == Some(basis.commit));
        let root = match current.or_else(|| roots.get(&basis.commit).cloned()) {
            Some(root) => root,
            None => {
                // This is the retained-root header roster, never a canonical timeline.
                // Read no other root's member, payload, directory or schema body.
                let mut selected = None;
                for header in directory.manifest.roots.iter() {
                    crate::durability::data::recovery_stop::account_work(execution, 1)?;
                    if header.commit_id == basis.commit {
                        selected = Some(header);
                        break;
                    }
                }
                let header = selected.ok_or_else(|| {
                    crate::durability::data::recovery_errors::DurabilityError::corrupt_checkpoint(
                        "index basis has no actual retained root image",
                    )
                })?;
                let root = regions.readmit_root(history, header, execution)?;
                roots
                    .insert_cloned_native_for_request(&basis.commit, &root, *execution)
                    .map_err(crate::durability::data::recovery_stop::map_stop)?;
                root
            }
        };
        let projection = projections
            .context()
            .project(lane, execution, &basis.branch, root)?;
        crate::indexes::authority::checkpoint_admission::verify_basis(
            lane,
            execution,
            indexes,
            basis,
            projection.view(),
        )?;
    }
    Ok(())
}
