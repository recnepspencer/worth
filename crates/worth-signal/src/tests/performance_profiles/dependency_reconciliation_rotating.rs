use std::time::Instant;

use serde_json::{json, Value};

use crate::facade::{DependencyEdge, SignalGraph};
use crate::tests::performance_support::{
    capture_and_certify_perf_samples, with_perf_topology_asserts_disabled, PerfMeasurement,
    PerfTimingPolicy,
};
use crate::tests::support::ASPECT_A;

use super::{graph_metrics_delta, perf_contract};

#[test]
#[ignore = "requires matched release capture runner; legacy goldens use obsolete sample counts"]
fn perf_dependency_reconciliation_rotating_window_serial() {
    let samples = with_perf_topology_asserts_disabled(|| {
        capture_and_certify_perf_samples(
            perf_contract(
                "dependency_reconciliation_rotating_window",
                "balanced",
                PerfTimingPolicy::StructuralOnly,
                &[
                    "reconcile_loop_nanos",
                    "dependency_reconcile_nanos",
                    "snapshot_batch_commit_nanos",
                ],
            ),
            || {
                let mut graph = SignalGraph::new();
                let sources = (0..64).map(|_| graph.node().build()).collect::<Vec<_>>();
                let leaves = (0..512).map(|_| graph.node().build()).collect::<Vec<_>>();
                let window = 8usize;
                let initial_desired = leaves
                    .iter()
                    .enumerate()
                    .map(|(index, _)| {
                        let mut desired = (0..window)
                            .map(|offset| {
                                DependencyEdge::new(
                                    sources[(index + offset) % sources.len()],
                                    ASPECT_A,
                                )
                            })
                            .collect::<Vec<_>>();
                        desired.sort_unstable_by_key(|edge| edge.sort_key());
                        desired
                    })
                    .collect::<Vec<_>>();
                let desired_by_round = (0..24)
                    .map(|round| {
                        leaves
                            .iter()
                            .enumerate()
                            .map(|(index, _)| {
                                let mut desired = (0..window)
                                    .map(|offset| {
                                        DependencyEdge::new(
                                            sources[(index + round + offset + 1) % sources.len()],
                                            ASPECT_A,
                                        )
                                    })
                                    .collect::<Vec<_>>();
                                desired.sort_unstable_by_key(|edge| edge.sort_key());
                                desired
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();

                for (leaf, desired) in leaves.iter().copied().zip(initial_desired.iter()) {
                    graph.reconcile_dependencies(leaf, desired).unwrap();
                }

                let before = graph.observe().metrics();
                let reconcile_start = Instant::now();
                for desired_round in &desired_by_round {
                    for (leaf, desired) in leaves.iter().copied().zip(desired_round.iter()) {
                        graph.reconcile_dependencies(leaf, desired).unwrap();
                    }
                }
                let reconcile_loop_nanos = reconcile_start.elapsed().as_nanos();
                let after = graph.observe().metrics();

                let mut metrics = graph_metrics_delta(before, after);
                if let Value::Object(ref mut map) = metrics {
                    map.insert("reconcile_loop_nanos".into(), json!(reconcile_loop_nanos));
                }
                PerfMeasurement::new(reconcile_loop_nanos / 1_000, metrics)
            },
        )
    });

    assert!(samples.iter().all(|sample| sample.elapsed_micros > 0));
}
