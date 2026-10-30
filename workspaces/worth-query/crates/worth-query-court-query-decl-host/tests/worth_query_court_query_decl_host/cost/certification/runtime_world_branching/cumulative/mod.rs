pub(crate) use worth_query_testkit_certification::facade::certification::runtime_world_branching::cumulative::*;

#[test]
#[ignore = "scheduled broader seeded model; ordinary CI runs the bounded action roster"]
fn scheduled_seeded_model_broadens_public_action_roster() {
    model::assert_scheduled_public_action_roster();
}

#[test]
#[ignore = "scheduled timing evidence; structural counters are the ordinary regression gate"]
fn scheduled_public_read_timings_report_cold_and_warm_distributions() {
    cost::run_scheduled_public_read_timings();
}

#[test]
fn structural_slopes_cross_the_real_composition_root() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    structural_slopes::assert_measured_bridge_and_result_slopes(request_execution);
}

#[test]
fn public_query_work_is_fixed_across_retained_populations() {
    cost::assert_public_population_slopes();
}

#[test]
fn public_query_work_is_fixed_across_independent_writers() {
    cost::assert_independent_writer_slopes();
}

#[test]
fn mutation_work_scales_only_with_touched_records() {
    cost::assert_touched_record_slopes();
}

#[test]
fn live_population_does_not_expand_publication_work() {
    cost::assert_live_publication_slopes();
}

#[test]
fn active_world_attempt_population_does_not_expand_query_work() {
    cost::assert_active_attempt_slopes();
}
