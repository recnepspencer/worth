use crate::facade::{
    mark_dirty, LineageRecordKind, NodeEvaluationResult, OutputChange, SignalError, SignalGraph,
    SignalRuntime, SignalRuntimePolicy, SnapshotRestoreLineageMode,
};
use crate::tests::support::{define_keyed_computation, version_ab, ASPECT_A};
use std::sync::atomic::{AtomicU32, Ordering};

#[test]
fn undo_redo_style_session_with_failures_and_memo_reuse_preserves_branch_local_truth() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let policy = SignalRuntimePolicy::development()
        .with_history_limit(4)
        .with_snapshot_restore_lineage_mode(SnapshotRestoreLineageMode::PerNode);
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    runtime.set_runtime_policy(policy);
    let source = runtime.graph_mut().node().output_identity().build();
    let family = define_keyed_computation(&mut runtime, "undo-redo-session", ());
    let keyed_def = family.keyed("bulkhead");
    let keyed = keyed_def.node(&mut runtime);
    let computation = keyed_def.memoized("shape-v1");
    let compute_calls = AtomicU32::new(0);
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("main-v1"),
                ))
            })?;
            tx.evaluate_keyed(keyed, &computation, &|view| {
                compute_calls.fetch_add(1, Ordering::Relaxed);
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("memo-artifact")
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
    let feature = runtime.create_branch("feature-undo").unwrap();
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
            tx.evaluate_keyed(keyed, &computation, &|view| {
                compute_calls.fetch_add(1, Ordering::Relaxed);
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(2, 0))
                        .with_output_identity("memo-artifact")
                        .with_output_change(OutputChange::Refreshed),
                ))
            })?;
            Ok(())
        })
        .unwrap();
    let feature_snapshot = runtime.capture_branch_snapshot(feature.clone()).unwrap();

    for cycle in 0..20 {
        runtime.switch_branch(main.clone()).unwrap();
        runtime
            .restore_branch_snapshot(main.clone(), &main_snapshot)
            .unwrap();

        runtime.switch_branch(feature.clone()).unwrap();
        runtime
            .restore_branch_snapshot(feature.clone(), &feature_snapshot)
            .unwrap();

        let err = runtime.transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(source, ASPECT_A)?;
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(100 + cycle, 0))
                        .with_output_identity(format!("bad-{cycle}")),
                ))
            })?;
            Err(SignalError::invalid_input("synthetic feature rollback"))
        });
        assert!(err.is_err());

        mark_dirty(runtime.graph_mut(), keyed, ASPECT_A).unwrap();
        runtime
            .transaction(request_execution, &mut runtime_ctx, |tx| {
                tx.evaluate_keyed(keyed, &computation, &|view| {
                    compute_calls.fetch_add(1, Ordering::Relaxed);
                    Ok(view.finish(NodeEvaluationResult::from_version(version_ab(999, 0))))
                })?;
                Ok(())
            })
            .unwrap();
    }

    assert_eq!(
        compute_calls.load(Ordering::Relaxed),
        1,
        "long undo/redo churn should keep reusing the memoized artifact instead of recomputing every cycle"
    );

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
    let feature_lineage = runtime.observe().lineage_chain_for_node(source);
    assert!(
        feature_lineage
            .iter()
            .any(|record| matches!(record.kind, LineageRecordKind::SnapshotRestore { .. })),
        "feature workflow should preserve restore lineage under undo/redo churn"
    );
    assert!(
        runtime
            .observe()
            .recent_execution_history_diagnostics()
            .len()
            <= policy.retention_budget.history_limit,
        "history must stay bounded under long undo/redo churn"
    );
    assert!(
        runtime.graph().replay_events().len() <= policy.retention_budget.history_limit.max(1) * 32,
        "replay must stay bounded under long undo/redo churn"
    );
}
