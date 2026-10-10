use super::support::*;

#[test]
fn pricing_shock_end_to_end_temporal_async_reference_workload_is_sufficient() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let pricing_skin = capture_pricing_workload_certification_bundle(
        BridgeRuntimePolicy::development(),
        BridgePreviewSessionIdentity::admit_bridge_owned("pricing:preview-reference-workload-skin"),
        resource_request,
    );
    let sufficiency = capture_pricing_reference_workload_sufficiency(
        BridgeRuntimePolicy::development(),
        resource_request,
    );
    let report = sufficiency.report();

    assert!(!pricing_skin.digest().is_empty());
    assert_eq!(sufficiency.fixture_evidence_digest(), pricing_skin.digest());
    assert_eq!(report.lane_reports().len(), 18);
    assert_eq!(report.comparison_reports().len(), 17);
    assert!(report.coverage_report().first_ship_lane_matrix_covered());
    assert!(report.coverage_report().required_phase_17_facets_covered());
    assert!(report.coverage_report().required_hostile_lane_set_covered());
    assert!(report.coverage_report().comparison_evidence_complete());
    assert!(report.coverage_report().expected_lane_outcomes_covered());
    assert_eq!(
        report.coverage_report().covered_required_facets(),
        BridgeSubscriptionReferenceWorkloadRequiredCoverageFacet::all()
    );
    assert_eq!(report.coverage_report().covered_required_facets().len(), 9);
    assert_eq!(report.counters().reference_workload_lane_count(), 18);
    assert_eq!(report.counters().reference_workload_report_count(), 1);
    assert_eq!(
        report.coverage_report().lane_kinds().len(),
        BridgeSubscriptionReferenceWorkloadLaneKind::first_ship_matrix().len()
    );
}
