use std::collections::HashMap;

use super::identity::{
    compose_causal_admission_counters_identity, compose_causal_admission_receipt_identity,
    compose_causal_decision_trace_identity, compose_causal_decision_trace_row_identity,
    CausalInspectionAdmissionCountersIdentity, CausalInspectionAdmissionDecisionIdentity,
    CausalInspectionAdmissionReceiptIdentity, CausalInspectionAdmissionSubjectIdentity,
    CausalInspectionDecisionTraceIdentity, CausalInspectionDecisionTraceRowIdentity,
};

use super::admission_decision::{
    CausalInspectionAdmissionDecision, CausalInspectionAdmissionDecisionKind,
    CausalInspectionAdmissionSubject,
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct CausalDecisionTraceLookupKey {
    value: String,
}

impl CausalDecisionTraceLookupKey {
    fn from_key(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalDecisionTraceRow {
    key: String,
    span: String,
    decision: CausalInspectionAdmissionDecisionKind,
    authority: String,
    reason: String,
    row_identity: CausalInspectionDecisionTraceRowIdentity,
}

impl CausalDecisionTraceRow {
    pub(super) fn new(
        key: &'static str,
        span: &'static str,
        decision: CausalInspectionAdmissionDecisionKind,
        authority: &'static str,
        reason: &'static str,
    ) -> Self {
        let mut row = Self {
            key: key.to_string(),
            span: span.to_string(),
            decision,
            authority: authority.to_string(),
            reason: reason.to_string(),
            row_identity: CausalInspectionDecisionTraceRowIdentity::from(
                crate::domain_computation::evidence_identity::WorthQueryEvidenceIdentity::compose(
                    crate::domain_computation::evidence_identity::WorthQueryEvidenceScope::CausalInspectionDecisionTraceRow,
                )
                .seal(),
            ),
        };
        row.row_identity = compose_causal_decision_trace_row_identity(&row);
        row
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn span(&self) -> &str {
        &self.span
    }

    pub fn decision(&self) -> CausalInspectionAdmissionDecisionKind {
        self.decision
    }

    pub fn authority(&self) -> &str {
        &self.authority
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }

    pub fn row_for_reporting(&self) -> &str {
        self.row_identity.as_str()
    }

    pub fn evidence_identity(
        &self,
    ) -> &crate::domain_computation::evidence_identity::WorthQueryEvidenceIdentity {
        self.row_identity.evidence_identity()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalDecisionTraceIndex {
    rows: Vec<CausalDecisionTraceRow>,
    lookup: HashMap<CausalDecisionTraceLookupKey, usize>,
    trace_identity: CausalInspectionDecisionTraceIdentity,
}

impl CausalDecisionTraceIndex {
    pub(super) fn new(rows: Vec<CausalDecisionTraceRow>) -> Self {
        let lookup = rows
            .iter()
            .enumerate()
            .map(|(index, row)| (CausalDecisionTraceLookupKey::from_key(row.key()), index))
            .collect();
        let mut trace = Self {
            rows,
            lookup,
            trace_identity: CausalInspectionDecisionTraceIdentity::from(
                crate::domain_computation::evidence_identity::WorthQueryEvidenceIdentity::compose(
                    crate::domain_computation::evidence_identity::WorthQueryEvidenceScope::CausalInspectionDecisionTraceIndex,
                )
                .seal(),
            ),
        };
        trace.trace_identity = compose_causal_decision_trace_identity(trace.rows());
        trace
    }

    pub fn rows(&self) -> &[CausalDecisionTraceRow] {
        &self.rows
    }

    pub fn row_for_key(&self, key: &str) -> Option<&CausalDecisionTraceRow> {
        self.lookup
            .get(&CausalDecisionTraceLookupKey::from_key(key))
            .and_then(|index| self.rows.get(*index))
    }

    pub fn trace_for_reporting(&self) -> &str {
        self.trace_identity.as_str()
    }

    pub(super) fn trace_identity(&self) -> &CausalInspectionDecisionTraceIdentity {
        &self.trace_identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalInspectionAdmissionCounters {
    causal_inspection_proof_transition_count: usize,
    causal_inspection_proof_outcome_count: usize,
    causal_inspection_proof_readmission_count: usize,
    causal_inspection_request_count: usize,
    causal_inspection_admission_count: usize,
    causal_inspection_advisory_count: usize,
    causal_inspection_denial_count: usize,
    causal_decision_trace_lookup_count: usize,
    causal_decision_trace_index_hit_count: usize,
    bridge_causal_envelope_request_count: usize,
    counter_identity: CausalInspectionAdmissionCountersIdentity,
}

impl CausalInspectionAdmissionCounters {
    pub(super) fn new(
        kind: CausalInspectionAdmissionDecisionKind,
        trace: &CausalDecisionTraceIndex,
    ) -> Self {
        let advisory_count = usize::from(kind == CausalInspectionAdmissionDecisionKind::Advisory);
        let denial_count = usize::from(kind == CausalInspectionAdmissionDecisionKind::Violation);
        let trace_lookup_count = trace.rows().len();
        let trace_hit_count = trace
            .rows()
            .iter()
            .filter(|row| trace.row_for_key(row.key()).is_some())
            .count();
        let mut counters = Self {
            causal_inspection_proof_transition_count: 1,
            causal_inspection_proof_outcome_count: 1,
            causal_inspection_proof_readmission_count: 0,
            causal_inspection_request_count: 1,
            causal_inspection_admission_count: 1,
            causal_inspection_advisory_count: advisory_count,
            causal_inspection_denial_count: denial_count,
            causal_decision_trace_lookup_count: trace_lookup_count,
            causal_decision_trace_index_hit_count: trace_hit_count,
            bridge_causal_envelope_request_count: 0,
            counter_identity: CausalInspectionAdmissionCountersIdentity::from(
                crate::domain_computation::evidence_identity::WorthQueryEvidenceIdentity::compose(
                    crate::domain_computation::evidence_identity::WorthQueryEvidenceScope::CausalInspectionAdmissionCounters,
                )
                .seal(),
            ),
        };
        counters.counter_identity = compose_causal_admission_counters_identity(&counters);
        counters
    }

    pub fn causal_inspection_proof_transition_count(&self) -> usize {
        self.causal_inspection_proof_transition_count
    }

    pub fn causal_inspection_proof_outcome_count(&self) -> usize {
        self.causal_inspection_proof_outcome_count
    }

    pub fn causal_inspection_proof_readmission_count(&self) -> usize {
        self.causal_inspection_proof_readmission_count
    }

    pub fn causal_inspection_request_count(&self) -> usize {
        self.causal_inspection_request_count
    }

    pub fn causal_inspection_admission_count(&self) -> usize {
        self.causal_inspection_admission_count
    }

    pub fn causal_inspection_advisory_count(&self) -> usize {
        self.causal_inspection_advisory_count
    }

    pub fn causal_inspection_denial_count(&self) -> usize {
        self.causal_inspection_denial_count
    }

    pub fn causal_decision_trace_lookup_count(&self) -> usize {
        self.causal_decision_trace_lookup_count
    }

    pub fn causal_decision_trace_index_hit_count(&self) -> usize {
        self.causal_decision_trace_index_hit_count
    }

    pub fn bridge_causal_envelope_request_count(&self) -> usize {
        self.bridge_causal_envelope_request_count
    }

    pub fn counter_snapshot(&self) -> &str {
        self.counter_identity.as_str()
    }

    pub(super) fn counter_identity(&self) -> &CausalInspectionAdmissionCountersIdentity {
        &self.counter_identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalInspectionAdmissionReceipt {
    receipt_identity: CausalInspectionAdmissionReceiptIdentity,
    subject_identity: CausalInspectionAdmissionSubjectIdentity,
    decision_identity: CausalInspectionAdmissionDecisionIdentity,
    decision_trace_identity: CausalInspectionDecisionTraceIdentity,
    counter_identity: CausalInspectionAdmissionCountersIdentity,
}

impl CausalInspectionAdmissionReceipt {
    pub(super) fn new(
        subject: &CausalInspectionAdmissionSubject,
        decision: &CausalInspectionAdmissionDecision,
        trace: &CausalDecisionTraceIndex,
        counters: &CausalInspectionAdmissionCounters,
    ) -> Self {
        let mut receipt = Self {
            receipt_identity: CausalInspectionAdmissionReceiptIdentity::from(
                crate::domain_computation::evidence_identity::WorthQueryEvidenceIdentity::compose(
                    crate::domain_computation::evidence_identity::WorthQueryEvidenceScope::CausalInspectionAdmissionReceipt,
                )
                .seal(),
            ),
            subject_identity: subject.subject_identity().clone(),
            decision_identity: decision.decision_identity().clone(),
            decision_trace_identity: trace.trace_identity().clone(),
            counter_identity: counters.counter_identity().clone(),
        };
        receipt.receipt_identity = compose_causal_admission_receipt_identity(&receipt);
        receipt
    }

    pub fn receipt_digest(&self) -> &str {
        self.receipt_identity.as_str()
    }

    pub fn evidence_identity(
        &self,
    ) -> &crate::domain_computation::evidence_identity::WorthQueryEvidenceIdentity {
        self.receipt_identity.evidence_identity()
    }

    pub fn subject_for_reporting(&self) -> &str {
        self.subject_identity.as_str()
    }

    pub fn decision_for_reporting(&self) -> &str {
        self.decision_identity.as_str()
    }

    pub fn decision_trace_index_for_reporting(&self) -> &str {
        self.decision_trace_identity.as_str()
    }

    pub fn counter_snapshot_for_reporting(&self) -> &str {
        self.counter_identity.as_str()
    }

    pub fn counter_snapshot(&self) -> &str {
        self.counter_snapshot_for_reporting()
    }

    pub(super) fn receipt_identity(&self) -> &CausalInspectionAdmissionReceiptIdentity {
        &self.receipt_identity
    }

    pub(super) fn subject_identity(&self) -> &CausalInspectionAdmissionSubjectIdentity {
        &self.subject_identity
    }

    pub(super) fn decision_identity(&self) -> &CausalInspectionAdmissionDecisionIdentity {
        &self.decision_identity
    }

    pub(super) fn decision_trace_identity(&self) -> &CausalInspectionDecisionTraceIdentity {
        &self.decision_trace_identity
    }

    pub(super) fn counter_identity(&self) -> &CausalInspectionAdmissionCountersIdentity {
        &self.counter_identity
    }
}
