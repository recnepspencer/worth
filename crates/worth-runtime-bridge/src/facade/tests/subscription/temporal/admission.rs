use super::super::support::*;

#[test]
fn runtime_admits_wake_driven_temporal_subscription_for_current_snapshot_basis() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let admitted = admitted_detail_subscription_in_runtime(&runtime, resource_request);
    let temporal_basis = admitted_temporal_basis(BridgeTemporalTruthViewBasis::authoritative(
        crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    ));

    let temporal = runtime
        .admit_temporal_subscription(
            &admitted,
            temporal_basis,
            BridgeTemporalSubscriptionFamilyKind::WakeDriven,
        )
        .expect("current temporal subscription should admit");
    let ready = runtime.prepare_temporal_subscription_activation(&temporal);

    assert_eq!(
        temporal.family().kind(),
        BridgeTemporalSubscriptionFamilyKind::WakeDriven
    );
    assert_eq!(
        temporal.counters().subscription_temporal_admission_count(),
        1
    );
    assert_eq!(
        ready.temporal_admission().temporal_admission_identity(),
        temporal.temporal_admission_identity()
    );
    assert_eq!(
        ready
            .ordinary_activation_ready()
            .admitted()
            .admitted_subscription_identity(),
        admitted.admitted_subscription_identity()
    );
    assert_eq!(
        ready
            .counters()
            .subscription_temporal_activation_ready_count(),
        1
    );
}

#[test]
fn runtime_admits_historical_temporal_subscription_against_snapshot_bound_basis() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let admitted = admitted_detail_subscription_in_runtime(&runtime, resource_request);
    let temporal_basis = admitted_temporal_basis(BridgeTemporalTruthViewBasis::historical(
        crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-historical"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    ));

    let temporal = runtime
        .admit_temporal_subscription(
            &admitted,
            temporal_basis,
            BridgeTemporalSubscriptionFamilyKind::HistoricalReplay,
        )
        .expect("historical temporal subscription should admit");

    assert_eq!(
        temporal.temporal_basis().kind(),
        crate::facade::BridgeTemporalBasisKind::Historical
    );
    assert_eq!(
        temporal.family().kind(),
        BridgeTemporalSubscriptionFamilyKind::HistoricalReplay
    );
}

#[test]
fn runtime_rejects_temporal_subscription_when_family_does_not_support_basis_kind() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let admitted = admitted_detail_subscription_in_runtime(&runtime, resource_request);
    let temporal_basis = admitted_temporal_basis(BridgeTemporalTruthViewBasis::historical(
        crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-historical"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    ));

    let rejection = runtime
        .admit_temporal_subscription(
            &admitted,
            temporal_basis,
            BridgeTemporalSubscriptionFamilyKind::WakeDriven,
        )
        .expect_err("wake-driven family should reject historical-only temporal basis");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeTemporalSubscriptionAdmissionRejectionKind::TemporalFamilyDoesNotSupportBasisKind
    );
    assert_eq!(
        rejection.counters().subscription_temporal_rejection_count(),
        1
    );
}

#[test]
fn runtime_rejects_temporal_subscription_when_branch_head_basis_drifts_by_branch() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let admitted = branch_head_detail_subscription_in_runtime(&runtime, resource_request);
    let temporal_basis = admitted_temporal_basis(BridgeTemporalTruthViewBasis::branch_head(
        crate::truth_identity_fixtures::truth_branch_fixture("wrong-branch"),
        crate::truth_identity_fixtures::truth_commit_fixture("head-wrong"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    ));

    let rejection = runtime
        .admit_temporal_subscription(
            &admitted,
            temporal_basis,
            BridgeTemporalSubscriptionFamilyKind::WakeDriven,
        )
        .expect_err("branch drift should reject temporal admission");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeTemporalSubscriptionAdmissionRejectionKind::BranchIdentityMismatch
    );
}

#[test]
fn runtime_rejects_temporal_subscription_when_snapshot_identity_drifts() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let admitted = admitted_detail_subscription_in_runtime(&runtime, resource_request);
    let temporal_basis = admitted_temporal_basis(BridgeTemporalTruthViewBasis::authoritative(
        crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-b"),
    ));

    let rejection = runtime
        .admit_temporal_subscription(
            &admitted,
            temporal_basis,
            BridgeTemporalSubscriptionFamilyKind::WakeDriven,
        )
        .expect_err("snapshot drift should reject temporal admission");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeTemporalSubscriptionAdmissionRejectionKind::SnapshotIdentityMismatch
    );
}
