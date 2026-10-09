//! Exact retained snapshot release through the issuing source owner.
use super::WorthQueryPrimaryGraphIntegrationHandle;

impl WorthQueryPrimaryGraphIntegrationHandle {
    pub(in crate::domain_computation::primary_graph) fn release_query_snapshot(
        &self,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) {
        self.source_owner.release_query_snapshot(snapshot);
    }
}
