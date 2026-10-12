use std::time::Instant;

use serde_json::{json, Value};

use crate::facade::{
    mark_dirty, EvaluationRequestMode, NodeEvaluationResult, PreparedEvaluation, SignalGraph,
    SignalRuntimePolicy,
};
use crate::logic::prepared::PreparedDependencyCapture;
use crate::tests::performance_support::{
    capture_and_certify_perf_samples, with_perf_topology_asserts_disabled, PerfMeasurement,
    PerfTimingPolicy,
};
use crate::tests::support::{version_ab, ASPECT_A};

use super::{graph_metrics_delta, perf_contract};

#[test]
#[ignore = "requires matched release capture runner; legacy goldens use obsolete sample counts"]
fn perf_dependency_reconciliation_stable_shape_staged_serial() {
    let samples = with_perf_topology_asserts_disabled(|| {
        capture_and_certify_perf_samples(
            perf_contract(
                "dependency_reconciliation_stable_shape_staged",
                "balanced",
                PerfTimingPolicy::StructuralOnly,
                &[
                    "planning_nanos",
                    "report_stage_precompute_nanos",
                    "report_stage_apply_nanos",
                    "report_semantic_finalize_nanos",
                    "dependency_reconcile_nanos",
                    "snapshot_batch_commit_nanos",
                ],
            ),
            || {
                let mut graph = SignalGraph::new();
                graph.set_runtime_policy(SignalRuntimePolicy::development());
                let sources = (0..64).map(|_| graph.node().build()).collect::<Vec<_>>();
                let leaves = (0..512).map(|_| graph.node().build()).collect::<Vec<_>>();
                let window = 8usize;
                let all_nodes = sources
                    .iter()
                    .copied()
                    .chain(leaves.iter().copied())
                    .collect::<Vec<_>>();
                let max_index = all_nodes
                    .iter()
                    .map(|node| node.index() as usize)
                    .max()
                    .unwrap_or(0);
                let mut leaf_positions = vec![usize::MAX; max_index + 1];
                let mut source_positions = vec![usize::MAX; max_index + 1];
                for (index, leaf) in leaves.iter().enumerate() {
                    leaf_positions[leaf.index() as usize] = index;
                }
                for (index, source) in sources.iter().enumerate() {
                    source_positions[source.index() as usize] = index;
                }

                let bootstrap = graph
                    .build_evaluation_plan(&leaves, EvaluationRequestMode::Default)
                    .unwrap();
                graph
                    .execute_prepared_plan_with_precompute(&bootstrap, &|node, _view| {
                        let source_index = source_positions[node.index() as usize];
                        if source_index != usize::MAX {
                            return Ok(PreparedEvaluation::from_result(
                                NodeEvaluationResult::from_version(version_ab(
                                    (source_index + 1) as u64,
                                    0,
                                )),
                            ));
                        }
                        let leaf_index = leaf_positions[node.index() as usize];
                        let mut capture = PreparedDependencyCapture::new();
                        for offset in 0..window {
                            capture.record(
                                sources[(leaf_index + offset) % sources.len()],
                                ASPECT_A,
                                None,
                            );
                        }
                        Ok(
                            PreparedEvaluation::from_result(NodeEvaluationResult::from_version(
                                version_ab((leaf_index + 1) as u64, 0),
                            ))
                            .with_dependencies(capture),
                        )
                    })
                    .unwrap();

                let before = graph.observe().metrics();
                let start = Instant::now();
                let mut planning_nanos = 0_u128;
                let mut report_precompute_nanos = 0_u128;
                let mut report_apply_nanos = 0_u128;
                let mut report_semantic_finalize_nanos = 0_u128;
                let access_before_loop = crate::data::access_counters::snapshot();
                let mut planning_materialized_entry_reads = 0_u64;
                let mut planning_runtime_artifact_state_reads = 0_u64;
                let mut planning_runtime_artifact_warm_reads = 0_u64;
                let mut execute_materialized_entry_reads = 0_u64;
                let mut execute_runtime_artifact_state_reads = 0_u64;
                let mut execute_runtime_artifact_warm_reads = 0_u64;
                let mut execute_retained_artifact_reads = 0_u64;
                for round in 0..24 {
                    for &source in &sources {
                        mark_dirty(&mut graph, source, ASPECT_A).unwrap();
                    }
                    let access_before_planning = crate::data::access_counters::snapshot();
                    let planning_start = Instant::now();
                    let plan = graph
                        .build_evaluation_plan(&leaves, EvaluationRequestMode::Default)
                        .unwrap();
                    planning_nanos += planning_start.elapsed().as_nanos();
                    let access_after_planning = crate::data::access_counters::snapshot();
                    let planning_delta = access_after_planning.delta_since(access_before_planning);
                    planning_materialized_entry_reads += planning_delta.materialized_entry_reads;
                    planning_runtime_artifact_state_reads +=
                        planning_delta.runtime_artifact_state_reads;
                    planning_runtime_artifact_warm_reads +=
                        planning_delta.runtime_artifact_warm_reads;
                    let access_before_execute = crate::data::access_counters::snapshot();
                    let report =
                        graph
                            .execute_prepared_plan_with_precompute(&plan, &|node, _view| {
                                let source_index = source_positions[node.index() as usize];
                                if source_index != usize::MAX {
                                    return Ok(PreparedEvaluation::from_result(
                                        NodeEvaluationResult::from_version(version_ab(
                                            (round + 2) as u64,
                                            source_index as u64,
                                        )),
                                    ));
                                }
                                let leaf_index = leaf_positions[node.index() as usize];
                                let mut capture = PreparedDependencyCapture::new();
                                for offset in 0..window {
                                    capture.record(
                                        sources[(leaf_index + offset) % sources.len()],
                                        ASPECT_A,
                                        None,
                                    );
                                }
                                Ok(PreparedEvaluation::from_result(
                                    NodeEvaluationResult::from_version(version_ab(
                                        (round + 2) as u64,
                                        leaf_index as u64,
                                    )),
                                )
                                .with_dependencies(capture))
                            })
                            .unwrap();
                    let access_after_execute = crate::data::access_counters::snapshot();
                    let execute_delta = access_after_execute.delta_since(access_before_execute);
                    execute_materialized_entry_reads += execute_delta.materialized_entry_reads;
                    execute_runtime_artifact_state_reads +=
                        execute_delta.runtime_artifact_state_reads;
                    execute_runtime_artifact_warm_reads +=
                        execute_delta.runtime_artifact_warm_reads;
                    execute_retained_artifact_reads += execute_delta.retained_artifact_reads;
                    report_precompute_nanos += report.stage_precompute_nanos;
                    report_apply_nanos += report.stage_apply_nanos;
                    report_semantic_finalize_nanos += report.semantic_finalize_nanos;
                }
                let elapsed = start.elapsed();
                let after = graph.observe().metrics();
                let loop_access_delta =
                    crate::data::access_counters::snapshot().delta_since(access_before_loop);

                let mut metrics = graph_metrics_delta(before, after);
                if let Value::Object(ref mut map) = metrics {
                    map.insert("planning_nanos".into(), json!(planning_nanos));
                    map.insert(
                        "report_stage_precompute_nanos".into(),
                        json!(report_precompute_nanos),
                    );
                    map.insert("report_stage_apply_nanos".into(), json!(report_apply_nanos));
                    map.insert(
                        "report_semantic_finalize_nanos".into(),
                        json!(report_semantic_finalize_nanos),
                    );
                    map.insert(
                        "planning_materialized_entry_reads".into(),
                        json!(planning_materialized_entry_reads),
                    );
                    map.insert(
                        "planning_runtime_artifact_state_reads".into(),
                        json!(planning_runtime_artifact_state_reads),
                    );
                    map.insert(
                        "planning_runtime_artifact_warm_reads".into(),
                        json!(planning_runtime_artifact_warm_reads),
                    );
                    map.insert(
                        "execute_materialized_entry_reads".into(),
                        json!(execute_materialized_entry_reads),
                    );
                    map.insert(
                        "execute_runtime_artifact_state_reads".into(),
                        json!(execute_runtime_artifact_state_reads),
                    );
                    map.insert(
                        "execute_runtime_artifact_warm_reads".into(),
                        json!(execute_runtime_artifact_warm_reads),
                    );
                    map.insert(
                        "execute_retained_artifact_reads".into(),
                        json!(execute_retained_artifact_reads),
                    );
                    map.insert(
                        "loop_materialized_entry_reads".into(),
                        json!(loop_access_delta.materialized_entry_reads),
                    );
                }
                PerfMeasurement::new(elapsed.as_micros(), metrics)
            },
        )
    });

    assert!(samples.iter().all(|sample| sample.elapsed_micros > 0));
    assert!(samples.iter().all(|sample| {
        sample.metrics["dependency_input_stable_shape_count"]
            .as_u64()
            .unwrap_or(0)
            > sample.metrics["dependency_input_replacement_count"]
                .as_u64()
                .unwrap_or(u64::MAX / 2)
    }));
}
