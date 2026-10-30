use worth_runtime_bridge::facade::{
    BridgeCausalEnvelopeCounters, BridgeCausalEnvelopeDenial, BridgeCausalEnvelopeDenialKind,
    BridgeCausalEvidenceFamily, BridgeCausalEvidenceOwner,
};

fn main() {
    let counters = BridgeCausalEnvelopeCounters {
        evidence_reference_count: 1,
        lower_runtime_family_count: 1,
        bridge_retained_lookup_count: 1,
        retained_bridge_binding_count: 0,
        external_authority_reference_count: 0,
        materialized_detail_count: 0,
        missing_bridge_record_count: 1,
        bridge_record_unindexed_scan_count: 0,
        counter_digest: sealed_authority_placeholder(),
    };

    let _ = BridgeCausalEnvelopeDenial {
        kind: BridgeCausalEnvelopeDenialKind::MissingRetainedBridgeRecord,
        family: BridgeCausalEvidenceFamily::BridgeRoute,
        supplied_owner: BridgeCausalEvidenceOwner::RuntimeBridge,
        expected_owner: BridgeCausalEvidenceOwner::RuntimeBridge,
        reference_identity: "route".into(),
        counters,
        failure_digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
