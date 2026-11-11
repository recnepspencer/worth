#[path = "history_branch_cells/initial_schema.rs"]
mod initial_schema;
#[path = "history_branch_cells/source_empty_schema.rs"]
mod source_empty_schema;
use std::collections::BTreeMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use worth_foundational::FoundationalBranchTarget;

use crate::branch::{
    RelationalBranchCellCheckpoint, RelationalBranchCellDenial, RelationalBranchReferenceCell,
    RelationalBranchRoot,
};
use crate::history::data::{BranchId, CommitId};

use super::{history_recovery_validation, HistorySubsystem};

impl HistorySubsystem {
    /// Complete capture calls this inside the actual configuration/publication
    /// freeze, before those guards end; the output owns DATA only.
    pub(crate) fn capture_checkpoint_branch_metadata(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        runtime_name: &str,
    ) -> Result<
        crate::branch::CheckpointBranchMetadata,
        crate::branch::CheckpointBranchMetadataCaptureDenial,
    > {
        self.record_branch_population_scan();
        self.branch_cells
            .capture_checkpoint_metadata(request, runtime_name)
    }

    /// Published branch truth closes initial installation even before settlement.
    pub(crate) fn has_published_branch_basis(&self) -> bool {
        self.branch_cells
            .values()
            .into_iter()
            .any(|cell| !matches!(cell.observation().target(), FoundationalBranchTarget::Empty))
    }

    pub(crate) fn rebuild_branch_head_version_index(
        &self,
        lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    ) {
        let roots = self
            .branch_cells
            .values()
            .into_iter()
            .filter_map(|cell| cell.root())
            .collect::<Vec<_>>();
        self.branch_head_versions.rebuild(lane, roots);
    }

    pub(crate) fn admit_recovered_branch_cell(
        &self,
        lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        checkpoint: RelationalBranchCellCheckpoint,
        expected_branch_id: &BranchId,
        recovered_root: Option<Arc<crate::branch::RelationalBranchRoot>>,
        recovered_provenance_root: Option<Arc<crate::branch::RelationalBranchRoot>>,
        schema_registry: &crate::schema::data::RelationalSchemaRegistry,
        symbols: &crate::symbols::data::StringInterner,
    ) -> Result<(), super::history_catalog_recovery::RelationalHistoryRecoveryDenial> {
        // A replayed commit's prior cell state that equals a cell recovery
        // already admitted (from the checkpoint or an earlier tail commit),
        // once rebound to this runtime, was validated then, including its fork
        // provenance. Readmitting it again would demand the provenance root,
        // which a checkpoint taken after the source branch moved on no longer
        // retains.
        if self
            .branch_cell(expected_branch_id)
            .is_some_and(|existing| {
                RelationalBranchReferenceCell::from_checkpoint_with_root(
                    self.runtime_instance_id,
                    checkpoint.clone(),
                    None,
                )
                .is_ok_and(|rebound| {
                    history_recovery_validation::branch_cell_truth_matches(
                        &existing.checkpoint(),
                        &rebound.checkpoint(),
                    )
                })
            })
        {
            return Ok(());
        }
        if let Some(FoundationalBranchTarget::Basis(target)) = checkpoint
            .fork_provenance
            .as_ref()
            .map(|provenance| provenance.target())
        {
            let commit_id = CommitId(target.selected_commit_id());
            self.readmit_replayed_root_descriptor(
                lane,
                execution,
                commit_id,
                target.roots().clone(),
                recovered_provenance_root.ok_or_else(|| {
                    format!("recovery provenance root `{}` is unavailable", commit_id.0)
                })?,
                symbols,
            )?;
        }
        let readmission_root = match checkpoint.observation.target() {
            FoundationalBranchTarget::Empty => Some(RelationalBranchRoot::empty_with_schema(
                schema_registry,
                crate::schema::data::runtime_descriptor_semantics_policy().current_write_version(),
            )),
            FoundationalBranchTarget::Basis(target) => {
                let commit_id = CommitId(target.selected_commit_id());
                Some(
                    self.readmit_replayed_root_descriptor(
                        lane,
                        execution,
                        commit_id,
                        target.roots().clone(),
                        recovered_root.ok_or_else(|| {
                            format!("recovery root `{}` is unavailable", commit_id.0)
                        })?,
                        symbols,
                    )?,
                )
            }
        };
        history_recovery_validation::require_branch_target_artifact(
            self,
            expected_branch_id,
            checkpoint.observation.target(),
        )?;
        super::history_recovery_lineage::validate_branch_target_lineage(
            self,
            expected_branch_id,
            checkpoint.observation.target(),
            checkpoint.fork_source_branch_id.as_ref(),
            checkpoint.fork_provenance.as_ref(),
        )?;
        history_recovery_validation::validate_branch_target_artifact(
            self,
            expected_branch_id,
            checkpoint.observation.target(),
        )?;
        let cell = RelationalBranchReferenceCell::from_checkpoint_with_root(
            self.runtime_instance_id,
            checkpoint,
            readmission_root,
        )
        .map_err(|denial| format!("invalid durable branch-cell state: {denial:?}"))?;
        let branch_id = cell.identity().branch_id().clone();
        if &branch_id != expected_branch_id {
            return Err(format!(
                "recovery branch-cell `{}` does not match envelope branch `{}`",
                branch_id.0, expected_branch_id.0
            )
            .into());
        }
        history_recovery_validation::validate_recovered_branch_cell(self, &cell)?;
        if let Some(existing) = self.branch_cell(&branch_id) {
            let existing = existing.checkpoint();
            let incoming = cell.checkpoint();
            let pristine_bootstrap = self.recorded_commit_envelope_count() == 0
                && history_recovery_validation::empty_bootstrap_cells_are_equivalent(
                    &existing, &incoming,
                );
            if pristine_bootstrap {
                let previous = self.branch_cells.remove(&branch_id);
                drop(previous);
                let installed_root = cell
                    .root()
                    .ok_or_else(|| "recovered bootstrap branch has no owner root".to_owned())?;
                self.install_branch_head(
                    cell.identity().clone(),
                    &installed_root,
                    cell.head_retention(),
                )
                .map_err(|denial| {
                    format!("recovered bootstrap head retention denied: {denial:?}")
                })?;
                self.insert_branch_cell(cell);
                return Ok(());
            }
            if !history_recovery_validation::branch_cell_truth_matches(&existing, &incoming) {
                return Err(
                    format!("recovery branch-cell state conflicts for `{}`", branch_id.0).into(),
                );
            }
            return Ok(());
        }
        let installed_root = cell
            .root()
            .ok_or_else(|| "recovered branch head has no owner root".to_owned())?;
        self.install_branch_head(
            cell.identity().clone(),
            &installed_root,
            cell.head_retention(),
        )
        .map_err(|denial| format!("recovered branch head retention denied: {denial:?}"))?;
        self.insert_branch_cell(cell);
        self.move_branch_head_basis(None, Some(&installed_root));
        Ok(())
    }

    pub(crate) fn branch_cells_snapshot(&self) -> Vec<RelationalBranchCellCheckpoint> {
        self.record_branch_population_scan();
        self.branch_cells.checkpoints()
    }

    pub(crate) fn branch_ids_snapshot(&self) -> Vec<BranchId> {
        self.record_branch_population_scan();
        let mut branch_ids = self.branch_cells.keys();
        branch_ids.sort();
        branch_ids
    }

    pub(super) fn record_branch_population_scan(&self) {
        self.branch_population_scans.fetch_add(1, Ordering::Relaxed);
    }
}

fn branch_head_retirement_denial(
    denial: crate::history::retention::RelationalRetentionAcquisitionDenial,
) -> RelationalBranchCellDenial {
    match denial {
        crate::history::retention::RelationalRetentionAcquisitionDenial::CapacityExhausted => {
            RelationalBranchCellDenial::RetentionCapacityExhausted
        }
        crate::history::retention::RelationalRetentionAcquisitionDenial::IdentityExhausted => {
            RelationalBranchCellDenial::RetentionIdentityExhausted
        }
        crate::history::retention::RelationalRetentionAcquisitionDenial::OwnerUnavailable => {
            RelationalBranchCellDenial::RetentionOwnerUnavailable
        }
        crate::history::retention::RelationalRetentionAcquisitionDenial::RootSetTooLarge
        | crate::history::retention::RelationalRetentionAcquisitionDenial::BindingMismatch => {
            RelationalBranchCellDenial::RetentionRootSetTooLarge
        }
    }
}
