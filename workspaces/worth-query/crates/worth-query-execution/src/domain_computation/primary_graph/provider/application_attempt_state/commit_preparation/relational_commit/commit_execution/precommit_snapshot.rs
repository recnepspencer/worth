use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    snapshots::{RelationalSnapshotAdmissionDenial, SnapshotHandle},
};

/// The exact before-image observer needed by Query's post-publication work.
/// It closes on every pre-publication exit, including an owner-call unwind.
pub struct WorthQueryPrecommitSnapshot {
    graph: WorthQueryPrimaryGraphIntegrationHandle,
    snapshot: Option<SnapshotHandle>,
}

impl WorthQueryPrecommitSnapshot {
    pub(super) fn acquire(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        graph: WorthQueryPrimaryGraphIntegrationHandle,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<Self, RelationalSnapshotAdmissionDenial> {
        let snapshot = graph.with_runtime(execution, |runtime| {
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

    pub(super) fn as_snapshot(&self) -> &SnapshotHandle {
        self.snapshot
            .as_ref()
            .expect("live Query publication retains its before-image observer")
    }
}

impl Drop for WorthQueryPrecommitSnapshot {
    fn drop(&mut self) {
        if let Some(snapshot) = self.snapshot.take() {
            self.graph.release_query_snapshot(&snapshot);
        }
    }
}

mod court_doors_for_test;
