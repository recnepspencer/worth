use worth_runtime_bridge::facade::{
    BridgeSubscriptionReferenceWorkloadFamilyKind, BridgeSubscriptionReferenceWorkloadLaneCoverageRole,
    BridgeSubscriptionReferenceWorkloadLaneCoverageRow, BridgeSubscriptionReferenceWorkloadLaneKind,
};


fn main() {
    let _row = BridgeSubscriptionReferenceWorkloadLaneCoverageRow {
        lane_kind: BridgeSubscriptionReferenceWorkloadLaneKind::AuthoritativeLive,
        family_kind: BridgeSubscriptionReferenceWorkloadFamilyKind::DetailExact,
        coverage_role: BridgeSubscriptionReferenceWorkloadLaneCoverageRole::Control,
        lane_report_digest: sealed_authority_placeholder(),
        comparison_report_digest: sealed_authority_placeholder(),
        comparison_outcome: None,
        primary_failure_boundary: None,
        expected_outcome: None,
        expected_primary_failure_boundary: None,
        matches_expected_evidence: true,
        canonical_basis: sealed_authority_placeholder(),
        digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
