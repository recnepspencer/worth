use crate::facade::*;

#[test]
fn runtime_builder_uses_expected_defaults() {
    let graph = SignalGraph::new();
    let runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();

    assert_eq!(
        runtime.checkpoint().policy().barrier_for(()),
        CheckpointBarrier::PerOperation
    );
    assert_eq!(
        *runtime.config().fallback_comparator(),
        VersionComparatorPolicy::Exact
    );
}
