//! Sealed test doors over this module's private members.
use super::*;

impl<Query> WorthQueryObservedSource<Query> {
    #[doc(hidden)]
    pub fn original_row_footprints_for_test(
        &self) -> impl ExactSizeIterator<Item = &super::super::WorthQueryObservedSourceFootprint> {
        Self::original_row_footprints(self)
    }
}

impl<Query> WorthQueryObservedSource<Query> {
    #[doc(hidden)]
    pub fn original_settled_producer_for_test(
        &self) -> Option<&super::super::super::super::application_output_demand::OriginalSettledProducerObservation> {
        Self::original_settled_producer(self)
    }
}
