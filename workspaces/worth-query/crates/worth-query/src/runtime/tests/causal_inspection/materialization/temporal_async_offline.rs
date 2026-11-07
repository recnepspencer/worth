use super::super::super::super::*;

fn denied_offline_temporal_async_artifact(
    outcome: CausalObservationOutcome,
    reason: CausalInspectionReason,
    evidence_identities: Vec<CausalObservationEvidenceIdentity>,
    requested_families: &[CausalEvidenceFamily],
) -> DeniedQueryCausalInspectionArtifact {
    let anchor = anchor_causal_observation(
        QueryObservationReceipt::fixture(outcome, evidence_identities),
        reason,
    )
    .expect("offline temporal/async receipt should anchor");
    let CausalEvidenceReferenceResolution::Resolved { reference_set, .. } =
        resolve_causal_evidence_references(anchor, requested_families)
    else {
        panic!("offline temporal/async references should resolve");
    };
    let receipt = reference_set.anchor().observation_receipt();
    let target = causal_inspection_target(
        receipt.observation_target().clone(),
        receipt.result_shape_context().clone(),
    )
    .expect("offline temporal/async target should match");
    let request = request_causal_inspection(
        reference_set,
        target,
        CausalInspectionExplanationFamily::DurableCausalArchive,
        CausalInspectionRichness::ReferenceOnly,
        requested_families,
    )
    .expect("unsupported family should still reach admission boundary");
    let CausalInspectionProofFlow::Denied(denied) = admit_causal_inspection(request) else {
        panic!("durable archive should deny while preserving retained evidence");
    };
    let QueryCausalInspectionArtifact::Denied(artifact) = materialize_denied_causal_inspection(
        &denied,
        None,
        CausalInspectionRedactionPolicy::PreserveDetail,
        CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
    ) else {
        panic!("expected denied offline temporal/async artifact");
    };
    artifact
}

#[test]
fn offline_temporal_async_diagnostics_preserve_remask_replay_resume_and_stale_completion() {
    let remask = denied_offline_temporal_async_artifact(
        CausalObservationOutcome::BranchPreview,
        CausalInspectionReason::BranchPreviewResult,
        vec![
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::QueryInspection,
                crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                    "query-inspection:offline-remask",
                ),
            )
            .expect("fixture digest must fit"),
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::BridgePreview,
                crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                    "bridge-preview:offline-remask",
                ),
            ),
        ],
        &[CausalEvidenceFamily::BridgePreview],
    );
    let replay_drift = denied_offline_temporal_async_artifact(
        CausalObservationOutcome::Replayed,
        CausalInspectionReason::HistoricalReplayResult,
        vec![
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::QueryInspection,
                crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                    "query-inspection:offline-replay-drift",
                ),
            )
            .expect("fixture digest must fit"),
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::BridgeReplay,
                crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                    "bridge-replay:offline-replay-drift",
                ),
            ),
        ],
        &[CausalEvidenceFamily::BridgeReplay],
    );
    let resume_mismatch = denied_offline_temporal_async_artifact(
        CausalObservationOutcome::Replayed,
        CausalInspectionReason::HistoricalReplayResult,
        vec![
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::QueryInspection,
                crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                    "query-inspection:offline-resume-mismatch",
                ),
            )
            .expect("fixture digest must fit"),
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::BridgeReplay,
                crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                    "bridge-replay:offline-resume-mismatch",
                ),
            ),
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::SignalReplayCursor,
                crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                    "signal-replay-cursor:offline-resume-mismatch",
                ),
            ),
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::BridgeContinuity,
                crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                    "bridge-continuity:offline-resume-mismatch",
                ),
            ),
        ],
        &[
            CausalEvidenceFamily::BridgeReplay,
            CausalEvidenceFamily::SignalReplayCursor,
            CausalEvidenceFamily::BridgeContinuity,
        ],
    );
    let stale_completion = denied_offline_temporal_async_artifact(
        CausalObservationOutcome::Denied,
        CausalInspectionReason::DeniedResult,
        vec![
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::QueryInspection,
                crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                    "query-inspection:offline-stale-completion",
                ),
            )
            .expect("fixture digest must fit"),
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::SignalEvaluation,
                crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                    "signal-evaluation:offline-stale-completion",
                ),
            ),
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::BridgeSourceFailure,
                crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                    "bridge-source-failure:offline-stale-completion",
                ),
            ),
        ],
        &[
            CausalEvidenceFamily::SignalEvaluation,
            CausalEvidenceFamily::BridgeSourceFailure,
        ],
    );

    assert_eq!(
        remask.temporal_async_explanation().kind(),
        QueryCausalTemporalAsyncExplanationKind::PreviewRemask
    );
    assert_eq!(
        replay_drift.temporal_async_explanation().kind(),
        QueryCausalTemporalAsyncExplanationKind::ReplayDrift
    );
    assert_eq!(
        resume_mismatch.temporal_async_explanation().kind(),
        QueryCausalTemporalAsyncExplanationKind::ResumeMismatch
    );
    assert_eq!(
        stale_completion.temporal_async_explanation().kind(),
        QueryCausalTemporalAsyncExplanationKind::StaleCompletion
    );
    assert!(remask.temporal_async_explanation().offline_explainable());
    assert!(replay_drift
        .temporal_async_explanation()
        .offline_explainable());
    assert!(resume_mismatch
        .temporal_async_explanation()
        .offline_explainable());
    assert!(stale_completion
        .temporal_async_explanation()
        .offline_explainable());
}
