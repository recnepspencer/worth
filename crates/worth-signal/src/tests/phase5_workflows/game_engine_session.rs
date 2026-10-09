use crate::facade::{
    ArtifactTransitionKind, EvaluationCondition, LineageRecordKind, NodeEvaluationResult,
    ReplayEventKind, SignalError, SignalGraph, SignalRuntime, SignalRuntimePolicy,
    SnapshotRestoreLineageMode,
};
use crate::tests::support::{mask_b, version_ab, DependencyBatchBuilder, ASPECT_A, ASPECT_B};

#[test]
fn game_engine_frame_session_handles_threshold_flapping_branch_churn_and_posthoc_debugging() {
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
        SignalRuntimePolicy::game_engine()
            .with_history_limit(6)
            .with_detail_limit(2)
            .with_observation_activation(
                worth_foundational::ObservationActivationProfile::Continuous,
            )
            .with_snapshot_restore_lineage_mode(SnapshotRestoreLineageMode::PerNode),
    );
    let source = runtime.graph_mut().node().output_identity().build();
    let culled = runtime
        .graph_mut()
        .node()
        .condition(EvaluationCondition::DeltaThreshold(2.0))
        .output_identity()
        .build();
    let lod = runtime
        .graph_mut()
        .node()
        .aspect_filter(mask_b())
        .output_identity()
        .build();
    let render = runtime.graph_mut().node().output_identity().build();
    let mut dependencies = DependencyBatchBuilder::new(runtime.graph_mut());
    dependencies
        .append_dependency(culled, source, ASPECT_A)
        .unwrap()
        .append_dependency(lod, source, ASPECT_B)
        .unwrap()
        .append_dependency(render, culled, ASPECT_A)
        .unwrap()
        .append_dependency(render, lod, ASPECT_B)
        .unwrap();
    dependencies.commit().unwrap();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(10, 100))
                        .with_output_identity("source-frame-10-meta-100"),
                ))
            })?;
            tx.read(culled, &|view| {
                let version = view.read_aspect_version(source, ASPECT_A)?;
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version)
                        .with_output_identity("culled-frame-10"),
                ))
            })?;
            tx.read(lod, &|view| {
                let version = view.read_aspect_version(source, ASPECT_B)?;
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version)
                        .with_output_identity("lod-meta-100"),
                ))
            })?;
            tx.read(render, &|view| {
                let geometry = view.read_aspect_version(culled, ASPECT_A)?;
                let lod_meta = view.read_aspect_version(lod, ASPECT_B)?;
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(
                        geometry.get(ASPECT_A),
                        lod_meta.get(ASPECT_B),
                    ))
                    .with_output_identity("render-frame-10"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let editor = runtime.observe().current_branch();
    let editor_snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    let play = runtime.create_branch("play-session").unwrap();
    runtime.switch_branch(play.clone()).unwrap();

    for frame in [11_u64, 12, 13, 14, 15, 16, 17, 18] {
        let metadata_version = if frame % 3 == 0 { 100 + frame } else { 100 };
        runtime
            .transaction(request_execution, &mut runtime_ctx, |tx| {
                tx.mark_dirty(source, ASPECT_A)?;
                if frame % 3 == 0 {
                    tx.mark_dirty(source, ASPECT_B)?;
                }
                tx.read(source, &|view| {
                    Ok(view.finish(
                        NodeEvaluationResult::from_version(version_ab(frame, metadata_version))
                            .with_output_identity(format!(
                                "source-frame-{frame}-meta-{metadata_version}"
                            )),
                    ))
                })?;
                tx.read(culled, &|view| {
                    let version = view.read_aspect_version(source, ASPECT_A)?;
                    Ok(view.finish(
                        NodeEvaluationResult::from_version(version)
                            .with_output_identity(format!("culled-frame-{frame}")),
                    ))
                })?;
                tx.read(lod, &|view| {
                    let version = view.read_aspect_version(source, ASPECT_B)?;
                    Ok(view.finish(
                        NodeEvaluationResult::from_version(version)
                            .with_output_identity(format!("lod-meta-{metadata_version}")),
                    ))
                })?;
                tx.read(render, &|view| {
                    let geometry = view.read_aspect_version(culled, ASPECT_A)?;
                    let lod_meta = view.read_aspect_version(lod, ASPECT_B)?;
                    Ok(view.finish(
                        NodeEvaluationResult::from_version(version_ab(
                            geometry.get(ASPECT_A),
                            lod_meta.get(ASPECT_B),
                        ))
                        .with_output_identity(format!(
                            "render-frame-{frame}-meta-{metadata_version}"
                        )),
                    ))
                })?;
                Ok(())
            })
            .unwrap();

        if frame % 2 == 0 {
            let err = runtime.transaction(request_execution, &mut runtime_ctx, |tx| {
                tx.mark_dirty(source, ASPECT_A)?;
                tx.mark_dirty(source, ASPECT_B)?;
                tx.read(source, &|view| {
                    Ok(view.finish(
                        NodeEvaluationResult::from_version(version_ab(frame + 100, frame + 200))
                            .with_output_identity(format!("bad-frame-{frame}")),
                    ))
                })?;
                Err(SignalError::invalid_input(
                    "synthetic playtest frame rollback",
                ))
            });
            assert!(err.is_err());
        }
    }

    let play_snapshot = runtime.capture_branch_snapshot(play.clone()).unwrap();
    runtime.switch_branch(editor.clone()).unwrap();
    runtime
        .restore_branch_snapshot(editor.clone(), &editor_snapshot)
        .unwrap();
    runtime.switch_branch(play.clone()).unwrap();
    runtime
        .restore_branch_snapshot(play.clone(), &play_snapshot)
        .unwrap();

    let play_replay = runtime.observe().replay_for_branch(play.id);
    assert!(
        play_replay
            .frames
            .iter()
            .any(|frame| frame.kind == ReplayEventKind::TransactionRolledBack),
        "play-session replay should preserve failed-frame rollback evidence"
    );
    assert!(
        runtime
            .observe()
            .lineage_chain_for_node(render)
            .iter()
            .any(|record| matches!(record.kind, LineageRecordKind::SnapshotRestore { .. })),
        "render artifact lineage should expose restore transitions after editor/play churn"
    );
    assert!(
        runtime
            .observe()
            .lineage_chain_for_node(lod)
            .iter()
            .any(|record| matches!(
                record.kind,
                LineageRecordKind::ArtifactTransition {
                    transition: ArtifactTransitionKind::Replaced,
                    ..
                }
            )),
        "the aspect-filtered LOD node should participate in the same long workflow and keep its own lineage history"
    );

    runtime.switch_branch(editor.clone()).unwrap();
    assert_eq!(
        runtime
            .graph()
            .get_entry(source)
            .unwrap()
            .get_aspect_version()
            .get(ASPECT_A),
        10,
        "editor branch should recover the original frame state after play-session churn"
    );
    assert_eq!(
        runtime
            .graph()
            .get_entry(source)
            .unwrap()
            .get_aspect_version()
            .get(ASPECT_B),
        100,
        "editor branch should also recover the untouched metadata aspect"
    );
    let editor_replay = runtime.observe().replay_for_branch(editor.id);
    assert!(editor_replay
        .frames
        .iter()
        .all(|frame| frame.branch_id == editor.id));
}
