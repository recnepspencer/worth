use std::sync::Once;
use std::time::Instant;

use serde_json::{json, Value};
use stats_alloc::{Region, INSTRUMENTED_SYSTEM};

use crate::facade::{mark_dirty, EvaluationRequestMode, NodeState};
use crate::tests::performance_support::{
    build_chain_graph, capture_and_certify_perf_samples, with_perf_topology_asserts_disabled,
    PerfMeasurement, PerfTimingPolicy,
};
use crate::tests::support::version_ab;

use super::{
    graph_metrics_delta, hot_family_contract_with_scoped_allocations, ZERO_BROAD_ENTRY_ACCESS,
};

#[test]
#[ignore = "performance baseline capture; run with -- --ignored --nocapture"]
fn perf_chain_10k_bootstrap_serial() {
    let samples = with_perf_topology_asserts_disabled(|| {
        static CHAIN_10K_WARMUP: Once = Once::new();
        CHAIN_10K_WARMUP.call_once(|| {
            let (mut warm_graph, warm_chain) = build_chain_graph(10_000);
            let warm_plan = warm_graph
                .build_evaluation_plan(&warm_chain, EvaluationRequestMode::Default)
                .unwrap();
            warm_graph
                .execute_prepared_plan(&warm_plan, &(), &|_ctx| Ok(version_ab(0, 1)))
                .unwrap();
        });

        capture_and_certify_perf_samples(
            hot_family_contract_with_scoped_allocations(
                "chain_10k_bootstrap",
                "balanced",
                PerfTimingPolicy::MedianOnly,
                &[
                    "build_nanos",
                    "bootstrap_plan_nanos",
                    "bootstrap_execute_nanos",
                    "push_nanos",
                ],
                &[
                    "push_scoped_allocation_calls",
                    "push_scoped_requested_bytes",
                ],
                ZERO_BROAD_ENTRY_ACCESS,
            ),
            || {
                let build_start = Instant::now();
                let (mut graph, chain) = build_chain_graph(10_000);
                let build_nanos = build_start.elapsed().as_nanos();

                graph.reset_telemetry();
                let before = graph.observe().metrics();
                let plan_start = Instant::now();
                let plan = graph
                    .build_evaluation_plan(&chain, EvaluationRequestMode::Default)
                    .unwrap();
                let bootstrap_plan_nanos = plan_start.elapsed().as_nanos();

                let execute_start = Instant::now();
                graph
                    .execute_prepared_plan(&plan, &(), &|_ctx| Ok(version_ab(0, 1)))
                    .unwrap();
                let bootstrap_execute_nanos = execute_start.elapsed().as_nanos();

                let push_before = graph.observe().metrics();
                let push_allocation_region = Region::new(&INSTRUMENTED_SYSTEM);
                let push_start = Instant::now();
                mark_dirty(&mut graph, chain[0], crate::tests::support::ASPECT_B).unwrap();
                let push_nanos = push_start.elapsed().as_nanos();
                let push_allocation = push_allocation_region.change();
                let after = graph.observe().metrics();

                assert_eq!(graph.get_state(chain[0]), Ok(NodeState::Dirty));
                assert_eq!(
                    after.invalidation.batch_width - push_before.invalidation.batch_width,
                    1,
                    "one source invalidation must execute one admitted dirty entry"
                );
                assert_eq!(
                    after.invalidation.frontier_seed_count
                        - push_before.invalidation.frontier_seed_count,
                    1,
                    "one source invalidation must execute one frontier seed"
                );
                assert_eq!(
                    after.invalidation.frontier_direct_dirty_count
                        - push_before.invalidation.frontier_direct_dirty_count,
                    0,
                    "the source seed transition is not downstream direct-dirty work"
                );

                let mut metrics = graph_metrics_delta(before, after);
                if let Value::Object(ref mut map) = metrics {
                    map.insert("build_nanos".into(), json!(build_nanos));
                    map.insert("bootstrap_plan_nanos".into(), json!(bootstrap_plan_nanos));
                    map.insert(
                        "bootstrap_execute_nanos".into(),
                        json!(bootstrap_execute_nanos),
                    );
                    map.insert("push_nanos".into(), json!(push_nanos));
                    map.insert(
                        "push_scoped_allocation_calls".into(),
                        json!(push_allocation.allocations),
                    );
                    map.insert(
                        "push_scoped_requested_bytes".into(),
                        json!(push_allocation.bytes_allocated),
                    );
                    map.insert(
                        "plans_built".into(),
                        json!(graph.telemetry().planner.plans_built),
                    );
                    map.insert(
                        "tasks_scheduled".into(),
                        json!(graph.telemetry().planner.tasks_scheduled),
                    );
                    map.insert(
                        "stage_execution_nanos".into(),
                        json!(graph.telemetry().execution.stage_execution_nanos),
                    );
                }

                PerfMeasurement::new(
                    (build_nanos + bootstrap_plan_nanos + bootstrap_execute_nanos + push_nanos)
                        / 1_000,
                    metrics,
                )
            },
        )
    });

    assert!(samples.iter().all(|sample| sample.elapsed_micros > 0));
}
