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
    pub(in crate::domain_computation) handler_fact_count: usize,
}

impl CompletedHandlerFactBoundary {
    #[doc(hidden)]
    pub fn seal_decision_reuse_for_test(
        &self,
        prepared: PreparedDecisionReuseContext,
        actual: DecisionContextUse,
    ) -> Option<CompletedDecisionReuseProof> {
        Self::seal_decision_reuse(self, prepared, actual)
    }
}

impl CompletedHandlerFactBoundary {
    pub(in crate::domain_computation) const fn from_completed_read(
        handler_fact_count: usize,
    ) -> Self {
        Self { handler_fact_count }
    }

    pub const fn completed_for_test(handler_fact_count: usize) -> Self {
        Self::from_completed_read(handler_fact_count)
    }

    pub(in crate::domain_computation) const fn handler_fact_count(&self) -> usize {
        self.handler_fact_count
    }

    /// A republished output keeps the handler prefix its producer completed:
    /// restoring the exact retained output runs no handler and reads nothing.
    pub(in crate::domain_computation) const fn continued_by_republication(&self) -> Self {
        Self {
            handler_fact_count: self.handler_fact_count,
        }
    }
}

impl CompletedHandlerFactBoundary {
    #[doc(hidden)]
    pub fn handler_fact_count_for_test(&self) -> usize {
        Self::handler_fact_count(self)
    }
}
