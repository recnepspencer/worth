use crate::diagnostics::replay::ReplayEventDetail;
use crate::facade::*;
use crate::tests::support::{version_ab, ASPECT_A, ASPECT_B};

fn operational_graph_digest(graph: &SignalGraph) -> String {
    let rows = graph
        .live_node_ids()
        .into_iter()
        .map(|node| {
            let parts = graph.node_checkpoint_image(node).unwrap().into_parts();
            let causes = graph
                .pending_causes(node)
                .unwrap()
                .iter()
                .map(|cause| {
                    format!(
                        "{:?}:{:?}:{:?}:{:?}:{}:{}:{}:{:?}",
                        cause.key.producer,
                        cause.key.aspect,
                        cause.key.edge_scope,
                        cause.key.dependency_revision,
                        cause.binding_axes.cached_version,
                        cause.binding_axes.output_commit_ordinal.0,
                        cause.binding_axes.committed_version,
                        cause.changed_scopes,
                    )
                })
                .collect::<Vec<_>>();
            format!(
                "{node:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{causes:?}",
                parts.state,
                parts.dirty_aspects,
                parts.dirty_partition_scopes,
                parts.aspect_versions,
                parts.dependency_revision,
                parts.pending_dependency_revalidation,
                parts.direct_invalidation_basis,
                parts.eval_config,
                parts.runtime_artifact_state,
                graph.dependencies_of(node).unwrap(),
                graph.subscribers_of(node).unwrap(),
                graph.get_dep_snapshot(node).unwrap(),
            )
        })
        .collect::<Vec<_>>();
    canonical_digest(&rows)
}

fn build_scoped_proof_runtime() -> (
    SignalRuntime<(), (), (), (), ()>,
    SignalBranchHandle,
    SignalBranchHandle,
    NodeId,
    NodeId,
    NodeId,
) {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let graph = SignalGraph::new();
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let support = runtime
        .graph_mut()
        .node()
        .reads_aspects([ASPECT_A])
        .produces_aspects([ASPECT_A])
        .build();
    let primary = runtime
        .graph_mut()
        .node()
        .reads_aspects([ASPECT_A])
        .produces_aspects([ASPECT_A])
        .build();
    let companion = runtime
        .graph_mut()
        .node()
        .reads_aspects([ASPECT_A])
        .produces_aspects([ASPECT_A])
        .build();
    runtime
        .graph_mut()
        .append_dependency(primary, support, ASPECT_A)
        .unwrap();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(support, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(1, 0))))
            })?;
            tx.read(primary, &|view| {
                let upstream = view.read_aspect_version(support, ASPECT_A)?;
                Ok(view.finish(NodeEvaluationResult::from_version(upstream)))
            })?;
            tx.read(companion, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(2, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.current_branch();
    let feature = runtime.create_branch("feature-scoped-proof").unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(primary, ASPECT_A)?;
            tx.read(primary, &|view| {
                let upstream = view.read_aspect_version(support, ASPECT_A)?;
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(
                    upstream.get(ASPECT_A) + 100,
                    0,
                ))))
            })?;
            tx.mark_dirty(companion, ASPECT_A)?;
            tx.read(companion, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(202, 0))))
            })?;
            Ok(())
        })
        .unwrap();
    runtime.switch_branch(main.clone()).unwrap();

    (runtime, feature, main, support, primary, companion)
}

fn latest_retained_branch_merge_scoped_proof(
    runtime: &SignalRuntime<(), (), (), (), ()>,
    branch_id: SignalBranchId,
) -> ScopedMergeProofPacket {
    runtime
        .replay_for_branch(branch_id)
        .frames
        .iter()
        .rev()
        .find_map(|event| {
            if event.kind != ReplayEventKind::BranchMerged {
                return None;
            }
            event
                .detail
                .as_ref()
                .and_then(ReplayEventDetail::as_scoped_merge_proof)
                .cloned()
        })
        .expect("branch-merge replay history should retain scoped merge proof")
}

#[test]
fn scoped_merge_preview_execution_and_replay_preserve_the_same_proof_packet() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let (mut runtime, feature, main, _support, primary, companion) = build_scoped_proof_runtime();
    let request = [
        SignalSelectedAspectRequestEntry::new(primary, ASPECT_A),
        SignalSelectedAspectRequestEntry::new(companion, ASPECT_B),
    ];

    let planned = runtime
        .merge_raw()
        .from(feature.clone())
        .into(main.clone())
        .selected_aspects(request.clone())
        .plan()
        .expect("scoped aspect merge should plan");
    let plan_report =
        merge_plan_proof_report(planned.plan(), planned.plan().registry_bundle_digest());
    let result = runtime
        .merge_raw()
        .from(feature.clone())
        .into(main.clone())
        .selected_aspects(request.clone())
        .run()
        .expect("scoped aspect merge should execute");
    let result_report = merge_result_proof_report(&result);

    assert_eq!(
        plan_report.scoped_merge_proof,
        result_report.scoped_merge_proof
    );
    assert_eq!(
        plan_report.scoped_merge_proof.declaration_digest(),
        result_report.scoped_merge_proof.declaration_digest()
    );
    assert_eq!(
        plan_report.scoped_merge_proof.admitted_scope_digest(),
        result_report.scoped_merge_proof.admitted_scope_digest()
    );
    assert_eq!(
        plan_report.scoped_merge_proof.skipped_scope_digest(),
        result_report.scoped_merge_proof.skipped_scope_digest()
    );
    assert_eq!(
        plan_report.scoped_merge_proof.no_op_scope_digest(),
        result_report.scoped_merge_proof.no_op_scope_digest()
    );
    assert_eq!(
        plan_report.scoped_merge_proof.breadth_summary(),
        result_report.scoped_merge_proof.breadth_summary()
    );
    assert_eq!(
        latest_retained_branch_merge_scoped_proof(&runtime, main.id),
        result.scoped_merge_proof
    );

    let merged_snapshot = runtime.capture_branch_snapshot(main.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.mark_dirty(companion, ASPECT_A)?;
            tx.read(companion, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(303, 0))))
            })?;
            Ok(())
        })
        .unwrap();
    runtime
        .restore_branch_snapshot(main.clone(), &merged_snapshot)
        .expect("restoring the merged branch snapshot should succeed");
    let restored_snapshot = runtime.capture_branch_snapshot(main.clone()).unwrap();

    let parity = replay_artifact_proof_report(
        ReplayArtifactProofInput {
            proof_schema_version: MERGE_PROOF_SCHEMA_VERSION.to_owned(),
            registry_bundle_digest: Some(result_report.registry_bundle_digest.clone()),
            lowered_strategy_bundle_digest: Some(
                result_report.lowered_strategy_bundle_digest.clone(),
            ),
            merge_plan_digest: Some(plan_report.plan_digest.clone()),
            merge_result_digest: Some(result_report.result_digest.clone()),
            lineage_digest: Some(result_report.lineage_digest.clone()),
            strategy_witness: Some(result.strategy_witness.clone()),
            compatibility_witness: Some(result.compatibility_witness.clone()),
            scoped_merge_proof: Some(result.scoped_merge_proof.clone()),
            branch_state_digest: operational_graph_digest(
                &merged_snapshot.authority_graph().unwrap(),
            ),
        },
        ReplayArtifactProofInput {
            proof_schema_version: MERGE_PROOF_SCHEMA_VERSION.to_owned(),
            registry_bundle_digest: Some(result_report.registry_bundle_digest.clone()),
            lowered_strategy_bundle_digest: Some(
                result_report.lowered_strategy_bundle_digest.clone(),
            ),
            merge_plan_digest: Some(plan_report.plan_digest.clone()),
            merge_result_digest: Some(result_report.result_digest.clone()),
            lineage_digest: Some(result_report.lineage_digest.clone()),
            strategy_witness: Some(result.strategy_witness.clone()),
            compatibility_witness: Some(result.compatibility_witness.clone()),
            scoped_merge_proof: Some(result.scoped_merge_proof.clone()),
            branch_state_digest: operational_graph_digest(
                &restored_snapshot.authority_graph().unwrap(),
            ),
        },
    );
    assert!(parity.parity);
}

#[test]
fn full_branch_scoped_merge_proof_carries_the_normalized_request_truth_forward() {
    let (mut runtime, feature, main, _support, _primary, _companion) = build_scoped_proof_runtime();
    let expected_scope_digest = BranchMergeRequest::full_branch(feature.clone(), main.clone())
        .normalize()
        .expect("full-branch request should normalize")
        .normalized_scope()
        .scope_digest()
        .to_owned();

    let planned = runtime
        .merge_raw()
        .from(feature.clone())
        .into(main.clone())
        .plan()
        .expect("full-branch merge should plan");
    let planned_proof = planned.plan().scoped_merge_proof().clone();
    let result = runtime
        .merge_raw()
        .from(feature)
        .into(main.clone())
        .run()
        .expect("full-branch merge should execute");

    assert_eq!(planned_proof.declaration_digest(), expected_scope_digest);
    assert_eq!(planned_proof.admitted_scope_digest(), expected_scope_digest);
    assert_eq!(
        result.scoped_merge_proof.declaration_digest(),
        expected_scope_digest
    );
    assert_eq!(
        result.scoped_merge_proof.admitted_scope_digest(),
        expected_scope_digest
    );
    assert_eq!(&planned_proof, &result.scoped_merge_proof);
    assert_eq!(planned_proof.skipped_scope_digest(), None);
    assert_eq!(planned_proof.no_op_scope_digest(), None);

    let retained_replay_proof = latest_retained_branch_merge_scoped_proof(&runtime, main.id);
    assert_eq!(retained_replay_proof, result.scoped_merge_proof);
}

#[test]
fn restore_after_merge_preserves_branch_local_scoped_merge_truth_without_widening() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let (mut runtime, feature, main, _support, primary, companion) = build_scoped_proof_runtime();
    let merged = runtime
        .merge_raw()
        .from(feature.clone())
        .into(main.clone())
        .selected_nodes([primary])
        .run()
        .expect("selected-node merge should execute");
    let merged_replay = canonical_digest(&runtime.observe().replay_for_branch(main.id));
    let merged_snapshot = runtime.capture_branch_snapshot(main.clone()).unwrap();

    runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.mark_dirty(companion, ASPECT_A)?;
            tx.read(companion, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(303, 0))))
            })?;
            Ok(())
        })
        .unwrap();
    let drifted_replay = canonical_digest(&runtime.observe().replay_for_branch(main.id));
    assert_ne!(
        merged_replay, drifted_replay,
        "post-merge drift should perturb the branch-local replay digest before restore"
    );

    runtime
        .restore_branch_snapshot(main.clone(), &merged_snapshot)
        .expect("restoring the merged target branch should succeed");
    let restored_snapshot = runtime.capture_branch_snapshot(main.clone()).unwrap();

    let (mut repeated_runtime, repeated_feature, repeated_main, _support, repeated_primary, _) =
        build_scoped_proof_runtime();
    let replayed = repeated_runtime
        .merge_raw()
        .from(repeated_feature)
        .into(repeated_main)
        .selected_nodes([repeated_primary])
        .run()
        .expect("same scoped merge on an equivalent fresh branch basis should stay bounded");

    assert_eq!(
        operational_graph_digest(&merged_snapshot.authority_graph().unwrap()),
        operational_graph_digest(&restored_snapshot.authority_graph().unwrap()),
        "restoring the merged snapshot should recover the same branch-local authority graph"
    );
    assert_eq!(merged.scoped_merge_proof, replayed.scoped_merge_proof);
    assert_eq!(merged.scoped_merge_proof.admitted_nodes(), &[primary]);
    assert_eq!(
        merged.scoped_merge_proof.declaration_digest(),
        replayed.scoped_merge_proof.declaration_digest()
    );
    assert_eq!(
        merged.scoped_merge_proof.admitted_scope_digest(),
        replayed.scoped_merge_proof.admitted_scope_digest()
    );
    assert_eq!(
        merged.scoped_merge_proof.support_closure_nodes(),
        replayed.scoped_merge_proof.support_closure_nodes()
    );
    assert_eq!(
        merged.scoped_merge_proof.breadth_summary(),
        replayed.scoped_merge_proof.breadth_summary()
    );
}
