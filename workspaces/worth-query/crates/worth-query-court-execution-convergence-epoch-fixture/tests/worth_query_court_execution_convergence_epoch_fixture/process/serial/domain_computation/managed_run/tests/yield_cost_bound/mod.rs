use crate::process::domain_computation::managed_run::tests::yield_cost_bound::*;
#[test]
fn yield_transition_work_is_invariant_to_unrelated_live_authority_width() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let baseline = execute_target(execution, 0, resource_request);
        let wide = execute_target(execution, UNRELATED_WIDTH, resource_request);
        assert_eq!(baseline, wide);
        assert_eq!(
            baseline,
            YieldCostEvidence {
                provider_steps: 1,
                safe_point_lookups: 2,
                pressure_classifications: 2,
                output_capacity_classifications: 0,
                suspension_count: 1,
                retained_probe_count: 1,
                retained_capacity_count: 2,
                yield_counters: expected_direct_yield_counters(),
            }
        );
    });
}

#[test]
fn yield_transition_allocation_count_is_width_invariant() {
    let output = worth_query_testkit_execution_primary_graph_application_attempt_2::process_deadline::output(
        std::process::Command::new(std::env::current_exe().unwrap())
            .arg("isolated_yield_transition_allocation_slope_probe")
            .env("WORTH_QUERY_YIELD_ALLOCATION_PROBE", "1"),
    )
    .unwrap();
    assert!(
        output.status.success(),
        "yield allocation probe failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
fn isolated_yield_transition_allocation_slope_probe() {
    if std::env::var_os("WORTH_QUERY_YIELD_ALLOCATION_PROBE").is_none() {
        return;
    }
    let baseline = measured_target(0);
    let wide = measured_target(UNRELATED_WIDTH);
    assert_eq!(baseline.allocations, wide.allocations);
    assert_eq!(baseline.reallocations, wide.reallocations);
}

#[test]
fn workflow_yield_transition_allocation_is_unrelated_authority_invariant() {
    let output = worth_query_testkit_execution_primary_graph_application_attempt_2::process_deadline::output(
        std::process::Command::new(std::env::current_exe().unwrap())
            .arg("isolated_workflow_yield_transition_allocation_slope_probe")
            .env("WORTH_QUERY_WORKFLOW_YIELD_ALLOCATION_PROBE", "1"),
    )
    .unwrap();
    assert!(
        output.status.success(),
        "workflow yield allocation probe failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}
