//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::computation_retention::producer_edition::InstalledProducerEdition;
use crate::domain_computation::primary_graph::computation_retention::full_cause::WorthQueryPartitionedComputationFullCause;

impl ComputationPrior {
    #[doc(hidden)]
    pub fn hand_in_test_for_test(prior: Option<Self>) {
        Self::hand_in_test(prior)
    }
}

impl SealedComputationRun {
    #[doc(hidden)]
    pub fn keep_in_test_for_test(run: Option<Self>) {
        Self::keep_in_test(run)
    }
}

impl SealedComputationRun {
    #[doc(hidden)]
    pub fn kept_in_test_for_test() -> Option<Self> {
        Self::kept_in_test()
    }
}

impl ComputationPrior {
    #[doc(hidden)]
    pub fn new_for_test(
        edition: InstalledProducerEdition,
        retained: Result<Arc<RetainedComputation>, WorthQueryPartitionedComputationFullCause>,
        record: Option<PriorComputationRecord>,
    ) -> Self {
        Self::new(edition, retained, record)
    }
}
