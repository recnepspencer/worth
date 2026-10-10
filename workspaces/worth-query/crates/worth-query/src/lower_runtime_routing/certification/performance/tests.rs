use super::counters::WorthQueryLowerRuntimePerformanceCounters;
use super::profiles::{
    WorthQueryLowerRuntimePerformanceProfile, WorthQueryLowerRuntimePerformanceProfileLabel,
};
use super::report::{
    certify_lower_runtime_performance_slopes, test_slope_digest_for_profiles,
    WorthQueryLowerRuntimePerformanceFamily,
};
use crate::lower_runtime_routing::certification::surface::worth_query_lower_runtime_representative_surface;

#[test]
fn scenario_profiles_are_monotonic_across_width_variants() {
    let host_request = worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let surface = worth_query_lower_runtime_representative_surface(resource_request);
    let report = certify_lower_runtime_performance_slopes(&surface);
    let profiles = report.profiles();

    assert_eq!(profiles.len(), 3);
    assert!(
        profiles[0].counters().crossing_inventory_width()
            < profiles[1].counters().crossing_inventory_width()
    );
    assert!(
        profiles[1].counters().crossing_inventory_width()
            < profiles[2].counters().crossing_inventory_width()
    );
    assert!(
        profiles[0].counters().support_lookup_operations()
            < profiles[1].counters().support_lookup_operations()
    );
    assert!(
        profiles[1].counters().support_lookup_operations()
            < profiles[2].counters().support_lookup_operations()
    );
    assert!(
        profiles[0].counters().debt_registry_lookup_operations()
            < profiles[1].counters().debt_registry_lookup_operations()
    );
    assert!(
        profiles[1].counters().debt_registry_lookup_operations()
            < profiles[2].counters().debt_registry_lookup_operations()
    );
}

#[test]
fn slope_report_emits_all_phase_seven_outputs_from_observed_profiles() {
    let host_request = worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let surface = worth_query_lower_runtime_representative_surface(resource_request);
    let report = certify_lower_runtime_performance_slopes(&surface);

    assert_eq!(report.rows().len(), 6);
    for row in report.rows() {
        assert_eq!(
            report.digest_for_output(row.family().output_name()),
            Some(row.slope_digest())
        );
    }
    assert!(!report
        .full_profile()
        .counters()
        .counter_snapshot_digest()
        .is_empty());
}

#[test]
fn full_profile_counter_snapshot_matches_exact_producer_widths() {
    let host_request = worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let surface = worth_query_lower_runtime_representative_surface(resource_request);
    let report = certify_lower_runtime_performance_slopes(&surface);
    let full = report.full_profile().counters();

    assert_eq!(full.crossing_inventory_width(), surface.requests().len());
    assert_eq!(full.route_plan_width(), surface.route_plans().len());
    assert_eq!(full.boundary_evidence_width(), surface.envelopes().len());
    assert_eq!(
        full.capability_eligibility_operations(),
        surface.eligibilities().len()
    );
    assert_eq!(
        full.route_plan_assembly_operations(),
        surface.route_plans().len()
    );
    assert_eq!(
        full.boundary_receipt_assembly_operations(),
        surface.boundary_receipts().len()
    );
    assert_eq!(
        full.boundary_envelope_assembly_operations(),
        surface.envelopes().len()
    );
}

#[test]
fn slope_digests_ignore_unrelated_width_drift() {
    let base_profiles = vec![
        profile(
            WorthQueryLowerRuntimePerformanceProfileLabel::Small,
            counters(4, 0, 3, 3, 5, 2, 4, 3, 3, 3, 9, 2),
        ),
        profile(
            WorthQueryLowerRuntimePerformanceProfileLabel::Medium,
            counters(8, 0, 6, 6, 9, 4, 8, 6, 6, 6, 25, 4),
        ),
        profile(
            WorthQueryLowerRuntimePerformanceProfileLabel::Full,
            counters(16, 0, 12, 12, 17, 8, 16, 12, 12, 12, 65, 8),
        ),
    ];
    let drifted_profiles = vec![
        profile(
            WorthQueryLowerRuntimePerformanceProfileLabel::Small,
            counters(4, 11, 9, 3, 12, 2, 4, 99, 3, 88, 9, 2),
        ),
        profile(
            WorthQueryLowerRuntimePerformanceProfileLabel::Medium,
            counters(8, 11, 15, 6, 18, 4, 8, 99, 6, 88, 25, 4),
        ),
        profile(
            WorthQueryLowerRuntimePerformanceProfileLabel::Full,
            counters(16, 11, 27, 12, 32, 8, 16, 99, 12, 88, 65, 8),
        ),
    ];

    assert_eq!(
        test_slope_digest_for_profiles(
            WorthQueryLowerRuntimePerformanceFamily::CapabilityEligibility,
            &base_profiles
        ),
        test_slope_digest_for_profiles(
            WorthQueryLowerRuntimePerformanceFamily::CapabilityEligibility,
            &drifted_profiles
        )
    );
    assert_eq!(
        test_slope_digest_for_profiles(
            WorthQueryLowerRuntimePerformanceFamily::BoundaryReceiptAssembly,
            &base_profiles
        ),
        test_slope_digest_for_profiles(
            WorthQueryLowerRuntimePerformanceFamily::BoundaryReceiptAssembly,
            &drifted_profiles
        )
    );
    assert_eq!(
        test_slope_digest_for_profiles(
            WorthQueryLowerRuntimePerformanceFamily::DebtRegistryLookup,
            &base_profiles
        ),
        test_slope_digest_for_profiles(
            WorthQueryLowerRuntimePerformanceFamily::DebtRegistryLookup,
            &drifted_profiles
        )
    );
}

fn counters(
    crossing_inventory_width: usize,
    compatibility_debt_width: usize,
    route_plan_width: usize,
    boundary_evidence_width: usize,
    support_width: usize,
    deferred_width: usize,
    capability_eligibility_operations: usize,
    route_plan_assembly_operations: usize,
    boundary_receipt_assembly_operations: usize,
    boundary_envelope_assembly_operations: usize,
    support_lookup_operations: usize,
    debt_registry_lookup_operations: usize,
) -> WorthQueryLowerRuntimePerformanceCounters {
    WorthQueryLowerRuntimePerformanceCounters::new_for_tests(
        crossing_inventory_width,
        compatibility_debt_width,
        route_plan_width,
        boundary_evidence_width,
        support_width,
        deferred_width,
        capability_eligibility_operations,
        route_plan_assembly_operations,
        boundary_receipt_assembly_operations,
        boundary_envelope_assembly_operations,
        support_lookup_operations,
        debt_registry_lookup_operations,
    )
}

fn profile(
    label: WorthQueryLowerRuntimePerformanceProfileLabel,
    counters: WorthQueryLowerRuntimePerformanceCounters,
) -> WorthQueryLowerRuntimePerformanceProfile {
    WorthQueryLowerRuntimePerformanceProfile::new_for_tests(label, counters)
}
