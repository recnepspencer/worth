use super::support::{
    activation_ready_for_branch_head, activation_ready_for_snapshot,
    admit_request_response_identity, admit_subscription_backed_identity,
    preview_active_subscription,
};
use super::*;
use crate::facade::{
    BridgeAsyncRequestAdmissionRequest, BridgeAsyncRequestIdentityRejectionKind,
    BridgeAsyncRequestSubscriptionInstance, BridgeAsyncRequestTruthViewBasis,
};
use worth_signal::facade::NodeId;

#[test]
fn equivalent_request_response_requests_admit_identically_across_equivalent_runtimes() {
    let first_runtime = runtime(BridgeRuntimePolicy::development());
    let second_runtime = runtime(BridgeRuntimePolicy::development());
    let first = admit_request_response_identity(
        &first_runtime,
        NodeId::new(11, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
    );
    let second = admit_request_response_identity(
        &second_runtime,
        NodeId::new(88, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
    );

    assert_eq!(first.digest(), second.digest());
    assert_eq!(first.request_identity(), second.request_identity());
    assert_eq!(
        first.in_flight_identity().digest(),
        second.in_flight_identity().digest()
    );
    assert_eq!(first.counters().async_request_identity_count(), 1);
    assert_eq!(first.counters().request_response_request_count(), 1);
}

#[test]
fn repeated_request_response_admissions_on_one_runtime_change_request_identity() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let first = admit_request_response_identity(
        &runtime,
        NodeId::new(23, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
    );
    let second = admit_request_response_identity(
        &runtime,
        NodeId::new(23, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
    );

    assert_ne!(first.digest(), second.digest());
    assert_ne!(first.request_identity(), second.request_identity());
    assert_ne!(first.request_handle(), second.request_handle());
    assert_ne!(
        first.request_handle().generation(),
        second.request_handle().generation()
    );
    assert_ne!(
        first.in_flight_identity().digest(),
        second.in_flight_identity().digest()
    );
}

#[test]
fn truth_basis_drift_changes_request_response_identity() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let left = admit_request_response_identity(
        &runtime,
        NodeId::new(19, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
    );
    let right = admit_request_response_identity(
        &runtime,
        NodeId::new(19, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-b"),
        ),
    );

    assert_ne!(left.digest(), right.digest());
    assert_ne!(left.request_identity(), right.request_identity());
}

#[test]
fn equivalent_subscription_backed_requests_require_and_preserve_subscription_instance_identity() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let first_runtime = runtime(BridgeRuntimePolicy::development());
    let second_runtime = runtime(BridgeRuntimePolicy::development());
    let first_ready = activation_ready_for_snapshot(
        &first_runtime,
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        resource_request,
    );
    let second_ready = activation_ready_for_snapshot(
        &second_runtime,
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        resource_request,
    );
    let first = admit_subscription_backed_identity(
        &first_runtime,
        NodeId::new(41, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
        BridgeAsyncRequestSubscriptionInstance::authoritative(&first_ready),
    )
    .expect("subscription-backed request should admit");
    let second = admit_subscription_backed_identity(
        &second_runtime,
        NodeId::new(105, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
        BridgeAsyncRequestSubscriptionInstance::authoritative(&second_ready),
    )
    .expect("equivalent subscription-backed request should admit");

    assert_eq!(first.digest(), second.digest());
    assert_eq!(
        first
            .subscription_instance()
            .expect("subscription-backed request should retain instance")
            .subscription_instance_identity(),
        second
            .subscription_instance()
            .expect("equivalent subscription-backed request should retain instance")
            .subscription_instance_identity()
    );
    assert_eq!(first.counters().subscription_backed_request_count(), 1);
    assert_eq!(first.counters().signal_async_request_admission_count(), 1);
}

#[test]
fn subscription_instance_drift_changes_subscription_backed_identity() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let first_ready = activation_ready_for_snapshot(
        &runtime,
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        resource_request,
    );
    let second_ready = activation_ready_for_branch_head(
        &runtime,
        crate::truth_identity_fixtures::truth_branch_fixture("main"),
        resource_request,
    );
    let first = admit_subscription_backed_identity(
        &runtime,
        NodeId::new(61, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
        BridgeAsyncRequestSubscriptionInstance::authoritative(&first_ready),
    )
    .expect("first subscription-backed request should admit");
    let second = admit_subscription_backed_identity(
        &runtime,
        NodeId::new(61, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
        BridgeAsyncRequestSubscriptionInstance::authoritative(&second_ready),
    )
    .expect("second subscription-backed request should admit");

    assert_ne!(first.digest(), second.digest());
    assert_ne!(
        first
            .subscription_instance()
            .expect("first subscription-backed request should retain instance")
            .digest(),
        second
            .subscription_instance()
            .expect("second subscription-backed request should retain instance")
            .digest()
    );
}

#[test]
fn preview_truth_basis_must_match_preview_subscription_instance() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let preview_active = preview_active_subscription(&runtime, "phase6-preview", resource_request);
    let authoritative_ready = activation_ready_for_snapshot(
        &runtime,
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        resource_request,
    );
    let rejection = admit_subscription_backed_identity(
        &runtime,
        NodeId::new(81, 0),
        BridgeAsyncRequestTruthViewBasis::preview(&preview_active),
        BridgeAsyncRequestSubscriptionInstance::authoritative(&authoritative_ready),
    )
    .expect_err(
        "preview truth-view basis must reject mismatched authoritative subscription instance",
    );

    assert_eq!(
        rejection.kind(),
        BridgeAsyncRequestIdentityRejectionKind::PreviewBasisSubscriptionInstanceMismatch
    );
    assert_eq!(
        rejection
            .counters()
            .async_request_identity_rejection_count(),
        1
    );
}

#[test]
fn request_identity_runtime_rejects_cross_thread_admission() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let seed = admit_request_response_identity(
        &runtime,
        NodeId::new(91, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
    );
    let runtime_for_thread = runtime.clone();

    let rejection = std::thread::spawn(move || {
        let lowered = runtime_for_thread
            .validate_async_source_declaration(super::support::request_response_draft(NodeId::new(
                91, 0,
            )))
            .and_then(|validated| runtime_for_thread.lower_async_source_declaration(&validated))
            .expect("threaded request-response declaration should lower");
        let binding = runtime_for_thread.bind_async_request_basis(
            &lowered,
            BridgeAsyncRequestTruthViewBasis::authoritative(
                crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            ),
        );
        let request = BridgeAsyncRequestAdmissionRequest::request_response(&lowered, &binding)
            .expect("threaded request should construct");
        runtime_for_thread
            .admit_async_request_identity(request)
            .expect_err("cross-thread admission should fail closed")
    })
    .join()
    .expect("thread should complete");

    assert_ne!(seed.request_identity().as_str(), "");
    assert_eq!(
        rejection.kind(),
        BridgeAsyncRequestIdentityRejectionKind::SignalRuntimeThreadAffinityViolation
    );
}
