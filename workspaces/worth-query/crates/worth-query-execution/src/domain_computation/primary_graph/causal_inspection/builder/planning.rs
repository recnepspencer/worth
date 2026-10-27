use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalInspectionPlan {
    pub(in crate::domain_computation::primary_graph::causal_inspection) inspection_basis:
        ScopedInspectionBasis,
    pub(in crate::domain_computation::primary_graph::causal_inspection) reference_set:
        CausalEvidenceReferenceSet,
    pub(in crate::domain_computation::primary_graph::causal_inspection) request:
        CausalInspectionRequest,
    pub(in crate::domain_computation::primary_graph::causal_inspection) admission:
        CausalInspectionProofFlow,
    pub(in crate::domain_computation::primary_graph::causal_inspection) redaction_policy:
        CausalInspectionRedactionPolicy,
    pub(in crate::domain_computation::primary_graph::causal_inspection) materialization_policy:
        CausalInspectionMaterializationPolicy,
}

impl CausalInspectionPlan {
    pub fn from_resolved_request(
        reference_set: CausalEvidenceReferenceSet,
        request: CausalInspectionRequest,
        redaction_policy: CausalInspectionRedactionPolicy,
        materialization_policy: CausalInspectionMaterializationPolicy,
    ) -> Self {
        let admission = admit_causal_inspection(request.clone());
        let inspection_basis = request
            .reference_set()
            .anchor()
            .observation_receipt()
            .inspection_basis()
            .clone();
        Self {
            inspection_basis,
            reference_set,
            request,
            admission,
            redaction_policy,
            materialization_policy,
        }
    }

    pub fn support_posture(&self) -> CausalInspectionSupportPosture {
        match &self.admission {
            CausalInspectionProofFlow::Admitted(_) => CausalInspectionSupportPosture::Admitted,
            CausalInspectionProofFlow::Advisory(_) => CausalInspectionSupportPosture::Advisory,
            CausalInspectionProofFlow::Denied(_) => CausalInspectionSupportPosture::Denied,
        }
    }

    pub fn inspection_basis(&self) -> &ScopedInspectionBasis {
        &self.inspection_basis
    }

    pub fn required_evidence(&self) -> &[CausalEvidenceReference] {
        self.reference_set.references()
    }

    pub fn admission(&self) -> &CausalInspectionProofFlow {
        &self.admission
    }

    pub fn decision_trace(&self) -> &CausalDecisionTraceIndex {
        self.admission.decision_trace()
    }

    pub fn estimated_cost(&self) -> CausalInspectionEstimatedCost {
        CausalInspectionEstimatedCost {
            anchor_derivation_count: 1,
            evidence_reference_resolution_count: 1,
            admission_count: 1,
            bridge_envelope_assembly_count: if self.support_posture()
                == CausalInspectionSupportPosture::Denied
            {
                0
            } else {
                1
            },
            evidence_reference_count: self.reference_set.references().len(),
        }
    }

    pub fn explain(&self) -> CausalInspectionPlanExplanation {
        CausalInspectionPlanExplanation {
            posture: self.support_posture(),
            reason: match &self.admission {
                CausalInspectionProofFlow::Admitted(_) => {
                    "query admission accepted the causal inspection request"
                }
                CausalInspectionProofFlow::Advisory(inspection) => {
                    inspection.decision().advisory_kind().map_or(
                        "query admission narrowed the causal inspection request",
                        |kind| kind.as_str(),
                    )
                }
                CausalInspectionProofFlow::Denied(inspection) => {
                    inspection.decision().violation_kind().map_or(
                        "query admission denied the causal inspection request",
                        |kind| kind.as_str(),
                    )
                }
            },
        }
    }

    pub fn anchor_for_reporting(&self) -> &str {
        self.reference_set.anchor().anchor_digest().as_str()
    }

    pub fn reference_set_digest(&self) -> &str {
        self.reference_set.reference_set_digest().as_str()
    }

    pub fn request_for_reporting(&self) -> &str {
        self.request.request_for_reporting()
    }

    pub fn admission_digest(&self) -> &str {
        match &self.admission {
            CausalInspectionProofFlow::Admitted(inspection) => {
                inspection.admitted_inspection_for_reporting()
            }
            CausalInspectionProofFlow::Advisory(inspection) => {
                inspection.advisory_inspection_for_reporting()
            }
            CausalInspectionProofFlow::Denied(inspection) => {
                inspection.denied_inspection_for_reporting()
            }
        }
    }

    pub fn redaction_policy(&self) -> CausalInspectionRedactionPolicy {
        self.redaction_policy
    }

    pub fn materialization_policy(&self) -> CausalInspectionMaterializationPolicy {
        self.materialization_policy
    }

    pub fn requested_richness(&self) -> CausalInspectionRichness {
        self.request.requested_richness()
    }

    pub fn explanation_family(&self) -> CausalInspectionExplanationFamily {
        self.request.explanation_family()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CausalInspectionEstimatedCost {
    anchor_derivation_count: usize,
    evidence_reference_resolution_count: usize,
    admission_count: usize,
    bridge_envelope_assembly_count: usize,
    evidence_reference_count: usize,
}

impl CausalInspectionEstimatedCost {
    pub fn anchor_derivation_count(&self) -> usize {
        self.anchor_derivation_count
    }

    pub fn evidence_reference_resolution_count(&self) -> usize {
        self.evidence_reference_resolution_count
    }

    pub fn admission_count(&self) -> usize {
        self.admission_count
    }

    pub fn bridge_envelope_assembly_count(&self) -> usize {
        self.bridge_envelope_assembly_count
    }

    pub fn evidence_reference_count(&self) -> usize {
        self.evidence_reference_count
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CausalInspectionPlanExplanation {
    posture: CausalInspectionSupportPosture,
    reason: &'static str,
}

impl CausalInspectionPlanExplanation {
    pub fn posture(&self) -> CausalInspectionSupportPosture {
        self.posture
    }

    pub fn reason(&self) -> &'static str {
        self.reason
    }
}

pub(super) fn reason_for_outcome(outcome: CausalObservationOutcome) -> CausalInspectionReason {
    match outcome {
        CausalObservationOutcome::Changed => CausalInspectionReason::ChangedResult,
        CausalObservationOutcome::Suppressed => CausalInspectionReason::SuppressedResult,
        CausalObservationOutcome::Denied => CausalInspectionReason::DeniedResult,
        CausalObservationOutcome::BranchPreview => CausalInspectionReason::BranchPreviewResult,
        CausalObservationOutcome::Replayed => CausalInspectionReason::HistoricalReplayResult,
    }
}
