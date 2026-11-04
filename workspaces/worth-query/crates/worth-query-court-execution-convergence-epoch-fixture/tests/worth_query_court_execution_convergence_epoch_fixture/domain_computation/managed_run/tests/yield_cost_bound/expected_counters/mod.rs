pub fn expected_direct_yield_counters(
) -> worth_query_execution::facade::domain_computation::WorthQueryYieldTransitionCounters {
    let mut counters = worth_query_execution::facade::domain_computation::WorthQueryYieldTransitionCounters::default();
    counters.classified_eligibility_for_test();
    counters.attempted_bridge_finalization_for_test();
    counters.attempted_checkpoint_suspension_for_test();
    counters.observed_checkpoint_retained_bytes_for_test(1);
    counters.validated_retained_resources_for_test();
    counters.minted_yielded_capability_for_test();
    counters
}

pub fn expected_workflow_yield_counters(
) -> worth_query_execution::facade::domain_computation::WorthQueryYieldTransitionCounters {
    let mut counters = expected_direct_yield_counters();
    counters.observed_artifact_registry_for_test();
    counters.observed_artifact_registry_for_test();
    counters.validated_retained_resources_for_test();
    counters
}
