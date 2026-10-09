use crate::facade::{
    mark_dirty, ArtifactTransitionKind, LineageRecordKind, NodeEvaluationResult, NodeId,
    OutputChange, SignalGraph, SignalRuntime,
};
use crate::tests::support::{define_keyed_computation, evaluate, version_ab, ASPECT_A};

#[test]
fn lineage_distinguishes_replacement_refresh_and_memoized_reuse() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut graph = SignalGraph::new();
    let source = graph.node().output_identity().build();

    let mut replaced = |_id: NodeId, _graph: &SignalGraph| {
        Ok(NodeEvaluationResult::from_version(version_ab(1, 0)).with_output_identity("artifact-a"))
    };
    let mut refreshed = |_id: NodeId, _graph: &SignalGraph| {
        Ok(NodeEvaluationResult::from_version(version_ab(2, 0))
            .with_output_identity("artifact-a")
            .with_output_change(OutputChange::Refreshed))
    };

    evaluate(&mut graph, source, &mut replaced).unwrap();
    mark_dirty(&mut graph, source, ASPECT_A).unwrap();
    evaluate(&mut graph, source, &mut refreshed).unwrap();

    let lineage = graph.observe().lineage_for_node(source);
    assert!(
        lineage.iter().any(|record| {
            matches!(
                record.kind,
                LineageRecordKind::ArtifactTransition {
                    transition: ArtifactTransitionKind::Replaced,
                    ..
                }
            )
        }),
        "first materialized artifact should record replacement semantics"
    );
    assert!(
        lineage.iter().any(|record| {
            matches!(
                record.kind,
                LineageRecordKind::ArtifactTransition {
                    transition: ArtifactTransitionKind::Refreshed { .. },
                    ..
                }
            )
        }),
        "stable artifact continuity should record refresh semantics"
    );

    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let family = define_keyed_computation(&mut runtime, "projection", ());
    let bulkhead = family.keyed("bulkhead");
    let keyed = bulkhead.node(&mut runtime);
    let computation = bulkhead.memoized("shape-v1");
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.evaluate_keyed(keyed, &computation, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("memo-artifact")
                        .with_output_change(OutputChange::Refreshed),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    mark_dirty(runtime.graph_mut(), keyed, ASPECT_A).unwrap();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.evaluate_keyed(keyed, &computation, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(99, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    assert!(
        runtime
            .graph()
            .observe()
            .lineage_for_node(keyed)
            .iter()
            .any(|record| {
                matches!(
                    record.kind,
                    LineageRecordKind::ArtifactTransition {
                        transition: ArtifactTransitionKind::MemoizedReuse,
                        ..
                    }
                )
            }),
        "memoized reuse should emit memoized lineage semantics"
    );
}

#[test]
fn continuity_token_preserves_lineage_without_requiring_output_identity() {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();

    evaluate(&mut graph, source, &mut |_id, _graph| {
        Ok(NodeEvaluationResult::from_version(version_ab(1, 0)).with_continuity_token("stable-a"))
    })
    .unwrap();
    let first_artifact = graph.observe().current_lineage_artifact(source).unwrap();

    mark_dirty(&mut graph, source, ASPECT_A).unwrap();
    evaluate(&mut graph, source, &mut |_id, _graph| {
        Ok(NodeEvaluationResult::from_version(version_ab(2, 0))
            .with_continuity_token("stable-a")
            .with_output_change(OutputChange::Refreshed))
    })
    .unwrap();

    assert_eq!(
        graph.observe().current_lineage_artifact(source),
        Some(first_artifact),
        "matching continuity token should preserve lineage even without output identity"
    );
    assert!(
        graph
            .observe()
            .lineage_for_node(source)
            .iter()
            .any(|record| {
                matches!(
                    record.kind,
                    LineageRecordKind::ArtifactTransition {
                        transition: ArtifactTransitionKind::Refreshed { .. },
                        ..
                    }
                )
            }),
        "continuity-token continuity should record refresh semantics"
    );
}
