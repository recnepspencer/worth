use crate::facade::{
    mark_dirty, ArtifactTransitionKind, LineageRecordKind, NodeEvaluationResult, OutputChange,
    ReplayEventKind, SignalError, SignalGraph, SignalRuntime, SignalRuntimePolicy,
};
use crate::tests::support::{
    define_keyed_computation, version_ab, GraphDependencyBatchExt, ASPECT_A,
};

#[test]
fn posthoc_forensics_after_long_session_answers_branch_and_artifact_questions() {
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
    runtime.set_runtime_policy(SignalRuntimePolicy::forensic().with_history_limit(8));
    let source = runtime.graph_mut().node().output_identity().build();
    let dependent = runtime.graph_mut().node().output_identity().build();
    runtime
        .graph_mut()
        .append_dependency(dependent, source, ASPECT_A)
        .unwrap();
    let family = define_keyed_computation(&mut runtime, "posthoc-session", ());
    let keyed_def = family.keyed("skin-panel");
    let keyed = keyed_def.node(&mut runtime);
    let computation = keyed_def.memoized("shape-a");
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("source-main"),
                ))
            })?;
            tx.read(dependent, &|view| {
                let version = view.read_aspect_version(source, ASPECT_A)?;
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version)
                        .with_output_identity("dependent-main"),
                ))
            })?;
            tx.evaluate_keyed(keyed, &computation, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("skin-a")
                        .with_output_change(OutputChange::Refreshed),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let main_snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    let feature = runtime.create_branch("feature-posthoc").unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(source, ASPECT_A)?;
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(2, 0))
                        .with_output_identity("source-feature"),
                ))
            })?;
            tx.read(dependent, &|view| {
                let version = view.read_aspect_version(source, ASPECT_A)?;
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version)
                        .with_output_identity("dependent-feature"),
                ))
            })?;
            Ok(())
        })
        .unwrap();
    let feature_snapshot = runtime.capture_branch_snapshot(feature.clone()).unwrap();

    let analysis = runtime.create_branch("analysis-posthoc").unwrap();
    runtime.switch_branch(analysis.clone()).unwrap();
    let analysis_snapshot = runtime.capture_branch_snapshot(analysis.clone()).unwrap();
    mark_dirty(runtime.graph_mut(), source, ASPECT_A).unwrap();
    runtime
        .restore_branch_snapshot(analysis.clone(), &analysis_snapshot)
        .unwrap();
    mark_dirty(runtime.graph_mut(), dependent, ASPECT_A).unwrap();
    runtime
        .restore_branch_snapshot(analysis.clone(), &analysis_snapshot)
        .unwrap();
    let err = runtime.transaction(request_execution, &mut runtime_ctx, |tx| {
        tx.mark_dirty(source, ASPECT_A)?;
        tx.read(source, &|view| {
            Ok(view.finish(
                NodeEvaluationResult::from_version(version_ab(99, 0))
                    .with_output_identity("analysis-bad"),
            ))
        })?;
        Err(SignalError::invalid_input("analysis failure"))
    });
    assert!(err.is_err());

    runtime.switch_branch(main.clone()).unwrap();

    let main_head = runtime.observe().branch_head_snapshot_id(main.id);
    let feature_head = runtime.observe().branch_head_snapshot_id(feature.id);
    let analysis_head = runtime.observe().branch_head_snapshot_id(analysis.id);
    let main_replay = runtime.observe().replay_for_branch(main.id);
    let feature_replay = runtime.observe().replay_for_branch(feature.id);
    let analysis_replay = runtime.observe().replay_for_branch(analysis.id);
    let artifact = runtime
        .switch_branch(feature.clone())
        .map(|_| runtime.observe().current_lineage_artifact(source).unwrap())
        .unwrap();
    let artifact_replay = runtime.observe().replay_for_artifact(artifact);
    let artifact_lineage = runtime.observe().lineage_chain_for_artifact(artifact);
    let around_feature_snapshot = runtime
        .observe()
        .replay_around_snapshot(feature_snapshot.meta.snapshot_id);

    assert_eq!(main_head, Some(main_snapshot.meta.snapshot_id));
    assert_eq!(feature_head, Some(feature_snapshot.meta.snapshot_id));
    assert_eq!(
        analysis_head,
        Some(analysis_snapshot.meta.snapshot_id),
        "analysis branch should point at its own snapshot after repeated local restores"
    );
    assert!(main_replay
        .frames
        .iter()
        .all(|frame| frame.branch_id == main.id));
    assert!(feature_replay
        .frames
        .iter()
        .all(|frame| frame.branch_id == feature.id));
    assert!(analysis_replay
        .frames
        .iter()
        .all(|frame| frame.branch_id == analysis.id));
    assert!(
        artifact_replay
            .frames
            .iter()
            .all(|frame| frame.lineage_artifact_id == Some(artifact)),
        "post-hoc artifact replay should isolate the requested artifact timeline"
    );
    assert!(
        artifact_lineage.iter().any(|record| matches!(
            record.kind,
            LineageRecordKind::ArtifactTransition {
                transition: ArtifactTransitionKind::Replaced,
                ..
            }
        )),
        "post-hoc lineage should answer where the artifact came from"
    );
    assert!(
        around_feature_snapshot
            .frames
            .iter()
            .any(|frame| frame.snapshot_id == Some(feature_snapshot.meta.snapshot_id)),
        "post-hoc replay should answer which events surrounded the feature snapshot"
    );
    assert!(
        analysis_replay
            .frames
            .iter()
            .any(|frame| frame.kind == ReplayEventKind::TransactionRolledBack),
        "post-hoc replay should preserve the failed analysis transaction"
    );
}
