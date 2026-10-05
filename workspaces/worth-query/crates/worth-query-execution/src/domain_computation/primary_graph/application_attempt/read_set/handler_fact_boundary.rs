/// The handler prefix of an exactly covered, completed decision read.
/// Source-selection facts follow this prefix in the sealed fact sequence.
pub(in crate::domain_computation) struct CompletedHandlerFactBoundary {
    handler_fact_count: usize,
}

impl CompletedHandlerFactBoundary {
    pub(super) const fn from_completed_read(handler_fact_count: usize) -> Self {
        Self { handler_fact_count }
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) const fn completed_for_test(
        handler_fact_count: usize,
    ) -> Self {
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
