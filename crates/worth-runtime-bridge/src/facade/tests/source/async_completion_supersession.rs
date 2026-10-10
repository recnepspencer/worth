use super::support::{
    activation_ready_for_branch_head, activation_ready_for_snapshot,
    denied_preview_subscription_backed_completion_after_discard,
    denied_request_response_completion_with_displacing_identity,
    denied_subscription_backed_completion_with_displacing_identity,
    preview_active_subscription_with_basis,
};
use super::*;
use crate::facade::{
    BridgeAsyncCompletionSupersessionClass, BridgeAsyncCompletionSupersessionClassificationRequest,
    BridgeAsyncRequestSubscriptionInstance, BridgeAsyncRequestTruthViewBasis,
};
use worth_signal::facade::NodeId;

#[test]
fn same_basis_replacement_classifies_signal_generation_supersession() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let truth_basis = BridgeAsyncRequestTruthViewBasis::authoritative(
        crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    );
    let (denied, displacing_request) = denied_request_response_completion_with_displacing_identity(
        &runtime,
        NodeId::new(141, 0),
        truth_basis.clone(),
        truth_basis.clone(),
    );

    let classified = runtime
        .classify_async_completion_supersession(
            BridgeAsyncCompletionSupersessionClassificationRequest::request_response(
                &denied,
                truth_basis,
            )
            .with_displacing_request_identity(&displacing_request),
        )
        .expect("same-basis replacement should classify");

    assert_eq!(
        classified.supersession_class(),
        BridgeAsyncCompletionSupersessionClass::SignalGenerationSuperseded
    );
    assert_eq!(
        classified.evidence().displacing_request_identity(),
        Some(displacing_request.request_identity().as_str())
    );
    assert_eq!(
        classified.counters().signal_generation_supersession_count(),
        1
    );
}

#[test]
fn branch_switch_classifies_branch_drift() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let original_truth_basis = BridgeAsyncRequestTruthViewBasis::authoritative(
        crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    );
    let current_truth_basis = BridgeAsyncRequestTruthViewBasis::authoritative(
        crate::truth_identity_fixtures::truth_branch_fixture("truth-feature"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-b"),
    );
    let (denied, displacing_request) = denied_request_response_completion_with_displacing_identity(
        &runtime,
        NodeId::new(142, 0),
        original_truth_basis,
        current_truth_basis.clone(),
    );

    let classified = runtime
        .classify_async_completion_supersession(
            BridgeAsyncCompletionSupersessionClassificationRequest::request_response(
                &denied,
                current_truth_basis,
            )
            .with_displacing_request_identity(&displacing_request),
        )
        .expect("branch switch should classify");

    assert_eq!(
        classified.supersession_class(),
        BridgeAsyncCompletionSupersessionClass::BranchDrifted
    );
    assert_eq!(classified.counters().branch_drift_supersession_count(), 1);
}

#[test]
fn same_branch_new_commit_classifies_truth_basis_superseded() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let original_truth_basis = BridgeAsyncRequestTruthViewBasis::authoritative(
        crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    );
    let current_truth_basis = BridgeAsyncRequestTruthViewBasis::authoritative(
        crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-b"),
    );
    let (denied, displacing_request) = denied_request_response_completion_with_displacing_identity(
        &runtime,
        NodeId::new(143, 0),
        original_truth_basis,
        current_truth_basis.clone(),
    );

    let classified = runtime
        .classify_async_completion_supersession(
            BridgeAsyncCompletionSupersessionClassificationRequest::request_response(
                &denied,
                current_truth_basis,
            )
            .with_displacing_request_identity(&displacing_request),
        )
        .expect("same-branch commit drift should classify");

    assert_eq!(
        classified.supersession_class(),
        BridgeAsyncCompletionSupersessionClass::TruthBasisSuperseded
    );
    assert_eq!(classified.counters().truth_basis_supersession_count(), 1);
}

#[test]
fn subscription_instance_replacement_classifies_subscription_instance_superseded() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let truth_basis = BridgeAsyncRequestTruthViewBasis::authoritative(
        crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    );
    let original_subscription_instance =
        BridgeAsyncRequestSubscriptionInstance::authoritative(&activation_ready_for_snapshot(
            &runtime,
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            resource_request,
        ));
    let current_subscription_instance =
        BridgeAsyncRequestSubscriptionInstance::authoritative(&activation_ready_for_branch_head(
            &runtime,
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            resource_request,
        ));
    let (denied, displacing_request) =
        denied_subscription_backed_completion_with_displacing_identity(
            &runtime,
            NodeId::new(144, 0),
            truth_basis.clone(),
            original_subscription_instance,
            truth_basis.clone(),
            current_subscription_instance.clone(),
        );

    let classified = runtime
        .classify_async_completion_supersession(
            BridgeAsyncCompletionSupersessionClassificationRequest::subscription_backed(
                &denied,
                truth_basis,
                current_subscription_instance,
            )
            .with_displacing_request_identity(&displacing_request),
        )
        .expect("subscription-backed replacement should classify");

    assert_eq!(
        classified.supersession_class(),
        BridgeAsyncCompletionSupersessionClass::SubscriptionInstanceSuperseded
    );
    assert_eq!(
        classified
            .counters()
            .subscription_instance_supersession_count(),
        1
    );
}

#[test]
fn preview_discard_classifies_preview_discarded() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let (denied, current_truth_basis, current_subscription_instance) =
        denied_preview_subscription_backed_completion_after_discard(
            &runtime,
            NodeId::new(145, 0),
            "preview-discard",
            resource_request,
        );

    let classified = runtime
        .classify_async_completion_supersession(
            BridgeAsyncCompletionSupersessionClassificationRequest::subscription_backed(
                &denied,
                current_truth_basis,
                current_subscription_instance,
            )
            .mark_preview_discarded(),
        )
        .expect("preview discard should classify");

    assert_eq!(
        classified.supersession_class(),
        BridgeAsyncCompletionSupersessionClass::PreviewDiscarded
    );
    assert_eq!(
        classified.counters().preview_discarded_supersession_count(),
        1
    );
}

#[test]
fn preview_basis_evolution_classifies_preview_basis_drift() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let truth_branch = crate::truth_identity_fixtures::truth_branch_fixture("truth-preview");
    let original_preview_active = preview_active_subscription_with_basis(
        &runtime,
        "preview-a",
        truth_branch.clone(),
        crate::truth_identity_fixtures::truth_snapshot_fixture("preview-snapshot-a"),
        resource_request,
    );
    let current_preview_active = preview_active_subscription_with_basis(
        &runtime,
        "preview-b",
        truth_branch,
        crate::truth_identity_fixtures::truth_snapshot_fixture("preview-snapshot-b"),
        resource_request,
    );
    let original_truth_basis = BridgeAsyncRequestTruthViewBasis::preview(&original_preview_active);
    let original_subscription_instance =
        BridgeAsyncRequestSubscriptionInstance::preview(&original_preview_active);
    let current_truth_basis = BridgeAsyncRequestTruthViewBasis::preview(&current_preview_active);
    let current_subscription_instance =
        BridgeAsyncRequestSubscriptionInstance::preview(&current_preview_active);
    let (denied, displacing_request) =
        denied_subscription_backed_completion_with_displacing_identity(
            &runtime,
            NodeId::new(146, 0),
            original_truth_basis,
            original_subscription_instance,
            current_truth_basis.clone(),
            current_subscription_instance.clone(),
        );

    let classified = runtime
        .classify_async_completion_supersession(
            BridgeAsyncCompletionSupersessionClassificationRequest::subscription_backed(
                &denied,
                current_truth_basis.clone(),
                current_subscription_instance.clone(),
            )
            .with_displacing_request_identity(&displacing_request),
        )
        .expect("preview basis evolution should classify");

    assert_eq!(
        classified.supersession_class(),
        BridgeAsyncCompletionSupersessionClass::PreviewBasisDrifted
    );
    assert_eq!(
        classified.evidence().original_truth_view_basis_digest(),
        denied
            .request_identity()
            .basis_binding()
            .truth_view_basis()
            .digest()
    );
    assert_eq!(
        classified.evidence().current_truth_view_basis_digest(),
        current_truth_basis.digest()
    );
    assert_eq!(
        classified
            .evidence()
            .original_subscription_instance_digest(),
        denied
            .request_identity()
            .subscription_instance()
            .map(BridgeAsyncRequestSubscriptionInstance::digest)
    );
    assert_eq!(
        classified.evidence().current_subscription_instance_digest(),
        Some(current_subscription_instance.digest())
    );
    assert_eq!(
        classified.evidence().displacing_request_identity(),
        Some(displacing_request.request_identity().as_str())
    );
    assert_eq!(
        classified.receipt().supersession_identity(),
        classified.evidence().supersession_identity().as_str()
    );
    assert_eq!(
        classified
            .counters()
            .preview_basis_drift_supersession_count(),
        1
    );
}
