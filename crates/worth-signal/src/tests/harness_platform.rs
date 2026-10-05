use serde_json::json;
use worth_harness::facade::{ComparisonMode, ComparisonProfile};
use worth_harness::facade::{
    ExecutionProfile, ExecutionRequest, HarnessAdapter, ObservationStatus,
    PerformanceHarnessAdapter,
};

use crate::facade::*;
use crate::tests::support::{evaluate, version_ab, ASPECT_A};

struct FixedEvaluator {
    source: NodeId,
}

impl SignalEvaluationDriver for FixedEvaluator {
    fn evaluate(
        &self,
        ctx: &mut EvaluationContext<'_, ()>,
    ) -> Result<EvaluationOutput, SignalError> {
        let version = if ctx.node() == self.source { 1 } else { 10 };
        Ok(EvaluationOutput::from_result(version_ab(version, 0)))
    }
}

struct ToleranceEvaluator {
    source: NodeId,
    middle: NodeId,
}

fn capture_performance_after_run(
    fixture: &worth_harness::facade::ScenarioFixture<SignalFixtureFactory>,
    mutation: &worth_harness::facade::MutationBatch<SignalMutationAction>,
    request: &ExecutionRequest<String>,
    profile: &ExecutionProfile,
) -> serde_json::Value {
    let adapter = SignalHarnessBridge;
    let mut session = adapter.create_runtime().unwrap();
    adapter.load_fixture(&mut session, fixture).unwrap();
    adapter
        .apply_mutation_batch(&mut session, mutation)
        .unwrap();
    let _ = adapter
        .execute(&mut session, fixture, request, profile)
        .unwrap();
    adapter
        .capture_performance(&session, fixture, profile)
        .unwrap()
}

impl SignalEvaluationDriver for ToleranceEvaluator {
    fn evaluate(
        &self,
        ctx: &mut EvaluationContext<'_, ()>,
    ) -> Result<EvaluationOutput, SignalError> {
        let result = if ctx.node() == self.source {
            let current = ctx
                .graph()
                .get_entry(self.source)?
                .get_aspect_version()
                .get(ASPECT_A);
            let version = if current == 0 { 10 } else { 12 };
            version_ab(version, 0)
        } else if ctx.node() == self.middle {
            let source_version = ctx
                .read_aspect_version(self.source, ASPECT_A)?
                .get(ASPECT_A);
            let version = if source_version <= 10 { 100 } else { 102 };
            version_ab(version, 0)
        } else {
            version_ab(1_000, 0)
        };
        Ok(EvaluationOutput::from_result(result))
    }
}

#[test]
fn signal_scenario_builder_resolves_initial_on_demand_obligation_before_ordinary_gating() {
    let mut scenario = SignalScenario::new("signal-ondemand");
    let source = scenario.node("source");
    let dependent = scenario.build_node("dependent", |graph| graph.node().on_demand().build());
    scenario
        .graph_mut()
        .append_dependency(dependent, source, ASPECT_A)
        .unwrap();

    let fixture = scenario
        .input("source")
        .observe("dependent")
        .with_evaluator(FixedEvaluator { source })
        .fixture()
        .unwrap();

    let request = ExecutionRequest::target("pull-dependent", "dependent".to_string());
    let bundle = signal_bench(fixture, request)
        .observe(&SignalProfileCatalog::development("development"))
        .unwrap();

    SignalHarnessAssert::assert_run_target_status(
        &bundle.core.run,
        "dependent",
        ObservationStatus::Clean,
    );
    assert_eq!(
        SignalHarnessAssert::execution_report(&bundle.core.run).tasks_deferred_by_condition,
        0
    );
    assert_eq!(bundle.explanations.len(), 1);
    assert_eq!(bundle.provenance.len(), 1);
}

#[test]
fn signal_harness_platform_surfaces_comparator_skips() {
    let mut scenario = SignalScenario::new("signal-tolerance");
    let source = scenario.node("source");
    let middle = scenario.node("middle");
    let dependent = scenario.build_node("dependent", |graph| graph.node().tolerance(2).build());
    scenario
        .graph_mut()
        .append_dependency(middle, source, ASPECT_A)
        .unwrap();
    scenario
        .graph_mut()
        .append_dependency(dependent, middle, ASPECT_A)
        .unwrap();

    let mut bootstrap_source = |_id: NodeId, graph: &SignalGraph| {
        let current = graph.get_entry(source)?.get_aspect_version().get(ASPECT_A);
        let version = if current == 0 { 10 } else { 12 };
        Ok(version_ab(version, 0))
    };
    let mut bootstrap_middle = |_id: NodeId, graph: &SignalGraph| {
        let source_version = graph.get_entry(source)?.get_aspect_version().get(ASPECT_A);
        let version = if source_version <= 10 { 100 } else { 102 };
        Ok(version_ab(version, 0))
    };
    let mut bootstrap_dependent = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(1_000, 0));
    evaluate(scenario.graph_mut(), source, &mut bootstrap_source).unwrap();
    evaluate(scenario.graph_mut(), middle, &mut bootstrap_middle).unwrap();
    evaluate(scenario.graph_mut(), dependent, &mut bootstrap_dependent).unwrap();

    let fixture = scenario
        .input("source")
        .observe("dependent")
        .with_evaluator(ToleranceEvaluator { source, middle })
        .fixture()
        .unwrap();
    let mutation = SignalMutationBatch::new("mark-source-dirty")
        .mark_dirty("source", ASPECT_A)
        .build();
    let request = ExecutionRequest::target("pull-dependent", "dependent".to_string());
    let profile = SignalProfileCatalog::development("development");
    let performance = capture_performance_after_run(&fixture, &mutation, &request, &profile);
    let bundle = signal_bench(fixture, request)
        .mutate(mutation)
        .observe(&profile)
        .unwrap();

    SignalHarnessAssert::assert_run_target_status(
        &bundle.core.run,
        "dependent",
        ObservationStatus::Clean,
    );
    assert!(
        performance["evaluation"]["skipped_by_comparator"]
            .as_u64()
            .unwrap_or(0)
            >= 1
    );
}

#[test]
fn signal_harness_platform_captures_diagnostics_explanations_and_provenance() {
    let mut scenario = SignalScenario::new("signal-observability");
    let source = scenario.node("source");
    let dependent = scenario.node("dependent");
    scenario
        .graph_mut()
        .append_dependency(dependent, source, ASPECT_A)
        .unwrap();
    scenario
        .graph_mut()
        .set_causality(
            dependent,
            Some(CausalityMetadata {
                kind: "bridge".to_string(),
                fields: [("commit".to_string(), "42".to_string())]
                    .into_iter()
                    .collect(),
            }),
        )
        .unwrap();

    let mut source_compute = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(1, 0));
    let mut dependent_compute = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(10, 0));
    evaluate(scenario.graph_mut(), source, &mut source_compute).unwrap();
    evaluate(scenario.graph_mut(), dependent, &mut dependent_compute).unwrap();

    let fixture = scenario
        .input("source")
        .observe("dependent")
        .with_evaluator(FixedEvaluator { source })
        .fixture()
        .unwrap();
    let request = ExecutionRequest::target("observe-dependent", "dependent".to_string());
    let bundle = signal_bench(fixture, request)
        .observe(&SignalProfileCatalog::development("development"))
        .unwrap();

    assert!(bundle.diagnostics.is_some());
    assert_ne!(
        bundle.diagnostics.as_ref().unwrap().summary,
        json!({}),
        "diagnostics summary should not be empty",
    );
    assert_eq!(bundle.explanations.len(), 1);
    assert_eq!(bundle.provenance.len(), 1);
}

#[test]
fn signal_harness_platform_runs_serial_parallel_parity() {
    let mut scenario = SignalScenario::new("signal-parity");
    let a = scenario.build_node("a", |graph| {
        graph
            .node()
            .with_contract(
                NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()),
            )
            .build()
    });
    let _b = scenario.build_node("b", |graph| {
        graph
            .node()
            .with_contract(
                NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()),
            )
            .build()
    });
    scenario.set_checked_evaluator(
        move |ctx: &mut CheckedEvaluationContext<'_, '_, '_, '_, ()>| {
            Ok::<_, SignalError>(version_ab(if ctx.node() == a { 7 } else { 9 }, 0))
        },
    );
    let fixture = scenario
        .with_execution_authority(
            crate::tests::leased_execution::support::shared_authority().clone(),
            crate::tests::leased_execution::support::request(4, 1_000_000),
        )
        .observe("a")
        .observe("b")
        .with_evaluator(move |ctx: &mut EvaluationContext<'_, ()>| {
            let version = if ctx.node() == a { 7 } else { 9 };
            Ok::<EvaluationOutput, SignalError>(EvaluationOutput::from_result(version_ab(
                version, 0,
            )))
        })
        .fixture()
        .unwrap();

    let request = ExecutionRequest::new("pull-pair", vec!["a".to_string(), "b".to_string()]);
    let report = signal_parity_suite(
        fixture,
        request,
        SignalProfileCatalog::serial("serial-baseline"),
    )
    .comparison_profile(ComparisonProfile {
        mode: ComparisonMode::Semantic,
        include_extensions: false,
        numeric_tolerance: None,
    })
    .candidates([
        SignalProfileCatalog::staged_parallel("staged-parallel-candidate"),
        SignalProfileCatalog::full_parallel("full-parallel-candidate"),
    ])
    .compare()
    .unwrap();

    assert!(report.matched);
    assert_eq!(report.results.len(), 2);
}
