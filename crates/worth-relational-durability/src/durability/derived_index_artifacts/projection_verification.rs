//! Readmit the selected original roots before running the index verifier.
use crate::durability::checkpoints::region_directory::{
    ReadmittedRegionDirectory, ReadmittedSchemas,
};
use crate::durability::data::RelationalCheckpointDirectory;
use contracts::durability::data::recovery_errors::DurabilityErrorOwnerPort as _;
use contracts::durability::data::recovery_stop::RelationalRecoveryStop;
use contracts::indexes::data::basis_checkpoint::IndexBasisCheckpoint;
use contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane;
use std::sync::Arc;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};
use worth_relational_branch::internal_port::runtime::state::subsystems::history::HistorySubsystem;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_indexes::internal_port::indexes::authority::checkpoint_admission::CheckpointIndexProjections;
use worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::IndexingState;
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
        contracts::durability::data::recovery_stop::account_work(execution, 1)?;
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
                    contracts::durability::data::recovery_stop::account_work(execution, 1)?;
                    if header.commit_id == basis.commit {
                        selected = Some(header);
                        break;
                    }
                }
                let header = selected.ok_or_else(|| {
                    contracts::durability::data::recovery_errors::DurabilityError::corrupt_checkpoint(
                        "index basis has no actual retained root image",
                    )
                })?;
                let root = regions.readmit_root(history, header, execution)?;
                roots
                    .insert_cloned_native_for_request(&basis.commit, &root, *execution)
                    .map_err(contracts::durability::data::recovery_stop::map_stop)?;
                root
            }
        };
        let selection = root.admit_checkpoint_projection(lane, execution, &basis.branch)?;
        let projection = projections.context().project(lane, selection);
        worth_relational_indexes::internal_port::indexes::authority::checkpoint_admission::verify_basis(
            lane,
            execution,
            indexes,
            basis,
            projection.view(),
        )?;
    }
    Ok(())
}
