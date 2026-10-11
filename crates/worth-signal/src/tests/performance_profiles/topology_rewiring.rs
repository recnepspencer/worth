use std::sync::Once;
use std::time::Instant;

use serde_json::{json, Value};

use crate::facade::SignalGraph;
use crate::tests::performance_support::{
    capture_and_certify_perf_samples, with_perf_topology_asserts_disabled, PerfMeasurement,
    PerfTimingPolicy,
};
use crate::tests::support::{GraphDependencyBatchExt, ASPECT_A};

use super::{graph_metrics_delta, hot_family_contract, ZERO_BROAD_AND_ARTIFACT_ACCESS};

#[test]
#[ignore = "requires matched release capture runner; legacy goldens use obsolete sample counts"]
fn perf_topology_rewiring_churn_serial() {
    let samples = with_perf_topology_asserts_disabled(|| {
        static TOPOLOGY_REWIRE_CHURN_WARMUP: Once = Once::new();
        TOPOLOGY_REWIRE_CHURN_WARMUP.call_once(|| {
            let mut graph = SignalGraph::new();
            let sources = (0..32).map(|_| graph.node().build()).collect::<Vec<_>>();
            let leaves = (0..256).map(|_| graph.node().build()).collect::<Vec<_>>();

            for (index, &leaf) in leaves.iter().enumerate() {
                graph
                    .append_dependency(leaf, sources[index % sources.len()], ASPECT_A)
                    .unwrap();
            }

            for round in 0..48 {
                for (index, &leaf) in leaves.iter().enumerate() {
                    let old = sources[(index + round) % sources.len()];
                    let new = sources[(index + round + 1) % sources.len()];
                    graph.rewire_dependency(leaf, old, new, ASPECT_A).unwrap();
                }
            }
        });

        capture_and_certify_perf_samples(
            hot_family_contract(
                "topology_rewiring_churn",
                "balanced",
                PerfTimingPolicy::MedianOnly,
                &["rewire_loop_nanos"],
                ZERO_BROAD_AND_ARTIFACT_ACCESS,
            ),
            || {
                let mut graph = SignalGraph::new();
                let sources = (0..32).map(|_| graph.node().build()).collect::<Vec<_>>();
                let leaves = (0..256).map(|_| graph.node().build()).collect::<Vec<_>>();

                for (index, &leaf) in leaves.iter().enumerate() {
                    graph
                        .append_dependency(leaf, sources[index % sources.len()], ASPECT_A)
                        .unwrap();
                }

                let before = graph.observe().metrics();
                let rewire_start = Instant::now();
                for round in 0..48 {
                    for (index, &leaf) in leaves.iter().enumerate() {
                        let old = sources[(index + round) % sources.len()];
                        let new = sources[(index + round + 1) % sources.len()];
                        graph.rewire_dependency(leaf, old, new, ASPECT_A).unwrap();
                    }
                }
                let rewire_loop_nanos = rewire_start.elapsed().as_nanos();
                let after = graph.observe().metrics();

                let mut metrics = graph_metrics_delta(before, after);
                if let Value::Object(ref mut map) = metrics {
                    map.insert("rewire_loop_nanos".into(), json!(rewire_loop_nanos));
                }
                PerfMeasurement::new(rewire_loop_nanos / 1_000, metrics)
            },
        )
    });

    assert!(samples.iter().all(|sample| sample.elapsed_micros > 0));
}

#[test]
#[ignore = "requires matched release capture runner; legacy goldens use obsolete sample counts"]
fn perf_topology_rewiring_rotating_window_serial() {
    let samples = with_perf_topology_asserts_disabled(|| {
        static TOPOLOGY_REWIRE_WINDOW_WARMUP: Once = Once::new();
        TOPOLOGY_REWIRE_WINDOW_WARMUP.call_once(|| {
            let mut graph = SignalGraph::new();
            let sources = (0..64).map(|_| graph.node().build()).collect::<Vec<_>>();
            let leaves = (0..512).map(|_| graph.node().build()).collect::<Vec<_>>();
            let window = 8usize;

            for (index, &leaf) in leaves.iter().enumerate() {
                for offset in 0..window {
                    let source = sources[(index + offset) % sources.len()];
                    graph.append_dependency(leaf, source, ASPECT_A).unwrap();
                }
            }

            for round in 0..24 {
                for (index, &leaf) in leaves.iter().enumerate() {
                    for offset in 0..window {
                        let old = sources[(index + round + offset) % sources.len()];
                        let new = sources[(index + round + offset + 1) % sources.len()];
                        graph.rewire_dependency(leaf, old, new, ASPECT_A).unwrap();
                    }
                }
            }
        });

        capture_and_certify_perf_samples(
            hot_family_contract(
                "topology_rewiring_rotating_window",
                "balanced",
                PerfTimingPolicy::MedianOnly,
                &["rewire_loop_nanos"],
                ZERO_BROAD_AND_ARTIFACT_ACCESS,
            ),
            || {
                let mut graph = SignalGraph::new();
                let sources = (0..64).map(|_| graph.node().build()).collect::<Vec<_>>();
                let leaves = (0..512).map(|_| graph.node().build()).collect::<Vec<_>>();
                let window = 8usize;

                for (index, &leaf) in leaves.iter().enumerate() {
                    for offset in 0..window {
                        let source = sources[(index + offset) % sources.len()];
                        graph.append_dependency(leaf, source, ASPECT_A).unwrap();
                    }
                }

                let before = graph.observe().metrics();
                let rewire_start = Instant::now();
                for round in 0..24 {
                    for (index, &leaf) in leaves.iter().enumerate() {
                        for offset in 0..window {
                            let old = sources[(index + round + offset) % sources.len()];
                            let new = sources[(index + round + offset + 1) % sources.len()];
                            graph.rewire_dependency(leaf, old, new, ASPECT_A).unwrap();
                        }
                    }
                }
                let rewire_loop_nanos = rewire_start.elapsed().as_nanos();
                let after = graph.observe().metrics();

                let mut metrics = graph_metrics_delta(before, after);
                if let Value::Object(ref mut map) = metrics {
                    map.insert("rewire_loop_nanos".into(), json!(rewire_loop_nanos));
                }
                PerfMeasurement::new(rewire_loop_nanos / 1_000, metrics)
            },
        )
    });

    assert!(samples.iter().all(|sample| sample.elapsed_micros > 0));
}
