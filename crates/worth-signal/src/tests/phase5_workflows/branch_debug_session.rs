use crate::facade::{
    NodeEvaluationResult, OutputChange, ReplayEventKind, SignalError, SignalGraph, SignalRuntime,
    SignalRuntimePolicy,
};
use crate::tests::support::{
    define_keyed_computation, version_ab, GraphDependencyBatchExt, ASPECT_A,
};

#[test]
fn branch_debug_session_mixed_churn_stays_forensically_coherent() {
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
    runtime.set_runtime_policy(SignalRuntimePolicy::development().with_history_limit(6));
    let source = runtime.graph_mut().node().output_identity().build();
    let dependent = runtime.graph_mut().node().output_identity().build();
    runtime
        .graph_mut()
        .append_dependency(dependent, source, ASPECT_A)
        .unwrap();
    let family = define_keyed_computation(&mut runtime, "workflow-projection", ());
    let keyed_def = family.keyed("wing-left");
    let keyed = keyed_def.node(&mut runtime);
    let keyed_computation = keyed_def.memoized("shape-v1");
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("main-source-v1"),
                ))
            })?;
            tx.read(dependent, &|view| {
                let version = view.read_aspect_version(source, ASPECT_A)?;
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version)
                        .with_output_identity("main-dependent-v1"),
                ))
            })?;
            tx.evaluate_keyed(keyed, &keyed_computation, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("keyed-main-v1")
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
    let feature = runtime.create_branch("feature-debug").unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(source, ASPECT_A)?;
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(2, 0))
                        .with_output_identity("feature-source-v2"),
                ))
            })?;
            tx.read(dependent, &|view| {
                let version = view.read_aspect_version(source, ASPECT_A)?;
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version)
                        .with_output_identity("feature-dependent-v2"),
                ))
            })?;
            Ok(())
        })
        .unwrap();
    let feature_snapshot = runtime.capture_branch_snapshot(feature.clone()).unwrap();

    runtime.switch_branch(main.clone()).unwrap();
    let analysis = runtime.create_branch("analysis-debug").unwrap();
    runtime.switch_branch(analysis.clone()).unwrap();
    let analysis_snapshot = runtime.capture_branch_snapshot(analysis.clone()).unwrap();

    for cycle in 0..10 {
        let err = runtime.transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(source, ASPECT_A)?;
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(50 + cycle, 0))
                        .with_output_identity(format!("analysis-bad-{cycle}")),
                ))
            })?;
            Err(SignalError::invalid_input("synthetic analysis rollback"))
        });
        assert!(err.is_err());

        runtime
            .restore_branch_snapshot(analysis.clone(), &analysis_snapshot)
            .unwrap();
        runtime.switch_branch(feature.clone()).unwrap();
        runtime
            .restore_branch_snapshot(feature.clone(), &feature_snapshot)
            .unwrap();
        runtime.switch_branch(analysis.clone()).unwrap();
    }

    runtime.switch_branch(main.clone()).unwrap();
    assert_eq!(
        runtime
            .graph()
            .get_entry(source)
            .unwrap()
            .get_aspect_version()
            .get(ASPECT_A),
        1
    );
    let main_replay = runtime.observe().replay_for_branch(main.id);
    assert!(main_replay
        .frames
        .iter()
        .all(|frame| frame.branch_id == main.id));

    runtime.switch_branch(feature.clone()).unwrap();
    assert_eq!(
        runtime
            .graph()
            .get_entry(source)
            .unwrap()
            .get_aspect_version()
            .get(ASPECT_A),
        2
    );
    let feature_replay = runtime.observe().replay_for_branch(feature.id);
    assert!(feature_replay
        .frames
        .iter()
        .all(|frame| frame.branch_id == feature.id));
    assert!(
        feature_replay
            .frames
            .iter()
            .any(|frame| frame.snapshot_id == Some(feature_snapshot.meta.snapshot_id)),
        "feature replay should still point back to the saved branch head snapshot"
    );

    runtime.switch_branch(analysis.clone()).unwrap();
    assert_eq!(
        runtime
            .graph()
            .get_entry(source)
            .unwrap()
            .get_aspect_version()
            .get(ASPECT_A),
        1,
        "analysis restores should keep the branch aligned with the captured main snapshot"
    );
    let analysis_replay = runtime.observe().replay_for_branch(analysis.id);
    assert!(analysis_replay
        .frames
        .iter()
        .all(|frame| frame.branch_id == analysis.id));
    assert!(
        analysis_replay
            .frames
            .iter()
            .any(|frame| frame.kind == ReplayEventKind::TransactionRolledBack),
        "analysis debug session should preserve rollback evidence for post-hoc inspection"
    );
    assert!(
        analysis_replay
            .frames
            .iter()
            .any(|frame| frame.kind == ReplayEventKind::SnapshotRestored),
        "analysis debug session should preserve restore evidence for post-hoc inspection"
    );
    assert_eq!(
        runtime.observe().branch_head_snapshot_id(main.id),
        Some(main_snapshot.meta.snapshot_id)
    );
    assert_eq!(
        runtime.observe().branch_head_snapshot_id(feature.id),
        Some(feature_snapshot.meta.snapshot_id)
    );
    assert_eq!(
        runtime.observe().branch_head_snapshot_id(analysis.id),
        Some(analysis_snapshot.meta.snapshot_id),
        "analysis branch should stay pinned to its own restored snapshot"
    );
}
