use super::HistorySubsystem;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use super::RelationalBranchBasisRegistryMetrics;

impl HistorySubsystem {
    pub(crate) fn detached_owner_snapshot(
        &self,
    ) -> Result<Self, crate::runtime::RelationalRuntimeForkDenial> {
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
                .ok_or(crate::runtime::RelationalRuntimeForkDenial::CanonicalInventoryInvalid)?;
            canonical_publication_routes
                .install_recovered(
                    positioned,
                    artifact.envelope_residency(),
                    artifact.roots().clone(),
                )
                .map_err(|_| {
                    crate::runtime::RelationalRuntimeForkDenial::CanonicalInventoryInvalid
                })?;
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
        snapshot
            .try_reset_retention_owner(snapshot.runtime_instance_id)
            .map_err(|_| crate::runtime::RelationalRuntimeForkDenial::CanonicalInventoryInvalid)?;
        Ok(snapshot)
    }

    pub(crate) fn restore_detached_recovery_snapshot(&mut self, snapshot: Self) {
        *self = snapshot;
    }
}

#[cfg(test)]
mod tests {
    use crate::runtime::RelationalRuntimeForkDenial;
    use crate::tests::support::{create_entity, runtime_with_test_schema};

    #[test]
    fn fork_refuses_a_missing_performed_artifact_without_reconstructing_inventory() {
        let host = crate::test_execution_policy::serial_request();
        let execution = worth_execution::ExecutionRequest::serial(&host);
        let runtime = runtime_with_test_schema();
        create_entity(&execution, &runtime, "performed-inventory");
        runtime.history.install_commit_catalog(Default::default());
        assert!(matches!(
            runtime.fork(),
            Err(RelationalRuntimeForkDenial::CanonicalInventoryInvalid)
        ));
    }
}
