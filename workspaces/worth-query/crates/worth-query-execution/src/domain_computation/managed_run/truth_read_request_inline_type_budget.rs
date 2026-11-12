//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryManagedTruthReadRequest>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/managed_run/truth_read_request.rs", "WorthQueryManagedTruthReadRequest", 128 * 1024);
        owner.field::<RelationalBranchBasisDescriptor>("relational_basis");
        owner.field::<SnapshotReadPacket>("packet");
        owner.field::<BridgeReplayMode>("replay_mode");
        owner.field::<BridgeDiagnosticsTier>("diagnostics_tier");
        owner.field::<BridgeDeliveryIntent>("delivery_intent");
        owner.field::<Option<worth_runtime_world::facade::ProductBranchObservation>>(
            "product_observation",
        );
    }
}
