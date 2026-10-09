use crate::facade::{
    mark_dirty, ArtifactTransitionKind, EvaluationCondition, LineageRecordKind,
    NodeEvaluationResult, OutputChange, ReplayEventKind, SignalError, SignalGraph, SignalRuntime,
    SignalRuntimePolicy,
};
use crate::tests::support::{
    define_keyed_computation, mask_b, version_ab, DependencyBatchBuilder, ASPECT_A, ASPECT_B,
};
use std::sync::atomic::{AtomicU32, Ordering};

#[test]
fn fintech_tick_correction_session_preserves_auditability_under_branching_replay_and_memo_reuse() {
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
    runtime.set_runtime_policy(
        SignalRuntimePolicy::fintech()
            .with_history_limit(8)
            .with_detail_limit(3),
    );
    let ticks = runtime.graph_mut().node().output_identity().build();
    let price = runtime.graph_mut().node().output_identity().build();
    let alert = runtime
        .graph_mut()
        .node()
        .aspect_filter(mask_b())
        .output_identity()
        .build();
    let throttle = runtime
        .graph_mut()
        .node()
        .condition(EvaluationCondition::DeltaThreshold(2.0))
        .output_identity()
        .build();
    let mut dependencies = DependencyBatchBuilder::new(runtime.graph_mut());
    dependencies
        .append_dependency(price, ticks, ASPECT_A)
        .unwrap()
        .append_dependency(alert, ticks, ASPECT_B)
        .unwrap()
        .append_dependency(throttle, ticks, ASPECT_A)
        .unwrap();
    dependencies.commit().unwrap();
    let family = define_keyed_computation(&mut runtime, "pricing-book", ());
    let risk_def = family.keyed("book-a");
    let risk = risk_def.node(&mut runtime);
    let risk_computation = risk_def.memoized("book-a-day-1");
    let compute_calls = AtomicU32::new(0);
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(ticks, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(100, 5))
                        .with_output_identity("ticks-100-vol-5"),
                ))
            })?;
            tx.read(price, &|view| {
                let version = view.read_aspect_version(ticks, ASPECT_A)?;
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version).with_output_identity("price-100"),
                ))
            })?;
            tx.read(alert, &|view| {
                let version = view.read_aspect_version(ticks, ASPECT_B)?;
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version).with_output_identity("alert-vol-5"),
                ))
            })?;
            tx.read(throttle, &|view| {
                let version = view.read_aspect_version(ticks, ASPECT_A)?;
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version)
                        .with_output_identity("throttle-100"),
                ))
            })?;
            tx.evaluate_keyed(risk, &risk_computation, &|view| {
                compute_calls.fetch_add(1, Ordering::Relaxed);
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(100, 0))
                        .with_output_identity("risk-100")
                        .with_output_change(OutputChange::Refreshed),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let audit_snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    let what_if = runtime.create_branch("what-if-shock").unwrap();
    runtime.switch_branch(what_if.clone()).unwrap();

    for tick in [101_u64, 102, 103, 104] {
        let volatility = if tick % 2 == 0 { 9 } else { 5 };
        runtime
            .transaction(request_execution, &mut runtime_ctx, |tx| {
                tx.mark_dirty(ticks, ASPECT_A)?;
                if tick % 2 == 0 {
                    tx.mark_dirty(ticks, ASPECT_B)?;
                }
                tx.read(ticks, &|view| {
                    Ok(view.finish(
                        NodeEvaluationResult::from_version(version_ab(tick, volatility))
                            .with_output_identity(format!("ticks-{tick}-vol-{volatility}")),
                    ))
                })?;
                tx.read(price, &|view| {
                    let version = view.read_aspect_version(ticks, ASPECT_A)?;
                    Ok(view.finish(
                        NodeEvaluationResult::from_version(version)
                            .with_output_identity(format!("price-{tick}")),
                    ))
                })?;
                tx.read(alert, &|view| {
                    let version = view.read_aspect_version(ticks, ASPECT_B)?;
                    Ok(view.finish(
                        NodeEvaluationResult::from_version(version)
                            .with_output_identity(format!("alert-vol-{volatility}")),
                    ))
                })?;
                tx.read(throttle, &|view| {
                    let version = view.read_aspect_version(ticks, ASPECT_A)?;
                    Ok(view.finish(
                        NodeEvaluationResult::from_version(version)
                            .with_output_identity(format!("throttle-{tick}")),
                    ))
                })?;
                Ok(())
            })
            .unwrap();

        mark_dirty(runtime.graph_mut(), risk, ASPECT_A).unwrap();
        runtime
            .transaction(request_execution, &mut runtime_ctx, |tx| {
                tx.evaluate_keyed(risk, &risk_computation, &|view| {
                    compute_calls.fetch_add(1, Ordering::Relaxed);
                    Ok(view.finish(NodeEvaluationResult::from_version(version_ab(9999, 0))))
                })?;
                Ok(())
            })
            .unwrap();
    }

    let what_if_snapshot = runtime.capture_branch_snapshot(what_if.clone()).unwrap();
    let correction = runtime.create_branch("late-tick-correction").unwrap();
    runtime.switch_branch(correction.clone()).unwrap();
    let correction_snapshot = runtime.capture_branch_snapshot(correction.clone()).unwrap();

    let err = runtime.transaction(request_execution, &mut runtime_ctx, |tx| {
        tx.mark_dirty(ticks, ASPECT_A)?;
        tx.read(ticks, &|view| {
            Ok(view.finish(
                NodeEvaluationResult::from_version(version_ab(80, 0))
                    .with_output_identity("bad-correction"),
            ))
        })?;
        Err(SignalError::invalid_input("synthetic late-tick rollback"))
    });
    assert!(err.is_err());

    runtime
        .restore_branch_snapshot(correction.clone(), &correction_snapshot)
        .unwrap();
    runtime.switch_branch(main.clone()).unwrap();
    runtime
        .restore_branch_snapshot(main.clone(), &audit_snapshot)
        .unwrap();
    runtime.switch_branch(what_if.clone()).unwrap();
    runtime
        .restore_branch_snapshot(what_if.clone(), &what_if_snapshot)
        .unwrap();

    assert_eq!(
        compute_calls.load(Ordering::Relaxed),
        1,
        "risk memoization should survive tick-session churn without recomputing every audit pass"
    );

    let correction_replay = runtime.observe().replay_for_branch(correction.id);
    assert!(
        correction_replay
            .frames
            .iter()
            .any(|frame| frame.kind == ReplayEventKind::TransactionRolledBack),
        "correction branch replay should preserve rollback evidence for audit"
    );
    runtime.switch_branch(what_if.clone()).unwrap();
    let risk_artifact = runtime.observe().current_lineage_artifact(risk).unwrap();
    let risk_replay = runtime.observe().replay_for_artifact(risk_artifact);
    assert!(
        risk_replay
            .frames
            .iter()
            .all(|frame| frame.lineage_artifact_id == Some(risk_artifact)),
        "artifact replay should isolate the memoized risk artifact timeline"
    );
    assert!(
        runtime
            .observe()
            .lineage_chain_for_artifact(risk_artifact)
            .iter()
            .any(|record| matches!(
                record.kind,
                LineageRecordKind::ArtifactTransition {
                    transition: ArtifactTransitionKind::MemoizedReuse,
                    ..
                }
            )),
        "artifact lineage should explain memoized reuse in the audit workflow"
    );
    assert!(
        runtime
            .observe()
            .lineage_chain_for_node(alert)
            .iter()
            .any(|record| matches!(
                record.kind,
                LineageRecordKind::ArtifactTransition {
                    transition: ArtifactTransitionKind::Replaced,
                    ..
                }
            )),
        "the aspect-filtered alert node should keep its own branch-local lineage under the same workflow"
    );
    let around_snapshot = runtime
        .observe()
        .replay_around_snapshot(what_if_snapshot.meta.snapshot_id);
    assert!(
        around_snapshot
            .frames
            .iter()
            .any(|frame| frame.snapshot_id == Some(what_if_snapshot.meta.snapshot_id)),
        "audit replay should answer what happened around the what-if snapshot"
    );

    runtime.switch_branch(main).unwrap();
    assert_eq!(
        runtime.graph().get_entry(ticks).unwrap().get_aspect_version().get(ASPECT_A),
        100,
        "main branch should remain on the authoritative baseline after what-if and correction churn"
    );
    assert_eq!(
        runtime
            .graph()
            .get_entry(ticks)
            .unwrap()
            .get_aspect_version()
            .get(ASPECT_B),
        5,
        "main branch should also preserve the baseline volatility aspect after churn"
    );
}
