use crate::data::dependency::DependencySnapshot;
use crate::facade::{
    ArtifactMergeAction, BranchMergeConflictKind, BranchMergeDivergence, BranchMergeKind,
    ConflictMergePolicy, LineageRecordKind, NodeEvaluationResult, ReplayEventKind, SignalGraph,
    SignalRuntime,
};
use crate::logic::transaction::BranchMergeResolutionRequirement;
use crate::tests::support::{version_ab, GraphDependencyBatchExt, ASPECT_A, ASPECT_B};

#[test]
fn merge_branch_runtime_artifact_conflict_can_resolve_by_adopting_source() {
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
    let shared = runtime.graph_mut().node().output_identity().build();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(shared, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(141, 0))
                        .with_output_identity("base-runtime-conflict"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("feature-runtime-conflict-resolve")
        .unwrap();

    runtime.switch_branch(feature.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(shared, ASPECT_A)?;
            tx.read(shared, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(142, 0))
                        .with_output_identity("feature-runtime-conflict"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    runtime.switch_branch(main.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(shared, ASPECT_A)?;
            tx.read(shared, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(143, 0))
                        .with_output_identity("main-runtime-conflict"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let result = runtime.merge_branch_raw(feature, main).unwrap();

    assert!(
        matches!(result.merge_kind, BranchMergeKind::Applied | BranchMergeKind::ConflictResolved),
        "stable-shape snapshot reconciliation may classify as applied or resolved conflict, but it should still use the narrow snapshot delta path"
    );
    assert_eq!(
        result.divergence,
        BranchMergeDivergence::SharedStateConflict
    );
    assert_eq!(
        result.reconciliation_policy.conflict,
        ConflictMergePolicy::ResolveSourceStateWhenStructureMatches
    );
    assert!(result.resolution_plan.is_some());
    let merged_record = result
        .records
        .iter()
        .find(|record| record.source_node == shared)
        .expect("shared node should be part of the resolved merge");
    assert!(
        matches!(merged_record.action, ArtifactMergeAction::Adopted),
        "runtime-artifact conflict resolution should adopt source authority"
    );
    assert!(merged_record
        .resolved_conflict_kinds
        .contains(&BranchMergeConflictKind::RuntimeArtifactMismatch));
    assert_eq!(
        merged_record
            .target_comparable
            .as_ref()
            .and_then(|comparable| comparable.output_identity.as_ref())
            .map(|identity| identity.as_str()),
        Some("feature-runtime-conflict"),
        "resolved runtime-artifact conflict should adopt source runtime state"
    );
    assert!(
        runtime
            .graph()
            .observe()
            .lineage_records()
            .iter()
            .any(|record| matches!(
                record.kind,
                LineageRecordKind::BranchMerge {
                    merge_kind: BranchMergeKind::ConflictResolved,
                    resolution_plan: Some(_),
                    ..
                }
            )),
        "resolved conflicts should emit real conflict-resolved lineage"
    );
}

#[test]
fn merge_branch_dependency_snapshot_conflict_can_resolve_by_adopting_source_snapshot() {
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
    let shared = runtime.graph_mut().node().output_identity().build();
    let source = runtime.graph_mut().node().output_identity().build();
    runtime
        .graph_mut()
        .append_dependency(shared, source, ASPECT_A)
        .unwrap();

    let mut runtime_ctx = ();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(shared, &|view| {
                let result = if view.node() == source {
                    view.finish(
                        NodeEvaluationResult::from_version(version_ab(101, 0))
                            .with_output_identity("snapshot-source"),
                    )
                } else {
                    view.finish(
                        NodeEvaluationResult::from_version(version_ab(102, 0))
                            .with_output_identity("snapshot-shared"),
                    )
                };
                Ok(result)
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime.create_branch("feature-snapshot-conflict").unwrap();

    let mut feature_snapshot = DependencySnapshot::empty();
    feature_snapshot.record(source, ASPECT_A, 3, None);
    runtime.switch_branch(feature.clone()).unwrap();
    runtime
        .graph_mut()
        .set_dep_snapshot(shared, feature_snapshot)
        .unwrap();

    let mut main_snapshot = DependencySnapshot::empty();
    main_snapshot.record(source, ASPECT_A, 5, None);
    main_snapshot.record(source, ASPECT_B, 7, None);
    runtime.switch_branch(main.clone()).unwrap();
    runtime
        .graph_mut()
        .set_dep_snapshot(shared, main_snapshot)
        .unwrap();

    let result = runtime.merge_branch_raw(feature, main).unwrap();
    let merged_record = result
        .records
        .iter()
        .find(|record| record.source_node == shared)
        .expect("shared node should be part of the resolved snapshot merge");

    assert!(
        matches!(result.merge_kind, BranchMergeKind::Applied | BranchMergeKind::ConflictResolved),
        "stable-shape snapshot reconciliation may classify as applied or resolved conflict, but it should still use the narrow snapshot delta path"
    );
    assert_eq!(
        result.divergence,
        BranchMergeDivergence::SharedStateConflict
    );
    assert_eq!(
        result.reconciliation_policy.conflict,
        ConflictMergePolicy::ResolveSourceStateWhenStructureMatches
    );
    assert!(result.resolution_plan.is_some());
    assert!(matches!(
        merged_record.action,
        ArtifactMergeAction::EquivalentUnchanged
    ));
    assert!(merged_record
        .resolved_conflict_kinds
        .contains(&BranchMergeConflictKind::DependencySnapshotMismatch));
    let merged_snapshot = runtime.graph().get_dep_snapshot(shared).unwrap();
    assert_eq!(
        merged_snapshot.entries().len(),
        1,
        "resolved dependency snapshot conflict should adopt the source snapshot shape"
    );
    assert_eq!(merged_snapshot.entries()[0].cached_version, 3);
    assert_eq!(merged_snapshot.entries()[0].aspect, ASPECT_A);
}

#[test]
fn merge_branch_conflict_resolved_emits_resolution_traceability() {
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
    let shared = runtime.graph_mut().node().output_identity().build();
    let mut runtime_ctx = ();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(shared, ASPECT_A)?;
            tx.read(shared, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(150, 0))
                        .with_output_identity("main-conflict-trace"),
                ))
            })?;
            Ok(())
        })
        .unwrap();
    let main = runtime.observe().current_branch();
    let feature = runtime.create_branch("feature-trace").unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(shared, ASPECT_A)?;
            tx.read(shared, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(151, 0))
                        .with_output_identity("feature-conflict-trace"),
                ))
            })?;
            Ok(())
        })
        .unwrap();
    runtime.switch_branch(main.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(shared, ASPECT_A)?;
            tx.read(shared, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(152, 0))
                        .with_output_identity("main-conflict-trace-advanced"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let result = runtime.merge_branch_raw(feature, main).unwrap();
    assert_eq!(result.merge_kind, BranchMergeKind::ConflictResolved);
    let resolution_plan = result
        .resolution_plan
        .as_ref()
        .expect("conflict-resolved merge should retain resolution plan");
    assert_eq!(resolution_plan.records.len(), 1);
    assert!(resolution_plan.records[0]
        .required_resolution
        .contains(&BranchMergeResolutionRequirement::ReconcileRuntimeArtifactState));

    let replay_detail = runtime
        .graph()
        .observe()
        .replay_events()
        .iter()
        .rev()
        .find(|event| matches!(event.kind, ReplayEventKind::BranchMerged))
        .and_then(|event| event.detail.as_ref().and_then(|detail| detail.as_message()))
        .expect("conflict-resolved merge should emit branch merge replay detail");
    assert!(
        replay_detail.contains("resolved_requirements"),
        "conflict-resolved replay should expose resolved requirements"
    );

    let branch_merge_lineage = runtime
        .graph()
        .observe()
        .lineage_records()
        .iter()
        .rev()
        .find(|record| {
            matches!(
                record.kind,
                LineageRecordKind::BranchMerge {
                    merge_kind: BranchMergeKind::ConflictResolved,
                    ..
                }
            )
        })
        .expect("conflict-resolved merge should emit branch merge lineage");
    match &branch_merge_lineage.kind {
        LineageRecordKind::BranchMerge {
            resolution_plan: Some(plan),
            ..
        } => {
            assert_eq!(plan.records.len(), 1);
            assert!(plan.records[0]
                .required_resolution
                .contains(&BranchMergeResolutionRequirement::ReconcileRuntimeArtifactState));
        }
        other => panic!("expected conflict-resolved branch merge lineage, got {other:?}"),
    }

    let artifact_merge_lineage = runtime
        .graph()
        .observe()
        .lineage_records()
        .iter()
        .rev()
        .find(|record| {
            matches!(
                record.kind,
                LineageRecordKind::ArtifactMerge {
                    source_node,
                    merge_kind: BranchMergeKind::ConflictResolved,
                    ..
                } if source_node == shared
            )
        })
        .expect("conflict-resolved merge should emit artifact merge lineage");
    match &artifact_merge_lineage.kind {
        LineageRecordKind::ArtifactMerge {
            resolved_conflict_kinds,
            ..
        } => {
            assert!(
                resolved_conflict_kinds.contains(&BranchMergeConflictKind::RuntimeArtifactMismatch)
            );
        }
        other => panic!("expected conflict-resolved artifact merge lineage, got {other:?}"),
    }
}
