use crate::diagnostics::ExplanationFact;
use crate::diagnostics::ProvenanceFact;
use crate::facade::{
    mark_dirty, ArtifactRetentionPolicy, DiagnosticsAvailability, NodeEvaluationResult,
    SignalGraph, SignalObservationRequest, SignalRuntime, SignalRuntimePolicy,
};
use crate::tests::support::{evaluate, version_ab, ASPECT_A};

#[test]
fn snapshot_artifact_retention_policy_changes_richness_not_restore_truth() {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(SignalRuntimePolicy::development());
    let node = graph.node().output_identity().build();

    evaluate(&mut graph, node, &mut |_id, _graph| {
        Ok(NodeEvaluationResult::from_version(version_ab(1, 0))
            .with_output_identity("snapshot-richness")
            .with_label("retained"))
    })
    .unwrap();
    let retained_explanation = graph
        .observe()
        .materialize()
        .materialize_explanation_artifact(node)
        .unwrap()
        .0
        .expect("development policy should materialize explanation artifacts");
    graph
        .diagnostics_state_mut()
        .record_explanation_fact(ExplanationFact::from_explanation(&retained_explanation));
    graph
        .diagnostics_state_mut()
        .record_provenance_fact(ProvenanceFact::from_explanation(&retained_explanation));

    let retained_snapshot = graph.capture_snapshot();
    assert_eq!(
        retained_snapshot
            .meta
            .artifact_retention
            .explanation_retention,
        ArtifactRetentionPolicy::Retain
    );
    assert_eq!(
        retained_snapshot
            .meta
            .artifact_retention
            .provenance_retention,
        ArtifactRetentionPolicy::Retain
    );
    assert!(
        retained_snapshot
            .diagnostics
            .explanation_facts
            .contains_key(&node),
        "development snapshot capture should retain explanation facts eagerly"
    );
    assert!(
        retained_snapshot
            .diagnostics
            .provenance_facts
            .contains_key(&node),
        "development snapshot capture should retain provenance facts eagerly"
    );

    graph.set_runtime_policy(
        SignalRuntimePolicy::operational()
            .with_explanation_retention(ArtifactRetentionPolicy::Omit)
            .with_provenance_retention(ArtifactRetentionPolicy::Omit),
    );
    let session = graph
        .begin_observation_session(SignalObservationRequest::operation())
        .unwrap();
    graph.cancel_observation_session(&session).unwrap();
    let omitted_snapshot = graph.capture_snapshot();
    assert_eq!(
        omitted_snapshot
            .meta
            .artifact_retention
            .explanation_retention,
        ArtifactRetentionPolicy::Omit
    );
    assert_eq!(
        omitted_snapshot
            .meta
            .artifact_retention
            .provenance_retention,
        ArtifactRetentionPolicy::Omit
    );
    assert!(
        omitted_snapshot.diagnostics.explanation_facts.is_empty(),
        "snapshot capture should omit cold explanation richness under an omit policy"
    );
    assert!(
        omitted_snapshot.diagnostics.provenance_facts.is_empty(),
        "snapshot capture should omit cold provenance richness under an omit policy"
    );

    mark_dirty(&mut graph, node, ASPECT_A).unwrap();
    evaluate(&mut graph, node, &mut |_id, _graph| {
        Ok(NodeEvaluationResult::from_version(version_ab(2, 0))
            .with_output_identity("snapshot-richness-2"))
    })
    .unwrap();
    assert_eq!(
        graph
            .get_entry(node)
            .unwrap()
            .get_aspect_version()
            .get(ASPECT_A),
        2
    );

    graph.restore_snapshot(&omitted_snapshot).unwrap();

    assert_eq!(
        graph
            .get_entry(node)
            .unwrap()
            .get_aspect_version()
            .get(ASPECT_A),
        1,
        "snapshot restore should rewind operational truth even when cold artifact richness was omitted"
    );
    let (explanation, materialization_mode) = graph
        .observe()
        .materialize()
        .materialize_explanation_artifact(node)
        .unwrap();
    assert!(
        explanation.is_none(),
        "omitted snapshot richness should remain absent after restore under the active runtime policy"
    );
    assert_eq!(materialization_mode, DiagnosticsAvailability::OmittedByTier);
}

#[test]
fn branch_snapshot_records_explicit_artifact_retention_for_non_active_branches() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let node = runtime.graph_mut().node().output_identity().build();
    let mut runtime_ctx = ();

    runtime.set_runtime_policy(SignalRuntimePolicy::development());
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(node, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("branch-retain")
                        .with_label("retain"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime.create_branch("feature-retention").unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    runtime.set_runtime_policy(
        SignalRuntimePolicy::operational()
            .with_explanation_retention(ArtifactRetentionPolicy::Omit)
            .with_provenance_retention(ArtifactRetentionPolicy::Omit),
    );
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(node, ASPECT_A)?;
            tx.read(node, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(2, 0))
                        .with_output_identity("branch-omit"),
                ))
            })?;
            Ok(())
        })
        .unwrap();
    runtime.switch_branch(main).unwrap();

    let feature_snapshot = runtime.capture_branch_snapshot(feature).unwrap();
    assert_eq!(
        feature_snapshot
            .meta
            .artifact_retention
            .explanation_retention,
        ArtifactRetentionPolicy::Omit
    );
    assert_eq!(
        feature_snapshot
            .meta
            .artifact_retention
            .provenance_retention,
        ArtifactRetentionPolicy::Omit
    );
    assert!(
        feature_snapshot.diagnostics.explanation_facts.is_empty(),
        "non-active branch snapshots should respect the branch-local snapshot artifact retention contract"
    );
    assert!(
        feature_snapshot.diagnostics.provenance_facts.is_empty(),
        "non-active branch snapshots should not retain omitted provenance richness"
    );
}
