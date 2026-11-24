use super::HistorySubsystem;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use super::RelationalBranchBasisRegistryMetrics;

impl HistorySubsystem {
    pub fn set_runtime_instance_id(
        &mut self,
        runtime_instance_id: u64,
    ) -> Result<
        (),
        crate::branch::reference::observation::RelationalBranchObservationConstructionDenial,
    > {
        let metrics = Arc::new(RelationalBranchBasisRegistryMetrics::default());
        self.record_branch_population_scan();
        // Complete every fallible identity rebind before replacing live cells.
        let cells = self
            .branch_cells
            .values()
            .into_iter()
            .map(|cell| {
                let mut rebound = cell.rebind_runtime(runtime_instance_id)?;
                rebound.bind_basis_registry_metrics(Arc::clone(&metrics));
                Ok((rebound.identity().branch_id().clone(), rebound))
            })
            .collect::<Result<_, crate::branch::reference::observation::RelationalBranchObservationConstructionDenial>>()?;
        self.branch_cells.restore_all(cells);
        self.runtime_instance_id = runtime_instance_id;
        self.basis_registry_metrics = metrics;
        self.try_reset_retention_owner(runtime_instance_id)
            .map_err(|_| {
                crate::branch::reference::observation::RelationalBranchObservationConstructionDenial::RetentionUnavailable
            })?;
        Ok(())
    }

    pub fn detached_owner_snapshot(
        &self,
    ) -> Result<Self, crate::runtime::fork_denial::RelationalRuntimeForkDenial> {
        let basis_registry_metrics = Arc::new(RelationalBranchBasisRegistryMetrics::default());
        let branch_cells = self.branch_cells.detached_owner_snapshot();
        let mut cells = branch_cells.take_all();
        for cell in cells.values_mut() {
            cell.bind_basis_registry_metrics(Arc::clone(&basis_registry_metrics));
        }
        branch_cells.restore_all(cells);
        let mut snapshot = Self {
            runtime_instance_id: self.runtime_instance_id,
            main_branch: self.main_branch.clone(),
            branch_cells,
            phase4_costs: self.phase4_costs.detached_owner_snapshot(),
            root_identity_issuer: self.root_identity_issuer.detached_owner_snapshot(),
            branch_population_scans: Arc::new(AtomicU64::new(
                self.branch_population_scans.load(Ordering::Relaxed),
            )),
            branch_head_versions: self.branch_head_versions.detached(),
            basis_registry_metrics,
            retention_owner: crate::history::retention::RelationalBranchRetentionOwner::new(
                self.runtime_instance_id,
            ),
            root_capture_sabotage: Arc::clone(&self.root_capture_sabotage),
            ledger: super::RuntimeOwnedState::new(self.ledger_snapshot()),
            commit_identity_allocator: Arc::new(AtomicU64::new(
                self.commit_identity_allocator.load(Ordering::Relaxed),
            )),
            version_identity_allocator: Arc::new(AtomicU64::new(
                self.version_identity_allocator.load(Ordering::Relaxed),
            )),
        };
        snapshot
            .try_reset_retention_owner(snapshot.runtime_instance_id)
            .map_err(|_| {
                crate::runtime::fork_denial::RelationalRuntimeForkDenial::CanonicalInventoryInvalid
            })?;
        Ok(snapshot)
    }
}
