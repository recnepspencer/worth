use super::super::super::super::*;
use super::artifact_support::admitted_artifact_for;
use super::slot_support::artifact_with_lower_runtime_slot_evidence;

pub(super) fn representative_matrix(
    changed: &QueryCausalInspectionArtifact,
    redacted: &QueryCausalInspectionArtifact,
    denied: &QueryCausalInspectionArtifact,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> CausalInspectionRepresentativeMatrix {
    let suppressed = admitted_artifact_for(
        super::super::causal_truth_commit_identity("commit-query-cert-suppressed"),
        CausalObservationOutcome::Suppressed,
        CausalInspectionReason::SuppressedResult,
        resource_request,
    );
    let branch_preview = admitted_artifact_for(
        super::super::causal_truth_commit_identity("commit-query-cert-branch-preview"),
        CausalObservationOutcome::BranchPreview,
        CausalInspectionReason::BranchPreviewResult,
        resource_request,
    );
    let replay = admitted_artifact_for(
        super::super::causal_truth_commit_identity("commit-query-cert-replay"),
        CausalObservationOutcome::Replayed,
        CausalInspectionReason::HistoricalReplayResult,
        resource_request,
    );
    let lower_runtime_slots = artifact_with_lower_runtime_slot_evidence(
        super::super::causal_truth_commit_identity("commit-query-cert-lower-runtime-slots"),
        resource_request,
    );
    let missing_signal_invalidation =
        missing_evidence_digest(CausalEvidenceFamily::SignalInvalidation);
    let missing_signal_evaluation = missing_evidence_digest(CausalEvidenceFamily::SignalEvaluation);
    let missing_bridge = missing_evidence_digest(CausalEvidenceFamily::BridgeRoute);

    CausalInspectionRepresentativeMatrix::from_representatives(&[
        artifact_row(CausalInspectionRepresentativeKind::ChangedResult, changed),
        artifact_row(
            CausalInspectionRepresentativeKind::SuppressedResult,
            &suppressed,
        ),
        artifact_row(
            CausalInspectionRepresentativeKind::QueryDeniedBeforeBridgeEnvelope,
            denied,
        ),
        missing_row(
            CausalInspectionRepresentativeKind::MissingSignalInvalidationEvidenceDenied,
            CausalEvidenceFamily::SignalInvalidation,
            &missing_signal_invalidation,
        ),
        missing_row(
            CausalInspectionRepresentativeKind::MissingSignalEvaluationEvidenceDenied,
            CausalEvidenceFamily::SignalEvaluation,
            &missing_signal_evaluation,
        ),
        missing_row(
            CausalInspectionRepresentativeKind::MissingBridgeRouteEvidenceDenied,
            CausalEvidenceFamily::BridgeRoute,
            &missing_bridge,
        ),
        artifact_row(
            CausalInspectionRepresentativeKind::AdvisoryRedactedCausalEnvelope,
            redacted,
        ),
        artifact_row(CausalInspectionRepresentativeKind::PolicyRedacted, redacted),
        artifact_row(
            CausalInspectionRepresentativeKind::BranchPreview,
            &branch_preview,
        ),
        artifact_row(
            CausalInspectionRepresentativeKind::HistoricalReplay,
            &replay,
        ),
        artifact_row(
            CausalInspectionRepresentativeKind::WorthStyleQueryOnlyConsumer,
            changed,
        ),
        artifact_row(
            CausalInspectionRepresentativeKind::BridgeRouteAndSignalEvidenceBindSameObservation,
            &lower_runtime_slots,
        ),
        artifact_row(
            CausalInspectionRepresentativeKind::ObservationAnchorBindsOneQueryReceipt,
            changed,
        ),
        artifact_row(
            CausalInspectionRepresentativeKind::BridgeRecordsBindThroughExistingDiagnostics,
            &lower_runtime_slots,
        ),
        artifact_row(
            CausalInspectionRepresentativeKind::SignalForensicAvailabilityAndReplayCursor,
            &lower_runtime_slots,
        ),
        artifact_row(
            CausalInspectionRepresentativeKind::CausalRichnessDoesNotChangeQueryMeaning,
            changed,
        ),
        artifact_row(
            CausalInspectionRepresentativeKind::CausalInspectionScaleHonesty,
            changed,
        ),
        failure_row(CausalInspectionRepresentativeKind::RelationalAuthorityMismatchDenied),
        failure_row(CausalInspectionRepresentativeKind::RedactionPolicyOverclaimDenied),
        failure_row(CausalInspectionRepresentativeKind::UnsupportedExplanationFamilyDenied),
        failure_row(
            CausalInspectionRepresentativeKind::DirectBridgeDiagnosticsDomainExplanationForbidden,
        ),
        failure_row(
            CausalInspectionRepresentativeKind::DirectRelationalRuntimeDomainExplanationForbidden,
        ),
        failure_row(
            CausalInspectionRepresentativeKind::DirectSignalGraphDomainExplanationForbidden,
        ),
        failure_row(CausalInspectionRepresentativeKind::DurableCausalArchiveOverclaimForbidden),
        failure_row(
            CausalInspectionRepresentativeKind::StoreBackedReplayReconstructionOverclaimForbidden,
        ),
    ])
    .expect("complete representative matrix should certify")
}

fn artifact_row(
    kind: CausalInspectionRepresentativeKind,
    artifact: &QueryCausalInspectionArtifact,
) -> CausalInspectionRepresentativeEvidence {
    CausalInspectionRepresentativeEvidence::from_query_artifact(kind, artifact).unwrap()
}

fn missing_row(
    kind: CausalInspectionRepresentativeKind,
    family: CausalEvidenceFamily,
    failure_digest: &str,
) -> CausalInspectionRepresentativeEvidence {
    CausalInspectionRepresentativeEvidence::from_missing_evidence(kind, family, failure_digest)
        .unwrap()
}

fn failure_row(kind: CausalInspectionRepresentativeKind) -> CausalInspectionRepresentativeEvidence {
    let evidence = CausalInspectionCertificationFailureEvidence::for_representative_kind(kind)
        .expect("failure row kind should have typed certification evidence");
    CausalInspectionRepresentativeEvidence::from_failure_evidence(kind, &evidence).unwrap()
}

fn missing_evidence_digest(family: CausalEvidenceFamily) -> String {
    let anchor = anchor_causal_observation(
        QueryObservationReceipt::fixture(
            CausalObservationOutcome::Changed,
            vec![CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::QueryInspection,
                crate::runtime::tests::causal_inspection::causal_test_reference_digest(format!(
                    "query-inspection:missing-{}",
                    family.as_str()
                )),
            )],
        ),
        CausalInspectionReason::ChangedResult,
    )
    .expect("missing-evidence fixture should anchor");
    let CausalEvidenceReferenceResolution::MissingRequiredEvidence { denial, .. } =
        resolve_causal_evidence_references(anchor, &[family])
    else {
        panic!("fixture should miss requested evidence family");
    };
    denial.failure_digest().to_string()
}
