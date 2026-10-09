use crate::facade::{
    ArtifactTransitionKind, LineageRecordKind, NodeEvaluationResult, ReplayEventKind, SignalGraph,
    SignalRuntime,
};
use crate::tests::support::{version_ab, ASPECT_A};

#[test]
fn replay_slices_and_lineage_chains_are_branch_and_snapshot_queryable() {
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

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(node, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("artifact-main"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let main_branch = runtime.observe().current_branch();
    let snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    let before_cursor = runtime
        .replay_for_branch(main_branch.id)
        .frames
        .last()
        .map(|frame| frame.cursor)
        .expect("main branch should have replay");

    let feature_branch = runtime.create_branch("feature-query").unwrap();
    runtime.switch_branch(feature_branch.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(node, ASPECT_A)?;
            tx.read(node, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(2, 0))
                        .with_output_identity("artifact-feature"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let feature_replay = runtime.observe().replay_for_branch(feature_branch.id);
    assert!(
        feature_replay
            .frames
            .iter()
            .all(|frame| frame.branch_id == feature_branch.id),
        "branch replay slices should stay branch-local"
    );
    assert!(
        feature_replay
            .frames
            .iter()
            .any(|frame| frame.node == Some(node)),
        "branch replay should include node-local evaluation events"
    );

    let node_replay = runtime.observe().replay_for_node(node);
    assert!(
        node_replay
            .frames
            .iter()
            .all(|frame| frame.node == Some(node)),
        "node replay slices should filter to the requested node"
    );

    let around_snapshot = runtime
        .observe()
        .replay_around_snapshot(snapshot.meta.snapshot_id);
    assert!(
        around_snapshot
            .frames
            .iter()
            .any(|frame| frame.snapshot_id == Some(snapshot.meta.snapshot_id)),
        "snapshot-centered replay queries should include the matching snapshot id"
    );

    runtime.switch_branch(main_branch).unwrap();
    let tail = runtime.observe().replay_from_cursor(before_cursor);
    assert!(
        tail.frames
            .iter()
            .any(|frame| frame.kind == ReplayEventKind::BranchSwitched),
        "cursor-based replay slices should include later branch transitions"
    );

    let artifact_id = runtime
        .observe()
        .current_lineage_artifact(node)
        .expect("node should have a current lineage artifact");
    let artifact_chain = runtime.observe().lineage_chain_for_artifact(artifact_id);
    assert!(
        artifact_chain.iter().any(|record| matches!(
            record.kind,
            LineageRecordKind::ArtifactTransition {
                transition: ArtifactTransitionKind::Replaced,
                ..
            }
        )),
        "artifact lineage chain should expose the replacement event that materialized it"
    );
    let artifact_replay = runtime.observe().replay_for_artifact(artifact_id);
    assert!(
        artifact_replay
            .frames
            .iter()
            .all(|frame| frame.lineage_artifact_id == Some(artifact_id)),
        "artifact replay slices should filter to the requested lineage artifact"
    );
    let node_chain = runtime.observe().lineage_chain_for_node(node);
    assert_eq!(
        node_chain
            .last()
            .and_then(|record| record.subject_artifact_id()),
        Some(artifact_id),
        "node lineage chain should end at the current artifact"
    );
    let refreshed_feature_snapshot = runtime
        .capture_branch_snapshot(feature_branch.clone())
        .unwrap();
    assert_eq!(
        runtime.observe().branch_head_snapshot_id(feature_branch.id),
        Some(refreshed_feature_snapshot.meta.snapshot_id),
        "capturing a non-active branch snapshot should keep the branch catalog in sync"
    );
}

#[test]
fn branched_runtime_preserves_unique_lineage_ids_and_sequences() {
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
                        .with_output_identity("main-v1"),
                ))
            })?;
            Ok(())
        })
        .unwrap();
    let main_branch = runtime.observe().current_branch();
    let main_artifact = runtime.observe().current_lineage_artifact(source).unwrap();

    let feature = runtime.create_branch("feature-unique").unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(source, ASPECT_A)?;
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(2, 0))
                        .with_output_identity("feature-v2"),
                ))
            })?;
            Ok(())
        })
        .unwrap();
    let feature_artifact = runtime.observe().current_lineage_artifact(source).unwrap();

    runtime.switch_branch(main_branch).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(source, ASPECT_A)?;
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(3, 0))
                        .with_output_identity("main-v3"),
                ))
            })?;
            Ok(())
        })
        .unwrap();
    let updated_main_artifact = runtime.observe().current_lineage_artifact(source).unwrap();

    assert_ne!(main_artifact, feature_artifact);
    assert_ne!(feature_artifact, updated_main_artifact);

    let sequences = runtime
        .graph()
        .observe()
        .lineage_records()
        .iter()
        .map(|record| record.sequence)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        sequences.len(),
        runtime.graph().observe().lineage_records().len(),
        "active branch lineage history should not contain duplicate lineage sequence ids"
    );
}
