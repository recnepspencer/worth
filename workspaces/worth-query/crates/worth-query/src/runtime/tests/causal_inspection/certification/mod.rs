mod artifact_support;
mod closure_support;
mod matrix_support;
mod proof_shape;
mod row_digest;
mod slot_support;
mod writeback_support;

use super::super::super::*;
use artifact_support::{
    admitted_artifact, advisory_artifacts, denied_artifact_and_missing_evidence,
};
pub(in crate::runtime::tests) use closure_support::runtime_backed_causal_certification_bundle;
use matrix_support::representative_matrix;

#[test]
fn causal_inspection_certification_bundle_closes_runtime_backed_rows() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let changed = admitted_artifact(
        super::causal_truth_commit_identity("commit-query-cert-changed"),
        resource_request,
    );
    let (full, redacted) = advisory_artifacts(
        super::causal_truth_commit_identity("commit-query-cert-redacted"),
        resource_request,
    );
    let (denied, missing_evidence_digest) = denied_artifact_and_missing_evidence(resource_request);
    let representatives = representative_matrix(&changed, &redacted, &denied, resource_request);
    assert_eq!(representatives.representative_digests().len(), 25);
    assert_eq!(representatives.missing_evidence_row_count(), 3);
    let boundary_audit =
        CausalInspectionBoundaryAudit::from_query_artifact_public_surface(&changed);
    let proof_shape = CausalInspectionProofShapeCertification::from_runtime_path(
        &changed,
        &representatives,
        &boundary_audit,
    );
    let proof_shape_digest = proof_shape.proof_shape_digest().to_string();
    let phase_progression_digest = proof_shape.phase_progression_digest().to_string();
    let witness_authority_digest = proof_shape.witness_authority_digest().to_string();
    let bridge_readmission_proof_digest = changed
        .bridge_readmission_proof_for_reporting()
        .expect("changed artifact should carry bridge readmission proof")
        .to_string();
    let small = CausalInspectionScaleCounterSnapshot::from_artifact(
        CausalInspectionScaleFixtureSize::Small,
        &changed,
    );
    let medium = CausalInspectionScaleCounterSnapshot::from_artifact(
        CausalInspectionScaleFixtureSize::Medium,
        &changed,
    );
    let large = CausalInspectionScaleCounterSnapshot::from_artifact(
        CausalInspectionScaleFixtureSize::Large,
        &changed,
    );

    let scope = build_causal_inspection_certification_scope(
        &changed,
        &full,
        &redacted,
        &denied,
        &missing_evidence_digest,
        boundary_audit,
        representatives,
        proof_shape,
        small,
        medium,
        large,
    )
    .expect("complete runtime-backed rows should build certification scope");
    let artifact_serialization_slope_digest = scope
        .performance_certification()
        .artifact_serialization_slope_digest()
        .to_string();
    let scale_slope_digest = scope
        .performance_certification()
        .scale_slope_digest()
        .to_string();
    let anchor_derivation_slope_digest = scope
        .performance_certification()
        .anchor_derivation_slope_digest()
        .to_string();
    let reference_resolution_slope_digest = scope
        .performance_certification()
        .reference_resolution_slope_digest()
        .to_string();
    let admission_slope_digest = scope
        .performance_certification()
        .admission_slope_digest()
        .to_string();
    let bridge_envelope_slope_digest = scope
        .performance_certification()
        .bridge_envelope_slope_digest()
        .to_string();
    let materialization_slope_digest = scope
        .performance_certification()
        .materialization_slope_digest()
        .to_string();
    let bundle = certify_causal_inspection_runtime_path(scope);

    assert_eq!(bundle.certification_row_count(), 35);
    assert_eq!(bundle.hostile_row_count(), 10);
    assert_eq!(bundle.representative_row_count(), 25);
    assert_eq!(bundle.scale_fixture_row_count(), 3);
    assert!(!bundle.certification_bundle_digest().is_empty());
    assert!(!bundle.performance_certification_digest().is_empty());
    assert!(!bundle.representative_matrix_digest().is_empty());
    assert_eq!(
        bundle.bridge_readmission_proof_digest(),
        bridge_readmission_proof_digest
    );
    assert_eq!(bundle.scale_slope_digest(), scale_slope_digest);
    assert_eq!(
        bundle.anchor_derivation_slope_digest(),
        anchor_derivation_slope_digest
    );
    assert_eq!(
        bundle.reference_resolution_slope_digest(),
        reference_resolution_slope_digest
    );
    assert_eq!(bundle.admission_slope_digest(), admission_slope_digest);
    assert_eq!(
        bundle.bridge_envelope_slope_digest(),
        bridge_envelope_slope_digest
    );
    assert_eq!(
        bundle.materialization_slope_digest(),
        materialization_slope_digest
    );
    assert_eq!(
        bundle.artifact_serialization_slope_digest(),
        artifact_serialization_slope_digest
    );
    assert_eq!(bundle.proof_shape_digest(), proof_shape_digest);
    assert_eq!(bundle.phase_progression_digest(), phase_progression_digest);
    assert_eq!(bundle.witness_authority_digest(), witness_authority_digest);
}

#[test]
fn causal_inspection_certification_rejects_bridge_envelope_slope_drift() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let changed = admitted_artifact(
        super::causal_truth_commit_identity("commit-query-cert-bridge-slope-changed"),
        resource_request,
    );
    let (full, redacted) = advisory_artifacts(
        super::causal_truth_commit_identity("commit-query-cert-bridge-slope-redacted"),
        resource_request,
    );
    let (denied, missing_evidence_digest) = denied_artifact_and_missing_evidence(resource_request);
    let representatives = representative_matrix(&changed, &redacted, &denied, resource_request);
    let boundary_audit =
        CausalInspectionBoundaryAudit::from_query_artifact_public_surface(&changed);
    let proof_shape = CausalInspectionProofShapeCertification::from_runtime_path(
        &changed,
        &representatives,
        &boundary_audit,
    );
    let small = CausalInspectionScaleCounterSnapshot::from_artifact(
        CausalInspectionScaleFixtureSize::Small,
        &changed,
    )
    .with_bridge_envelope_slope_for_tests(2);
    let medium = CausalInspectionScaleCounterSnapshot::from_artifact(
        CausalInspectionScaleFixtureSize::Medium,
        &changed,
    );
    let large = CausalInspectionScaleCounterSnapshot::from_artifact(
        CausalInspectionScaleFixtureSize::Large,
        &changed,
    );

    let error = build_causal_inspection_certification_scope(
        &changed,
        &full,
        &redacted,
        &denied,
        &missing_evidence_digest,
        boundary_audit,
        representatives,
        proof_shape,
        small,
        medium,
        large,
    )
    .unwrap_err();

    assert_eq!(
        error.kind(),
        CausalInspectionCertificationErrorKind::ScaleSlopeDrift
    );
}

#[test]
fn causal_inspection_certification_rejects_redaction_identity_drift() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let changed = admitted_artifact(
        super::causal_truth_commit_identity("commit-query-cert-drift-changed"),
        resource_request,
    );
    let unrelated_full = admitted_artifact(
        super::causal_truth_commit_identity("commit-query-cert-drift-unrelated"),
        resource_request,
    );
    let (_, redacted) = advisory_artifacts(
        super::causal_truth_commit_identity("commit-query-cert-drift-redacted"),
        resource_request,
    );
    let (denied, missing_evidence_digest) = denied_artifact_and_missing_evidence(resource_request);
    let representatives = representative_matrix(&changed, &redacted, &denied, resource_request);
    let boundary_audit =
        CausalInspectionBoundaryAudit::from_query_artifact_public_surface(&changed);
    let proof_shape = CausalInspectionProofShapeCertification::from_runtime_path(
        &changed,
        &representatives,
        &boundary_audit,
    );
    let small = CausalInspectionScaleCounterSnapshot::from_artifact(
        CausalInspectionScaleFixtureSize::Small,
        &changed,
    );
    let medium = CausalInspectionScaleCounterSnapshot::from_artifact(
        CausalInspectionScaleFixtureSize::Medium,
        &changed,
    );
    let large = CausalInspectionScaleCounterSnapshot::from_artifact(
        CausalInspectionScaleFixtureSize::Large,
        &changed,
    );

    let error = build_causal_inspection_certification_scope(
        &changed,
        &unrelated_full,
        &redacted,
        &denied,
        &missing_evidence_digest,
        boundary_audit,
        representatives,
        proof_shape,
        small,
        medium,
        large,
    )
    .unwrap_err();

    assert_eq!(
        error.kind(),
        CausalInspectionCertificationErrorKind::RedactionIdentityDrift
    );
}

#[test]
fn causal_inspection_certification_rejects_incomplete_representative_matrix() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let changed = admitted_artifact(
        super::causal_truth_commit_identity("commit-query-cert-matrix-changed"),
        resource_request,
    );
    let rows = [CausalInspectionRepresentativeEvidence::from_query_artifact(
        CausalInspectionRepresentativeKind::ChangedResult,
        &changed,
    )
    .unwrap()];

    let error = CausalInspectionRepresentativeMatrix::from_representatives(&rows).unwrap_err();

    assert_eq!(
        error.kind(),
        CausalInspectionCertificationErrorKind::MissingRepresentativeMatrixRow
    );
}

#[test]
fn causal_inspection_certification_rejects_mislabeled_missing_evidence() {
    let error = CausalInspectionRepresentativeEvidence::from_missing_evidence(
        CausalInspectionRepresentativeKind::MissingBridgeRouteEvidenceDenied,
        CausalEvidenceFamily::SignalInvalidation,
        "typed-failure-digest",
    )
    .unwrap_err();

    assert!(matches!(
        error.kind(),
        CausalInspectionCertificationErrorKind::RepresentativeMatrixMismatch
    ));
}

#[test]
fn causal_inspection_certification_rejects_mislabeled_failure_row() {
    let error = CausalInspectionRepresentativeEvidence::from_failure(
        CausalInspectionRepresentativeKind::ChangedResult,
        "typed-failure-digest",
    )
    .unwrap_err();

    assert!(matches!(
        error.kind(),
        CausalInspectionCertificationErrorKind::RepresentativeMatrixMismatch
    ));
}

#[test]
fn causal_inspection_certification_requires_typed_failure_evidence_digest() {
    let error = CausalInspectionRepresentativeEvidence::from_failure(
        CausalInspectionRepresentativeKind::DurableCausalArchiveOverclaimForbidden,
        "synthetic-durable-overclaim-digest",
    )
    .unwrap_err();

    assert_eq!(
        error.kind(),
        CausalInspectionCertificationErrorKind::RepresentativeMatrixMismatch
    );
}

#[test]
fn causal_inspection_certification_rejects_swapped_failure_evidence() {
    let evidence = CausalInspectionCertificationFailureEvidence::for_representative_kind(
        CausalInspectionRepresentativeKind::DirectBridgeDiagnosticsDomainExplanationForbidden,
    )
    .unwrap();

    let error = CausalInspectionRepresentativeEvidence::from_failure_evidence(
        CausalInspectionRepresentativeKind::DirectSignalGraphDomainExplanationForbidden,
        &evidence,
    )
    .unwrap_err();

    assert_eq!(
        error.kind(),
        CausalInspectionCertificationErrorKind::RepresentativeMatrixMismatch
    );
}

#[test]
fn causal_inspection_certification_failure_evidence_names_forbidden_and_debt_posture() {
    let direct = CausalInspectionCertificationFailureEvidence::for_representative_kind(
        CausalInspectionRepresentativeKind::DirectRelationalRuntimeDomainExplanationForbidden,
    )
    .unwrap();
    let durable = CausalInspectionCertificationFailureEvidence::for_representative_kind(
        CausalInspectionRepresentativeKind::DurableCausalArchiveOverclaimForbidden,
    )
    .unwrap();

    assert_eq!(
        direct.source(),
        CausalInspectionCertificationFailureSource::PublicBoundaryAudit
    );
    assert!(direct.ordinary_path_forbidden());
    assert!(!direct.later_milestone_debt());
    assert_eq!(
        durable.source(),
        CausalInspectionCertificationFailureSource::LaterMilestoneDebt
    );
    assert!(durable.ordinary_path_forbidden());
    assert!(durable.later_milestone_debt());
    assert_ne!(direct.failure_digest(), durable.failure_digest());
}

#[test]
fn causal_inspection_certification_rejects_rich_slot_row_without_named_slots() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let changed = admitted_artifact(
        super::causal_truth_commit_identity("commit-query-cert-thin-rich-row"),
        resource_request,
    );
    for kind in [
        CausalInspectionRepresentativeKind::BridgeRouteAndSignalEvidenceBindSameObservation,
        CausalInspectionRepresentativeKind::BridgeRecordsBindThroughExistingDiagnostics,
        CausalInspectionRepresentativeKind::SignalForensicAvailabilityAndReplayCursor,
    ] {
        let error = CausalInspectionRepresentativeEvidence::from_query_artifact(kind, &changed)
            .unwrap_err();

        assert_eq!(
            error.kind(),
            CausalInspectionCertificationErrorKind::RepresentativeMatrixMismatch
        );
    }
}
