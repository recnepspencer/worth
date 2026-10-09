use crate::facade::{
    NodeEvaluationResult, ReplayEventKind, SignalError, SignalGraph, SignalRuntime,
    SignalRuntimePolicy,
};
use crate::tests::support::{version_ab, ASPECT_A};

#[test]
fn branch_local_transaction_failure_does_not_advance_heads_or_leak_committed_artifacts() {
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
    let feature = runtime.create_branch("feature-failure").unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(source, ASPECT_A)?;
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(2, 0))
                        .with_output_identity("feature-stable"),
                ))
            })?;
            Ok(())
        })
        .unwrap();
    let feature_snapshot = runtime.capture_branch_snapshot(feature.clone()).unwrap();
    let feature_head_before = runtime.observe().branch_head_snapshot_id(feature.id);
    let feature_artifact_before = runtime.observe().current_lineage_artifact(source);
    let feature_lineage_before = runtime
        .graph()
        .observe()
        .lineage_for_node(source)
        .to_owned_records();
    let feature_replay_before = runtime.observe().replay_for_branch(feature.id);
    let policy_before_failure = runtime.runtime_policy();

    let err = runtime.transaction(request_execution, &mut runtime_ctx, |tx| {
        tx.try_set_runtime_policy(SignalRuntimePolicy::forensic())
            .map_err(|_| {
                SignalError::invalid_input("forensic rollback policy should be admissible")
            })?;
        tx.mark_dirty(source, ASPECT_A)?;
        tx.read(source, &|view| {
            Ok(view.finish(
                NodeEvaluationResult::from_version(version_ab(3, 0))
                    .with_output_identity("feature-bad"),
            ))
        })?;
        Err(SignalError::invalid_input("force branch-local rollback"))
    });
    assert!(err.is_err(), "failing transaction should surface an error");
    assert_eq!(
        runtime.runtime_policy(),
        policy_before_failure,
        "failed transactions must restore the installed runtime policy"
    );

    assert_eq!(
        runtime.observe().branch_head_snapshot_id(feature.id),
        feature_head_before,
        "failed branch-local work must not advance the branch head"
    );
    assert_eq!(
        runtime.observe().current_lineage_artifact(source),
        feature_artifact_before,
        "failed branch-local work must not replace the committed lineage artifact"
    );
    assert_eq!(
        runtime
            .graph()
            .get_entry(source)
            .unwrap()
            .get_aspect_version()
            .get(ASPECT_A),
        2,
        "failed branch-local work must rewind the active branch graph state"
    );
    assert_eq!(
        runtime
            .graph()
            .observe()
            .lineage_for_node(source)
            .to_owned_records(),
        feature_lineage_before,
        "failed branch-local work must not leak committed lineage transitions for the node"
    );
    let feature_replay_after = runtime.observe().replay_for_branch(feature.id);
    assert!(
        feature_replay_after.frames.len() >= feature_replay_before.frames.len(),
        "failed branch-local work may append rollback/failure events, but it must not erase prior replay"
    );
    assert!(
        feature_replay_after
            .frames
            .iter()
            .all(|frame| frame.branch_id == feature.id),
        "failed branch-local work must keep replay isolation inside the active branch"
    );
    assert!(
        feature_replay_after
            .frames
            .iter()
            .filter(|frame| frame.kind == ReplayEventKind::TransactionCommitted)
            .count()
            == feature_replay_before
                .frames
                .iter()
                .filter(|frame| frame.kind == ReplayEventKind::TransactionCommitted)
                .count(),
        "failed branch-local work must not leak a committed replay outcome"
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
        "branch-local rollback must not contaminate sibling branches"
    );
    let main_replay = runtime.observe().replay_for_branch(main.id);
    assert!(
        main_replay
            .frames
            .iter()
            .all(|frame| frame.branch_id == main.id),
        "sibling branch replay should remain branch-local after feature failure"
    );

    runtime.switch_branch(feature).unwrap();
    runtime
        .restore_branch_snapshot(runtime.observe().current_branch(), &feature_snapshot)
        .unwrap();
    assert_eq!(
        runtime
            .graph()
            .get_entry(source)
            .unwrap()
            .get_aspect_version()
            .get(ASPECT_A),
        2,
        "restoring the saved feature snapshot should still be possible after failure churn"
    );
}
