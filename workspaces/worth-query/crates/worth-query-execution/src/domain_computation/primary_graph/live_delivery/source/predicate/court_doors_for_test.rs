//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryLiveDeliverySource {
    #[doc(hidden)]
    pub fn begin_predicate_preparation_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        observation: &ProductBranchObservation,
    ) -> Result<WorthQueryLivePredicatePreparation, WorthQueryLiveAdmissionDenial> {
        Self::begin_predicate_preparation(self, execution, observation)
    }
}

impl WorthQueryLiveDeliverySource {
    #[doc(hidden)]
    pub fn join_predicate_opening_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        subscription: &WorthQueryLivePredicateSubscription,
        observation: &ProductBranchObservation,
    ) -> Result<WorthQueryLivePredicateOpening, WorthQueryLiveAdmissionDenial> {
        Self::join_predicate_opening(self, execution, subscription, observation)
    }
}

impl WorthQueryLiveDeliverySource {
    #[doc(hidden)]
    pub fn open_predicate_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        observation: &ProductBranchObservation,
    ) -> Result<WorthQueryLivePredicateSubscription, WorthQueryLiveAdmissionDenial> {
        Self::open_predicate(self, execution, observation)
    }
}

impl WorthQueryLiveDeliverySource {
    #[doc(hidden)]
    pub fn validate_predicate_progress_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        subscription: &WorthQueryLivePredicateSubscription,
        current: &ProductBranchObservation,
    ) -> Result<(), WorthQueryLiveAdmissionDenial> {
        Self::validate_predicate_progress(self, execution, subscription, current)
    }
}
