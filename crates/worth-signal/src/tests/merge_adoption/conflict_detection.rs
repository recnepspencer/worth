use crate::data::error::SignalError;
use crate::facade::{
    ArtifactAuthorityClass, ArtifactMergeAuthority, BranchMergeConflictKind, BranchMergeDivergence,
    BranchMergeFailureEvidence, BranchMergeFailureKind, ConflictMergePolicy, DependencyEdge,
    MergeAdoptability, NodeEvaluationResult, ReplayEventKind, SignalGraph, SignalRuntime,
};
use crate::logic::transaction::{BranchMergeResolutionRequirement, ConflictResolutionStrategy};
use crate::tests::support::{version_ab, GraphDependencyBatchExt, ASPECT_A};

#[test]
fn merge_branch_self_merge_surfaces_typed_failure() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let branch = runtime.observe().current_branch();

    let err = runtime
        .merge_branch_raw(branch.clone(), branch)
        .unwrap_err();
    match err {
        SignalError::BranchMergeFailed { kind, .. } => {
            assert_eq!(kind, BranchMergeFailureKind::SelfMergeRejected);
        }
        other => panic!("expected typed branch merge failure, got {other:?}"),
    }
}

#[test]
fn merge_branch_divergent_shared_node_requires_typed_conflict_surface() {
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
                    NodeEvaluationResult::from_version(version_ab(41, 0))
                        .with_output_identity("base-shared"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime.create_branch("feature-conflict").unwrap();

    runtime.switch_branch(feature.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(shared, ASPECT_A)?;
            tx.read(shared, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(42, 0))
                        .with_output_identity("feature-shared"),
                ))
            })?;
            Ok(())
        })
        .unwrap();
    {
        let mut graph = runtime.graph_mut();
        let mut entry = graph.get_entry_mut(shared).unwrap();
        let mut runtime_artifact = entry
            .get_runtime_artifact_state()
            .cloned()
            .expect("feature shared node should have runtime artifact state");
        runtime_artifact.warm_mut().merge_authority = ArtifactMergeAuthority {
            authority_class: ArtifactAuthorityClass::BranchLocalSpeculative,
            adoptability: MergeAdoptability::NonAdoptableBranchLocal,
        };
        entry.set_runtime_artifact_state(Some(runtime_artifact));
    }

    runtime.switch_branch(main.clone()).unwrap();
    let main_only = runtime.graph_mut().node().output_identity().build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(main_only, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(43, 0))
                        .with_output_identity("main-only"),
                ))
            })?;
            tx.mark_dirty(shared, ASPECT_A)?;
            tx.read(shared, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(44, 0))
                        .with_output_identity("main-shared"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let err = runtime.merge_branch_raw(feature, main).unwrap_err();
    match err {
        SignalError::BranchMergeFailed { kind, evidence, .. } => {
            assert_eq!(
                kind,
                BranchMergeFailureKind::DivergenceRequiresConflictResolution
            );
            let evidence = match *evidence.expect("conflict evidence should be present") {
                BranchMergeFailureEvidence::Conflict(evidence) => evidence,
                other => panic!("expected conflict evidence, got {other:?}"),
            };
            assert_eq!(
                evidence.divergence,
                BranchMergeDivergence::SharedStateConflict
            );
            assert_eq!(
                evidence.reconciliation_policy.conflict,
                ConflictMergePolicy::ResolveSourceStateWhenStructureMatches
            );
            assert_eq!(evidence.summary.total_conflict_count, 1);
            assert_eq!(evidence.summary.comparable_mismatch_count, 1);
            assert_eq!(
                evidence.summary.primary_conflict_kind,
                Some(BranchMergeConflictKind::MergeAuthorityMismatch)
            );
            assert!(evidence
                .summary
                .required_resolution
                .contains(&BranchMergeResolutionRequirement::ReconcileComparableState));
            assert!(evidence
                .summary
                .required_resolution
                .contains(&BranchMergeResolutionRequirement::ReconcileMergeAuthority));
            assert!(evidence
                .summary
                .required_resolution
                .contains(&BranchMergeResolutionRequirement::ReconcileRuntimeArtifactState));
            assert_eq!(evidence.resolution_plan.records.len(), 1);
            assert_eq!(evidence.resolution_plan.records[0].source_node, shared);
            assert!(evidence.resolution_plan.records[0]
                .required_resolution
                .contains(&BranchMergeResolutionRequirement::ReconcileRuntimeArtifactState));
            assert!(evidence.resolution_plan.records[0]
                .supported_strategies
                .contains(&ConflictResolutionStrategy::AdoptSourceRuntimeArtifactState));
            assert!(evidence.resolution_plan.records[0]
                .supported_strategies
                .contains(&ConflictResolutionStrategy::PreserveTargetRuntimeArtifactState));
            let failure = runtime
                .observe()
                .latest_failure_diagnostics()
                .expect("failed merge should record failure diagnostics");
            assert!(
                failure
                    .message
                    .contains("primary=Some(MergeAuthorityMismatch)"),
                "failure diagnostics should surface the primary conflict class"
            );
            assert!(
                failure.message.contains("ReconcileRuntimeArtifactState"),
                "failure diagnostics should surface required merge resolution"
            );
            assert!(
                runtime.graph().replay_events().iter().any(|event| {
                    event.kind == ReplayEventKind::FailureRecorded
                        && event
                            .detail
                            .as_ref()
                            .and_then(|detail| detail.as_message())
                            .map(|detail| detail.contains("ReconcileRuntimeArtifactState"))
                            .unwrap_or(false)
                }),
                "failed merge should emit a failure replay detail with required resolution"
            );
            assert!(
                !runtime
                    .graph()
                    .replay_events()
                    .iter()
                    .any(|event| { event.kind == ReplayEventKind::BranchMerged }),
                "failed merge must not emit a false branch-merged replay boundary"
            );
            assert_eq!(evidence.records.len(), 1);
            assert_eq!(evidence.records[0].source_node, shared);
            assert!(evidence.records[0]
                .conflict_kinds
                .contains(&BranchMergeConflictKind::ComparableMismatch));
            assert!(evidence.records[0]
                .conflict_kinds
                .contains(&BranchMergeConflictKind::RuntimeArtifactMismatch));
            assert_eq!(
                evidence.records[0]
                    .source_comparable
                    .as_ref()
                    .and_then(|comparable| comparable.output_identity.as_ref())
                    .map(|identity| identity.as_str()),
                Some("feature-shared")
            );
            assert_eq!(
                evidence.records[0]
                    .target_comparable
                    .as_ref()
                    .and_then(|comparable| comparable.output_identity.as_ref())
                    .map(|identity| identity.as_str()),
                Some("main-shared")
            );
        }
        other => panic!("expected typed divergence failure, got {other:?}"),
    }
}

#[test]
fn merge_branch_dependency_topology_conflict_surfaces_structural_requirement() {
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
    let source_a = runtime.graph_mut().node().output_identity().build();
    let source_b = runtime.graph_mut().node().output_identity().build();
    runtime
        .graph_mut()
        .append_dependency(shared, source_a, ASPECT_A)
        .unwrap();

    let mut runtime_ctx = ();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(shared, &|view| {
                let result = match view.node() {
                    node if node == source_a => view.finish(
                        NodeEvaluationResult::from_version(version_ab(91, 0))
                            .with_output_identity("topology-source-a"),
                    ),
                    node if node == source_b => view.finish(
                        NodeEvaluationResult::from_version(version_ab(92, 0))
                            .with_output_identity("topology-source-b"),
                    ),
                    _ => view.finish(
                        NodeEvaluationResult::from_version(version_ab(93, 0))
                            .with_output_identity("topology-shared"),
                    ),
                };
                Ok(result)
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime.create_branch("feature-topology-conflict").unwrap();

    runtime.switch_branch(feature.clone()).unwrap();
    runtime
        .graph_mut()
        .set_dependencies(shared, [DependencyEdge::new(source_b, ASPECT_A)])
        .unwrap();

    runtime.switch_branch(main.clone()).unwrap();
    runtime
        .graph_mut()
        .set_dependencies(
            shared,
            [
                DependencyEdge::new(source_a, ASPECT_A),
                DependencyEdge::new(source_b, ASPECT_A),
            ],
        )
        .unwrap();

    let err = runtime.merge_branch_raw(feature, main).unwrap_err();
    match err {
        SignalError::BranchMergeFailed { kind, evidence, .. } => {
            assert_eq!(
                kind,
                BranchMergeFailureKind::DivergenceRequiresConflictResolution
            );
            let evidence = match *evidence.expect("topology conflict evidence should be present") {
                BranchMergeFailureEvidence::Conflict(evidence) => evidence,
                other => panic!("expected conflict evidence, got {other:?}"),
            };
            assert_eq!(
                evidence.divergence,
                BranchMergeDivergence::SharedStateConflict
            );
            assert_eq!(
                evidence.summary.primary_conflict_kind,
                Some(BranchMergeConflictKind::DependencyTopologyMismatch)
            );
            assert!(evidence
                .summary
                .required_resolution
                .contains(&BranchMergeResolutionRequirement::ReconcileDependencyTopology));
            assert_eq!(evidence.records.len(), 1);
            assert!(evidence.records[0]
                .conflict_kinds
                .contains(&BranchMergeConflictKind::DependencyTopologyMismatch));
        }
        other => panic!("expected topology conflict failure, got {other:?}"),
    }
}
