use worth_runtime_bridge::facade::{
    BridgeSubscriptionCertificationComparisonReport, BridgeSubscriptionCertificationCounterSnapshot,
    BridgeSubscriptionOfflineAuditOutcomeSummary, BridgeSubscriptionOfflineAuditReport,
    BridgeSubscriptionReferenceWorkloadCoverageProof, BridgeSubscriptionReferenceWorkloadCoverageReport,
    BridgeSubscriptionReferenceWorkloadLaneArtifactSet, BridgeSubscriptionReferenceWorkloadLaneReport,
    BridgeSubscriptionReferenceWorkloadReport,
};

fn fake<T>() -> T {
    panic!("fixture should never run")
}

fn main() {
    let _ = BridgeSubscriptionReferenceWorkloadReport {
        manifest_digest: fake(),
        declaration_digest: fake(),
        lane_artifact_set_digest: fake(),
        coverage_proof_digest: fake(),
        lane_reports: fake::<Vec<BridgeSubscriptionReferenceWorkloadLaneReport>>(),
        comparison_reports: fake::<Vec<BridgeSubscriptionCertificationComparisonReport>>(),
        offline_audit_report: fake::<BridgeSubscriptionOfflineAuditReport>(),
        outcome_summary: fake::<BridgeSubscriptionOfflineAuditOutcomeSummary>(),
        coverage_report: fake::<BridgeSubscriptionReferenceWorkloadCoverageReport>(),
        counters: fake::<BridgeSubscriptionCertificationCounterSnapshot>(),
        canonical_basis: fake(),
        digest: fake(),
    };
}
