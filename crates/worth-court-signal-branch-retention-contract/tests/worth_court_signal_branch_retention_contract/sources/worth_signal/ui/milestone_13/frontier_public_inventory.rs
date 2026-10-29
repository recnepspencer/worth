use worth_signal::facade::adapters as surface;
use worth_signal::facade::{SignalGraph, SignalRuntime};

fn assert_type<T>() {}

fn main() {
    assert_type::<surface::InvalidationPlanningEstimate>();
    assert_type::<surface::SignalInvalidationExecutionReceipt>();
    assert_type::<surface::SignalInvalidationRealizedCounters>();
    assert_type::<surface::InvalidationExecutionSummary>();

    let estimate = surface::InvalidationPlanningEstimate::default();
    let _ = estimate.seed_count();
    let _ = estimate.direct_candidate_count();
    let _ = estimate.partition_scoped_check_count();

    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let observation = runtime
        .begin_invalidation_execution_observation()
        .expect("observation admission should succeed for the sample runtime");
    let _ = runtime.finish_invalidation_execution_observation(&observation);
    let _ = runtime.observe_invalidation_execution(|_| Ok(()));
    let _: Option<&surface::InvalidationPlanningEstimate> =
        runtime.graph().observe().latest_invalidation_planning_estimate();

    fn inspect(receipt: surface::SignalInvalidationExecutionReceipt) {
        let summary = receipt.summary();
        let _: &surface::SignalInvalidationRealizedCounters = summary.realized_counters();
    }
    let _ = inspect;
}
