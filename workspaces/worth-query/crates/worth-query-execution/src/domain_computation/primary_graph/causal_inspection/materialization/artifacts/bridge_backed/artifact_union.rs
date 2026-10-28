use super::*;

macro_rules! bridge_backed_accessors {
    ($ty:ty) => {
        impl $ty {
            pub fn evidence_references(&self) -> &[QueryCausalEvidenceReferenceArtifact] {
                &self.evidence_references
            }

            pub fn temporal_async_explanation(&self) -> &QueryCausalTemporalAsyncExplanation {
                &self.temporal_async_explanation
            }

            pub fn boundary_categories(&self) -> &[CausalInspectionBoundaryEnvelopeCategory] {
                &self.boundary_categories
            }

            pub fn bridge_envelope_for_reporting(&self) -> &str {
                self.bridge_envelope_identity.as_str()
            }

            pub fn bridge_envelope_identity(&self) -> &WorthQueryEvidenceIdentity {
                &self.bridge_envelope_identity
            }

            pub fn bridge_receipt_for_reporting(&self) -> &str {
                self.bridge_receipt_identity.as_str()
            }

            pub fn bridge_receipt_identity(&self) -> &WorthQueryEvidenceIdentity {
                &self.bridge_receipt_identity
            }

            pub fn performance(&self) -> &CausalInspectionPerformanceEnvelope {
                &self.performance
            }

            pub fn receipt(&self) -> &CausalMaterializationReceipt {
                &self.receipt
            }

            pub fn readmission_proof(&self) -> &CausalBridgeReadmissionProof {
                &self.readmission_proof
            }

            pub fn bridge_readmission_proof_for_reporting(&self) -> &str {
                self.readmission_proof.readmission_proof_for_reporting()
            }

            pub fn bridge_readmission_proof_identity(&self) -> &WorthQueryEvidenceIdentity {
                self.readmission_proof.readmission_proof_identity()
            }

            pub fn causal_identity_for_reporting(&self) -> &str {
                self.causal_identity.as_str()
            }

            pub fn causal_identity(&self) -> &CausalInspectionArtifactIdentity {
                &self.causal_identity
            }

            pub fn artifact_for_reporting(&self) -> &str {
                self.artifact_identity.as_str()
            }

            pub fn artifact_identity(&self) -> &CausalInspectionArtifactIdentity {
                &self.artifact_identity
            }
        }
    };
}

bridge_backed_accessors!(AdmittedQueryCausalInspectionArtifact);
bridge_backed_accessors!(AdvisoryQueryCausalInspectionArtifact);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryCausalInspectionArtifact {
    Admitted(AdmittedQueryCausalInspectionArtifact),
    Advisory(AdvisoryQueryCausalInspectionArtifact),
    Denied(DeniedQueryCausalInspectionArtifact),
}

impl QueryCausalInspectionArtifact {
    pub fn kind(&self) -> CausalInspectionArtifactKind {
        match self {
            Self::Admitted(_) => CausalInspectionArtifactKind::Admitted,
            Self::Advisory(_) => CausalInspectionArtifactKind::Advisory,
            Self::Denied(_) => CausalInspectionArtifactKind::Denied,
        }
    }

    pub fn artifact_for_reporting(&self) -> &str {
        match self {
            Self::Admitted(artifact) => artifact.artifact_for_reporting(),
            Self::Advisory(artifact) => artifact.artifact_for_reporting(),
            Self::Denied(artifact) => artifact.artifact_for_reporting(),
        }
    }

    pub fn artifact_identity(&self) -> &CausalInspectionArtifactIdentity {
        match self {
            Self::Admitted(artifact) => artifact.artifact_identity(),
            Self::Advisory(artifact) => artifact.artifact_identity(),
            Self::Denied(artifact) => artifact.artifact_identity(),
        }
    }

    pub fn causal_identity_for_reporting(&self) -> &str {
        match self {
            Self::Admitted(artifact) => artifact.causal_identity_for_reporting(),
            Self::Advisory(artifact) => artifact.causal_identity_for_reporting(),
            Self::Denied(artifact) => artifact.causal_identity_for_reporting(),
        }
    }

    pub fn causal_identity(&self) -> &CausalInspectionArtifactIdentity {
        match self {
            Self::Admitted(artifact) => artifact.causal_identity(),
            Self::Advisory(artifact) => artifact.causal_identity(),
            Self::Denied(artifact) => artifact.causal_identity(),
        }
    }

    pub fn query_observation_for_reporting(&self) -> &str {
        match self {
            Self::Admitted(artifact) => artifact.query_observation_for_reporting(),
            Self::Advisory(artifact) => artifact.query_observation_for_reporting(),
            Self::Denied(artifact) => artifact.query_observation_for_reporting(),
        }
    }

    pub fn query_observation_identity(&self) -> &CausalObservationReceiptIdentity {
        match self {
            Self::Admitted(artifact) => artifact.query_observation_identity(),
            Self::Advisory(artifact) => artifact.query_observation_identity(),
            Self::Denied(artifact) => artifact.query_observation_identity(),
        }
    }

    pub fn bridge_envelope_for_reporting(&self) -> Option<&str> {
        match self {
            Self::Admitted(artifact) => Some(artifact.bridge_envelope_for_reporting()),
            Self::Advisory(artifact) => Some(artifact.bridge_envelope_for_reporting()),
            Self::Denied(_) => None,
        }
    }

    pub fn evidence_reference_count(&self) -> usize {
        match self {
            Self::Admitted(artifact) => artifact.evidence_references().len(),
            Self::Advisory(artifact) => artifact.evidence_references().len(),
            Self::Denied(_) => 0,
        }
    }

    pub fn performance(&self) -> &CausalInspectionPerformanceEnvelope {
        match self {
            Self::Admitted(artifact) => artifact.performance(),
            Self::Advisory(artifact) => artifact.performance(),
            Self::Denied(artifact) => artifact.performance(),
        }
    }

    pub fn temporal_async_explanation(&self) -> &QueryCausalTemporalAsyncExplanation {
        match self {
            Self::Admitted(artifact) => artifact.temporal_async_explanation(),
            Self::Advisory(artifact) => artifact.temporal_async_explanation(),
            Self::Denied(artifact) => artifact.temporal_async_explanation(),
        }
    }

    pub fn bridge_readmission_proof_for_reporting(&self) -> Option<&str> {
        match self {
            Self::Admitted(artifact) => Some(artifact.bridge_readmission_proof_for_reporting()),
            Self::Advisory(artifact) => Some(artifact.bridge_readmission_proof_for_reporting()),
            Self::Denied(_) => None,
        }
    }

    pub fn bridge_readmission_proof_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        match self {
            Self::Admitted(artifact) => Some(artifact.bridge_readmission_proof_identity()),
            Self::Advisory(artifact) => Some(artifact.bridge_readmission_proof_identity()),
            Self::Denied(_) => None,
        }
    }

    pub fn is_admitted(&self) -> bool {
        matches!(self, Self::Admitted(_))
    }

    pub fn is_denied(&self) -> bool {
        matches!(self, Self::Denied(_))
    }
}
