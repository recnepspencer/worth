use crate::facade::mark_dirty;
use crate::facade::{
    ArtifactTransitionKind, DiagnosticsTier, EvaluationContext, LineageRecordKind, NodeContract,
    NodeEvaluationResult, OutputChange, Recipe, ReplayEventKind, SignalGraph, SignalRuntime,
    SignalRuntimePolicy, VersionComparatorPolicy,
};
use crate::tests::support::{version_ab, ASPECT_A, ASPECT_B};

#[test]
fn cross_identity_lineage_and_history_preserve_correspondence_family() {
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
    let projection = runtime
        .define(Recipe {
            family: "projection".into(),
            contract: NodeContract::reads([ASPECT_A])
                .with_produces([ASPECT_B])
                .with_cross_identity_persistent_matching(),
            tier: (),
            comparator: VersionComparatorPolicy::OutputIdentity,
            evaluator: |view: &mut EvaluationContext<'_, ()>| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("lineage-family-artifact")
                        .with_output_change(OutputChange::Refreshed),
                ))
            },
        })
        .unwrap();
    let source = projection.keyed("source");
    let alias = projection.keyed("alias-lineage-family");
    let alias_node = alias.node(&mut runtime);
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            source.evaluate_memoized(tx, "shape-v1")
        })
        .unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            alias.evaluate_cross_identity_with_lineage_mapping(
                tx,
                "source",
                "shape-v1",
                "lineage-map:mesh-42->mesh-77",
            )
        })
        .unwrap();

    let replay_event = runtime
        .graph()
        .replay_events()
        .iter()
        .rev()
        .find(|event| event.kind == ReplayEventKind::TaskApplied && event.node == Some(alias_node))
        .expect("cross-identity replay event");
    assert_eq!(
        replay_event.persistent_correspondence_kind,
        Some(crate::data::reuse::PersistentCorrespondenceKind::LineageBackedMapping)
    );

    let history = runtime
        .observe()
        .execution_history_summary(DiagnosticsTier::Development);
    let node_summary = history
        .nodes
        .iter()
        .find(|node| node.node == alias_node)
        .expect("history summary for alias");
    assert_eq!(
        node_summary.persistent_correspondence_kind,
        Some(crate::data::reuse::PersistentCorrespondenceKind::LineageBackedMapping)
    );

    let lineage = runtime.observe().lineage_chain_for_node(alias_node);
    assert!(lineage.iter().any(|record| matches!(
        &record.kind,
        LineageRecordKind::ArtifactTransition {
            transition: ArtifactTransitionKind::CrossIdentityPersistentReuse {
                correspondence_kind:
                    crate::data::reuse::PersistentCorrespondenceKind::LineageBackedMapping
            },
            ..
        }
    )));
}

#[test]
fn branch_local_cross_identity_rejection_preserves_main_correspondence_and_lineage() {
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
    runtime
        .graph_mut()
        .set_runtime_policy(SignalRuntimePolicy::development());
    let projection = runtime
        .define(Recipe {
            family: "projection".into(),
            contract: NodeContract::reads([ASPECT_A])
                .with_produces([ASPECT_B])
                .with_cross_identity_persistent_matching(),
            tier: (),
            comparator: VersionComparatorPolicy::OutputIdentity,
            evaluator: |view: &mut EvaluationContext<'_, ()>| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("branch-correspondence-artifact")
                        .with_output_change(OutputChange::Refreshed),
                ))
            },
        })
        .unwrap();
    let source = projection.keyed("source");
    let alias = projection.keyed("alias-branch");
    let alias_node = alias.node(&mut runtime);
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            source.evaluate_memoized(tx, "shape-v1")
        })
        .unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            alias.evaluate_cross_identity(tx, "source", "shape-v1", "mesh-branch-001")
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let main_lineage_before = runtime.observe().lineage_chain_for_node(alias_node);
    let main_replay_before = runtime.observe().replay_for_branch(main.id);
    let main_artifact_before = runtime
        .observe()
        .current_lineage_artifact(alias_node)
        .expect("main branch should have a lineage artifact");

    let feature = runtime.create_branch("feature-cross-identity").unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    mark_dirty(runtime.graph_mut(), alias_node, ASPECT_A).unwrap();

    let err = runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            alias.evaluate_cross_identity_with_lineage_mapping(
                tx,
                "source",
                "shape-v1",
                "lineage-map:mesh-branch-001->mesh-branch-777",
            )
        })
        .expect_err("feature branch should reject stale cross-identity evidence");
    assert!(err.to_string().contains("reuse certification failed"));
    assert_eq!(
        runtime.observe().current_lineage_artifact(alias_node),
        Some(main_artifact_before),
        "failed feature-branch admission must not replace the branch-local committed artifact"
    );
    assert_eq!(
        runtime
            .graph()
            .observe()
            .materialize()
            .materialize_trace_summary(alias_node)
            .unwrap()
            .and_then(|summary| summary.reuse_boundary_context)
            .as_ref()
            .and_then(|ctx| ctx.persistent_correspondence()),
        Some(&crate::data::reuse::PersistentCorrespondenceEvidence::HostSuppliedKey(
            "mesh-branch-001".to_string()
        )),
        "feature branch should preserve the last committed certified correspondence after rejection"
    );
    let feature_replay_after = runtime.observe().replay_for_branch(feature.id);
    assert!(
        feature_replay_after
            .frames
            .iter()
            .all(|frame| frame.branch_id == feature.id),
        "feature replay must remain branch-local after stale correspondence rejection"
    );

    runtime.switch_branch(main.clone()).unwrap();
    assert_eq!(
        runtime.observe().lineage_chain_for_node(alias_node),
        main_lineage_before,
        "feature-branch rejection must not contaminate main-branch lineage"
    );
    let main_replay_after = runtime.observe().replay_for_branch(main.id);
    assert_eq!(
        main_replay_after
            .frames
            .iter()
            .filter(|frame| frame.kind == ReplayEventKind::TaskApplied)
            .count(),
        main_replay_before
            .frames
            .iter()
            .filter(|frame| frame.kind == ReplayEventKind::TaskApplied)
            .count(),
        "feature-branch rejection must not append task-apply replay on main"
    );
    assert_eq!(
        main_replay_after
            .frames
            .iter()
            .filter(|frame| frame.kind == ReplayEventKind::TransactionCommitted)
            .count(),
        main_replay_before
            .frames
            .iter()
            .filter(|frame| frame.kind == ReplayEventKind::TransactionCommitted)
            .count(),
        "feature-branch rejection must not append committed execution replay on main"
    );
    assert_eq!(
        runtime
            .graph()
            .observe()
            .materialize()
            .materialize_trace_summary(alias_node)
            .unwrap()
            .and_then(|summary| summary.reuse_boundary_context)
            .as_ref()
            .and_then(|ctx| ctx.persistent_correspondence()),
        Some(
            &crate::data::reuse::PersistentCorrespondenceEvidence::HostSuppliedKey(
                "mesh-branch-001".to_string()
            )
        ),
        "main branch should retain its original certified correspondence"
    );
}

#[test]
fn branch_local_cross_identity_history_retains_committed_family_after_rejected_evolution() {
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
    let projection = runtime
        .define(Recipe {
            family: "projection".into(),
            contract: NodeContract::reads([ASPECT_A])
                .with_produces([ASPECT_B])
                .with_cross_identity_persistent_matching(),
            tier: (),
            comparator: VersionComparatorPolicy::OutputIdentity,
            evaluator: |view: &mut EvaluationContext<'_, ()>| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("branch-history-artifact")
                        .with_output_change(OutputChange::Refreshed),
                ))
            },
        })
        .unwrap();
    let source = projection.keyed("source");
    let alias = projection.keyed("alias-branch-history");
    let alias_node = alias.node(&mut runtime);
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            source.evaluate_memoized(tx, "shape-v1")
        })
        .unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            alias.evaluate_cross_identity_with_contract_basis(
                tx,
                "source",
                "shape-v1",
                "contract:mesh-family:v2",
            )
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("feature-cross-identity-history")
        .unwrap();

    runtime.switch_branch(feature.clone()).unwrap();
    mark_dirty(runtime.graph_mut(), alias_node, ASPECT_A).unwrap();
    let _ = runtime.transaction(request_execution, &mut runtime_ctx, |tx| {
        alias.evaluate_cross_identity_with_region_identity(tx, "source", "shape-v1", "region:wing")
    });

    let feature_history = runtime
        .observe()
        .execution_history_summary(DiagnosticsTier::Development);
    let feature_summary = feature_history
        .nodes
        .iter()
        .find(|node| node.node == alias_node)
        .expect("feature summary for alias");
    assert_eq!(
        feature_summary.reuse_origin,
        Some(crate::data::reuse::ReuseOrigin::CrossIdentityPersistentReuse),
        "rejected evolution must not erase the last committed advanced reuse origin"
    );
    assert_eq!(
        feature_summary.persistent_correspondence_kind,
        Some(crate::data::reuse::PersistentCorrespondenceKind::ContractDeclaredBasis),
        "history should keep the committed correspondence family after rejected branch-local evolution"
    );

    runtime.switch_branch(main.clone()).unwrap();
    let main_history = runtime
        .observe()
        .execution_history_summary(DiagnosticsTier::Development);
    let main_summary = main_history
        .nodes
        .iter()
        .find(|node| node.node == alias_node)
        .expect("main summary for alias");
    assert_eq!(
        main_summary.persistent_correspondence_kind,
        Some(crate::data::reuse::PersistentCorrespondenceKind::ContractDeclaredBasis),
        "main branch should still report the committed correspondence family"
    );
}
