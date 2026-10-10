use worth_proof::TransitionOutcome;
use worth_signal::facade::{ClockAdvanceOrdinal, ClockDomain, ClockTick};

use super::support::*;
use crate::facade::{
    BridgeDiagnosticsTier, BridgeExecutionPolicyClass, BridgeFailureLocalizationRequest,
    BridgePolicyDeclaration, BridgePolicyDeclarationIdentity, BridgeRequestKind,
    BridgeTemporalAsyncFailureClass, BridgeTemporalAsyncFailureSubcode, BridgeTemporalSignalBasis,
    BridgeTemporalTruthViewBasis, BridgeTemporalWakeEvidence,
};

#[test]
fn temporal_basis_denial_localizes_as_cross_branch_failure() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let denial = match crate::facade::AdmittedBridgeTemporalBasis::admit(
        BridgeTemporalTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
        BridgeTemporalSignalBasis::new(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-other"),
            ClockDomain::MonotonicExecution,
            ClockTick::new(7),
            ClockAdvanceOrdinal::new(2),
            None,
        ),
        Some(BridgeTemporalWakeEvidence::new(
            worth_signal::facade::TemporalWakeId::new(11),
            worth_signal::facade::WakeOrdinal::new(3),
            ClockTick::new(7),
        )),
    ) {
        TransitionOutcome::Denied(denial) => denial,
        outcome => panic!("expected temporal basis denial, got {outcome:?}"),
    };

    let localized = runtime
        .localize_temporal_async_failure(BridgeFailureLocalizationRequest::TemporalBasisDenial(
            denial,
        ))
        .expect("temporal basis denial should localize");

    assert_eq!(
        localized.failure_class(),
        BridgeTemporalAsyncFailureClass::TemporalBasisFailure
    );
    assert_eq!(
        localized.subcode(),
        BridgeTemporalAsyncFailureSubcode::TemporalBasisCrossBranch
    );
}

#[test]
fn historical_temporal_previous_value_gap_localizes_as_temporal_readiness_failure() {
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
            crate::facade::BridgeTemporalSubscriptionFamilyKind::HistoricalReplay,
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
        .expect_err("missing previous value evidence should reject");

    let localized = runtime
        .localize_temporal_async_failure(
            BridgeFailureLocalizationRequest::HistoricalTemporalReplayRejection(rejection),
        )
        .expect("historical temporal replay rejection should localize");

    assert_eq!(
        localized.failure_class(),
        BridgeTemporalAsyncFailureClass::TemporalReadinessFailure
    );
    assert_eq!(
        localized.subcode(),
        BridgeTemporalAsyncFailureSubcode::TemporalReadinessPreviousValueMissing
    );
}

#[test]
fn duplicate_temporal_wake_localizes_as_duplicate_ordering_cause() {
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
            crate::facade::BridgeTemporalSubscriptionFamilyKind::WakeDriven,
        )
        .expect("temporal subscription should admit");
    let ready = runtime.prepare_temporal_subscription_activation(&temporal);
    let request = runtime.prepare_temporal_wake_routing(&ready);
    let first = runtime
        .route_temporal_wake(&request, None)
        .expect("first wake should route");
    let rejection = runtime
        .route_temporal_wake(&request, Some(&first))
        .expect_err("duplicate wake should reject");

    let localized = runtime
        .localize_temporal_async_failure(
            BridgeFailureLocalizationRequest::TemporalWakeRoutingRejection(rejection),
        )
        .expect("duplicate wake rejection should localize");

    assert_eq!(
        localized.failure_class(),
        BridgeTemporalAsyncFailureClass::OrderingFailure
    );
    assert_eq!(
        localized.subcode(),
        BridgeTemporalAsyncFailureSubcode::OrderingDuplicateCause
    );
}

#[test]
fn preview_subscription_instance_mismatch_localizes_as_async_identity_failure() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let preview_active =
        preview_active_subscription(&runtime, "phase15-preview-mismatch", resource_request);
    let authoritative_ready = activation_ready_for_snapshot(
        &runtime,
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        resource_request,
    );
    let rejection = admit_subscription_backed_identity(
        &runtime,
        NodeId::new(771, 0),
        BridgeAsyncRequestTruthViewBasis::preview(&preview_active),
        BridgeAsyncRequestSubscriptionInstance::authoritative(&authoritative_ready),
    )
    .expect_err("preview truth basis must reject authoritative subscription instances");

    let localized = runtime
        .localize_temporal_async_failure(
            BridgeFailureLocalizationRequest::AsyncRequestIdentityRejection(rejection),
        )
        .expect("request identity rejection should localize");

    assert_eq!(
        localized.failure_class(),
        BridgeTemporalAsyncFailureClass::AsyncIdentityFailure
    );
    assert_eq!(
        localized.subcode(),
        BridgeTemporalAsyncFailureSubcode::AsyncIdentityPreviewMismatch
    );
}

#[test]
fn payload_contract_mismatch_localizes_as_completion_envelope_failure() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let (request_identity, raw) = mismatched_payload_completion(&runtime);
    let rejection = runtime
        .validate_async_completion_envelope(&request_identity, raw)
        .expect_err("payload contract mismatch should reject before completion admission");

    let localized = runtime
        .localize_temporal_async_failure(
            BridgeFailureLocalizationRequest::AsyncCompletionRejection(rejection),
        )
        .expect("completion rejection should localize");

    assert_eq!(
        localized.failure_class(),
        BridgeTemporalAsyncFailureClass::CompletionAdmissionFailure
    );
    assert_eq!(
        localized.subcode(),
        BridgeTemporalAsyncFailureSubcode::CompletionAdmissionEnvelopeInvalid
    );
}

#[test]
fn duplicate_truth_patch_suppression_localizes_as_ordering_suppressed_cause() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let truth_patch = committed_patch(
        crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
    );
    let ordering = runtime.order_mixed_causes(&BridgeMixedCauseOrderingRequest::new(
        BridgeMixedCauseOrderingLaneKind::Authoritative,
        vec![
            BridgeMixedCauseOrderingInput::TruthPatch(truth_patch.clone()),
            BridgeMixedCauseOrderingInput::TruthPatch(truth_patch),
        ],
    ));

    let localized = runtime
        .localize_temporal_async_failure(BridgeFailureLocalizationRequest::MixedCauseSuppressed(
            ordering.suppressed()[0].clone(),
        ))
        .expect("suppressed mixed cause should localize");

    assert_eq!(
        localized.failure_class(),
        BridgeTemporalAsyncFailureClass::OrderingFailure
    );
    assert_eq!(
        localized.subcode(),
        BridgeTemporalAsyncFailureSubcode::OrderingSuppressedCause
    );
}

#[test]
fn stale_signal_generation_revalidation_localizes_as_revalidation_rejected() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let rejection = request_response_revalidation_rejection_for_stale_signal_generation(
        &runtime,
        worth_signal::facade::NodeId::new(777, 0),
        crate::facade::BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
    );

    let localized = runtime
        .localize_temporal_async_failure(
            BridgeFailureLocalizationRequest::AsyncForwardCausalityRejection(rejection),
        )
        .expect("forward causality rejection should localize");

    assert_eq!(
        localized.failure_class(),
        BridgeTemporalAsyncFailureClass::RetryRevalidationFailure
    );
    assert_eq!(
        localized.subcode(),
        BridgeTemporalAsyncFailureSubcode::RetryRevalidationRevalidationRejected
    );
}

#[test]
fn duplicate_completion_noop_localizes_as_writeback_idempotent_noop() {
    let runtime = runtime_with_authority();
    let completion = admitted_authoritative_request_response_completion(
        &runtime,
        worth_signal::facade::NodeId::new(880, 0),
        "phase15-noop",
    );
    let request = authoritative_writeback_request(&completion, "phase15-noop");
    let admitted = runtime
        .admit_async_writeback(request)
        .expect("writeback admission should succeed");
    let staged = runtime
        .stage_async_writeback_effect(&admitted)
        .expect("writeback staging should succeed");
    let _first = runtime.commit_async_writeback(&staged);
    let second = runtime.commit_async_writeback(&staged);

    let localized = runtime
        .localize_temporal_async_failure(
            BridgeFailureLocalizationRequest::AsyncWritebackCommitReport(second),
        )
        .expect("noop writeback report should localize");

    assert_eq!(
        localized.failure_class(),
        BridgeTemporalAsyncFailureClass::WritebackBoundaryFailure
    );
    assert_eq!(
        localized.subcode(),
        BridgeTemporalAsyncFailureSubcode::WritebackBoundaryIdempotentNoop
    );
}

#[test]
fn policy_rejection_localizes_as_policy_remask_failure() {
    let runtime = runtime(BridgeRuntimePolicy::operational().with_replay_artifacts(false));
    let declaration = BridgePolicyDeclaration::new(
        BridgePolicyDeclarationIdentity::admit_bridge_owned("policy:phase15-replay-required"),
        BridgeRequestKind::Preview,
        BridgeExecutionPolicyClass::Optimized,
        BridgeDiagnosticsTier::Minimal,
        true,
        false,
    );
    let rejection = runtime
        .admit_policy_declaration(declaration)
        .expect_err("replay-required policy should reject against non-replay baseline");

    let localized = runtime
        .localize_temporal_async_failure(BridgeFailureLocalizationRequest::PolicyRejection(
            rejection,
        ))
        .expect("policy rejection should localize");

    assert_eq!(
        localized.failure_class(),
        BridgeTemporalAsyncFailureClass::PolicyRemaskFailure
    );
    assert_eq!(
        localized.subcode(),
        BridgeTemporalAsyncFailureSubcode::PolicyRemaskPolicyDrift
    );
}
