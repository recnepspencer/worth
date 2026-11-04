use crate::process::domain_computation::managed_run::tests::step_cost_bound::*;
#[test]
fn one_provider_step_has_constant_work_under_unrelated_authority_width() {
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
                StepCostEvidence {
                    issued_calls: 1,
                    admitted_receipts: 1,
                    completed_work_units: 1,
                    provider_steps: 1,
                    safe_point_lookups: 3,
                    pressure_classifications: 3,
                    output_capacity_classifications: 1,
                    queue_lookups: 2,
                    queue_mutations: 2,
                    retained_bytes: 0,
                }
            );
        });
}

#[test]
fn admitted_chunk_count_has_only_the_declared_linear_step_cost() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::advancement::with_test_advancement(|active_phase| {
            let bootstrap = active_phase.bootstrap_for_test();
            let resource_request = bootstrap.execution_request();

            let phase = &active_phase;
            let execution = phase;

            let one_chunk = execute_target_with_chunks(execution, 0, 1, resource_request);
            let four_chunks = execute_target_with_chunks(execution, 0, 4, resource_request);
            assert_eq!(
                one_chunk,
                StepCostEvidence {
                    issued_calls: 1,
                    admitted_receipts: 1,
                    completed_work_units: 1,
                    provider_steps: 1,
                    safe_point_lookups: 3,
                    pressure_classifications: 3,
                    output_capacity_classifications: 1,
                    queue_lookups: 2,
                    queue_mutations: 2,
                    retained_bytes: 0,
                }
            );
            assert_eq!(
                four_chunks,
                StepCostEvidence {
                    issued_calls: 1,
                    admitted_receipts: 1,
                    completed_work_units: 4,
                    provider_steps: 4,
                    safe_point_lookups: 12,
                    pressure_classifications: 12,
                    output_capacity_classifications: 4,
                    queue_lookups: 8,
                    queue_mutations: 8,
                    retained_bytes: 0,
                }
            );
        });
}

#[test]
fn one_provider_step_allocation_count_is_width_invariant() {
    let output = worth_query_testkit_execution_primary_graph_application_attempt_2::process_deadline::output(
            std::process::Command::new(std::env::current_exe().unwrap())
                .arg("isolated_provider_step_allocation_slope_probe")
                .env("WORTH_QUERY_STEP_ALLOCATION_PROBE", "1"),
        )
        .unwrap();
    assert!(
        output.status.success(),
        "provider-step allocation probe failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
fn isolated_provider_step_allocation_slope_probe() {
    if std::env::var_os("WORTH_QUERY_STEP_ALLOCATION_PROBE").is_none() {
        return;
    }
    let baseline = measured_target(0);
    let wide = measured_target(UNRELATED_WIDTH);
    assert_eq!(baseline.allocations, wide.allocations);
    assert_eq!(baseline.reallocations, wide.reallocations);
}
