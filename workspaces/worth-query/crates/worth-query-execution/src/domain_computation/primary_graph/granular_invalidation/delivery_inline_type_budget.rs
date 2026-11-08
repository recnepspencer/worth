//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryGranularSourceReadBasis>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/granular_invalidation/delivery.rs", "WorthQueryGranularSourceReadBasis", 128 * 1024);
        owner.field::<worth_runtime_bridge::facade::TruthSnapshotIdentity>("snapshot");
        owner.field::<worth_runtime_bridge::facade::TruthBranchIdentity>("branch");
        owner.field::<std::sync::Arc<worth_runtime_bridge::facade::RelationalBridgeObservationLease>>("observation");
    }
}
