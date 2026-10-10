use super::support::{
    admit_request_response_completion, admit_request_response_identity,
    admit_subscription_backed_completion, denied_request_response_completion_after_cancellation,
    denied_request_response_completion_after_rejection,
    denied_request_response_completion_after_restore_staleness,
    denied_request_response_completion_after_supersession,
    denied_request_response_completion_after_timeout, mismatched_payload_completion,
    request_response_raw_completion,
};
use super::*;
use crate::facade::{
    BridgeAsyncCompletionDenialClass, BridgeAsyncCompletionRejectionKind,
    BridgeAsyncRequestTruthViewBasis,
};
use worth_signal::facade::NodeId;

#[test]
fn equivalent_request_response_fulfilled_completions_admit_identically() {
    let first_runtime = runtime(BridgeRuntimePolicy::development());
    let second_runtime = runtime(BridgeRuntimePolicy::development());
    let first = admit_request_response_completion(
        &first_runtime,
        NodeId::new(11, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
        64,
    );
    let second = admit_request_response_completion(
        &second_runtime,
        NodeId::new(88, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
        64,
    );
    let first = first
        .admitted_completion()
        .expect("request-response completion should admit");
    let second = second
        .admitted_completion()
        .expect("equivalent request-response completion should admit");

    assert_eq!(first.digest(), second.digest());
    assert_eq!(first.receipt().digest(), second.receipt().digest());
    assert_eq!(first.completion_class(), second.completion_class());
    assert_eq!(first.counters().completion_admission_count(), 1);
    assert_eq!(first.counters().request_response_completion_count(), 1);
}

#[test]
fn equivalent_subscription_backed_fulfilled_completions_admit_identically() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let first_runtime = runtime(BridgeRuntimePolicy::development());
    let second_runtime = runtime(BridgeRuntimePolicy::development());
    let first = admit_subscription_backed_completion(
        &first_runtime,
        NodeId::new(31, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
        48,
        resource_request,
    )
    .expect("subscription-backed completion should admit");
    let second = admit_subscription_backed_completion(
        &second_runtime,
        NodeId::new(67, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
        48,
        resource_request,
    )
    .expect("equivalent subscription-backed completion should admit");
    let first = first
        .admitted_completion()
        .expect("subscription-backed completion should admit");
    let second = second
        .admitted_completion()
        .expect("equivalent subscription-backed completion should admit");

    assert_eq!(first.digest(), second.digest());
    assert_eq!(first.receipt().digest(), second.receipt().digest());
    assert_eq!(first.counters().subscription_backed_completion_count(), 1);
}

#[test]
fn oversized_payload_is_retained_as_denied_completion() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let report = admit_request_response_completion(
        &runtime,
        NodeId::new(29, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
        4096,
    );
    let denied = report
        .denied_completion()
        .expect("oversized payload should deny canonically");

    assert_eq!(
        denied.denial_class(),
        BridgeAsyncCompletionDenialClass::SignalLifecycleDenied
    );
    assert_eq!(denied.counters().completion_denial_count(), 1);
    assert_eq!(denied.counters().signal_completion_denial_count(), 1);
    assert_eq!(denied.receipt().state(), denied.state());
}

#[test]
fn late_completion_after_rejection_maps_to_rejected_denial() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let report = denied_request_response_completion_after_rejection(
        &runtime,
        NodeId::new(45, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
    );
    let denied = report
        .denied_completion()
        .expect("late completion after rejection should deny");

    assert_eq!(
        denied.denial_class(),
        BridgeAsyncCompletionDenialClass::Rejected
    );
    assert_eq!(denied.receipt().state(), denied.state());
}

#[test]
fn late_completion_after_cancellation_maps_to_cancelled_denial() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let report = denied_request_response_completion_after_cancellation(
        &runtime,
        NodeId::new(46, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
    );
    let denied = report
        .denied_completion()
        .expect("late completion after cancellation should deny");

    assert_eq!(
        denied.denial_class(),
        BridgeAsyncCompletionDenialClass::Cancelled
    );
    assert_eq!(denied.receipt().state(), denied.state());
}

#[test]
fn late_completion_after_supersession_maps_to_superseded_denial() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let report = denied_request_response_completion_after_supersession(
        &runtime,
        NodeId::new(47, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
    );
    let denied = report
        .denied_completion()
        .expect("late completion after supersession should deny");

    assert_eq!(
        denied.denial_class(),
        BridgeAsyncCompletionDenialClass::Superseded
    );
    assert_eq!(denied.receipt().state(), denied.state());
}

#[test]
fn late_completion_after_timeout_maps_to_timed_out_denial() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let report = denied_request_response_completion_after_timeout(
        &runtime,
        NodeId::new(48, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
        3,
    );
    let denied = report
        .denied_completion()
        .expect("late completion after timeout should deny");

    assert_eq!(
        denied.denial_class(),
        BridgeAsyncCompletionDenialClass::TimedOut
    );
    assert_eq!(denied.receipt().state(), denied.state());
}

#[test]
fn pre_restore_completion_maps_to_stale_denial() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let report = denied_request_response_completion_after_restore_staleness(
        &runtime,
        NodeId::new(49, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
    );
    let denied = report
        .denied_completion()
        .expect("pre-restore completion should deny as stale");

    assert_eq!(
        denied.denial_class(),
        BridgeAsyncCompletionDenialClass::StaleDenied
    );
    assert_eq!(denied.receipt().state(), denied.state());
}

#[test]
fn mismatched_payload_contract_rejects_before_signal_admission() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let (request_identity, raw) = mismatched_payload_completion(&runtime);
    let rejection = runtime
        .validate_async_completion_envelope(&request_identity, raw)
        .expect_err("mismatched payload contract should reject before admission");

    assert_eq!(
        rejection.kind(),
        BridgeAsyncCompletionRejectionKind::PayloadContractDigestMismatch
    );
    assert_eq!(rejection.counters().completion_rejection_count(), 1);
    assert_eq!(
        rejection
            .counters()
            .invalid_completion_envelope_rejection_count(),
        1
    );
}

#[test]
fn diagnostics_tier_variation_does_not_change_completion_digest() {
    let development = runtime(BridgeRuntimePolicy::development());
    let operational = runtime(BridgeRuntimePolicy::operational());
    let development = admit_request_response_completion(
        &development,
        NodeId::new(101, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
        64,
    );
    let operational = admit_request_response_completion(
        &operational,
        NodeId::new(101, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
        64,
    );
    let development = development
        .admitted_completion()
        .expect("development completion should admit");
    let operational = operational
        .admitted_completion()
        .expect("operational completion should admit");

    assert_eq!(development.digest(), operational.digest());
    assert_eq!(
        development.receipt().digest(),
        operational.receipt().digest()
    );
}

#[test]
fn completion_runtime_rejects_cross_thread_admission() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let request_identity = admit_request_response_identity(
        &runtime,
        NodeId::new(91, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
    );
    let validated = runtime
        .validate_async_completion_envelope(
            &request_identity,
            request_response_raw_completion(&request_identity, 64),
        )
        .expect("completion envelope should validate");
    let runtime_for_thread = runtime.clone();
    let request_identity_for_thread = request_identity.clone();
    let validated_for_thread = validated.clone();

    let rejection = std::thread::spawn(move || {
        runtime_for_thread
            .admit_async_completion(&request_identity_for_thread, &validated_for_thread)
            .expect_err("cross-thread completion admission should fail closed")
    })
    .join()
    .expect("thread should complete");

    assert_eq!(
        rejection.kind(),
        BridgeAsyncCompletionRejectionKind::SignalRuntimeThreadAffinityViolation
    );
}
