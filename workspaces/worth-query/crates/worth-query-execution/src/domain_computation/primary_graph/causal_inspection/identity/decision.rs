use super::*;

pub(in crate::domain_computation::primary_graph::causal_inspection) fn compose_causal_admission_decision_identity(
    decision: &CausalInspectionAdmissionDecision,
) -> CausalInspectionAdmissionDecisionIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::CausalInspectionAdmissionDecision)
        .field_shape(WorthQueryEvidenceTag::new("kind"), decision.kind().as_str())
        .optional_shape(
            WorthQueryEvidenceTag::new("advisory"),
            decision.advisory_kind().map(|kind| kind.as_str()),
        )
        .optional_shape(
            WorthQueryEvidenceTag::new("violation"),
            decision.violation_kind().map(|kind| kind.as_str()),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("richness"),
            decision.admitted_richness().as_str(),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("families"),
            decision
                .permitted_evidence_families()
                .iter()
                .map(CausalEvidenceFamily::as_str),
        )
        .seal()
        .into()
}

pub(in crate::domain_computation::primary_graph::causal_inspection) fn compose_causal_decision_trace_row_identity(
    row: &CausalDecisionTraceRow,
) -> CausalInspectionDecisionTraceRowIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::CausalInspectionDecisionTraceRow)
        .field_shape(WorthQueryEvidenceTag::new("key"), row.key())
        .field_shape(WorthQueryEvidenceTag::new("span"), row.span())
        .field_shape(
            WorthQueryEvidenceTag::new("decision"),
            row.decision().as_str(),
        )
        .field_shape(WorthQueryEvidenceTag::new("authority"), row.authority())
        .field_value(WorthQueryEvidenceTag::new("reason"), row.reason())
        .seal()
        .into()
}

pub(in crate::domain_computation::primary_graph::causal_inspection) fn compose_causal_decision_trace_identity(
    rows: &[CausalDecisionTraceRow],
) -> CausalInspectionDecisionTraceIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::CausalInspectionDecisionTraceIndex)
        .field_evidence_identity_sequence(
            WorthQueryEvidenceTag::new("rows"),
            rows.iter().map(CausalDecisionTraceRow::evidence_identity),
        )
        .seal()
        .into()
}

pub(in crate::domain_computation::primary_graph::causal_inspection) fn compose_causal_admission_counters_identity(
    counters: &CausalInspectionAdmissionCounters,
) -> CausalInspectionAdmissionCountersIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::CausalInspectionAdmissionCounters)
        .field_usize(
            WorthQueryEvidenceTag::new("proof_transition_count"),
            counters.causal_inspection_proof_transition_count(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("proof_outcome_count"),
            counters.causal_inspection_proof_outcome_count(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("proof_readmission_count"),
            counters.causal_inspection_proof_readmission_count(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("request_count"),
            counters.causal_inspection_request_count(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("admission_count"),
            counters.causal_inspection_admission_count(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("advisory_count"),
            counters.causal_inspection_advisory_count(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("denial_count"),
            counters.causal_inspection_denial_count(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("trace_lookup_count"),
            counters.causal_decision_trace_lookup_count(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("trace_hit_count"),
            counters.causal_decision_trace_index_hit_count(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("bridge_envelope_request_count"),
            counters.bridge_causal_envelope_request_count(),
        )
        .seal()
        .into()
}

pub(in crate::domain_computation::primary_graph::causal_inspection) fn compose_causal_admission_receipt_identity(
    receipt: &CausalInspectionAdmissionReceipt,
) -> CausalInspectionAdmissionReceiptIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::CausalInspectionAdmissionReceipt)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("subject"),
            receipt.subject_identity().evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("decision"),
            receipt.decision_identity().evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("trace"),
            receipt.decision_trace_identity().evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("counters"),
            receipt.counter_identity().evidence_identity(),
        )
        .seal()
        .into()
}
