use std::sync::Arc;

use crate::branch::{RelationalBranchIdentity, RelationalBranchRoot};
use crate::history::data::BranchId;
use crate::history::retention::{
    RelationalBranchRetentionBinding, RelationalRetentionAcquisitionDenial,
    RelationalRetentionCostCounters,
};

use super::HistorySubsystem;

impl HistorySubsystem {
    pub(crate) fn retained_index_versions(
        &self,
    ) -> std::collections::BTreeSet<(
        crate::identity::data::VersionId,
        crate::schema::data::SchemaVersionId,
    )> {
        let mut versions = self.retention_owner.retained_index_versions();
        for cell in self.branch_cells.values() {
            if let Some(root) = cell.root() {
                if let Some(envelope) = root.canonical_envelope() {
                    versions.insert((
                        envelope.commit.version_id,
                        root.schema_authority().schema_version(),
                    ));
                }
            }
        }
        versions
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
        let identity = self
            .branch_cell(branch_id)
            .map(|cell| cell.identity().clone())
            .ok_or(RelationalRetentionAcquisitionDenial::OwnerUnavailable)?;
        self.branch_cell_mut(branch_id)
            .expect("branch identity came from the live registry")
            .install_root(Arc::clone(&root));
        let head_retention = Arc::clone(
            self.branch_cell(branch_id)
                .expect("branch identity came from the live registry")
                .head_retention(),
        );
        self.install_branch_head(identity, &root, &head_retention)
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

    pub(super) fn reset_retention_owner(&mut self, runtime_instance_id: u64) {
        self.try_reset_retention_owner_from(
            crate::history::retention::RelationalBranchRetentionOwner::new(runtime_instance_id),
        )
        .expect("proven live branch-head population fits the owner retention budget");
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
