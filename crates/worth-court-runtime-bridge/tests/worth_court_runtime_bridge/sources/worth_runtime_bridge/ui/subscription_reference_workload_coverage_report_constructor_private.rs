use worth_runtime_bridge::facade::{
    BridgeSubscriptionCertificationCounterSnapshot,
    BridgeSubscriptionReferenceWorkloadCoverageReport,
};


fn main() {
    let _coverage = BridgeSubscriptionReferenceWorkloadCoverageReport {
        lane_kinds: Vec::new(),
        family_kinds: Vec::new(),
        lane_coverage_rows: Vec::new(),
        first_ship_lane_matrix_covered: true,
        multi_family_covered: true,
        comparison_evidence_complete: true,
        expected_lane_outcomes_covered: true,
        counters: BridgeSubscriptionCertificationCounterSnapshot::default(),
        canonical_basis: sealed_authority_placeholder(),
        digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
