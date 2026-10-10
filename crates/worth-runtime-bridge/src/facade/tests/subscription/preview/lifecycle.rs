use super::super::support::*;

#[test]
fn preview_lifecycle_discard_requires_zero_residue_across_all_lifecycle_kinds() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, preview_active) =
        preview_active_detail_subscription("preview-lifecycle-discard", resource_request);
    let preview_identity = preview_active
        .preview_active_subscription_identity()
        .clone();
    let work_trace = preview_work_trace(&runtime, &preview_active, "preview-lifecycle-discard");
    let residue_envelope = runtime
        .capture_preview_lifecycle_residue_envelope(
            &preview_active,
            &work_trace,
            zero_preview_lifecycle_residue_inputs(&work_trace),
        )
        .expect("preview lifecycle residue envelope should capture");

    let proof = runtime
        .admit_preview_lifecycle_discard(preview_active, residue_envelope)
        .expect("zero lifecycle residue should admit discard");

    assert_eq!(
        proof.preview_active_subscription_identity(),
        &preview_identity
    );
    assert_eq!(proof.total_residue_count(), 0);
    assert_eq!(proof.residue_records().len(), 4);
    assert_eq!(proof.counters().subscription_preview_discard_count(), 1);
}

#[test]
fn preview_lifecycle_discard_rejects_nonzero_completion_residue() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, preview_active) =
        preview_active_detail_subscription("preview-lifecycle-nonzero", resource_request);
    let work_trace = preview_work_trace(&runtime, &preview_active, "preview-lifecycle-nonzero");
    let residue_envelope = runtime
        .capture_preview_lifecycle_residue_envelope(
            &preview_active,
            &work_trace,
            preview_lifecycle_residue_inputs_with_count(
                &work_trace,
                crate::facade::BridgeSubscriptionPreviewLifecycleResidueKind::CompletionWriteback,
                1,
            ),
        )
        .expect("residue envelope should capture");

    let rejection = runtime
        .admit_preview_lifecycle_discard(preview_active, residue_envelope)
        .expect_err("nonzero completion residue must reject discard");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionPreviewLifecycleDiscardRejectionKind::NonzeroResidue
    );
    assert_eq!(
        rejection.rejection_context().nonzero_kinds()[0].kind(),
        crate::facade::BridgeSubscriptionPreviewLifecycleResidueKind::CompletionWriteback
    );
}

#[test]
fn preview_lifecycle_promotion_re_admits_authoritative_boundary() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, preview_active, promotion_record, promoted_ready) =
        preview_promotion_detail_subscription("preview-lifecycle-promote", resource_request);
    let preview_identity = preview_active
        .preview_active_subscription_identity()
        .clone();
    let authoritative_identity = promoted_ready
        .admitted()
        .admitted_subscription_identity()
        .clone();
    let work_trace = preview_work_trace(&runtime, &preview_active, "preview-lifecycle-promote");
    let residue_envelope = runtime
        .capture_preview_lifecycle_residue_envelope(
            &preview_active,
            &work_trace,
            zero_preview_lifecycle_residue_inputs(&work_trace),
        )
        .expect("residue envelope should capture");
    let promotion = runtime
        .admit_preview_lifecycle_promotion(
            &preview_active,
            &work_trace,
            &residue_envelope,
            &promotion_record,
        )
        .expect("promotion should admit");

    let readmission = runtime
        .prepare_authoritative_preview_readmission(promotion, &promoted_ready)
        .expect("authoritative readmission should admit");

    assert_eq!(
        readmission.readmission_class(),
        crate::facade::BridgeSubscriptionAuthoritativePreviewReadmissionClass::ReAdmittedAuthoritativeBoundary
    );
    assert_eq!(
        readmission.preview_active_subscription_identity(),
        preview_identity.as_str()
    );
    assert_eq!(
        readmission.authoritative_admitted_subscription_identity(),
        &authoritative_identity
    );
    assert_ne!(
        readmission.preview_active_subscription_identity(),
        readmission
            .authoritative_admitted_subscription_identity()
            .as_str()
    );
    assert_eq!(
        readmission
            .counters()
            .subscription_preview_authoritative_readmission_count(),
        1
    );
}

#[test]
fn preview_lifecycle_promotion_rejects_preview_crossed_completion() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, preview_active, promotion_record, _promoted_ready) =
        preview_promotion_detail_subscription("preview-lifecycle-completion", resource_request);
    let work_trace = preview_work_trace(&runtime, &preview_active, "preview-lifecycle-completion");
    let residue_envelope = runtime
        .capture_preview_lifecycle_residue_envelope(
            &preview_active,
            &work_trace,
            preview_lifecycle_residue_inputs_with_count(
                &work_trace,
                crate::facade::BridgeSubscriptionPreviewLifecycleResidueKind::CompletionWriteback,
                1,
            ),
        )
        .expect("residue envelope should capture");

    let rejection = runtime
        .admit_preview_lifecycle_promotion(
            &preview_active,
            &work_trace,
            &residue_envelope,
            &promotion_record,
        )
        .expect_err("preview-crossed completion must reject promotion");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionPreviewLifecyclePromotionRejectionKind::PreviewCrossedCompletion
    );
    assert_eq!(
        rejection
            .counters()
            .subscription_preview_crossed_completion_rejection_count(),
        1
    );
}

#[test]
fn preview_lifecycle_promotion_rejects_temporal_evidence_drift() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, preview_active, promotion_record, _promoted_ready) =
        preview_promotion_detail_subscription("preview-lifecycle-temporal-drift", resource_request);
    let work_trace = preview_work_trace(
        &runtime,
        &preview_active,
        "preview-lifecycle-temporal-drift",
    );
    let mut residue_inputs = zero_preview_lifecycle_residue_inputs(&work_trace);
    residue_inputs[0] = crate::facade::BridgeSubscriptionPreviewLifecycleResidueInput::custom(
        crate::facade::BridgeSubscriptionPreviewLifecycleResidueKind::TemporalWake,
        0,
        Arc::<str>::from("drifted-temporal-evidence"),
    );
    let residue_envelope = runtime
        .capture_preview_lifecycle_residue_envelope(&preview_active, &work_trace, residue_inputs)
        .expect("residue envelope should capture");

    let rejection = runtime
        .admit_preview_lifecycle_promotion(
            &preview_active,
            &work_trace,
            &residue_envelope,
            &promotion_record,
        )
        .expect_err("temporal evidence drift must reject promotion");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionPreviewLifecyclePromotionRejectionKind::TemporalEvidenceDrift
    );
    assert_eq!(
        rejection
            .counters()
            .subscription_preview_temporal_evidence_drift_rejection_count(),
        1
    );
}

#[test]
fn preview_lifecycle_promotion_rejects_residue_envelope_from_another_work_trace() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, preview_active, promotion_record, _promoted_ready) =
        preview_promotion_detail_subscription("preview-lifecycle-envelope-drift", resource_request);
    let work_trace = preview_work_trace(
        &runtime,
        &preview_active,
        "preview-lifecycle-envelope-drift-a",
    );
    let other_work_trace = runtime
        .record_preview_subscription_work(
            &preview_active,
            vec![
                BridgeSubscriptionPreviewWorkInput::delivery(&preview_active),
                BridgeSubscriptionPreviewWorkInput::routing(&preview_active),
                BridgeSubscriptionPreviewWorkInput::diagnostics(&preview_active),
                BridgeSubscriptionPreviewWorkInput::continuation(&preview_active),
            ],
        )
        .expect("alternate preview work trace should record");
    let residue_envelope = runtime
        .capture_preview_lifecycle_residue_envelope(
            &preview_active,
            &other_work_trace,
            zero_preview_lifecycle_residue_inputs(&other_work_trace),
        )
        .expect("residue envelope should capture");

    let rejection = runtime
        .admit_preview_lifecycle_promotion(
            &preview_active,
            &work_trace,
            &residue_envelope,
            &promotion_record,
        )
        .expect_err("promotion must reject residue envelopes from another work trace");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionPreviewLifecyclePromotionRejectionKind::ResidueEnvelopeMismatch
    );
}
