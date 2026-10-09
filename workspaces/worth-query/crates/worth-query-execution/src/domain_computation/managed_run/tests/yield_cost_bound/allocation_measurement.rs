use super::*;

#[cfg(feature = "allocation-probes")]
pub(super) fn measured_workflow_target(unrelated_width: usize) -> stats_alloc::Stats {
    let (paused, unrelated) =
        crate::domain_computation::primary_graph::with_test_advancement(|execution| {
            prepared_workflow_target(&execution, unrelated_width)
        });
    let region = stats_alloc::Region::new(&stats_alloc::INSTRUMENTED_SYSTEM);
    let yielded = match paused.yield_run() {
        crate::domain_computation::WorthQueryWorkflowYieldOutcome::Yielded(yielded) => yielded,
        _ => panic!("eligible workflow cost target did not yield"),
    };
    let stats = region.change();
    assert_eq!(
        yielded.inspection().yield_counters(),
        expected_workflow_yield_counters()
    );
    match yielded.cleanup() {
        crate::domain_computation::WorthQueryWorkflowYieldCleanupOutcome::Complete(_) => {}
        _ => panic!("artifact-free workflow cost target did not clean up"),
    }
    drop(unrelated);
    stats
}

#[cfg(feature = "allocation-probes")]
pub(super) fn measured_target(unrelated_width: usize) -> stats_alloc::Stats {
    let (paused, unrelated, _, _) =
        crate::domain_computation::primary_graph::with_test_advancement(|execution| {
            prepared_target(&execution, unrelated_width)
        });
    let region = stats_alloc::Region::new(&stats_alloc::INSTRUMENTED_SYSTEM);
    let yielded = yield_target(paused);
    let stats = region.change();
    let _ = yielded.cleanup();
    drop(unrelated);
    stats
}
