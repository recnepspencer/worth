use std::sync::Arc;

use crate::branch::{RelationalBranchIdentity, RelationalBranchRoot};
use crate::history::data::BranchId;
use crate::history::retention::{
    RelationalBranchRetentionBinding, RelationalRetentionAcquisitionDenial,
    RelationalRetentionCostCounters,
};
use crate::identity::data::VersionId;

use super::HistorySubsystem;

impl HistorySubsystem {
    pub(crate) fn retired_root_storage_inventory(
        &self,
    ) -> Vec<crate::history::retention::RelationalRetiredRootStorageSnapshot> {
        self.retention_owner.storage_inventory()
    }

    pub(crate) fn current_root_storage_inventory(&self) -> Vec<Arc<RelationalBranchRoot>> {
        self.record_branch_population_scan();
        self.branch_cells
            .values()
            .into_iter()
            .filter_map(|cell| cell.root())
            .collect()
    }

    pub(crate) fn index_generation_basis_is_held(
        &self,
        generation: &crate::indexes::data::DerivedIndexGeneration,
        branch_scoped: bool,
    ) -> bool {
        let basis = &generation.applicability;
        self.index_basis_is_held(
            basis.version_id,
            basis.schema_version,
            branch_scoped.then_some(&basis.branch_id),
        )
    }

    pub(crate) fn index_basis_is_held(
        &self,
        version: VersionId,
        schema: crate::schema::data::SchemaVersionId,
        branch: Option<&BranchId>,
    ) -> bool {
        let head_is_live = if let Some(branch) = branch {
            self.branch_cell(branch)
                .and_then(|cell| cell.root())
                .is_some_and(|root| {
                    root.canonical_envelope()
                        .map(|envelope| envelope.commit.version_id)
                        == Some(version)
                        && root.schema_authority().schema_version() == schema
                })
        } else {
            self.branch_head_versions.contains(version, schema)
        };
        head_is_live
            || self
                .retention_owner
                .index_basis_is_held(version, schema, branch)
    }

    pub(crate) fn has_detached_retention_obligations(&self) -> bool {
        self.retention_owner.has_detached_obligations()
    }

    #[cfg(test)]
    pub(crate) fn retired_branch_binding_count(&self) -> usize {
        self.retention_owner.retired_branch_binding_count()
    }

    pub(crate) fn retained_index_roots(&self) -> crate::history::retention::RetainedIndexRoots {
        let mut roots = self.retention_owner.retained_index_roots();
        for cell in self.branch_cells.values() {
            let branch = cell.identity().branch_id().clone();
            roots.latest_branches.insert(branch.clone());
            if let Some(root) = cell.root()
                && let Some(envelope) = root.canonical_envelope()
            {
                roots.live.insert((
                    branch,
                    envelope.commit.version_id,
                    root.schema_authority().schema_version(),
                ));
            }
        }
        roots
    }

    pub(crate) fn retention_binding(&self) -> RelationalBranchRetentionBinding {
        self.retention_owner.binding()
    }

    pub(crate) fn retention_cost_counters(&self) -> RelationalRetentionCostCounters {
        self.retention_owner.counters()
    }

    pub(crate) fn retain_historical_root(
        &self,
        commit_id: crate::history::data::CommitId,
    ) -> Result<
        Option<crate::history::retention::RelationalRetainedHistoricalRoot>,
        RelationalRetentionAcquisitionDenial,
    > {
        crate::history::retention::RelationalRetainedHistoricalRoot::from_owner(
            &self.retention_owner,
            commit_id,
        )
    }

    pub(crate) fn install_branch_head(
        &self,
        identity: RelationalBranchIdentity,
        root: &Arc<RelationalBranchRoot>,
        head_retention: &Arc<crate::history::retention::RelationalBranchHeadRetentionCell>,
    ) -> Result<(), RelationalRetentionAcquisitionDenial> {
        let obligation = self.retention_owner.install_head(identity, root)?;
        head_retention
            .install(obligation)
            .map_err(|_| RelationalRetentionAcquisitionDenial::OwnerUnavailable)
    }

    pub(crate) fn install_branch_root(
        &self,
        branch_id: &BranchId,
        root: Arc<RelationalBranchRoot>,
    ) -> Result<(), RelationalRetentionAcquisitionDenial> {
        let mut cell = self
            .branch_cell(branch_id)
            .ok_or(RelationalRetentionAcquisitionDenial::OwnerUnavailable)?;
        let identity = cell.identity().clone();
        let head_retention = Arc::clone(cell.head_retention());
        self.install_branch_head(identity, &root, &head_retention)?;
        cell.install_root(root);
        Ok(())
    }

    pub(crate) fn reserve_branch_head_retirement(
        &self,
        identity: &RelationalBranchIdentity,
        root: &Arc<RelationalBranchRoot>,
        head_retention: &Arc<crate::history::retention::RelationalBranchHeadRetentionCell>,
    ) -> Result<
        crate::history::retention::RelationalHeadRetirementReservation,
        RelationalRetentionAcquisitionDenial,
    > {
        self.retention_binding()
            .reserve_head_retirement(identity, root, head_retention)
    }

    pub(crate) fn reclaim_retired_branch_roots(
        &self,
        maximum_roots: usize,
    ) -> crate::history::retention::RelationalBranchRootReclamationOutcome {
        self.retention_owner.reclaim_retired_roots(maximum_roots)
    }

    pub(crate) fn retired_branch_root_count(&self) -> usize {
        self.retention_owner.retired_root_count()
    }

    pub(super) fn try_reset_retention_owner(
        &mut self,
        runtime_instance_id: u64,
    ) -> Result<(), RelationalRetentionAcquisitionDenial> {
        self.try_reset_retention_owner_from(
            crate::history::retention::RelationalBranchRetentionOwner::new(runtime_instance_id),
        )
    }

    fn try_reset_retention_owner_from(
        &mut self,
        owner: crate::history::retention::RelationalBranchRetentionOwner,
    ) -> Result<(), RelationalRetentionAcquisitionDenial> {
        let heads = self
            .branch_cells
            .values()
            .into_iter()
            .filter_map(|cell| {
                cell.root().map(|root| {
                    (
                        cell.identity().clone(),
                        root,
                        Arc::clone(cell.head_retention()),
                    )
                })
            })
            .collect::<Vec<_>>();
        let mut admitted_heads = Vec::with_capacity(heads.len());
        for (identity, root, head_retention) in heads {
            let obligation = owner.install_head(identity, &root)?;
            admitted_heads.push((head_retention, obligation));
        }
        self.retention_owner = owner;
        for (head_retention, obligation) in admitted_heads {
            head_retention.reset(obligation);
        }
        let mut cells = self.branch_cells.take_all();
        for cell in cells.values_mut() {
            cell.reset_basis_registry(Arc::clone(&self.basis_registry_metrics));
        }
        self.branch_cells.restore_all(cells);
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn reset_retention_owner_with_test_limits(
        &mut self,
        max_live_root_obligations: usize,
        max_retired_branch_roots: usize,
    ) {
        self.try_reset_retention_owner_from(
            crate::history::retention::RelationalBranchRetentionOwner::with_test_limits(
                self.runtime_instance_id,
                max_live_root_obligations,
                max_retired_branch_roots,
            ),
        )
        .expect("test retention limits must admit the installed branch heads");
    }
}

#[cfg(test)]
impl crate::runtime::RelationalRuntime {
    pub(crate) fn set_retention_capacity_for_test(
        &mut self,
        max_live_root_obligations: usize,
        max_retired_branch_roots: usize,
    ) {
        self.unshared_state_mut()
            .expect("retention capacity test support requires the only handle to this runtime")
            .history
            .reset_retention_owner_with_test_limits(
                max_live_root_obligations,
                max_retired_branch_roots,
            );
    }
}

#[cfg(test)]
mod retained_owner_tests {
    use super::*;
    use crate::branch::RelationalBranchBasisDenial as Denial;
    use crate::tests::support::runtime_with_test_schema;

    #[test]
    fn retained_basis_extension_requires_actual_current_history_owner_after_reset() {
        let mut runtime = runtime_with_test_schema();
        let identity = runtime.main_branch_identity();
        let (_, before) = runtime.observe_branch(&identity).unwrap();
        // Keep the actual prior owner alive: a runtime-id-only check would still
        // be able to acquire through that old binding and wrongly succeed.
        let prior_owner = runtime.history.retention_owner.clone();
        let foreign = runtime_with_test_schema();
        assert!(matches!(
            foreign.retain_component_basis(&before),
            Err(Denial::ForeignRuntime { .. })
        ));
        drop(runtime.retain_component_basis(&before).unwrap());
        runtime.set_retention_capacity_for_test(64, 64);
        assert_eq!(runtime.main_branch_identity(), identity);
        assert_eq!(
            before.identity().runtime_instance_id(),
            runtime.runtime_instance_id()
        );
        assert!(matches!(
            runtime.retain_component_basis(&before),
            Err(Denial::UnavailableRetainedTarget)
        ));
        let (_, current) = runtime.observe_branch(&identity).unwrap();
        assert_eq!(
            current.descriptor(),
            before.descriptor(),
            "matching metadata cannot replace actual owner custody"
        );
        drop(runtime.retain_component_basis(&current).unwrap());
        drop(prior_owner);
    }
}
