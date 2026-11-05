//! Moved whole from `domain_computation::primary_graph::application_attempt::read_set::decision_reuse`: the items below and every impl they own.
use crate::domain_computation::vocabulary::primary_graph::decision_context_use::DecisionContextUse;
use crate::domain_computation::vocabulary::primary_graph::decision_reuse::CompletedDecisionReuseProof;
use crate::domain_computation::vocabulary::primary_graph::decision_reuse::PreparedDecisionReuseContext;
use crate::domain_computation::vocabulary::primary_graph::handler_fact_boundary::CompletedHandlerFactBoundary;

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
    pub(in crate::domain_computation::primary_graph) const fn from_completed_read(
        handler_fact_count: usize,
    ) -> Self {
        Self { handler_fact_count }
    }

    pub const fn completed_for_test(handler_fact_count: usize) -> Self {
        Self::from_completed_read(handler_fact_count)
    }

    pub(in crate::domain_computation::primary_graph) const fn handler_fact_count(&self) -> usize {
        self.handler_fact_count
    }

    /// A republished output keeps the handler prefix its producer completed:
    /// restoring the exact retained output runs no handler and reads nothing.
    pub(in crate::domain_computation::primary_graph) const fn continued_by_republication(
        &self,
    ) -> Self {
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
