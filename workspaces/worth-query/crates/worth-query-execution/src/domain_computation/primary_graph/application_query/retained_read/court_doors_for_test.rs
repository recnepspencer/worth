//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationReadObservation {
    #[doc(hidden)]
    pub fn original_settled_producer_for_test(
        &self) -> Option<&super::super::super::application_output_demand::OriginalSettledProducerObservation> {
        Self::original_settled_producer(self)
    }
}

impl WorthQueryApplicationReadObservation {
    #[doc(hidden)]
    pub fn with_original_producer_for_test(
        &self, producer: Arc<super::super::super::application_output_demand::OriginalSettledProducerObservation>) -> Arc<Self> {
        Self::with_original_producer(self, producer)
    }
}
