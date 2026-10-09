use crate::facade::{mark_dirty, NodeEvaluationResult, NodeId, SignalGraph, SignalRuntime};
use crate::tests::support::{evaluate, version_ab, ASPECT_A};

#[test]
fn snapshot_contract_accepts_matching_schema_and_rejects_profile_or_schema_mismatch() {
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
    let source = runtime.graph_mut().node().output_identity().build();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("snapshot-contract"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    runtime.restore_snapshot(&snapshot).unwrap();
    assert_eq!(
        runtime
            .graph()
            .get_entry(source)
            .unwrap()
            .get_aspect_version()
            .get(ASPECT_A),
        1,
        "matching schema/profile snapshots should restore successfully"
    );

    let mut wrong_profile = snapshot.clone();
    wrong_profile.meta.core_storage_profile = "definitely-wrong-profile".to_string();
    let wrong_profile_err = runtime.restore_snapshot(&wrong_profile).unwrap_err();
    assert!(
        wrong_profile_err
            .to_string()
            .contains("core storage profile"),
        "profile mismatch should fail explicitly"
    );

    let mut wrong_schema = snapshot.clone();
    wrong_schema.meta.schema_version += 1;
    let wrong_schema_err = runtime.restore_snapshot(&wrong_schema).unwrap_err();
    assert!(
        wrong_schema_err.to_string().contains("schema version"),
        "schema mismatch should fail explicitly"
    );
}

#[test]
fn graph_restore_uses_checkpoint_image_not_raw_snapshot_graph_bundle() {
    let mut graph = SignalGraph::new();
    let source = graph.node().output_identity().build();

    let mut source_v1 =
        |_id: NodeId, _graph: &SignalGraph| {
            Ok(NodeEvaluationResult::from_version(version_ab(1, 0))
                .with_output_identity("artifact-v1"))
        };
    let mut source_v2 =
        |_id: NodeId, _graph: &SignalGraph| {
            Ok(NodeEvaluationResult::from_version(version_ab(9, 0))
                .with_output_identity("artifact-v9"))
        };

    evaluate(&mut graph, source, &mut source_v1).unwrap();
    let mut snapshot = graph.capture_snapshot();

    let mut tampered_graph = snapshot.diagnostic_graph.clone();
    tampered_graph
        .get_entry_mut(source)
        .unwrap()
        .set_aspect_version(version_ab(77, 0));
    snapshot.diagnostic_graph = tampered_graph;

    mark_dirty(&mut graph, source, ASPECT_A).unwrap();
    evaluate(&mut graph, source, &mut source_v2).unwrap();
    assert_eq!(
        graph
            .get_entry(source)
            .unwrap()
            .get_aspect_version()
            .get(ASPECT_A),
        9
    );

    graph.restore_snapshot(&snapshot).unwrap();

    assert_eq!(
        graph.get_entry(source).unwrap().get_aspect_version().get(ASPECT_A),
        1,
        "restore should follow the checkpoint image authority carrier rather than the raw snapshot graph bundle"
    );
}

#[test]
fn checkpoint_image_omits_diagnostic_richness_while_snapshot_bundle_retains_it() {
    let mut graph = SignalGraph::new();
    let source = graph.node().output_identity().build();

    let mut source_v1 =
        |_id: NodeId, _graph: &SignalGraph| {
            Ok(NodeEvaluationResult::from_version(version_ab(1, 0))
                .with_output_identity("artifact-v1"))
        };

    evaluate(&mut graph, source, &mut source_v1).unwrap();
    let snapshot = graph.capture_snapshot();

    assert!(
        snapshot
            .authority_graph()
            .unwrap()
            .observe()
            .replay_events()
            .is_empty(),
        "checkpoint image should not carry retained replay richness"
    );
    assert!(
        snapshot
            .authority_graph()
            .unwrap()
            .observe()
            .lineage_records()
            .is_empty(),
        "checkpoint image should not carry retained lineage richness"
    );
    assert!(
        snapshot
            .authority_graph()
            .unwrap()
            .diagnostics_state()
            .explanation_facts()
            .is_empty(),
        "checkpoint image should not carry retained explanation richness"
    );
    assert!(
        !snapshot.diagnostics.replay_frames.is_empty()
            || !snapshot
                .diagnostic_graph
                .observe()
                .replay_events()
                .is_empty(),
        "rich snapshot bundle should still carry explicit diagnostics/replay payloads"
    );
}
