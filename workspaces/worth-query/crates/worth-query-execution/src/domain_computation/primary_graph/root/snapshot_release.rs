//! Exact retained snapshot release through the issuing source owner.
use super::WorthQueryPrimaryGraphIntegrationHandle;

impl WorthQueryPrimaryGraphIntegrationHandle {
    #[doc(hidden)]
    pub fn release_query_snapshot(
        &self,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) {
        self.source_owner.release_query_snapshot(snapshot);
    }
    pub(in crate::domain_computation::primary_graph) fn release_component_basis(
        &self,
        retention: worth_relational::facade::branch::RelationalBranchRetentionLease,
    ) -> Result<
        worth_relational::facade::branch::RelationalBranchRetentionReleaseReceipt,
        worth_relational::facade::branch::RelationalBranchRetentionReleaseDenial,
    > {
        self.source_owner.release_component_basis(retention)
    }
}
