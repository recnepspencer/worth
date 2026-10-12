//! Merge snapshot release ownership.
use super::*;

impl WorthQueryMergeSnapshotOwner for StatefulBridgeRuntimeBackend {
    fn release_query_merge_snapshot(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) {
        self.state
            .borrow()
            .relational_source
            .with_runtime_mut(request_execution, |runtime| {
                runtime.snapshots().release_snapshot(snapshot)
            })
            .expect("merge fixture closes its exact published snapshot once");
    }
}
