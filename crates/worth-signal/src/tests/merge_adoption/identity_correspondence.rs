use super::schema_registry_scenarios::cross_identity_merge_schema_registry;
use crate::facade::{
    ArtifactMergeAction, IdentityCorrespondenceBasis, IdentityCorrespondenceStatus,
    IdentityMatcherSelectionBasis, NodeEvaluationResult, SignalGraph, SignalRuntime,
};
use crate::tests::support::version_ab;

#[test]
fn runtime_merge_identity_matcher_changes_source_only_correspondence_behavior() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let graph = SignalGraph::new().with_schema_registry(cross_identity_merge_schema_registry(None));
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let mut runtime_ctx = ();
    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("feature-identity-matcher-correspondence")
        .unwrap();

    runtime.switch_branch(feature.clone()).unwrap();
    let feature_only = runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.cross-identity-merge-owned")
        .expect("known schema")
        .output_identity()
        .build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(feature_only, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(10, 0))
                        .with_output_identity("gear-tooth-correspondence"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    runtime.switch_branch(main.clone()).unwrap();
    let target_only = runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.cross-identity-merge-owned")
        .expect("known schema")
        .output_identity()
        .build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(target_only, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(11, 0))
                        .with_output_identity("gear-tooth-correspondence"),
                ))
            })?;
            Ok(())
        })
        .unwrap();
    let target_node_count_before_exact = runtime.graph().active_node_count();

    let exact_result = runtime
        .merge_raw()
        .from(feature.clone())
        .into(main.clone())
        .identity_matcher_named("signal.identity.exact-node-id")
        .run()
        .unwrap();

    assert_eq!(
        exact_result.selected_identity_matcher_name.as_str(),
        "signal.identity.exact-node-id"
    );
    assert_eq!(
        exact_result.selected_identity_matcher_basis,
        IdentityMatcherSelectionBasis::RequestNamed
    );
    assert_eq!(
        runtime.graph().active_node_count(),
        target_node_count_before_exact + 1
    );
    assert!(exact_result
        .records
        .iter()
        .any(|record| record.source_node == feature_only
            && matches!(record.action, ArtifactMergeAction::IntroducedIntoTarget)));

    let graph = SignalGraph::new().with_schema_registry(cross_identity_merge_schema_registry(None));
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let mut runtime_ctx = ();
    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("feature-identity-matcher-correspondence")
        .unwrap();

    runtime.switch_branch(feature.clone()).unwrap();
    let feature_only = runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.cross-identity-merge-owned")
        .expect("known schema")
        .output_identity()
        .build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(feature_only, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(10, 0))
                        .with_output_identity("gear-tooth-correspondence"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    runtime.switch_branch(main.clone()).unwrap();
    let target_only = runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.cross-identity-merge-owned")
        .expect("known schema")
        .output_identity()
        .build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(target_only, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(11, 0))
                        .with_output_identity("gear-tooth-correspondence"),
                ))
            })?;
            Ok(())
        })
        .unwrap();
    let target_node_count_before_output_identity = runtime.graph().active_node_count();

    let matched_result = runtime
        .merge_raw()
        .from(feature)
        .into(main)
        .identity_matcher_named("signal.identity.output-identity-in-target-journal")
        .run()
        .unwrap();

    assert_eq!(
        matched_result.selected_identity_matcher_name.as_str(),
        "signal.identity.output-identity-in-target-journal"
    );
    assert_eq!(
        matched_result.selected_identity_matcher_basis,
        IdentityMatcherSelectionBasis::RequestNamed
    );
    assert_eq!(
        matched_result
            .identity_correspondence
            .rejected_admissibility_count,
        0
    );
    assert_eq!(
        runtime.graph().active_node_count(),
        target_node_count_before_output_identity
    );
    let matched_record = matched_result
        .records
        .iter()
        .find(|record| record.source_node == feature_only)
        .expect("identity matcher should reconcile the source-only node onto the target node");
    assert_eq!(matched_record.target_node, Some(target_only));
    assert_eq!(
        matched_record.identity_basis,
        Some(IdentityCorrespondenceBasis::OutputIdentityTargetJournal)
    );
    assert_eq!(
        matched_record.identity_status,
        Some(IdentityCorrespondenceStatus::Matched)
    );
    assert_eq!(matched_record.identity_candidate_count, 1);
    assert!(!matches!(
        matched_record.action,
        ArtifactMergeAction::IntroducedIntoTarget
    ));
}

#[test]
fn runtime_merge_output_identity_matcher_fails_closed_without_explicit_admissibility() {
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
    let mut runtime_ctx = ();
    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("feature-identity-matcher-not-admitted")
        .unwrap();

    runtime.switch_branch(feature.clone()).unwrap();
    let feature_only = runtime.graph_mut().node().output_identity().build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(feature_only, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(20, 0))
                        .with_output_identity("not-admitted-correspondence"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    runtime.switch_branch(main.clone()).unwrap();
    let target_only = runtime.graph_mut().node().output_identity().build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(target_only, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(21, 0))
                        .with_output_identity("not-admitted-correspondence"),
                ))
            })?;
            Ok(())
        })
        .unwrap();
    let target_node_count_before = runtime.graph().active_node_count();

    let result = runtime
        .merge_raw()
        .from(feature)
        .into(main)
        .identity_matcher_named("signal.identity.output-identity-in-target-journal")
        .run()
        .unwrap();

    assert_eq!(
        runtime.graph().active_node_count(),
        target_node_count_before + 1
    );
    assert_eq!(
        result.identity_correspondence.rejected_admissibility_count,
        1
    );
    let correspondence = result
        .identity_correspondence
        .records
        .iter()
        .find(|record| record.source_node == feature_only)
        .expect("source node should have identity correspondence record");
    assert_eq!(
        correspondence.status,
        IdentityCorrespondenceStatus::UnmatchedRejectedAdmissibility
    );
    assert!(correspondence.admissibility_rejection.is_some());
    assert!(result
        .records
        .iter()
        .any(|record| record.source_node == feature_only
            && matches!(record.action, ArtifactMergeAction::IntroducedIntoTarget)));
}
