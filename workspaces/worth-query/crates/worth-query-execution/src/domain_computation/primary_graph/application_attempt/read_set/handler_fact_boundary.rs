/// The handler prefix of an exactly covered, completed decision read.
/// Source-selection facts follow this prefix in the sealed fact sequence.
pub(in crate::domain_computation) struct CompletedHandlerFactBoundary {
    handler_fact_count: usize,
}

impl CompletedHandlerFactBoundary {
    pub(super) const fn from_completed_read(handler_fact_count: usize) -> Self {
        Self { handler_fact_count }
    }

    pub(in crate::domain_computation::primary_graph) const fn handler_fact_count(&self) -> usize {
        self.handler_fact_count
    }
}
