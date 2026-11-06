use super::*;

pub(in crate::runtime) fn runtime_remask_posture_identity(
    disposition_kind: WorthQueryRuntimeRemaskDispositionKind,
    reason_kind: WorthQueryRuntimeRemaskReasonKind,
    support_identity: &WorthQueryEvidenceIdentity,
    basis_identity: &WorthQueryEvidenceIdentity,
    policy_identity: &WorthQueryEvidenceIdentity,
    tenant_truth_identity: &WorthQueryEvidenceIdentity,
    tenant_schema_identity: &WorthQueryEvidenceIdentity,
    relationship_proof_identity: &WorthQueryEvidenceIdentity,
    schema_context_identity: &WorthQueryEvidenceIdentity,
) -> Result<
    WorthQueryEvidenceIdentity,
    crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial,
> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_runtime_remask_posture_v1",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("disposition"),
                disposition_kind.as_str(),
            )
            .field_shape(WorthQueryEvidenceTag::new("reason"), reason_kind.as_str())
            .field_evidence_identity(WorthQueryEvidenceTag::new("support"), support_identity)
            .field_evidence_identity(WorthQueryEvidenceTag::new("basis"), basis_identity)
            .field_evidence_identity(WorthQueryEvidenceTag::new("policy"), policy_identity)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("tenant_truth"),
                tenant_truth_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("tenant_schema"),
                tenant_schema_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("relationship_proof"),
                relationship_proof_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("schema_context"),
                schema_context_identity,
            )
            .seal()?,
    )
}

pub(in crate::runtime) fn runtime_downstream_resume_posture_identity(
    kind: WorthQueryRuntimeDownstreamResumePostureKind,
    required_basis_identity: Option<&WorthQueryEvidenceIdentity>,
    support_posture: WorthQueryLowerRuntimeSupportPosture,
    support_identity: &WorthQueryEvidenceIdentity,
) -> Result<
    WorthQueryEvidenceIdentity,
    crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial,
> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_runtime_downstream_resume_posture_v2",
            )
            .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("required_basis"),
                required_basis_identity,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("support_posture"),
                support_posture.as_str(),
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("support"), support_identity)
            .seal()?,
    )
}
