//! Explicit request options for rich inspection materialization.
use worth_query_execution_diagnostics::facade::causal_inspection::{
    inventory::CausalEvidenceFamily, materialization::CausalInspectionMaterializationPolicy,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationMaterializationRequest {
    families: u64,
    source_effects: bool,
    producer_evidence: bool,
    conditional_history: bool,
}

impl WorthQueryApplicationMaterializationRequest {
    pub const fn family(family: CausalEvidenceFamily) -> Self {
        Self {
            families: 1 << (family as u32),
            source_effects: false,
            producer_evidence: false,
            conditional_history: false,
        }
    }

    /// Select only the original source's canonical effects, without an unrelated family.
    pub const fn source_effects() -> Self {
        Self {
            families: 0,
            source_effects: true,
            producer_evidence: false,
            conditional_history: false,
        }
    }

    /// Select only optional conditional history; connected mode still requires both diagnostics.
    pub const fn conditional_history() -> Self {
        Self {
            families: 0,
            source_effects: false,
            producer_evidence: false,
            conditional_history: true,
        }
    }

    pub const fn with_conditional_history(mut self) -> Self {
        self.conditional_history = true;
        self
    }

    pub const fn requests_conditional_history(self) -> bool {
        self.conditional_history
    }

    pub const fn with_family(mut self, family: CausalEvidenceFamily) -> Self {
        self.families |= 1 << (family as u32);
        self
    }

    pub const fn with_source_effects(mut self) -> Self {
        self.source_effects = true;
        self
    }

    pub const fn with_producer_evidence(mut self) -> Self {
        self.producer_evidence = true;
        self
    }

    pub const fn requests_source_effects(self) -> bool {
        self.source_effects
    }

    pub const fn requests_producer_evidence(self) -> bool {
        self.producer_evidence
    }

    pub const fn requests(self, family: CausalEvidenceFamily) -> bool {
        self.families & (1 << (family as u32)) != 0
    }

    pub fn requested_families(self) -> impl Iterator<Item = CausalEvidenceFamily> {
        CausalEvidenceFamily::ALL
            .into_iter()
            .filter(move |family| self.requests(*family))
    }

    pub const fn policy(self) -> CausalInspectionMaterializationPolicy {
        CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact
    }
}
