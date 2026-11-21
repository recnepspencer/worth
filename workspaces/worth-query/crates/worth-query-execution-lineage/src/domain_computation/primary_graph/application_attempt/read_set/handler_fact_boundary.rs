/// The handler prefix of an exactly covered, completed decision read.
/// Source-selection facts follow this prefix in the sealed fact sequence.
pub struct CompletedHandlerFactBoundary {
    handler_fact_count: usize,
    historical: Option<std::sync::Arc<[crate::domain_computation::primary_graph::application_contribution::producer::observation_reads::WorthQueryHistoricalProducerRead]>>,
}

impl CompletedHandlerFactBoundary {
    pub fn insert_input_facts(&mut self, expected: usize, count: usize) -> bool {
        if self.handler_fact_count != expected {
            return false;
        }
        let Some(total) = expected.checked_add(count) else {
            return false;
        };
        self.handler_fact_count = total;
        true
    }

    pub const fn from_completed_read(handler_fact_count: usize) -> Self {
        Self {
            handler_fact_count,
            historical: None,
        }
    }

    pub fn retain_historical_reads(
        &mut self,
        reads: Option<std::sync::Arc<[crate::domain_computation::primary_graph::application_contribution::producer::observation_reads::WorthQueryHistoricalProducerRead]>>,
    ) {
        self.historical = reads;
    }
    pub fn retain_predecessor_reads(
        &self,
    ) -> Option<std::sync::Arc<[crate::domain_computation::primary_graph::application_contribution::producer::observation_reads::WorthQueryHistoricalProducerRead]>>{
        self.historical.clone()
    }
    pub const fn completed_for_test(handler_fact_count: usize) -> Self {
        Self::from_completed_read(handler_fact_count)
    }

    pub const fn handler_fact_count(&self) -> usize {
        self.handler_fact_count
    }

    /// A republished output keeps the handler prefix its producer completed:
    /// restoring the exact retained output runs no handler and reads nothing.
    pub fn continued_by_republication(&self) -> Self {
        Self {
            handler_fact_count: self.handler_fact_count,
            historical: self.historical.clone(),
        }
    }
}

mod court_doors_for_test;
