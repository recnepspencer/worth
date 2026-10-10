use super::artifact_support::{
    admitted_artifact, advisory_artifacts, denied_artifact_and_missing_evidence,
};
use super::matrix_support::representative_matrix;
use super::*;

pub(in crate::runtime::tests) fn runtime_backed_causal_certification_bundle(
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> CausalInspectionCertificationBundle {
    let changed = admitted_artifact(
        super::super::causal_truth_commit_identity("commit-query-cert-changed"),
        resource_request,
    );
    let (full, redacted) = advisory_artifacts(
        super::super::causal_truth_commit_identity("commit-query-cert-redacted"),
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

    certify_causal_inspection_runtime_path(scope)
}
