use worth_proof::TransitionOutcome;

use super::super::support::*;

#[test]
fn runtime_prepares_historical_temporal_readiness_from_pinned_truth_and_retained_values() {
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
            temporal_basis.clone(),
            BridgeTemporalSubscriptionFamilyKind::HistoricalReplay,
        )
        .expect("historical temporal subscription should admit");
    let historical_truth_basis = runtime
        .admit_historical_truth_view_basis(temporal_basis.truth_basis())
        .expect("historical truth basis should admit");
    let retained = runtime.retain_historical_previous_value_evidence(
        crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        vec![retained_previous_value_reference(11, "1:0", 5)],
    );

    let replay_basis = runtime
        .admit_historical_temporal_replay_basis(&temporal, &historical_truth_basis, retained)
        .expect("historical replay basis should admit");
    let request = runtime.prepare_historical_temporal_replay_request(&replay_basis);
    let readiness = runtime.prepare_historical_temporal_readiness(&request);

    assert_eq!(
        historical_truth_basis
            .counters()
            .subscription_historical_truth_basis_admission_count(),
        1
    );
    assert_eq!(
        replay_basis
            .counters()
            .subscription_historical_temporal_replay_basis_admission_count(),
        1
    );
    assert_eq!(
        readiness
            .counters()
            .subscription_historical_temporal_readiness_count(),
        1
    );
}

#[test]
fn runtime_rejects_historical_replay_without_retained_previous_values() {
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
            temporal_basis.clone(),
            BridgeTemporalSubscriptionFamilyKind::HistoricalReplay,
        )
        .expect("historical temporal subscription should admit");
    let historical_truth_basis = runtime
        .admit_historical_truth_view_basis(temporal_basis.truth_basis())
        .expect("historical truth basis should admit");
    let retained = runtime.retain_historical_previous_value_evidence(
        crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        vec![],
    );

    let rejection = runtime
        .admit_historical_temporal_replay_basis(&temporal, &historical_truth_basis, retained)
        .expect_err("missing previous values should reject");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeHistoricalTemporalReplayRejectionKind::MissingPreviousValueEvidence
    );
    assert_eq!(
        rejection
            .counters()
            .subscription_historical_temporal_replay_rejection_count(),
        1
    );
}

#[test]
fn runtime_rejects_historical_replay_when_previous_values_cross_branches() {
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
            temporal_basis.clone(),
            BridgeTemporalSubscriptionFamilyKind::HistoricalReplay,
        )
        .expect("historical temporal subscription should admit");
    let historical_truth_basis = runtime
        .admit_historical_truth_view_basis(temporal_basis.truth_basis())
        .expect("historical truth basis should admit");
    let retained = runtime.retain_historical_previous_value_evidence(
        crate::truth_identity_fixtures::truth_branch_fixture("analysis-drift"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        vec![retained_previous_value_reference(11, "1:0", 5)],
    );

    let rejection = runtime
        .admit_historical_temporal_replay_basis(&temporal, &historical_truth_basis, retained)
        .expect_err("cross-branch previous values should reject");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeHistoricalTemporalReplayRejectionKind::PreviousValueEvidenceBranchMismatch
    );
}

#[test]
fn runtime_rejects_nonhistorical_truth_basis_for_historical_replay() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let admitted_truth_basis = match crate::facade::AdmittedBridgeTemporalTruthViewBasis::admit(
        BridgeTemporalTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
    ) {
        TransitionOutcome::Success(admitted) => admitted,
        outcome => panic!("expected admitted truth basis, got {outcome:?}"),
    };

    let rejection = runtime
        .admit_historical_truth_view_basis(&admitted_truth_basis)
        .expect_err("authoritative truth basis should reject");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeHistoricalTruthBasisAdmissionRejectionKind::TemporalTruthBasisNotHistorical
    );
}

#[test]
fn runtime_historical_temporal_readiness_digest_ignores_unrelated_current_truth_churn() {
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
            temporal_basis.clone(),
            BridgeTemporalSubscriptionFamilyKind::HistoricalReplay,
        )
        .expect("historical temporal subscription should admit");
    let historical_truth_basis = runtime
        .admit_historical_truth_view_basis(temporal_basis.truth_basis())
        .expect("historical truth basis should admit");
    let retained_rows = vec![retained_previous_value_reference(11, "1:0", 5)];
    let replay_basis = runtime
        .admit_historical_temporal_replay_basis(
            &temporal,
            &historical_truth_basis,
            runtime.retain_historical_previous_value_evidence(
                crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                retained_rows.clone(),
            ),
        )
        .expect("replay basis should admit");
    let readiness = runtime.prepare_historical_temporal_readiness(
        &runtime.prepare_historical_temporal_replay_request(&replay_basis),
    );

    let _unrelated_current_basis =
        admitted_temporal_basis(BridgeTemporalTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-current"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-z"),
        ));
    let replay_basis_again = runtime
        .admit_historical_temporal_replay_basis(
            &temporal,
            &historical_truth_basis,
            runtime.retain_historical_previous_value_evidence(
                crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                retained_rows,
            ),
        )
        .expect("replay basis should stay stable");
    let readiness_again = runtime.prepare_historical_temporal_readiness(
        &runtime.prepare_historical_temporal_replay_request(&replay_basis_again),
    );

    assert_eq!(readiness.digest(), readiness_again.digest());
}

#[test]
fn runtime_rejects_historical_replay_when_historical_truth_snapshot_drifts() {
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
    let mismatched_truth_basis = match crate::facade::AdmittedBridgeTemporalTruthViewBasis::admit(
        BridgeTemporalTruthViewBasis::historical(
            crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-historical"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-b"),
        ),
    ) {
        TransitionOutcome::Success(admitted) => admitted,
        outcome => panic!("expected admitted mismatched truth basis, got {outcome:?}"),
    };
    let historical_truth_basis = runtime
        .admit_historical_truth_view_basis(&mismatched_truth_basis)
        .expect("historical truth basis should admit");
    let retained = runtime.retain_historical_previous_value_evidence(
        crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        vec![retained_previous_value_reference(11, "1:0", 5)],
    );

    let rejection = runtime
        .admit_historical_temporal_replay_basis(&temporal, &historical_truth_basis, retained)
        .expect_err("snapshot-drift historical truth basis should reject");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeHistoricalTemporalReplayRejectionKind::HistoricalTruthSnapshotIdentityMismatch
    );
}

#[test]
fn runtime_rejects_historical_replay_when_historical_truth_branch_drifts() {
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
    let mismatched_truth_basis = match crate::facade::AdmittedBridgeTemporalTruthViewBasis::admit(
        BridgeTemporalTruthViewBasis::historical(
            crate::truth_identity_fixtures::truth_branch_fixture("analysis-drift"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-historical"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
    ) {
        TransitionOutcome::Success(admitted) => admitted,
        outcome => panic!("expected admitted mismatched truth basis, got {outcome:?}"),
    };
    let historical_truth_basis = runtime
        .admit_historical_truth_view_basis(&mismatched_truth_basis)
        .expect("historical truth basis should admit");
    let retained = runtime.retain_historical_previous_value_evidence(
        crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        vec![retained_previous_value_reference(11, "1:0", 5)],
    );

    let rejection = runtime
        .admit_historical_temporal_replay_basis(&temporal, &historical_truth_basis, retained)
        .expect_err("branch-drift historical truth basis should reject");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeHistoricalTemporalReplayRejectionKind::HistoricalTruthBranchIdentityMismatch
    );
}
