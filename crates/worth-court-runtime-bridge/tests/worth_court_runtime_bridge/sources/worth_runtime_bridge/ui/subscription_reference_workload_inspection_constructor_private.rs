use worth_runtime_bridge::facade::BridgeSubscriptionReferenceWorkloadInspection;


fn main() {
    let _inspection = BridgeSubscriptionReferenceWorkloadInspection {
        reference_workload_report_digest: sealed_authority_placeholder(),
        manifest_digest: sealed_authority_placeholder(),
        offline_audit_report_digest: sealed_authority_placeholder(),
        outcome_summary_digest: sealed_authority_placeholder(),
        coverage_report_digest: sealed_authority_placeholder(),
        counter_digest: sealed_authority_placeholder(),
        lane_report_digests: Vec::new(),
        comparison_report_digests: Vec::new(),
        lane_report_count: 0,
        comparison_report_count: 0,
        host_log_dependency_count: 0,
        live_state_dependency_count: 0,
        canonical_basis: sealed_authority_placeholder(),
        digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
