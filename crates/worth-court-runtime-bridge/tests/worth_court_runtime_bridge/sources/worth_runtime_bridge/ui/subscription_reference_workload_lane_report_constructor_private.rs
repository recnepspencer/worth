use worth_runtime_bridge::facade::{
    BridgeSubscriptionCertificationCounterSnapshot, BridgeSubscriptionReferenceWorkloadFamilyKind,
    BridgeSubscriptionReferenceWorkloadLaneKind, BridgeSubscriptionReferenceWorkloadLaneReport,
};


fn main() {
    let _report = BridgeSubscriptionReferenceWorkloadLaneReport {
        lane_kind: BridgeSubscriptionReferenceWorkloadLaneKind::AuthoritativeLive,
        family_kind: BridgeSubscriptionReferenceWorkloadFamilyKind::DetailExact,
        source_artifact_index_digest: sealed_authority_placeholder(),
        certification_bundle_digest: sealed_authority_placeholder(),
        counters: BridgeSubscriptionCertificationCounterSnapshot::default(),
        canonical_basis: sealed_authority_placeholder(),
        digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
