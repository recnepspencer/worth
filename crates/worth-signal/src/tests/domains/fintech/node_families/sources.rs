use super::{bounded, FintechRuntime};
use crate::facade::NodeId;

pub(in crate::tests::domains::fintech) fn build_bucket_sources(
    runtime: &mut FintechRuntime,
    buckets: usize,
) -> Vec<NodeId> {
    let mut nodes = Vec::with_capacity(buckets);
    for _ in 0..buckets {
        nodes.push(
            runtime
                .graph_mut()
                .node()
                .with_contract(bounded([]))
                .reads_aspects(super::super::aspects::full_mask())
                .tolerance(1)
                .build(),
        );
    }
    nodes
}

pub(in crate::tests::domains::fintech) fn build_scenario_sources(
    runtime: &mut FintechRuntime,
    scenarios: usize,
) -> Vec<NodeId> {
    let mut nodes = Vec::with_capacity(scenarios);
    for _ in 0..scenarios {
        nodes.push(
            runtime
                .graph_mut()
                .node()
                .with_contract(bounded([]))
                .reads_aspects(super::super::aspects::full_mask())
                .tolerance(2)
                .build(),
        );
    }
    nodes
}
