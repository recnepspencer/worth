use super::super::runtime_with_writeback_authority;
use super::support::*;
use crate::facade::{
    BridgeAsyncWritebackRejectionKind, BridgeFailureLocalizationRequest,
    BridgeTemporalAsyncFailureClass, BridgeTemporalAsyncFailureSubcode,
};

#[test]
fn equivalent_stale_completion_localizations_seal_equal_offline_bundles() {
    let first_runtime = runtime(BridgeRuntimePolicy::development());
    let second_runtime = runtime(BridgeRuntimePolicy::development());
    let original_basis = BridgeAsyncRequestTruthViewBasis::authoritative(
        crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    );
    let current_basis = BridgeAsyncRequestTruthViewBasis::authoritative(
        crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-b"),
    );

    let (first_denied, first_displacing) =
        denied_request_response_completion_with_displacing_identity(
            &first_runtime,
            NodeId::new(401, 0),
            original_basis.clone(),
            current_basis.clone(),
        );
    let first_classified = first_runtime
        .classify_async_completion_supersession(
            BridgeAsyncCompletionSupersessionClassificationRequest::request_response(
                &first_denied,
                current_basis.clone(),
            )
            .with_displacing_request_identity(&first_displacing),
        )
        .expect("first classified denied completion should admit");
    let first_localized = first_runtime
        .localize_temporal_async_failure(
            BridgeFailureLocalizationRequest::AsyncClassifiedDeniedCompletion(first_classified),
        )
        .expect("first failure should localize");

    let (second_denied, second_displacing) =
        denied_request_response_completion_with_displacing_identity(
            &second_runtime,
            NodeId::new(401, 0),
            original_basis,
            current_basis,
        );
    let second_classified = second_runtime
        .classify_async_completion_supersession(
            BridgeAsyncCompletionSupersessionClassificationRequest::request_response(
                &second_denied,
                BridgeAsyncRequestTruthViewBasis::authoritative(
                    crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
                    crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
                    crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-b"),
                ),
            )
            .with_displacing_request_identity(&second_displacing),
        )
        .expect("second classified denied completion should admit");
    let second_localized = second_runtime
        .localize_temporal_async_failure(
            BridgeFailureLocalizationRequest::AsyncClassifiedDeniedCompletion(second_classified),
        )
        .expect("second failure should localize");

    let first_bundle = first_runtime
        .seal_temporal_async_offline_diagnosis_bundle(vec![first_localized.clone()])
        .expect("first bundle should seal");
    let second_bundle = second_runtime
        .seal_temporal_async_offline_diagnosis_bundle(vec![second_localized.clone()])
        .expect("second bundle should seal");
    let comparison =
        first_runtime.compare_temporal_async_failure_bundles(&first_bundle, &second_bundle);

    assert_eq!(
        first_localized.failure_class(),
        BridgeTemporalAsyncFailureClass::SupersessionFailure
    );
    assert_eq!(
        first_localized.subcode(),
        BridgeTemporalAsyncFailureSubcode::SupersessionTruthBasis
    );
    assert_eq!(first_localized.digest(), second_localized.digest());
    assert!(comparison.equivalent());
}

#[test]
fn preview_discard_and_promotion_mismatch_localize_to_distinct_subcodes() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (discard_runtime, discard_preview_active) =
        preview_active_detail_subscription("phase15-preview-discard", resource_request);
    let discard_work_trace = preview_work_trace(
        &discard_runtime,
        &discard_preview_active,
        "phase15-preview-discard",
    );
    let discard_residue = discard_runtime
        .capture_preview_lifecycle_residue_envelope(
            &discard_preview_active,
            &discard_work_trace,
            preview_lifecycle_residue_inputs_with_count(
                &discard_work_trace,
                BridgeSubscriptionPreviewLifecycleResidueKind::CompletionWriteback,
                1,
            ),
        )
        .expect("discard residue envelope should capture");
    let discard_rejection = discard_runtime
        .admit_preview_lifecycle_discard(discard_preview_active, discard_residue)
        .expect_err("nonzero residue should reject discard");
    let discard_localized = discard_runtime
        .localize_temporal_async_failure(BridgeFailureLocalizationRequest::PreviewDiscardRejection(
            discard_rejection,
        ))
        .expect("discard failure should localize");

    let (promotion_runtime, promotion_preview_active, promotion_record, _ready) =
        preview_promotion_detail_subscription("phase15-preview-promotion", resource_request);
    let promotion_work_trace = preview_work_trace(
        &promotion_runtime,
        &promotion_preview_active,
        "phase15-preview-promotion",
    );
    let other_work_trace = promotion_runtime
        .record_preview_subscription_work(
            &promotion_preview_active,
            vec![
                crate::facade::BridgeSubscriptionPreviewWorkInput::delivery(
                    &promotion_preview_active,
                ),
                crate::facade::BridgeSubscriptionPreviewWorkInput::routing(
                    &promotion_preview_active,
                ),
                crate::facade::BridgeSubscriptionPreviewWorkInput::diagnostics(
                    &promotion_preview_active,
                ),
                crate::facade::BridgeSubscriptionPreviewWorkInput::continuation(
                    &promotion_preview_active,
                ),
            ],
        )
        .expect("alternate work trace should record");
    let promotion_residue = promotion_runtime
        .capture_preview_lifecycle_residue_envelope(
            &promotion_preview_active,
            &other_work_trace,
            zero_preview_lifecycle_residue_inputs(&other_work_trace),
        )
        .expect("promotion residue envelope should capture");
    let promotion_rejection = promotion_runtime
        .admit_preview_lifecycle_promotion(
            &promotion_preview_active,
            &promotion_work_trace,
            &promotion_residue,
            &promotion_record,
        )
        .expect_err("mismatched envelope should reject promotion");
    let promotion_localized = promotion_runtime
        .localize_temporal_async_failure(
            BridgeFailureLocalizationRequest::PreviewPromotionRejection(promotion_rejection),
        )
        .expect("promotion failure should localize");

    assert_eq!(
        discard_localized.subcode(),
        BridgeTemporalAsyncFailureSubcode::PreviewBoundaryDiscardResidue
    );
    assert_eq!(
        promotion_localized.subcode(),
        BridgeTemporalAsyncFailureSubcode::PreviewBoundaryPromotionMismatch
    );
    assert_ne!(discard_localized.digest(), promotion_localized.digest());
}

#[test]
fn resume_basis_rejection_localizes_without_live_diagnostics_handle() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let checkpoint = checkpoint_from_sealed(
        &runtime,
        &active,
        &sealed_window_with_members(
            &runtime,
            &active,
            BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
            0,
            fixture_members(1),
        ),
        0,
        BridgeSubscriptionDuplicateReplayPolicyKind::SuppressAcknowledgedMembers,
    );
    let async_request = admitted_async_request_identity(
        &runtime,
        crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        809,
    );
    let retained = retained_subscription_resume_basis(
        &runtime,
        &active,
        &checkpoint,
        None,
        Some(retained_inflight_async_resume_basis_without_generation(
            &async_request,
            true,
        )),
        None,
        true,
    );
    let rejection = runtime
        .admit_subscription_resume_basis(&retained)
        .expect_err("missing inflight generation should reject");

    let localized = runtime
        .localize_temporal_async_failure(BridgeFailureLocalizationRequest::ResumeBasisRejection(
            rejection,
        ))
        .expect("resume basis failure should localize");
    let bundle = runtime
        .seal_temporal_async_offline_diagnosis_bundle(vec![localized.clone()])
        .expect("bundle should seal");
    let matrix = runtime.inspect_temporal_async_failure_matrix(&bundle);

    assert_eq!(
        localized.failure_class(),
        BridgeTemporalAsyncFailureClass::ResumeBasisFailure
    );
    assert_eq!(
        localized.subcode(),
        BridgeTemporalAsyncFailureSubcode::ResumeBasisStale
    );
    assert_eq!(matrix.rows().len(), 1);
    assert_eq!(
        matrix.rows()[0].localized_failure_digest(),
        localized.digest()
    );
}

#[test]
fn mixed_cause_denial_localizes_as_ordering_replay_drift() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let original_basis = BridgeAsyncRequestTruthViewBasis::authoritative(
        crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    );
    let current_basis = BridgeAsyncRequestTruthViewBasis::authoritative(
        crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-b"),
    );
    let (denied, displacing_request) = denied_request_response_completion_with_displacing_identity(
        &runtime,
        NodeId::new(402, 0),
        original_basis,
        current_basis.clone(),
    );
    let classified = runtime
        .classify_async_completion_supersession(
            BridgeAsyncCompletionSupersessionClassificationRequest::request_response(
                &denied,
                current_basis,
            )
            .with_displacing_request_identity(&displacing_request),
        )
        .expect("classified denied completion should admit");
    let ordering = runtime.order_mixed_causes(&BridgeMixedCauseOrderingRequest::new(
        BridgeMixedCauseOrderingLaneKind::Authoritative,
        vec![BridgeMixedCauseOrderingInput::AsyncClassifiedDeniedCompletion(classified)],
    ));

    let localized = runtime
        .localize_temporal_async_failure(BridgeFailureLocalizationRequest::MixedCauseDenied(
            ordering.denied()[0].clone(),
        ))
        .expect("mixed-cause denial should localize");

    assert_eq!(
        localized.failure_class(),
        BridgeTemporalAsyncFailureClass::OrderingFailure
    );
    assert_eq!(
        localized.subcode(),
        BridgeTemporalAsyncFailureSubcode::OrderingReplayDrift
    );
}

#[test]
fn writeback_mapper_failure_localization_is_diagnostics_tier_invariant() {
    let development_runtime = runtime_with_writeback_authority(BridgeRuntimePolicy::development());
    let forensic_runtime = runtime_with_writeback_authority(BridgeRuntimePolicy::forensic());

    let development_completion = admitted_authoritative_request_response_completion(
        &development_runtime,
        NodeId::new(501, 0),
        "phase15-writeback",
    );
    let forensic_completion = admitted_authoritative_request_response_completion(
        &forensic_runtime,
        NodeId::new(501, 0),
        "phase15-writeback",
    );
    let development_request = BridgeAsyncWritebackAdmissionRequest::authoritative_commit(
        &development_completion,
        integer_projected_state_diff_intent(9),
        development_completion
            .request_identity()
            .basis_binding()
            .truth_view_basis()
            .clone(),
    );
    let forensic_request = BridgeAsyncWritebackAdmissionRequest::authoritative_commit(
        &forensic_completion,
        integer_projected_state_diff_intent(9),
        forensic_completion
            .request_identity()
            .basis_binding()
            .truth_view_basis()
            .clone(),
    );

    let development_writeback = development_runtime
        .admit_async_writeback(development_request)
        .expect("development writeback should admit");
    let forensic_writeback = forensic_runtime
        .admit_async_writeback(forensic_request)
        .expect("forensic writeback should admit");
    let development_rejection = development_runtime
        .stage_async_writeback_effect(&development_writeback)
        .expect_err("integer mapper input should reject during staging");
    let forensic_rejection = forensic_runtime
        .stage_async_writeback_effect(&forensic_writeback)
        .expect_err("integer mapper input should reject during staging");

    assert_eq!(
        development_rejection.kind(),
        BridgeAsyncWritebackRejectionKind::MapperFailed
    );
    let development_localized = development_runtime
        .localize_temporal_async_failure(BridgeFailureLocalizationRequest::AsyncWritebackRejection(
            development_rejection,
        ))
        .expect("development mapper failure should localize");
    let forensic_localized = forensic_runtime
        .localize_temporal_async_failure(BridgeFailureLocalizationRequest::AsyncWritebackRejection(
            forensic_rejection,
        ))
        .expect("forensic mapper failure should localize");

    assert_eq!(
        development_localized.subcode(),
        BridgeTemporalAsyncFailureSubcode::WritebackBoundaryMapperFailed
    );
    assert_eq!(development_localized.digest(), forensic_localized.digest());
}
