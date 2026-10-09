use crate::facade::{
    ArtifactMergeAction, BranchMergeDivergence, BranchMergeKind, NodeEvaluationResult, SignalGraph,
    SignalRuntime,
};
use crate::tests::support::{version_ab, ASPECT_A};

#[test]
fn merge_branch_target_advanced_without_shared_conflict_surfaces_applied_divergence() {
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
                    NodeEvaluationResult::from_version(version_ab(51, 0))
                        .with_output_identity("base-shared"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime.create_branch("feature-applied").unwrap();

    runtime.switch_branch(feature.clone()).unwrap();
    let feature_only = runtime.graph_mut().node().output_identity().build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(shared, ASPECT_A)?;
            tx.read(shared, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(51, 0))
                        .with_output_identity("base-shared"),
                ))
            })?;
            tx.read(feature_only, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(52, 0))
                        .with_output_identity("feature-only"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    runtime.switch_branch(main.clone()).unwrap();
    let main_only = runtime.graph_mut().node().output_identity().build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(shared, ASPECT_A)?;
            tx.read(shared, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(51, 0))
                        .with_output_identity("base-shared"),
                ))
            })?;
            tx.read(main_only, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(53, 0))
                        .with_output_identity("main-only"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let result = runtime.merge_branch_raw(feature, main).unwrap();
    assert_eq!(result.merge_kind, BranchMergeKind::Applied);
    assert_eq!(result.divergence, BranchMergeDivergence::TargetAdvanced);
    assert!(result.counters.final_candidate_breadth > 0);
    assert!(result
        .records
        .iter()
        .any(|record| record.action == ArtifactMergeAction::IntroducedIntoTarget));
}

#[test]
fn merge_branch_unrelated_target_only_pending_work_does_not_degrade_fast_forward() {
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
                    NodeEvaluationResult::from_version(version_ab(61, 0))
                        .with_output_identity("base-shared"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime.create_branch("feature-fast-forward").unwrap();

    runtime.switch_branch(feature.clone()).unwrap();
    let feature_only = runtime.graph_mut().node().output_identity().build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(feature_only, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(62, 0))
                        .with_output_identity("feature-only"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    runtime.switch_branch(main.clone()).unwrap();
    let unrelated_main_only = runtime.graph_mut().node().output_identity().build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(unrelated_main_only, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(63, 0))
                        .with_output_identity("main-unrelated"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let result = runtime.merge_branch_raw(feature, main).unwrap();
    assert_eq!(result.merge_kind, BranchMergeKind::FastForward);
    assert_eq!(result.divergence, BranchMergeDivergence::None);
    assert!(result
        .records
        .iter()
        .any(|record| record.action == ArtifactMergeAction::IntroducedIntoTarget));
}
