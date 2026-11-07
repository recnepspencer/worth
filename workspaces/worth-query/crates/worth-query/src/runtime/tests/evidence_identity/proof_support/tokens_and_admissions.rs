use super::super::super::support::*;

pub(in super::super) fn assert_canonical_evidence_identity_token(token: impl AsRef<str>) {
    let token = token.as_ref();
    assert!(
        token.starts_with("worth.query.evidence-identity.v1:sha256:"),
        "expected canonical evidence identity token, got {token}"
    );
}
pub(in super::super) fn compose_basis_admission_identity(
    scope: crate::WorthQueryEvidenceScope,
    label: &WorthQuerySessionLabel,
    effect_policy: WorthQueryEffectPolicy,
    authority_lane: WorthQueryAuthorityLane,
    evidence: impl IntoIterator<Item = impl Into<String>>,
) -> crate::WorthQueryEvidenceIdentity {
    let evidence_rows = WorthQueryBasisAdmissionEvidenceRow::rows_from_values(evidence);
    crate::WorthQueryEvidenceIdentity::compose(scope)
        .field_value(
            crate::WorthQueryEvidenceTag::new("session_label_identity"),
            label.identity_digest().as_str(),
        )
        .field_shape(
            crate::WorthQueryEvidenceTag::new("effect_policy"),
            effect_policy.as_str(),
        )
        .field_shape(
            crate::WorthQueryEvidenceTag::new("authority_lane"),
            authority_lane.as_str(),
        )
        .field_value_sequence(
            crate::WorthQueryEvidenceTag::new("basis_evidence"),
            evidence_rows.iter().map(|row| row.row_digest().as_str()),
        )
        .seal()
        .expect("fixture digest must fit")
}

pub(in super::super) fn compose_receipt_identity(
    scope: crate::WorthQueryEvidenceScope,
    admission_identity: &crate::WorthQueryEvidenceIdentity,
    posture: &str,
) -> crate::WorthQueryEvidenceIdentity {
    crate::WorthQueryEvidenceIdentity::compose(scope)
        .field_evidence_identity(
            crate::WorthQueryEvidenceTag::new("admission_identity"),
            admission_identity,
        )
        .field_shape(crate::WorthQueryEvidenceTag::new("posture"), posture)
        .seal()
        .expect("fixture digest must fit")
}

pub(in super::super) fn compose_denial_evidence_identity(
    evidence: &WorthQueryIntentDenialEvidence,
) -> crate::WorthQueryEvidenceIdentity {
    crate::WorthQueryEvidenceIdentity::compose(crate::WorthQueryEvidenceScope::IntentDenialEvidence)
        .field_shape(
            crate::WorthQueryEvidenceTag::new("intent_name"),
            evidence.intent_name(),
        )
        .field_shape(crate::WorthQueryEvidenceTag::new("stage"), evidence.stage())
        .field_value(
            crate::WorthQueryEvidenceTag::new("message"),
            evidence.message(),
        )
        .field_shape(
            crate::WorthQueryEvidenceTag::new("strategy_identity"),
            evidence.strategy_identity(),
        )
        .field_shape(
            crate::WorthQueryEvidenceTag::new("strategy_version"),
            evidence.strategy_version(),
        )
        .optional_value(
            crate::WorthQueryEvidenceTag::new("returned_strategy_identity"),
            evidence.returned_strategy_identity(),
        )
        .optional_value(
            crate::WorthQueryEvidenceTag::new("returned_strategy_version"),
            evidence.returned_strategy_version(),
        )
        .optional_identity(
            crate::WorthQueryEvidenceTag::new("returned_strategy_descriptor_digest"),
            evidence.returned_strategy_descriptor_digest(),
        )
        .field_value(
            crate::WorthQueryEvidenceTag::new("canonical_input_digest"),
            evidence.canonical_input_digest(),
        )
        .field_shape(
            crate::WorthQueryEvidenceTag::new("source_lane"),
            evidence.source_lane().as_str(),
        )
        .field_shape(
            crate::WorthQueryEvidenceTag::new("target_lane"),
            evidence.target_lane().as_str(),
        )
        .optional_shape(
            crate::WorthQueryEvidenceTag::new("execution_kind"),
            evidence
                .execution_kind()
                .map(WorthQueryIntentExecutionKind::as_str),
        )
        .optional_identity(
            crate::WorthQueryEvidenceTag::new("attempt_digest"),
            evidence.attempt_digest(),
        )
        .field_value_sequence(
            crate::WorthQueryEvidenceTag::new("invariant_evidence"),
            evidence.invariant_evidence().iter().map(String::as_str),
        )
        .optional_evidence_identity(
            crate::WorthQueryEvidenceTag::new("snapshot_identity"),
            evidence.snapshot_evidence_identity().as_ref(),
        )
        .optional_identity(
            crate::WorthQueryEvidenceTag::new("execution_provenance"),
            evidence
                .execution_provenance()
                .map(WorthQueryIntentExecutionProvenance::execution_provenance_chain_digest),
        )
        .optional_identity(
            crate::WorthQueryEvidenceTag::new("decision_trace_digest"),
            evidence
                .decision_trace_envelope()
                .map(WorthQueryIntentDecisionTraceEnvelope::trace_digest),
        )
        .seal()
        .expect("fixture digest must fit")
}
