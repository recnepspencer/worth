use super::admission_decision::{
    CausalInspectionAdmissionDecision, CausalInspectionAdmissionDecisionKind,
    CausalInspectionAdmissionSubject, CausalInspectionAdvisoryKind, CausalInspectionViolationKind,
};
use super::admission_trace::{
    CausalDecisionTraceIndex, CausalDecisionTraceRow, CausalInspectionAdmissionCounters,
    CausalInspectionAdmissionReceipt,
};
use super::identity::{compose_causal_outcome_identity, CausalInspectionOutcomeIdentity};
use super::request::{
    CausalInspectionExplanationFamily, CausalInspectionRequest, CausalInspectionRichness,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedCausalInspection {
    subject: CausalInspectionAdmissionSubject,
    decision: CausalInspectionAdmissionDecision,
    decision_trace: CausalDecisionTraceIndex,
    receipt: CausalInspectionAdmissionReceipt,
    counters: CausalInspectionAdmissionCounters,
    admitted_inspection_digest: CausalInspectionOutcomeIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdvisoryCausalInspection {
    subject: CausalInspectionAdmissionSubject,
    decision: CausalInspectionAdmissionDecision,
    decision_trace: CausalDecisionTraceIndex,
    receipt: CausalInspectionAdmissionReceipt,
    counters: CausalInspectionAdmissionCounters,
    advisory_inspection_digest: CausalInspectionOutcomeIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeniedCausalInspection {
    subject: CausalInspectionAdmissionSubject,
    decision: CausalInspectionAdmissionDecision,
    decision_trace: CausalDecisionTraceIndex,
    receipt: CausalInspectionAdmissionReceipt,
    counters: CausalInspectionAdmissionCounters,
    denied_inspection_digest: CausalInspectionOutcomeIdentity,
}

macro_rules! inspection_accessors {
    ($ty:ty, $digest_name:ident) => {
        impl $ty {
            pub fn subject(&self) -> &CausalInspectionAdmissionSubject {
                &self.subject
            }

            pub fn decision(&self) -> &CausalInspectionAdmissionDecision {
                &self.decision
            }

            pub fn decision_trace(&self) -> &CausalDecisionTraceIndex {
                &self.decision_trace
            }

            pub fn receipt(&self) -> &CausalInspectionAdmissionReceipt {
                &self.receipt
            }

            pub fn counters(&self) -> &CausalInspectionAdmissionCounters {
                &self.counters
            }

            pub fn $digest_name(&self) -> &str {
                self.$digest_name.as_str()
            }

            pub fn inspection_for_reporting(&self) -> &str {
                self.$digest_name.as_str()
            }
        }
    };
}

inspection_accessors!(AdmittedCausalInspection, admitted_inspection_digest);
inspection_accessors!(AdvisoryCausalInspection, advisory_inspection_digest);
inspection_accessors!(DeniedCausalInspection, denied_inspection_digest);

impl AdmittedCausalInspection {
    pub(super) fn admitted_inspection_identity(&self) -> &CausalInspectionOutcomeIdentity {
        &self.admitted_inspection_digest
    }

    pub fn admitted_inspection_for_reporting(&self) -> &str {
        self.admitted_inspection_digest()
    }
}

impl AdvisoryCausalInspection {
    pub(super) fn advisory_inspection_identity(&self) -> &CausalInspectionOutcomeIdentity {
        &self.advisory_inspection_digest
    }

    pub fn advisory_inspection_for_reporting(&self) -> &str {
        self.advisory_inspection_digest()
    }
}

impl DeniedCausalInspection {
    pub(super) fn denied_inspection_identity(&self) -> &CausalInspectionOutcomeIdentity {
        &self.denied_inspection_digest
    }

    pub fn denied_inspection_for_reporting(&self) -> &str {
        self.denied_inspection_digest()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CausalInspectionProofFlow {
    Admitted(AdmittedCausalInspection),
    Advisory(AdvisoryCausalInspection),
    Denied(DeniedCausalInspection),
}

impl CausalInspectionProofFlow {
    pub fn counters(&self) -> &CausalInspectionAdmissionCounters {
        match self {
            Self::Admitted(inspection) => inspection.counters(),
            Self::Advisory(inspection) => inspection.counters(),
            Self::Denied(inspection) => inspection.counters(),
        }
    }

    pub fn decision_trace(&self) -> &CausalDecisionTraceIndex {
        match self {
            Self::Admitted(inspection) => inspection.decision_trace(),
            Self::Advisory(inspection) => inspection.decision_trace(),
            Self::Denied(inspection) => inspection.decision_trace(),
        }
    }
}

pub fn admit_causal_inspection(request: CausalInspectionRequest) -> CausalInspectionProofFlow {
    let subject = CausalInspectionAdmissionSubject::from_request(&request);
    let decision = admission_decision(&request);
    let trace = decision_trace(&request, &decision);
    let counters = CausalInspectionAdmissionCounters::new(decision.kind(), &trace);
    let receipt = CausalInspectionAdmissionReceipt::new(&subject, &decision, &trace, &counters);
    match decision.kind() {
        CausalInspectionAdmissionDecisionKind::Success => {
            CausalInspectionProofFlow::Admitted(AdmittedCausalInspection {
                admitted_inspection_digest: inspection_digest(
                    CausalInspectionAdmissionDecisionKind::Success,
                    &subject,
                    &decision,
                    &trace,
                    &receipt,
                ),
                subject,
                decision,
                decision_trace: trace,
                receipt,
                counters,
            })
        }
        CausalInspectionAdmissionDecisionKind::Advisory => {
            CausalInspectionProofFlow::Advisory(AdvisoryCausalInspection {
                advisory_inspection_digest: inspection_digest(
                    CausalInspectionAdmissionDecisionKind::Advisory,
                    &subject,
                    &decision,
                    &trace,
                    &receipt,
                ),
                subject,
                decision,
                decision_trace: trace,
                receipt,
                counters,
            })
        }
        CausalInspectionAdmissionDecisionKind::Violation => {
            CausalInspectionProofFlow::Denied(DeniedCausalInspection {
                denied_inspection_digest: inspection_digest(
                    CausalInspectionAdmissionDecisionKind::Violation,
                    &subject,
                    &decision,
                    &trace,
                    &receipt,
                ),
                subject,
                decision,
                decision_trace: trace,
                receipt,
                counters,
            })
        }
    }
}

fn admission_decision(request: &CausalInspectionRequest) -> CausalInspectionAdmissionDecision {
    match request.explanation_family() {
        CausalInspectionExplanationFamily::CrossRuntimeCausalExplanation => {
            if request.requested_richness() == CausalInspectionRichness::MaterializedDetail {
                CausalInspectionAdmissionDecision::advisory(
                    request,
                    CausalInspectionAdvisoryKind::MaterializedDetailDeferredUntilBridgeEnvelope,
                )
            } else {
                CausalInspectionAdmissionDecision::success(request)
            }
        }
        CausalInspectionExplanationFamily::DurableCausalArchive
        | CausalInspectionExplanationFamily::StoreBackedReplayReconstruction => {
            CausalInspectionAdmissionDecision::violation(
                request,
                CausalInspectionViolationKind::UnsupportedExplanationFamily,
            )
        }
    }
}

fn decision_trace(
    request: &CausalInspectionRequest,
    decision: &CausalInspectionAdmissionDecision,
) -> CausalDecisionTraceIndex {
    let explanation_decision = if decision.violation_kind().is_some() {
        CausalInspectionAdmissionDecisionKind::Violation
    } else {
        CausalInspectionAdmissionDecisionKind::Success
    };
    let richness_decision = if decision.advisory_kind().is_some() {
        CausalInspectionAdmissionDecisionKind::Advisory
    } else {
        explanation_decision
    };
    let family_reason = if decision.violation_kind().is_some() {
        "unsupported explanation family remains later-milestone debt"
    } else {
        "cross-runtime causal explanation is the supported Query inspection family"
    };
    let richness_reason =
        if request.requested_richness() == CausalInspectionRichness::MaterializedDetail {
            "materialized detail is narrowed until bridge envelope admission"
        } else {
            "reference-only inspection does not require bridge envelope materialization"
        };
    CausalDecisionTraceIndex::new(vec![
        CausalDecisionTraceRow::new(
            "explanation_family",
            request.explanation_family().as_str(),
            explanation_decision,
            "query_inspection_admission",
            family_reason,
        ),
        CausalDecisionTraceRow::new(
            "richness_policy",
            request.requested_richness().as_str(),
            richness_decision,
            "query_redaction_posture",
            richness_reason,
        ),
        CausalDecisionTraceRow::new(
            "evidence_family_scope",
            "resolved_reference_set",
            decision.kind(),
            "query_evidence_reference_set",
            "requested evidence families are bounded by Phase 2 reference proof",
        ),
    ])
}

fn inspection_digest(
    kind: CausalInspectionAdmissionDecisionKind,
    subject: &CausalInspectionAdmissionSubject,
    decision: &CausalInspectionAdmissionDecision,
    trace: &CausalDecisionTraceIndex,
    receipt: &CausalInspectionAdmissionReceipt,
) -> CausalInspectionOutcomeIdentity {
    compose_causal_outcome_identity(
        kind,
        subject.subject_identity(),
        decision.decision_identity(),
        trace.trace_identity(),
        receipt.receipt_identity(),
    )
}
