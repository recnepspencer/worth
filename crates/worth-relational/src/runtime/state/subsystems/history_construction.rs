use crate::history::data::BranchId;

use super::HistorySubsystem;
use crate::runtime::RuntimeSubsystem;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use super::RelationalBranchBasisRegistryMetrics;

impl RuntimeSubsystem for HistorySubsystem {
    type Config = BranchId;

    fn new(config: &Self::Config) -> Self {
        Self::build_with_main_branch(config.clone())
    }

    fn fork(&self) -> Self {
        self.detached_owner_snapshot()
    }
}

impl HistorySubsystem {
    pub(crate) fn detached_owner_snapshot(&self) -> Self {
        let basis_registry_metrics = Arc::new(RelationalBranchBasisRegistryMetrics::default());
        let branch_cells = self.branch_cells.detached_owner_snapshot();
        let mut cells = branch_cells.take_all();
        for cell in cells.values_mut() {
            cell.bind_basis_registry_metrics(Arc::clone(&basis_registry_metrics));
        }
        branch_cells.restore_all(cells);
        let canonical_publication_routes = Arc::new(
            self.canonical_publication_routes
                .detached_branch_transitions(),
        );
        canonical_publication_routes.install_checkpoint_catalog(self.commit_artifacts());
        for positioned in self.canonical_publication_routes.performed_snapshot() {
            let artifact = self
                .commit_artifact(positioned.commit.commit_id)
                .expect("performed owner inventory has its catalog entry");
            canonical_publication_routes
                .install_recovered(
                    positioned,
                    artifact.envelope_residency(),
                    artifact.roots().clone(),
                )
                .expect("owner canonical inventory is valid when detached");
        }
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
            #[cfg(test)]
            root_capture_sabotage: Arc::clone(&self.root_capture_sabotage),
            ledger: super::RuntimeOwnedState::new(self.ledger_snapshot()),
            commit_identity_allocator: Arc::new(AtomicU64::new(
                self.commit_identity_allocator.load(Ordering::Relaxed),
            )),
            version_identity_allocator: Arc::new(AtomicU64::new(
                self.version_identity_allocator.load(Ordering::Relaxed),
            )),
            canonical_publication_routes,
        };
        snapshot.reset_retention_owner(snapshot.runtime_instance_id);
        snapshot
    }

    pub(crate) fn restore_detached_recovery_snapshot(&mut self, snapshot: Self) {
        *self = snapshot;
    }
}
