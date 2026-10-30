use worth_runtime_bridge::facade::{
    BridgeTemporalCauseClassification, BridgeTemporalCauseRecord, BridgeTemporalRoutingLaneKind,
};

fn fake<T>() -> T {
    panic!("private")
}

fn main() {
    let _ = BridgeTemporalCauseRecord {
        cause_record_identity: fake(),
        routing_lane_kind: BridgeTemporalRoutingLaneKind::Authoritative,
        activation_lane_identity: fake(),
        temporal_basis_identity: fake(),
        preview_basis_identity: None,
        classification: BridgeTemporalCauseClassification::TimeOnly,
        wake_id: fake(),
        wake_ready_ordinal: fake(),
        wake_tick: fake(),
        truth_patch_identity: None,
        truth_patch_digest: None,
        counters: fake(),
        canonical_basis: fake(),
        digest: fake(),
    };
}
