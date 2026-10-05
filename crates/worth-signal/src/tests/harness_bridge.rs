use worth_harness::facade::{
    DiagnosticsHarnessAdapter, DiagnosticsLevel, ExecutionProfile, ExecutionRequest,
    ExplanationHarnessAdapter, HarnessAdapter, HarnessRunner, MutationBatch,
    ProvenanceHarnessAdapter, ReplayHarnessAdapter, ReplayRequest, ScenarioPlan,
};

use crate::facade::*;
use crate::tests::support::GraphDependencyBatchExt;

const ASPECT_A: Aspect = Aspect::new(0);

struct BasicEvaluator {
    source: NodeId,
}

impl SignalEvaluationDriver for BasicEvaluator {
    fn evaluate(
        &self,
        ctx: &mut EvaluationContext<'_, ()>,
    ) -> Result<EvaluationOutput, SignalError> {
        if ctx.node() == self.source {
            Ok(EvaluationOutput::from_result(AspectVersion::from_updates(
                [(ASPECT_A, 1)],
            )))
        } else {
            Ok(EvaluationOutput::from_result(AspectVersion::from_updates(
                [(ASPECT_A, 2)],
            )))
        }
    }
}

fn basic_fixture() -> worth_harness::facade::ScenarioFixture<SignalFixtureFactory> {
    ScenarioPlan::new(
        "signal-basic",
        SignalFixtureFactory::new(|| {
            let mut builder = SignalHarnessRuntimeBuilder::new();
            let source = builder.graph_mut().node().build();
            let dependent = builder.graph_mut().node().build();
            builder
                .graph_mut()
                .append_dependency(dependent, source, ASPECT_A)?;
            builder.insert_label("source", source);
            builder.insert_label("dependent", dependent);
            builder.set_evaluator(BasicEvaluator { source });
            builder.build()
        }),
    )
    .declare_input("source")
    .declare_observation("dependent")
    .compile()
}

fn checked_fixture() -> worth_harness::facade::ScenarioFixture<SignalFixtureFactory> {
    ScenarioPlan::new(
        "signal-checked",
        SignalFixtureFactory::new(|| {
            let mut builder = SignalHarnessRuntimeBuilder::new();
            let source = builder
                .graph_mut()
                .node()
                .with_contract(
                    NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()),
                )
                .build();
            let dependent = builder
                .graph_mut()
                .node()
                .with_contract(NodeContract::wildcard().with_bounded_inputs(
                    BoundedSignalInputs::new([DeclaredSignalInput::new(source, ASPECT_A)]),
                ))
                .build();
            builder
                .graph_mut()
                .append_dependency(dependent, source, ASPECT_A)?;
            builder.insert_label("source", source);
            builder.insert_label("dependent", dependent);
            builder.set_evaluator(BasicEvaluator { source });
            builder.set_checked_evaluator(
                move |ctx: &mut CheckedEvaluationContext<'_, '_, '_, '_, ()>| {
                    let value = if ctx.node() == source {
                        1
                    } else {
                        ctx.read(source, ASPECT_A)? + 1
                    };
                    Ok::<_, SignalError>(AspectVersion::zero().with(ASPECT_A, value))
                },
            );
            builder
                .with_execution_authority(
                    crate::tests::leased_execution::support::shared_authority().clone(),
                    crate::tests::leased_execution::support::request(2, 1_000_000),
                )
                .build()
        }),
    )
    .declare_input("source")
    .declare_observation("dependent")
    .compile()
}

#[test]
fn signal_harness_parallel_profiles_use_a_shared_lease_and_checked_inputs() {
    let adapter = SignalHarnessBridge;
    let fixture = checked_fixture();
    let request = ExecutionRequest::new("pull-dependent", vec!["dependent".to_string()]);
    for profile in [
        ExecutionProfile::staged_parallel("staged"),
        ExecutionProfile::full_parallel("full"),
    ] {
        let mut session = adapter.create_runtime().unwrap();
        adapter.load_fixture(&mut session, &fixture).unwrap();
        let run = adapter
            .execute(&mut session, &fixture, &request, &profile)
            .unwrap();
        assert_eq!(run.target_statuses.len(), 1);
        assert_eq!(
            session
                .runtime()
                .unwrap()
                .graph()
                .node_aspect_version(session.runtime().unwrap().resolve("dependent").unwrap(),)
                .unwrap()
                .get(ASPECT_A),
            2
        );
    }
}

#[test]
fn signal_harness_bridge_executes_serial_fixture() {
    let adapter = SignalHarnessBridge;
    let runner = HarnessRunner::new(adapter);
    let fixture = basic_fixture();
    let mutation = MutationBatch::new("mark-source-dirty").push(SignalMutationAction::mark_dirty(
        "dirty-source",
        "source",
        ASPECT_A,
    ));
    let request = ExecutionRequest::new("pull-dependent", vec!["dependent".to_string()]);
    let profile =
        ExecutionProfile::serial("serial").with_diagnostics_level(DiagnosticsLevel::Development);

    let bundle = runner
        .execute_core(&fixture, Some(&mutation), &request, &profile)
        .unwrap();

    assert_eq!(bundle.run.adapter_name, "worth-signal");
    assert_eq!(bundle.run.scenario_name, "signal-basic");
    assert_eq!(bundle.run.target_statuses.len(), 1);
    assert!(bundle.pre_snapshot.is_some());
    assert!(bundle.post_snapshot.is_some());
}

#[test]
fn signal_harness_bridge_captures_diagnostics_explanations_and_provenance() {
    let adapter = SignalHarnessBridge;
    let fixture = basic_fixture();
    let request = ExecutionRequest::new("pull-dependent", vec!["dependent".to_string()]);
    let profile =
        ExecutionProfile::serial("serial").with_diagnostics_level(DiagnosticsLevel::Development);
    let mut session = adapter.create_runtime().unwrap();

    adapter.load_fixture(&mut session, &fixture).unwrap();
    let _run = adapter
        .execute(&mut session, &fixture, &request, &profile)
        .unwrap();

    let diagnostics = adapter
        .capture_diagnostics(&session, &fixture, &profile)
        .unwrap();
    let explanations = adapter
        .capture_explanations(&session, &fixture, &request, &profile)
        .unwrap();
    let provenance = adapter
        .capture_provenance(&session, &fixture, &request, &profile)
        .unwrap();

    assert_eq!(diagnostics.adapter_name, "worth-signal");
    assert_eq!(explanations.len(), 1);
    assert_eq!(provenance.len(), 1);
    assert_eq!(
        diagnostics.extensions["core_storage_profile"],
        CORE_STORAGE_PROFILE_ID
    );
    assert_eq!(
        explanations[0].extensions["artifact_materialization"],
        "retained"
    );
    assert_eq!(
        provenance[0].extensions["artifact_materialization"],
        "retained"
    );
}

#[test]
fn signal_harness_bridge_captures_v2_replay_summary() {
    let adapter = SignalHarnessBridge;
    let fixture = basic_fixture();
    let request = ExecutionRequest::new("pull-dependent", vec!["dependent".to_string()]);
    let profile =
        ExecutionProfile::serial("serial").with_diagnostics_level(DiagnosticsLevel::Development);
    let mut session = adapter.create_runtime().unwrap();

    adapter.load_fixture(&mut session, &fixture).unwrap();
    let run = adapter
        .execute(&mut session, &fixture, &request, &profile)
        .unwrap();

    let replay = adapter
        .capture_replay(
            &session,
            &fixture,
            &ReplayRequest {
                name: "replay-dependent".to_string(),
                source_run: run.clone(),
                request: request.clone(),
                profile: profile.clone(),
            },
        )
        .unwrap();

    assert_eq!(
        replay.schema_version,
        worth_harness::facade::RecordSchemaVersion::V2
    );
    assert_eq!(replay.requested_targets, request.targets);
    let events = replay
        .summary
        .get("events")
        .and_then(|value| value.as_array())
        .expect("replay summary should expose runtime events");
    assert!(!events.is_empty());
    let mut last_cursor = None;
    for event in events {
        let cursor = event["cursor"]
            .as_u64()
            .expect("replay event should have a cursor");
        if let Some(previous) = last_cursor {
            assert!(previous < cursor, "replay events must be strictly ordered");
        }
        last_cursor = Some(cursor);
    }
}

#[test]
fn signal_runtime_materializes_native_explanation_and_provenance_facts() {
    let adapter = SignalHarnessBridge;
    let fixture = basic_fixture();
    let request = ExecutionRequest::new("pull-dependent", vec!["dependent".to_string()]);
    let profile =
        ExecutionProfile::serial("serial").with_diagnostics_level(DiagnosticsLevel::Development);
    let mut session = adapter.create_runtime().unwrap();

    adapter.load_fixture(&mut session, &fixture).unwrap();
    let _run = adapter
        .execute(&mut session, &fixture, &request, &profile)
        .unwrap();

    let runtime = session.runtime().unwrap();
    let node = runtime.resolve("dependent").unwrap();
    let explanation_fact = runtime
        .graph
        .explanation_fact(node)
        .expect("execution should materialize explanation facts");
    let provenance_fact = runtime
        .graph
        .provenance_fact(node)
        .expect("execution should materialize provenance facts");

    assert_eq!(explanation_fact.node, node);
    assert_eq!(provenance_fact.node, node);
    assert_eq!(
        explanation_fact.semantic_segment_id,
        provenance_fact.semantic_segment_id
    );
    assert!(
        provenance_fact
            .vertices
            .iter()
            .any(|vertex| vertex.node == node),
        "provenance graph should retain the target vertex"
    );
    assert!(
        provenance_fact.vertices.len() > provenance_fact.edges.len().min(1),
        "provenance graph should expose structured vertices, not only flattened edges"
    );
    let explanation = runtime.graph.observe().explain(node).unwrap();
    assert_eq!(explanation, explanation_fact.explanation);
}

#[test]
fn operational_profile_reconstructs_rich_artifacts_without_retaining_facts() {
    let adapter = SignalHarnessBridge;
    let fixture = basic_fixture();
    let request = ExecutionRequest::new("pull-dependent", vec!["dependent".to_string()]);
    let profile =
        ExecutionProfile::serial("serial").with_diagnostics_level(DiagnosticsLevel::Operational);
    let mut session = adapter.create_runtime().unwrap();

    adapter.load_fixture(&mut session, &fixture).unwrap();
    let _run = adapter
        .execute(&mut session, &fixture, &request, &profile)
        .unwrap();

    let runtime = session.runtime_mut().unwrap();
    let observation = runtime
        .graph
        .begin_observation_session(SignalObservationRequest::operation())
        .unwrap();
    runtime
        .graph
        .cancel_observation_session(&observation)
        .unwrap();

    let runtime = session.runtime().unwrap();
    let node = runtime.resolve("dependent").unwrap();
    assert!(runtime.graph.explanation_fact(node).is_none());
    assert!(runtime.graph.provenance_fact(node).is_none());

    let explanations = adapter
        .capture_explanations(&session, &fixture, &request, &profile)
        .unwrap();
    let provenance = adapter
        .capture_provenance(&session, &fixture, &request, &profile)
        .unwrap();
    let (explanation, explanation_mode) = runtime
        .graph
        .materialize_explanation_artifact(node)
        .unwrap();
    let (prov, provenance_mode) = runtime.graph.materialize_provenance_artifact(node).unwrap();

    assert!(explanation.is_some());
    assert!(prov.is_some());
    assert_eq!(
        explanation_mode,
        DiagnosticsAvailability::ReconstructedAvailable
    );
    assert_eq!(
        provenance_mode,
        DiagnosticsAvailability::ReconstructedAvailable
    );
    assert_eq!(
        explanations[0].extensions["artifact_materialization"],
        "reconstructed"
    );
    assert_eq!(
        provenance[0].extensions["artifact_materialization"],
        "reconstructed"
    );
}
