use std::sync::Arc;

use crate::domain_computation::{
    WorthQueryConvergenceDomainDecision, WorthQueryConvergenceDomainWorkEvidence,
    WorthQueryProviderWorkReport,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryBoundConvergenceReport {
    evidence_identity: Arc<str>,
    provider_receipt_identity: Arc<str>,
    graph_evidence_identity: Arc<str>,
    iteration_ordinal: usize,
    decision: WorthQueryConvergenceDomainDecision,
    domain_work: WorthQueryConvergenceDomainWorkEvidence,
    provider_work: WorthQueryProviderWorkReport,
}

impl WorthQueryBoundConvergenceReport {
    pub(super) fn new(
        evidence_identity: impl Into<Arc<str>>,
        provider_receipt_identity: impl Into<Arc<str>>,
        graph_evidence_identity: impl Into<Arc<str>>,
        iteration_ordinal: usize,
        decision: WorthQueryConvergenceDomainDecision,
        domain_work: WorthQueryConvergenceDomainWorkEvidence,
        provider_work: WorthQueryProviderWorkReport,
    ) -> Self {
        Self {
            evidence_identity: evidence_identity.into(),
            provider_receipt_identity: provider_receipt_identity.into(),
            graph_evidence_identity: graph_evidence_identity.into(),
            iteration_ordinal,
            decision,
            domain_work,
            provider_work,
        }
    }

    pub fn evidence_identity(&self) -> &str {
        &self.evidence_identity
    }

    pub fn provider_receipt_identity(&self) -> &str {
        &self.provider_receipt_identity
    }

    pub fn graph_evidence_identity(&self) -> &str {
        &self.graph_evidence_identity
    }

    pub const fn iteration_ordinal(&self) -> usize {
        self.iteration_ordinal
    }

    pub fn decision(&self) -> &WorthQueryConvergenceDomainDecision {
        &self.decision
    }

    pub const fn domain_work(&self) -> WorthQueryConvergenceDomainWorkEvidence {
        self.domain_work
    }

    pub const fn provider_work(&self) -> WorthQueryProviderWorkReport {
        self.provider_work
    }
}
