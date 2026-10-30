use worth_runtime_bridge::facade::{
    BridgeTemporalRoutingLaneKind, BridgeTemporalWakeRoutingRequest,
};

fn fake<T>() -> T {
    panic!("private")
}

fn main() {
    let _ = BridgeTemporalWakeRoutingRequest {
        routing_request_identity: fake(),
        routing_lane_kind: BridgeTemporalRoutingLaneKind::Authoritative,
        subscription_identity: fake(),
        activation_lane_identity: fake(),
        temporal_basis_identity: fake(),
        preview_basis_identity: None,
        truth_branch_identity: fake(),
        truth_snapshot_identity: fake(),
        wake_id: fake(),
        wake_ready_ordinal: fake(),
        wake_tick: fake(),
        canonical_basis: fake(),
        digest: fake(),
    };
}
