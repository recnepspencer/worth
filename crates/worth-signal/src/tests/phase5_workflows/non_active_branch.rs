use crate::facade::{
    ArtifactTransitionKind, LineageRecordKind, NodeEvaluationResult, SignalError, SignalGraph,
    SignalRuntime, SignalRuntimePolicy,
};
use crate::tests::support::{mask_b, version_ab, GraphDependencyBatchExt, ASPECT_A, ASPECT_B};

#[test]
fn non_active_branch_inspection_after_heavy_foreground_churn_uses_stored_branch_state() {
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
    let filtered = runtime
        .graph_mut()
        .node()
        .aspect_filter(mask_b())
        .output_identity()
        .build();
    runtime
        .graph_mut()
        .append_dependency(filtered, source, ASPECT_B)
        .unwrap();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 10))
                        .with_output_identity("seed"),
                ))
            })?;
            tx.read(filtered, &|view| {
                let version = view.read_aspect_version(source, ASPECT_B)?;
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version)
                        .with_output_identity("filtered-seed"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime.create_branch("feature-inspect").unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(source, ASPECT_B)?;
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 20))
                        .with_output_identity("feature"),
                ))
            })?;
            tx.read(filtered, &|view| {
                let version = view.read_aspect_version(source, ASPECT_B)?;
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version)
                        .with_output_identity("filtered-feature"),
                ))
            })?;
            Ok(())
        })
        .unwrap();
    let feature_snapshot = runtime.capture_branch_snapshot(feature.clone()).unwrap();

    runtime.switch_branch(main.clone()).unwrap();
    let analysis = runtime.create_branch("analysis-foreground").unwrap();
    runtime.switch_branch(analysis.clone()).unwrap();
    for step in 0..25_u64 {
        if step % 4 == 0 {
            let err = runtime.transaction(request_execution, &mut runtime_ctx, |tx| {
                tx.mark_dirty(source, ASPECT_B)?;
                tx.read(source, &|view| {
                    Ok(view.finish(
                        NodeEvaluationResult::from_version(version_ab(100 + step, 1000 + step))
                            .with_output_identity(format!("bad-{step}")),
                    ))
                })?;
                Err(SignalError::invalid_input("synthetic analysis rollback"))
            });
            assert!(err.is_err());
        } else {
            runtime
                .transaction(request_execution, &mut runtime_ctx, |tx| {
                    tx.mark_dirty(source, ASPECT_A)?;
                    tx.read(source, &|view| {
                        Ok(view.finish(
                            NodeEvaluationResult::from_version(version_ab(50 + step, 10))
                                .with_output_identity(format!("analysis-{step}")),
                        ))
                    })?;
                    Ok(())
                })
                .unwrap();
        }
    }

    let feature_replay = runtime.observe().replay_for_branch(feature.id);
    let feature_lineage = runtime.observe().lineage_chain_for_node(filtered);
    let feature_head = runtime.observe().branch_head_snapshot_id(feature.id);
    assert_eq!(feature_head, Some(feature_snapshot.meta.snapshot_id));
    assert!(feature_replay
        .frames
        .iter()
        .all(|frame| frame.branch_id == feature.id));
    assert!(
        feature_replay
            .frames
            .iter()
            .any(|frame| frame.snapshot_id == Some(feature_snapshot.meta.snapshot_id)),
        "non-active branch inspection should still see the stored feature snapshot head"
    );
    assert!(
        feature_lineage
            .iter()
            .any(|record| matches!(
                record.kind,
                LineageRecordKind::ArtifactTransition {
                    transition: ArtifactTransitionKind::Replaced,
                    ..
                }
            )),
        "non-active branch inspection should read stored lineage instead of the active branch state"
    );
}
