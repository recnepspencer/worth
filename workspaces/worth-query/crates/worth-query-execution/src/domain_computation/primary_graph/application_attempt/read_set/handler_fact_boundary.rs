/// The handler prefix of an exactly covered, completed decision read.
/// Source-selection facts follow this prefix in the sealed fact sequence.
pub struct CompletedHandlerFactBoundary {
    handler_fact_count: usize,
    historical: Option<std::sync::Arc<[crate::domain_computation::primary_graph::application_contribution::producer::predecessor_reads::HistoricalProducerRead]>>,
}

impl CompletedHandlerFactBoundary {
    pub(super) const fn from_completed_read(handler_fact_count: usize) -> Self {
        Self {
            handler_fact_count,
            historical: None,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn retain_historical_reads(
        &mut self,
        reads: Option<std::sync::Arc<[crate::domain_computation::primary_graph::application_contribution::producer::predecessor_reads::HistoricalProducerRead]>>,
    ) {
        self.historical = reads;
    }
    pub const fn completed_for_test(handler_fact_count: usize) -> Self {
        Self::from_completed_read(handler_fact_count)
    }

    pub(in crate::domain_computation::primary_graph) const fn handler_fact_count(&self) -> usize {
        self.handler_fact_count
    }

    /// A republished output keeps the handler prefix its producer completed:
    /// restoring the exact retained output runs no handler and reads nothing.
    pub(in crate::domain_computation::primary_graph) fn continued_by_republication(&self) -> Self {
        Self {
            handler_fact_count: self.handler_fact_count,
            historical: self.historical.clone(),
        }
    }
}

mod court_doors_for_test;
