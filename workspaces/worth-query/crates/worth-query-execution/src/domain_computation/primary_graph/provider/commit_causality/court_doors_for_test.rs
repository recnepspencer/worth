//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn active_application_commit_causality_partitions_for_test(&self) -> usize {
        Self::active_application_commit_causality_partitions(self)
    }
}

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn committed_application_emissions_for_test(
    &self, publication: &crate::domain_computation::primary_graph::vocabulary::application_attempt::committed_publication::WorthQueryCommittedProductPublication) -> Vec<crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationEmission>{
        Self::committed_application_emissions(self, publication)
    }
}

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn observe_application_commit_causality_for_test(
        &self,
        product: &crate::basis::WorthQueryProductBranchLease,
    ) -> crate::domain_computation::primary_graph::live_delivery::WorthQueryLiveSubscription {
        Self::observe_application_commit_causality(self, product)
    }
}

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn published_application_commit_count_for_test(&self) -> usize {
        Self::published_application_commit_count(self)
    }
}

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn set_application_commit_causality_limits_for_test(&self, batch_capacity: usize) {
        Self::set_application_commit_causality_limits(self, batch_capacity)
    }
}

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn set_application_commit_causality_reservation_hook_for_test(
        &self,
        hook: Option<std::sync::Arc<dyn Fn() + Send + Sync>>,
    ) {
        Self::set_application_commit_causality_reservation_hook(self, hook)
    }
}
