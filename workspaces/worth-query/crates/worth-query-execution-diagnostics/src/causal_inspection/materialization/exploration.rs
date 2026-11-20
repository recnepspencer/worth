use super::{
    CausalInspectionArtifactKind, CausalInspectionBoundaryEnvelopeCategory,
    CausalInspectionPerformanceEnvelope, CausalMaterializationReceipt,
    QueryCausalEvidenceReferenceArtifact, QueryCausalInspectionArtifact,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CausalInspectionArtifactDecisionTrace<'a> {
    query_decision_for_reporting: &'a str,
    bridge_envelope_for_reporting: Option<&'a str>,
    bridge_denial_for_reporting: Option<&'a str>,
}

impl<'a> CausalInspectionArtifactDecisionTrace<'a> {
    pub fn query_decision_for_reporting(&self) -> &'a str {
        self.query_decision_for_reporting
    }

    pub fn bridge_envelope_for_reporting(&self) -> Option<&'a str> {
        self.bridge_envelope_for_reporting
    }

    pub fn bridge_denial_for_reporting(&self) -> Option<&'a str> {
        self.bridge_denial_for_reporting
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CausalInspectionArtifactIntegrity<'a> {
    artifact_for_reporting: &'a str,
    causal_identity_for_reporting: &'a str,
    bridge_readmission_proof_for_reporting: Option<&'a str>,
}

impl<'a> CausalInspectionArtifactIntegrity<'a> {
    pub fn artifact_for_reporting(&self) -> &'a str {
        self.artifact_for_reporting
    }

    pub fn causal_identity_for_reporting(&self) -> &'a str {
        self.causal_identity_for_reporting
    }

    pub fn bridge_readmission_proof_for_reporting(&self) -> Option<&'a str> {
        self.bridge_readmission_proof_for_reporting
    }
}

impl QueryCausalInspectionArtifact {
    pub fn primary_result(&self) -> CausalInspectionArtifactKind {
        self.kind()
    }

    pub fn warnings(&self) -> Vec<&str> {
        match self {
            Self::Admitted(_) => Vec::new(),
            Self::Advisory(artifact) => vec![artifact.advisory_reason()],
            Self::Denied(artifact) => vec![artifact.denial_reason()],
        }
    }

    pub fn decision_trace(&self) -> CausalInspectionArtifactDecisionTrace<'_> {
        match self {
            Self::Admitted(artifact) => CausalInspectionArtifactDecisionTrace {
                query_decision_for_reporting: artifact.query_admission_for_reporting(),
                bridge_envelope_for_reporting: Some(artifact.bridge_envelope_for_reporting()),
                bridge_denial_for_reporting: None,
            },
            Self::Advisory(artifact) => CausalInspectionArtifactDecisionTrace {
                query_decision_for_reporting: artifact.query_advisory_for_reporting(),
                bridge_envelope_for_reporting: Some(artifact.bridge_envelope_for_reporting()),
                bridge_denial_for_reporting: None,
            },
            Self::Denied(artifact) => CausalInspectionArtifactDecisionTrace {
                query_decision_for_reporting: artifact.query_denial_for_reporting(),
                bridge_envelope_for_reporting: None,
                bridge_denial_for_reporting: artifact.bridge_denial_for_reporting(),
            },
        }
    }

    pub fn authority_bindings(&self) -> &[QueryCausalEvidenceReferenceArtifact] {
        self.evidence()
    }

    pub fn evidence(&self) -> &[QueryCausalEvidenceReferenceArtifact] {
        match self {
            Self::Admitted(artifact) => artifact.evidence_references(),
            Self::Advisory(artifact) => artifact.evidence_references(),
            Self::Denied(_) => &[],
        }
    }

    pub fn integrity(&self) -> CausalInspectionArtifactIntegrity<'_> {
        CausalInspectionArtifactIntegrity {
            artifact_for_reporting: self.artifact_for_reporting(),
            causal_identity_for_reporting: self.causal_identity_for_reporting(),
            bridge_readmission_proof_for_reporting: self.bridge_readmission_proof_for_reporting(),
        }
    }

    pub fn receipt(&self) -> &CausalMaterializationReceipt {
        match self {
            Self::Admitted(artifact) => artifact.receipt(),
            Self::Advisory(artifact) => artifact.receipt(),
            Self::Denied(artifact) => artifact.receipt(),
        }
    }

    pub fn denial_reason(&self) -> Option<&str> {
        match self {
            Self::Denied(artifact) => Some(artifact.denial_reason()),
            Self::Admitted(_) | Self::Advisory(_) => None,
        }
    }

    pub fn advisory_reason(&self) -> Option<&str> {
        match self {
            Self::Advisory(artifact) => Some(artifact.advisory_reason()),
            Self::Admitted(_) | Self::Denied(_) => None,
        }
    }

    pub fn boundary_categories(&self) -> &[CausalInspectionBoundaryEnvelopeCategory] {
        match self {
            Self::Admitted(artifact) => artifact.boundary_categories(),
            Self::Advisory(artifact) => artifact.boundary_categories(),
            Self::Denied(artifact) => artifact.boundary_categories(),
        }
    }

    pub fn is_advisory(&self) -> bool {
        matches!(self, Self::Advisory(_))
    }

    pub fn performance_envelope(&self) -> &CausalInspectionPerformanceEnvelope {
        self.performance()
    }
}
