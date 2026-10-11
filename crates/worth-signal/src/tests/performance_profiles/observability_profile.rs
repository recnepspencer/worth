use std::time::Instant;

use serde_json::json;

use crate::facade::{
    signal_bench, CausalityMetadata, EvaluationContext, EvaluationOutput, SignalError,
    SignalProfileCatalog, SignalScenario,
};
use crate::tests::performance_support::{
    capture_and_certify_perf_samples, PerfMeasurement, PerfTimingPolicy,
};
use crate::tests::support::{version_ab, GraphDependencyBatchExt, ASPECT_A};

use super::perf_contract;

#[test]
#[ignore = "requires matched release capture runner; legacy goldens use obsolete sample counts"]
fn perf_harness_observability_profile_delta() {
    for profile_name in ["development", "forensic"] {
        let samples = capture_and_certify_perf_samples(
            perf_contract(
                "harness_observability_profile",
                profile_name,
                PerfTimingPolicy::StructuralOnly,
                &["observe_loop_nanos"],
            ),
            || {
                let mut scenario = SignalScenario::new("perf-observability-profile");
                let mut sources = Vec::new();
                let mut dependents = Vec::new();
                for index in 0..12 {
                    let source = scenario.node(format!("source-{index}"));
                    let dependent = scenario.node(format!("dependent-{index}"));
                    scenario
                        .graph_mut()
                        .append_dependency(dependent, source, ASPECT_A)
                        .unwrap();
                    scenario
                        .graph_mut()
                        .set_causality(
                            dependent,
                            Some(CausalityMetadata {
                                kind: "perf-observe".to_string(),
                                fields: [
                                    ("source".to_string(), format!("source-{index}")),
                                    ("channel".to_string(), "obs".to_string()),
                                ]
                                .into_iter()
                                .collect(),
                            }),
                        )
                        .unwrap();
                    sources.push(source);
                    dependents.push(dependent);
                }

                let fixture = dependents
                    .iter()
                    .enumerate()
                    .fold(
                        scenario.with_evaluator(move |ctx: &mut EvaluationContext<'_, ()>| {
                            let node = ctx.node();
                            let version = sources
                                .iter()
                                .position(|source| *source == node)
                                .map(|index| (index + 1) as u64)
                                .unwrap_or(10_000 + node.index() as u64);
                            Ok::<EvaluationOutput, SignalError>(EvaluationOutput::from_result(
                                version_ab(version, 0),
                            ))
                        }),
                        |builder, (index, _)| {
                            builder
                                .input(format!("source-{index}"))
                                .observe(format!("dependent-{index}"))
                        },
                    )
                    .fixture()
                    .unwrap();

                let request = worth_harness::facade::ExecutionRequest::new(
                    "observe-dependent-fanout",
                    (0..dependents.len())
                        .map(|index| format!("dependent-{index}"))
                        .collect(),
                );
                let profile = match profile_name {
                    "development" => SignalProfileCatalog::development("development"),
                    "forensic" => SignalProfileCatalog::forensic("forensic"),
                    other => panic!("unexpected profile for perf test: {other}"),
                };
                let iterations = 6_u64;

                let observe_start = Instant::now();
                let mut explanations = 0_u64;
                let mut provenance = 0_u64;
                let mut tasks_executed = 0_u64;
                let mut tasks_pruned = 0_u64;
                let mut diagnostics_seen = false;
                for _ in 0..iterations {
                    let bundle = signal_bench(fixture.clone(), request.clone())
                        .observe(&profile)
                        .unwrap();
                    explanations += bundle.explanations.len() as u64;
                    provenance += bundle.provenance.len() as u64;
                    diagnostics_seen |= bundle.diagnostics.is_some();
                    tasks_executed += bundle
                        .core
                        .run
                        .summary
                        .get("tasks_executed")
                        .and_then(|value| value.as_u64())
                        .unwrap_or(0);
                    tasks_pruned += bundle
                        .core
                        .run
                        .summary
                        .get("tasks_pruned")
                        .and_then(|value| value.as_u64())
                        .unwrap_or(0);
                }
                let observe_loop_nanos = observe_start.elapsed().as_nanos();

                let metrics = json!({
                    "iterations": iterations,
                    "targets": dependents.len(),
                    "explanations": explanations,
                    "provenance": provenance,
                    "has_diagnostics": diagnostics_seen,
                    "tasks_executed": tasks_executed,
                    "tasks_pruned": tasks_pruned,
                    "observe_loop_nanos": observe_loop_nanos,
                });

                PerfMeasurement::new(observe_loop_nanos / 1_000, metrics)
            },
        );

        assert!(samples.iter().all(|sample| sample.elapsed_micros > 0));
    }
}
