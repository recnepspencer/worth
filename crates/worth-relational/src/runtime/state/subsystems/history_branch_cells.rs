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
    /// Published branch truth closes initial installation even before settlement.
    pub(crate) fn has_published_branch_basis(&self) -> bool {
        self.branch_cells
            .values()
            .into_iter()
            .any(|cell| !matches!(cell.observation().target(), FoundationalBranchTarget::Empty))
    }

    pub(crate) fn rebuild_branch_head_version_index(&self) {
        let versions = self
            .branch_cells
            .values()
            .into_iter()
            .filter_map(|cell| match cell.observation().target() {
                FoundationalBranchTarget::Empty => None,
                FoundationalBranchTarget::Basis(target) => {
                    Some(crate::identity::data::VersionId(target.version_id()))
                }
            })
            .collect::<Vec<_>>();
        self.branch_head_versions.rebuild(versions);
    }

    pub(crate) fn transition_empty_branches_to_initial_schema(
        &self,
        registry: &crate::schema::data::RelationalSchemaRegistry,
    ) -> Result<(), RelationalBranchCellDenial> {
        self.record_branch_population_scan();
        let existing = self.branch_cells.take_all();
        let transitioned = existing
            .iter()
            .map(|(branch_id, cell)| {
                let mut candidate = cell.clone_for_head_replacement();
                if matches!(
                    candidate.observation().target(),
                    FoundationalBranchTarget::Empty
                ) {
                    candidate.advance_metadata()?;
                    candidate.install_root(RelationalBranchRoot::empty_with_schema(
                        registry,
                        crate::schema::data::runtime_descriptor_semantics_policy()
                            .current_write_version(),
                    ));
                }
                Ok((branch_id.clone(), candidate))
            })
            .collect::<Result<BTreeMap<_, _>, RelationalBranchCellDenial>>();
        match transitioned {
            Ok(cells) => {
                let reservations = cells
                    .values()
                    .filter(|cell| {
                        matches!(cell.observation().target(), FoundationalBranchTarget::Empty)
                    })
                    .map(|cell| {
                        let previous_root = existing
                            .get(cell.identity().branch_id())
                            .and_then(RelationalBranchReferenceCell::root)
                            .expect("live empty branch retains its pre-transition root");
                        let next_root = cell
                            .root()
                            .expect("transitioned empty branch retains its schema root");
                        self.reserve_branch_head_retirement(
                            cell.identity(),
                            &previous_root,
                            cell.head_retention(),
                        )
                        .map(|reservation| (reservation, previous_root, next_root))
                        .map_err(branch_head_retirement_denial)
                    })
                    .collect::<Result<Vec<_>, _>>();
                let reservations = match reservations {
                    Ok(reservations) => reservations,
                    Err(denial) => {
                        self.branch_cells.restore_all(existing);
                        return Err(denial);
                    }
                };
                self.branch_cells.restore_all(cells);
                for (mut reservation, previous_root, next_root) in reservations {
                    reservation.transfer_head(&previous_root, &next_root);
                    reservation.replace_head(previous_root);
                }
                Ok(())
            }
            Err(denial) => {
                self.branch_cells.restore_all(existing);
                Err(denial)
            }
        }
    }

    pub(crate) fn admit_recovered_branch_cell(
        &self,
        checkpoint: RelationalBranchCellCheckpoint,
        expected_branch_id: &BranchId,
        recovered_root: Option<Arc<crate::branch::RelationalBranchRoot>>,
        recovered_provenance_root: Option<Arc<crate::branch::RelationalBranchRoot>>,
        schema_registry: &crate::schema::data::RelationalSchemaRegistry,
        symbols: &crate::symbols::data::StringInterner,
    ) -> Result<(), String> {
        if let Some(FoundationalBranchTarget::Basis(target)) = checkpoint
            .fork_provenance
            .as_ref()
            .map(|provenance| provenance.target())
        {
            let commit_id = CommitId(target.selected_commit_id());
            self.readmit_replayed_root_descriptor(
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
                        commit_id,
                        target.roots().clone(),
                        recovered_root.ok_or_else(|| {
                            format!("recovery root `{}` is unavailable", commit_id.0)
                        })?,
                        symbols,
                    )
                    .map_err(|detail| {
                        format!(
                            "tail branch cell references unavailable branch root `{}`: {detail}",
                            commit_id.0
                        )
                    })?,
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
            ));
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
                return Err(format!(
                    "recovery branch-cell state conflicts for `{}`",
                    branch_id.0
                ));
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
        let recovered_head_version = match cell.observation().target() {
            FoundationalBranchTarget::Empty => None,
            FoundationalBranchTarget::Basis(target) => {
                Some(crate::identity::data::VersionId(target.version_id()))
            }
        };
        self.insert_branch_cell(cell);
        self.move_branch_head_version(None, recovered_head_version);
        Ok(())
    }

    pub(crate) fn branch_cells_snapshot(&self) -> Vec<RelationalBranchCellCheckpoint> {
        self.record_branch_population_scan();
        self.branch_cells.checkpoints()
    }

    pub(crate) fn branch_root_checkpoints(
        &self,
    ) -> Result<Vec<crate::branch::RelationalBranchRootCheckpoint>, String> {
        self.record_branch_population_scan();
        let mut roots = BTreeMap::new();
        for cell in self.branch_cells.values() {
            let root = cell.root().ok_or_else(|| {
                format!(
                    "live branch `{}` has no complete owner root",
                    cell.identity().branch_id().0
                )
            })?;
            let commit_id = match cell.observation().target() {
                FoundationalBranchTarget::Empty => {
                    if root.id() != 0 || root.descriptor().is_some() {
                        return Err(format!(
                            "empty branch `{}` carries a committed root",
                            cell.identity().branch_id().0
                        ));
                    }
                    continue;
                }
                FoundationalBranchTarget::Basis(target) => {
                    let commit_id = CommitId(target.selected_commit_id());
                    if root.commit_id() != Some(commit_id) {
                        return Err(format!(
                            "committed branch `{}` root identity disagrees with its target",
                            cell.identity().branch_id().0
                        ));
                    }
                    commit_id
                }
            };
            if let Some(existing) = roots.get(&commit_id) {
                if !Arc::ptr_eq(existing, &root) {
                    return Err(format!(
                        "commit {} is represented by competing immutable roots",
                        commit_id.0
                    ));
                }
                continue;
            }
            roots.insert(commit_id, Arc::clone(&root));
        }
        Ok(roots
            .into_iter()
            .map(|(commit_id, root)| {
                let partitions = root
                    .partition_ids()
                    .into_iter()
                    .filter_map(|partition_id| root.partition_state(partition_id).cloned())
                    .collect();
                crate::branch::RelationalBranchRootCheckpoint::new(
                    commit_id,
                    partitions,
                    root.retained_schema_authority(),
                )
            })
            .collect())
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
        crate::history::retention::RelationalRetentionAcquisitionDenial::RootSetTooLarge => {
            RelationalBranchCellDenial::RetentionRootSetTooLarge
        }
    }
}
