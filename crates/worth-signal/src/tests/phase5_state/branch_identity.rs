use crate::facade::{NodeEvaluationResult, SignalGraph, SignalRuntime};
use crate::tests::support::{version_ab, ASPECT_A};

#[test]
fn branched_runtime_preserves_unique_branch_and_snapshot_ids() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();

    let main = runtime.observe().current_branch();
    let feature = runtime.create_branch("feature-runtime-ids").unwrap();

    runtime.switch_branch(feature.clone()).unwrap();
    let nested = runtime.create_branch("nested-runtime-ids").unwrap();
    let feature_snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");

    runtime.switch_branch(main.clone()).unwrap();
    let sibling = runtime.create_branch("sibling-runtime-ids").unwrap();
    let main_snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");

    assert!(
        sibling.id > nested.id,
        "restored main branch state must not reuse a branch id already allocated on another branch"
    );
    assert!(
        main_snapshot.meta.snapshot_id > feature_snapshot.meta.snapshot_id,
        "restored main branch state must not reuse a snapshot id already allocated on another branch"
    );
}

#[test]
fn branch_switch_and_restore_churn_preserve_branch_local_heads_and_replay_isolation() {
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
                        .with_output_identity("main-artifact"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime.create_branch("feature-churn").unwrap();
    let main_snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");

    runtime.switch_branch(feature.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(source, ASPECT_A)?;
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(2, 0))
                        .with_output_identity("feature-artifact"),
                ))
            })?;
            Ok(())
        })
        .unwrap();
    let feature_snapshot = runtime.capture_branch_snapshot(feature.clone()).unwrap();

    for _ in 0..6 {
        runtime.switch_branch(main.clone()).unwrap();
        runtime
            .restore_branch_snapshot(main.clone(), &main_snapshot)
            .unwrap();
        runtime.switch_branch(feature.clone()).unwrap();
        runtime
            .restore_branch_snapshot(feature.clone(), &feature_snapshot)
            .unwrap();
    }

    assert_eq!(
        runtime.observe().branch_head_snapshot_id(main.id),
        Some(main_snapshot.meta.snapshot_id),
        "main branch head should stay pinned to its own restored snapshot"
    );
    assert_eq!(
        runtime.observe().branch_head_snapshot_id(feature.id),
        Some(feature_snapshot.meta.snapshot_id),
        "feature branch head should stay pinned to its own restored snapshot"
    );

    runtime.switch_branch(main.clone()).unwrap();
    assert_eq!(
        runtime
            .graph()
            .get_entry(source)
            .unwrap()
            .get_aspect_version()
            .get(ASPECT_A),
        1,
        "main branch state should survive churn"
    );
    let main_replay = runtime.observe().replay_for_branch(main.id);
    assert!(
        main_replay
            .frames
            .iter()
            .all(|frame| frame.branch_id == main.id),
        "main branch replay should remain branch-local after churn"
    );

    runtime.switch_branch(feature.clone()).unwrap();
    assert_eq!(
        runtime
            .graph()
            .get_entry(source)
            .unwrap()
            .get_aspect_version()
            .get(ASPECT_A),
        2,
        "feature branch state should survive churn"
    );
    let feature_replay = runtime.observe().replay_for_branch(feature.id);
    assert!(
        feature_replay
            .frames
            .iter()
            .all(|frame| frame.branch_id == feature.id),
        "feature branch replay should remain branch-local after churn"
    );
}
