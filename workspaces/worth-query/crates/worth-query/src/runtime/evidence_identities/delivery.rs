use super::*;

pub(in crate::runtime) struct RuntimeDownstreamDeliveryIdentityParts<'a> {
    pub view_name: &'a str,
    pub delivery_batch_identity: &'a WorthQueryEvidenceIdentity,
    pub delivery_class: WorthQueryRuntimeDownstreamDeliveryClass,
    pub delivery_cause_kind: QuerySubscriptionDeliveryCauseKind,
    pub delivery_cause_identity: &'a WorthQueryEvidenceIdentity,
    pub sequence: u64,
    pub basis_identity: &'a WorthQueryEvidenceIdentity,
    pub support_posture: WorthQueryRuntimeDownstreamSupportPosture,
    pub support_identity: &'a WorthQueryEvidenceIdentity,
    pub mixed_cause_identity: Option<&'a WorthQueryEvidenceIdentity>,
    pub async_result_state_identity: Option<&'a WorthQueryEvidenceIdentity>,
    pub remask_identity: Option<&'a WorthQueryEvidenceIdentity>,
    pub runtime_resume_support_identity: &'a WorthQueryEvidenceIdentity,
    pub durable_resume_support_identity: &'a WorthQueryEvidenceIdentity,
}

pub(in crate::runtime) fn runtime_downstream_delivery_identity(
    parts: RuntimeDownstreamDeliveryIdentityParts<'_>,
) -> Result<WorthQueryEvidenceIdentity, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
    Ok(worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "worth_query_runtime_downstream_delivery_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("view"), parts.view_name)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("batch"),
            parts.delivery_batch_identity,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("class"),
            parts.delivery_class.as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("cause"),
            parts.delivery_cause_kind.as_str(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("cause_digest"),
            parts.delivery_cause_identity,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("sequence"),
            parts.sequence as usize,
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("basis"), parts.basis_identity)
        .field_shape(
            WorthQueryEvidenceTag::new("support_posture"),
            parts.support_posture.as_str(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("support"),
            parts.support_identity,
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("mixed_cause"),
            parts.mixed_cause_identity,
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("async_result_state"),
            parts.async_result_state_identity,
        )
        .optional_evidence_identity(WorthQueryEvidenceTag::new("remask"), parts.remask_identity)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("runtime_resume"),
            parts.runtime_resume_support_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("durable_resume"),
            parts.durable_resume_support_identity,
        )
        .seal()?)
}
