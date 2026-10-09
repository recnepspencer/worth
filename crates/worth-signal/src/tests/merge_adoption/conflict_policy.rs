use super::schema_registry_scenarios::merge_schema_registry;
use crate::data::error::SignalError;
use crate::facade::{
    BranchMergeConflictKind, BranchMergeFailureEvidence, BranchMergeFailureKind, BranchMergeKind,
    ConflictMergePolicy, ConflictPolicySelectionBasis, NodeEvaluationResult, SignalGraph,
    SignalRuntime,
};
use crate::tests::support::{version_ab, GraphDependencyBatchExt, ASPECT_A};

#[test]
fn runtime_merge_request_named_conflict_policy_changes_merge_outcome() {
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
                    NodeEvaluationResult::from_version(version_ab(201, 0))
                        .with_output_identity("base-request-policy"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let reject_feature = runtime
        .create_branch("feature-request-conflict-policy-reject")
        .unwrap();

    runtime.switch_branch(reject_feature.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(shared, ASPECT_A)?;
            tx.read(shared, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(202, 0))
                        .with_output_identity("feature-request-policy"),
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
                    NodeEvaluationResult::from_version(version_ab(203, 0))
                        .with_output_identity("main-request-policy"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let reject_err = runtime
        .merge_raw()
        .from(reject_feature.clone())
        .into(main.clone())
        .conflict_policy_named("signal.conflict.reject-shared-state")
        .run()
        .expect_err("reject policy should fail closed on shared-state conflict");

    match reject_err {
        SignalError::BranchMergeFailed { kind, evidence, .. } => {
            assert_eq!(
                kind,
                BranchMergeFailureKind::DivergenceRequiresConflictResolution
            );
            let evidence =
                match *evidence.expect("reject policy failure should expose conflict evidence") {
                    BranchMergeFailureEvidence::Conflict(evidence) => evidence,
                    other => panic!("expected conflict evidence, got {other:?}"),
                };
            assert_eq!(
                evidence.reconciliation_policy.conflict,
                ConflictMergePolicy::RejectSharedStateConflict
            );
            assert_eq!(
                evidence.summary.primary_conflict_kind,
                Some(BranchMergeConflictKind::RuntimeArtifactMismatch)
            );
        }
        other => panic!("expected typed merge failure, got {other:?}"),
    }

    let resolve_feature = runtime
        .create_branch("feature-request-conflict-policy-resolve")
        .unwrap();
    runtime.switch_branch(resolve_feature.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(shared, ASPECT_A)?;
            tx.read(shared, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(204, 0))
                        .with_output_identity("feature-request-policy-resolve"),
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
                    NodeEvaluationResult::from_version(version_ab(205, 0))
                        .with_output_identity("main-request-policy-resolve"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let resolve_result = runtime
        .merge_raw()
        .from(resolve_feature)
        .into(main)
        .conflict_policy_named("signal.conflict.resolve-source-when-structure-matches")
        .run()
        .expect("resolve policy should permit bounded auto-resolution");

    assert_eq!(
        resolve_result.selected_conflict_policy_name.as_str(),
        "signal.conflict.resolve-source-when-structure-matches"
    );
    assert_eq!(
        resolve_result.selected_conflict_policy_basis,
        ConflictPolicySelectionBasis::RequestNamed
    );
    assert_eq!(
        resolve_result.reconciliation_policy.conflict,
        ConflictMergePolicy::ResolveSourceStateWhenStructureMatches
    );
    assert_eq!(resolve_result.merge_kind, BranchMergeKind::ConflictResolved);
}

#[test]
fn runtime_merge_request_named_conflict_policy_selects_registered_descriptor() {
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
    let shared = runtime.graph_mut().node().build();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(shared, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(1, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("feature-request-named-conflict-policy")
        .unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    let feature_only = runtime.graph_mut().node().build();
    runtime
        .graph_mut()
        .append_dependency(feature_only, shared, ASPECT_A)
        .unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(feature_only, &|view| {
                let upstream = view.read_aspect_version(shared, ASPECT_A)?;
                Ok(view.finish(NodeEvaluationResult::from_version(upstream)))
            })?;
            Ok(())
        })
        .unwrap();
    runtime.switch_branch(main.clone()).unwrap();

    let planned = runtime
        .merge_raw()
        .from(feature.clone())
        .into(main.clone())
        .conflict_policy_named("signal.conflict.reject-shared-state")
        .plan()
        .unwrap();

    assert_eq!(
        planned.plan().selected_conflict_policy_name().as_str(),
        "signal.conflict.reject-shared-state"
    );
    assert_eq!(
        planned.plan().selected_conflict_policy_basis(),
        ConflictPolicySelectionBasis::RequestNamed
    );
    assert_eq!(
        planned.plan().reconciliation_policy().conflict,
        ConflictMergePolicy::RejectSharedStateConflict
    );
}

#[test]
fn runtime_merge_uses_schema_default_conflict_policy_when_request_is_silent() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let graph = SignalGraph::new().with_schema_registry(merge_schema_registry(
        "signal.merge.rebase-source-onto-target",
        Some("signal.conflict.reject-shared-state"),
        None,
    ));
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let shared = runtime.graph_mut().node().build();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(shared, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(1, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("feature-schema-default-conflict-policy")
        .unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    let feature_only = runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.merge-owned")
        .expect("known schema")
        .build();
    runtime
        .graph_mut()
        .append_dependency(feature_only, shared, ASPECT_A)
        .unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(feature_only, &|view| {
                let upstream = view.read_aspect_version(shared, ASPECT_A)?;
                Ok(view.finish(NodeEvaluationResult::from_version(upstream)))
            })?;
            Ok(())
        })
        .unwrap();
    runtime.switch_branch(main.clone()).unwrap();

    let planned = runtime
        .merge_raw()
        .from(feature.clone())
        .into(main.clone())
        .plan()
        .unwrap();

    assert_eq!(
        planned.plan().selected_conflict_policy_name().as_str(),
        "signal.conflict.reject-shared-state"
    );
    assert_eq!(
        planned.plan().selected_conflict_policy_basis(),
        ConflictPolicySelectionBasis::SchemaDefault
    );
    assert_eq!(
        planned.plan().reconciliation_policy().conflict,
        ConflictMergePolicy::RejectSharedStateConflict
    );
}

#[test]
fn runtime_merge_node_conflict_policy_override_precedes_schema_default() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let graph = SignalGraph::new().with_schema_registry(merge_schema_registry(
        "signal.merge.rebase-source-onto-target",
        Some("signal.conflict.resolve-source-when-structure-matches"),
        None,
    ));
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let shared = runtime.graph_mut().node().build();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(shared, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(1, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("feature-node-override-conflict-policy")
        .unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    let feature_only = runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.merge-owned")
        .expect("known schema")
        .conflict_policy_name("signal.conflict.reject-shared-state")
        .build();
    runtime
        .graph_mut()
        .append_dependency(feature_only, shared, ASPECT_A)
        .unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(feature_only, &|view| {
                let upstream = view.read_aspect_version(shared, ASPECT_A)?;
                Ok(view.finish(NodeEvaluationResult::from_version(upstream)))
            })?;
            Ok(())
        })
        .unwrap();
    runtime.switch_branch(main.clone()).unwrap();

    let planned = runtime
        .merge_raw()
        .from(feature.clone())
        .into(main.clone())
        .plan()
        .unwrap();

    assert_eq!(
        planned.plan().selected_conflict_policy_name().as_str(),
        "signal.conflict.reject-shared-state"
    );
    assert_eq!(
        planned.plan().selected_conflict_policy_basis(),
        ConflictPolicySelectionBasis::NodeOverride
    );
    assert_eq!(
        planned.plan().reconciliation_policy().conflict,
        ConflictMergePolicy::RejectSharedStateConflict
    );
}
