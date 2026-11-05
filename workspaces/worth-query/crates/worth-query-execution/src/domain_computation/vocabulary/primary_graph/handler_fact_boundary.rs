//! Moved whole from `domain_computation::primary_graph::vocabulary::application_attempt::handler_fact_boundary`: the items below and every impl they own.
use crate::domain_computation::vocabulary::primary_graph::decision_context_use::DecisionContextUse;
use crate::domain_computation::vocabulary::primary_graph::decision_reuse::CompletedDecisionReuseProof;
use crate::domain_computation::vocabulary::primary_graph::decision_reuse::PreparedDecisionReuseContext;

impl CompletedHandlerFactBoundary {
    /// Only the exactly completed read can turn a descriptive capture into
    /// reusable decision evidence. The handler fact prefix stays separate.
    pub(in crate::domain_computation) fn seal_decision_reuse(
        &self,
        prepared: PreparedDecisionReuseContext,
        actual: DecisionContextUse,
    ) -> Option<CompletedDecisionReuseProof> {
        prepared.seal_after_completed_read(actual)
    }
}

/// The handler prefix of an exactly covered, completed decision read.
/// Source-selection facts follow this prefix in the sealed fact sequence.
pub struct CompletedHandlerFactBoundary {
    handler_fact_count: usize,
}
