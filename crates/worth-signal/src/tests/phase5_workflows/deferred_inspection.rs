use crate::facade::{
    ArtifactTransitionKind, LineageRecordKind, NodeEvaluationResult, ReplayEventKind, SignalError,
    SignalGraph, SignalRuntime, SignalRuntimePolicy,
};
use crate::tests::support::{version_ab, GraphDependencyBatchExt, ASPECT_A};

#[test]
fn inspect_only_at_end_after_50_step_session_preserves_forensic_truth() {
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
    runtime.set_runtime_policy(SignalRuntimePolicy::forensic().with_history_limit(10));
    let source = runtime.graph_mut().node().output_identity().build();
    let dependent = runtime.graph_mut().node().output_identity().build();
    runtime
        .graph_mut()
        .append_dependency(dependent, source, ASPECT_A)
        .unwrap();
    let mut runtime_ctx = ();
    let main = runtime.observe().current_branch();
    let mut saved = None;

    for step in 0..50_u64 {
        if step == 0 {
            runtime
                .transaction(request_execution, &mut runtime_ctx, |tx| {
                    tx.read(source, &|view| {
                        Ok(view.finish(
                            NodeEvaluationResult::from_version(version_ab(1, 0))
                                .with_output_identity("seed"),
                        ))
                    })?;
                    tx.read(dependent, &|view| {
                        let version = view.read_aspect_version(source, ASPECT_A)?;
                        Ok(view.finish(
                            NodeEvaluationResult::from_version(version)
                                .with_output_identity("dependent-seed"),
                        ))
                    })?;
                    Ok(())
                })
                .unwrap();
            saved = Some(
                runtime
                    .capture_snapshot()
                    .expect("snapshot capture should succeed without managed queue bindings"),
            );
            continue;
        }

        if step % 10 == 0 {
            let branch = runtime.create_branch(format!("branch-{step}")).unwrap();
            runtime.switch_branch(branch).unwrap();
        }
        if step % 7 == 0 {
            let err = runtime.transaction(request_execution, &mut runtime_ctx, |tx| {
                tx.mark_dirty(source, ASPECT_A)?;
                tx.read(source, &|view| {
                    Ok(view.finish(
                        NodeEvaluationResult::from_version(version_ab(1000 + step, 0))
                            .with_output_identity(format!("bad-{step}")),
                    ))
                })?;
                Err(SignalError::invalid_input(
                    "synthetic long-session rollback",
                ))
            });
            assert!(err.is_err());
        } else {
            runtime
                .transaction(request_execution, &mut runtime_ctx, |tx| {
                    tx.mark_dirty(source, ASPECT_A)?;
                    tx.read(source, &|view| {
                        Ok(view.finish(
                            NodeEvaluationResult::from_version(version_ab(1 + step, 0))
                                .with_output_identity(format!("source-{step}")),
                        ))
                    })?;
                    tx.read(dependent, &|view| {
                        let version = view.read_aspect_version(source, ASPECT_A)?;
                        Ok(view.finish(
                            NodeEvaluationResult::from_version(version)
                                .with_output_identity(format!("dependent-{step}")),
                        ))
                    })?;
                    Ok(())
                })
                .unwrap();
        }

        if step % 13 == 0 && runtime.observe().current_branch().id == main.id {
            runtime
                .restore_branch_snapshot(main.clone(), saved.as_ref().unwrap())
                .unwrap();
        }
    }

    let replay = runtime
        .observe()
        .replay_for_branch(runtime.observe().current_branch().id);
    let lineage = runtime.observe().lineage_chain_for_node(dependent);
    let explanation = runtime.observe().explain(dependent).unwrap();

    assert!(
        !replay.frames.is_empty(),
        "end-of-session replay should still be queryable"
    );
    assert!(
        !lineage.is_empty(),
        "end-of-session lineage should still be queryable"
    );
    assert!(
        replay
            .frames
            .iter()
            .any(|frame| frame.kind == ReplayEventKind::TransactionRolledBack),
        "end-of-session replay should retain rollback evidence"
    );
    assert!(
        lineage.iter().any(|record| matches!(
            record.kind,
            LineageRecordKind::ArtifactTransition {
                transition: ArtifactTransitionKind::Replaced,
                ..
            }
        )),
        "end-of-session lineage should retain materialization history"
    );
    assert!(
        explanation.execution_record_id.is_some(),
        "end-of-session explanation should still resolve to a trace-bearing artifact"
    );
}
