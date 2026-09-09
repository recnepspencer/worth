use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    snapshots::{RelationalSnapshotAdmissionDenial, SnapshotHandle},
};

/// The exact before-image observer needed by Query's post-publication work.
/// It closes on every pre-publication exit, including an owner-call unwind.
pub(super) struct WorthQueryPrecommitSnapshot {
    graph: WorthQueryPrimaryGraphIntegrationHandle,
    snapshot: Option<SnapshotHandle>,
}

impl WorthQueryPrecommitSnapshot {
    pub(super) fn acquire(
        graph: WorthQueryPrimaryGraphIntegrationHandle,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<Self, RelationalSnapshotAdmissionDenial> {
        let snapshot = graph.with_runtime_mut(|runtime| {
            crate::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(
                runtime, basis,
            )
        })?;
        Ok(Self {
            graph,
            snapshot: Some(snapshot),
        })
    }

    pub(super) fn into_publication(mut self) -> SnapshotHandle {
        self.snapshot
            .take()
            .expect("live Query publication retains its before-image observer")
    }
}

impl Drop for WorthQueryPrecommitSnapshot {
    fn drop(&mut self) {
        if let Some(snapshot) = self.snapshot.take() {
            self.graph.with_runtime_mut(|runtime| {
                crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unwind_releases_only_its_precommit_snapshot() {
        let world =
            crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
                true,
            );
        let product = world.application.admit_product_publication().unwrap();
        let graph = world.application.primary_provider.graph.clone();
        let count = || {
            graph.with_runtime(|runtime| runtime.retention().inspect_plan().active_snapshot_count)
        };
        let baseline = count();
        let peer = WorthQueryPrecommitSnapshot::acquire(
            graph.clone(),
            product.observation().basis().relational_basis(),
        )
        .unwrap();
        let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _attempt = WorthQueryPrecommitSnapshot::acquire(
                graph.clone(),
                product.observation().basis().relational_basis(),
            )
            .unwrap();
            assert_eq!(count(), baseline + 2);
            panic!("owner call interrupted before Query publication");
        }));
        assert!(unwind.is_err());
        assert_eq!(count(), baseline + 1);
        drop(peer);
        assert_eq!(count(), baseline);
    }
}
