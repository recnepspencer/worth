use std::time::Instant;

use serde_json::{json, Value};

use crate::facade::{
    EvaluationContext, EvaluationOutput, NodeEvaluationResult, SignalError, SignalGraph,
    SignalRuntime, SignalRuntimePolicy, StageExecutor,
};
use crate::tests::performance_support::{
    capture_and_certify_perf_samples, with_perf_topology_asserts_disabled, PerfMeasurement,
    PerfTimingPolicy,
};
use crate::tests::support::{version_ab, DependencyBatchBuilder, ASPECT_A};

use super::{eval_metrics_delta, hot_family_contract, ZERO_BROAD_ENTRY_ACCESS};

#[test]
#[ignore = "performance baseline capture; run with -- --ignored --nocapture"]
fn perf_suppression_wide_fanout_serial() {
    let samples = with_perf_topology_asserts_disabled(|| {
        capture_and_certify_perf_samples(
            hot_family_contract(
                "suppression_wide_fanout",
                "balanced",
                PerfTimingPolicy::MedianOnly,
                &["leaf_reread_nanos", "stage_execution_nanos"],
                ZERO_BROAD_ENTRY_ACCESS,
            ),
            || {
                let mut runtime = SignalRuntime::builder(SignalGraph::new())
                    .with_kernel_defaults()
                    .build();
                runtime
                    .set_runtime_policy(SignalRuntimePolicy::operational().with_history_limit(4));

                let source = runtime.graph_mut().node().build();
                let middle = runtime.graph_mut().node().tolerance(2).build();
                let leaves = (0..128)
                    .map(|_| runtime.graph_mut().node().tolerance(2).build())
                    .collect::<Vec<_>>();

                let mut dependencies = DependencyBatchBuilder::new(runtime.graph_mut());
                dependencies
                    .append_dependency(middle, source, ASPECT_A)
                    .unwrap();
                for &leaf in &leaves {
                    dependencies
                        .append_dependency(leaf, middle, ASPECT_A)
                        .unwrap();
                }
                dependencies.commit().unwrap();

                let evaluator =
            move |ctx: &mut EvaluationContext<'_, ()>| -> Result<EvaluationOutput, SignalError> {
                let node = ctx.node();
                let result = if node == source {
                    let current = ctx.graph().node_aspect_version(source)?.get(ASPECT_A);
                    let next = if current == 0 { 10 } else { 12 };
                    version_ab(next, 0)
                } else if node == middle {
                    let source_version = ctx.read_aspect_version(source, ASPECT_A)?.get(ASPECT_A);
                    let version = if source_version <= 10 { 100 } else { 102 };
                    version_ab(version, 0)
                } else {
                    let middle_version = ctx.read_aspect_version(middle, ASPECT_A)?.get(ASPECT_A);
                    let version = if middle_version <= 100 { 1_000 } else { 1_002 };
                    version_ab(version, 0)
                };
                Ok(EvaluationOutput::from_result(result))
            };

                let _ = runtime
                    .read_with_executor(source, &(), &evaluator, StageExecutor::Serial)
                    .unwrap();
                let _ = runtime
                    .read_with_executor(middle, &(), &evaluator, StageExecutor::Serial)
                    .unwrap();
                for &leaf in &leaves {
                    let _ = runtime
                        .read_with_executor(leaf, &(), &evaluator, StageExecutor::Serial)
                        .unwrap();
                }

                assert_eq!(
                    runtime.graph().node_aspect_version(source).unwrap(),
                    version_ab(10, 0)
                );
                assert_eq!(
                    runtime.graph().node_aspect_version(middle).unwrap(),
                    version_ab(100, 0)
                );
                for &leaf in &leaves {
                    assert_eq!(
                        runtime.graph().node_aspect_version(leaf).unwrap(),
                        version_ab(1_000, 0)
                    );
                }

                let before = runtime.observe().metrics();
                let access_before_transaction = crate::data::access_counters::snapshot();
                let transaction_start = Instant::now();
                runtime
                    .transaction(&mut (), |tx| {
                        tx.mark_dirty(source, ASPECT_A)?;
                        tx.read(source, &|ctx| {
                            Ok(ctx.finish(NodeEvaluationResult::from_version(version_ab(12, 0))))
                        })?;
                        Ok(())
                    })
                    .unwrap();
                let transaction_nanos = transaction_start.elapsed().as_nanos();
                let access_after_transaction = crate::data::access_counters::snapshot();

                let access_before_reread = crate::data::access_counters::snapshot();
                let leaf_reread_start = Instant::now();
                for &leaf in &leaves {
                    let _ = runtime
                        .read_with_executor(leaf, &(), &evaluator, StageExecutor::Serial)
                        .unwrap();
                }
                let leaf_reread_nanos = leaf_reread_start.elapsed().as_nanos();
                let access_after_reread = crate::data::access_counters::snapshot();
                let after = runtime.observe().metrics();

                // Operational policy leaves optional telemetry inactive. Prove
                // suppression through canonical values, outside timed intervals.
                assert_eq!(
                    runtime.graph().node_aspect_version(source).unwrap(),
                    version_ab(12, 0)
                );
                assert_eq!(
                    runtime.graph().node_aspect_version(middle).unwrap(),
                    version_ab(100, 0)
                );
                for &leaf in &leaves {
                    assert_eq!(
                        runtime.graph().node_aspect_version(leaf).unwrap(),
                        version_ab(1_000, 0)
                    );
                }

                let mut metrics = eval_metrics_delta(before, after);
                if let Value::Object(ref mut map) = metrics {
                    map.insert("transaction_nanos".into(), json!(transaction_nanos));
                    map.insert("leaf_reread_nanos".into(), json!(leaf_reread_nanos));
                    map.insert(
                        "transaction_materialized_entry_reads".into(),
                        json!(
                            access_after_transaction
                                .delta_since(access_before_transaction)
                                .materialized_entry_reads
                        ),
                    );
                    map.insert(
                        "transaction_runtime_artifact_state_reads".into(),
                        json!(
                            access_after_transaction
                                .delta_since(access_before_transaction)
                                .runtime_artifact_state_reads
                        ),
                    );
                    map.insert(
                        "reread_materialized_entry_reads".into(),
                        json!(
                            access_after_reread
                                .delta_since(access_before_reread)
                                .materialized_entry_reads
                        ),
                    );
                    map.insert(
                        "reread_runtime_artifact_state_reads".into(),
                        json!(
                            access_after_reread
                                .delta_since(access_before_reread)
                                .runtime_artifact_state_reads
                        ),
                    );
                }
                PerfMeasurement::new((transaction_nanos + leaf_reread_nanos) / 1_000, metrics)
            },
        )
    });

    assert!(samples.iter().all(|sample| sample.elapsed_micros > 0));
}
