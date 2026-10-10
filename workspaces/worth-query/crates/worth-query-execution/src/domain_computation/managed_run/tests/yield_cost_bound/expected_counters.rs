pub(super) fn expected_direct_yield_counters(
) -> crate::domain_computation::WorthQueryYieldTransitionCounters {
    let mut counters = crate::domain_computation::WorthQueryYieldTransitionCounters::default();
    counters.classified_eligibility();
    counters.attempted_bridge_finalization();
    counters.attempted_checkpoint_suspension();
    counters.observed_checkpoint_retained_bytes(1);
    counters.validated_retained_resources();
    counters.minted_yielded_capability();
    counters
}

#[cfg(feature = "allocation-probes")]
pub(super) fn expected_workflow_yield_counters(
) -> crate::domain_computation::WorthQueryYieldTransitionCounters {
    let mut counters = expected_direct_yield_counters();
    counters.observed_artifact_registry();
    counters.observed_artifact_registry();
    counters.validated_retained_resources();
    counters
}
