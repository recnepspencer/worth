use super::WorthQueryOutputDemandDenial;
use crate::domain_computation::primary_graph::WorthQueryGeneratedOutputInvariantAdmissionDenial;
use crate::domain_computation::WorthQueryCustomInvariantDenial;

impl WorthQueryOutputDemandDenial {
    /// The original installed invariant evidence from the producer's publication.
    /// Diagnostic history clones retain the same declared value bytes.
    pub fn custom_invariant_denial(&self) -> Option<&WorthQueryCustomInvariantDenial> {
        self.evidence.as_ref()?.custom_invariant.as_ref()
    }

    pub(in crate::domain_computation::primary_graph) fn with_custom_invariant_denial(
        mut self,
        denial: Option<WorthQueryCustomInvariantDenial>,
    ) -> Self {
        if let Some(denial) = denial {
            self.evidence_mut().custom_invariant = Some(denial);
        }
        self
    }

    /// Why the producer's own required invariant did not admit its publication,
    /// with the receipt's verdict and non-applicability basis.
    pub fn producer_requirement_denial(
        &self,
    ) -> Option<WorthQueryGeneratedOutputInvariantAdmissionDenial> {
        self.evidence.as_ref()?.producer_requirement
    }

    pub(in crate::domain_computation::primary_graph) fn with_producer_requirement_denial(
        mut self,
        denial: Option<WorthQueryGeneratedOutputInvariantAdmissionDenial>,
    ) -> Self {
        if let Some(denial) = denial {
            self.evidence_mut().producer_requirement = Some(denial);
        }
        self
    }
}
