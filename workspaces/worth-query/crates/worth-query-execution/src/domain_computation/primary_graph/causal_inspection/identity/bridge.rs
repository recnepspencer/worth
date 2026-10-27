use super::*;

pub fn compose_bridge_causal_envelope_identity(
    identity: &BridgeCausalEnvelopeIdentity,
) -> WorthQueryEvidenceIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::CausalInspectionArtifact)
        .field_shape(WorthQueryEvidenceTag::new("role"), "bridge-causal-envelope")
        .field_bridge_retained_evidence_identity(
            WorthQueryEvidenceTag::new("request"),
            identity.request_evidence_identity(),
        )
        .field_bridge_retained_evidence_identity(
            WorthQueryEvidenceTag::new("anchor"),
            identity.causal_observation_anchor_evidence_identity(),
        )
        .field_bridge_retained_evidence_identity(
            WorthQueryEvidenceTag::new("bindings"),
            identity.evidence_binding_evidence_identity(),
        )
        .field_bridge_retained_evidence_identity(
            WorthQueryEvidenceTag::new("counters"),
            identity.counter_evidence_identity(),
        )
        .field_bridge_retained_evidence_identity(
            WorthQueryEvidenceTag::new("identity"),
            identity.envelope_evidence_identity(),
        )
        .seal()
}

pub fn compose_bridge_causal_explanation_envelope_identity(
    envelope: &BridgeCausalExplanationEnvelope,
) -> WorthQueryEvidenceIdentity {
    let envelope_identity = compose_bridge_causal_envelope_identity(envelope.identity());
    worth_query_evidence_identity(WorthQueryEvidenceScope::CausalInspectionArtifact)
        .field_shape(
            WorthQueryEvidenceTag::new("role"),
            "bridge-causal-explanation-envelope",
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("identity"), &envelope_identity)
        .field_shape(
            WorthQueryEvidenceTag::new("summary_kind"),
            bridge_causal_admission_summary_kind_label(envelope.admission_summary_kind()),
        )
        .field_bridge_retained_evidence_identity(
            WorthQueryEvidenceTag::new("admission_summary"),
            envelope.admission_summary_evidence_identity(),
        )
        .field_bridge_retained_evidence_identity(
            WorthQueryEvidenceTag::new("request"),
            envelope.request_evidence_identity(),
        )
        .field_bridge_retained_evidence_identity(
            WorthQueryEvidenceTag::new("anchor"),
            envelope.causal_observation_anchor_evidence_identity(),
        )
        .field_bridge_retained_evidence_identity(
            WorthQueryEvidenceTag::new("envelope"),
            envelope.envelope_evidence_identity(),
        )
        .seal()
}

pub fn bridge_causal_admission_summary_kind_label(
    kind: BridgeCausalInspectionAdmissionSummaryKind,
) -> &'static str {
    match kind {
        BridgeCausalInspectionAdmissionSummaryKind::Admitted => "admitted",
        BridgeCausalInspectionAdmissionSummaryKind::Advisory => "advisory",
    }
}

pub fn compose_bridge_causal_envelope_receipt_identity(
    receipt: &BridgeCausalEnvelopeReceipt,
) -> WorthQueryEvidenceIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::CausalInspectionArtifact)
        .field_shape(
            WorthQueryEvidenceTag::new("role"),
            "bridge-causal-envelope-receipt",
        )
        .field_bridge_retained_evidence_identity(
            WorthQueryEvidenceTag::new("envelope_identity"),
            receipt.envelope_identity_evidence(),
        )
        .field_bridge_retained_evidence_identity(
            WorthQueryEvidenceTag::new("envelope"),
            receipt.envelope_evidence_identity(),
        )
        .field_bridge_retained_evidence_identity(
            WorthQueryEvidenceTag::new("counters"),
            receipt.counter_evidence_identity(),
        )
        .field_bridge_retained_evidence_identity(
            WorthQueryEvidenceTag::new("receipt"),
            receipt.receipt_evidence_identity(),
        )
        .seal()
}

pub fn compose_bridge_causal_denial_identity(
    denial: &BridgeCausalEnvelopeDenial,
) -> WorthQueryEvidenceIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::CausalInspectionDeniedArtifactDetail)
        .field_shape(WorthQueryEvidenceTag::new("role"), "bridge-causal-denial")
        .field_shape(WorthQueryEvidenceTag::new("kind"), denial.kind().as_str())
        .field_shape(
            WorthQueryEvidenceTag::new("family"),
            denial.family().as_str(),
        )
        .field_bridge_retained_evidence_identity(
            WorthQueryEvidenceTag::new("failure"),
            &denial.failure_evidence_identity(),
        )
        .seal()
}
