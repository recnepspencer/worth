//! Sealed test doors over this module's private members.
use super::*;

impl PerformedMembers {
    #[doc(hidden)]
    pub fn current_for_test<T>(
        &self, value: T, basis: CurrentBasis, record: Arc<crate::domain_computation::primary_graph::application_output_demand::RequiredWorkMembership>) -> CurrentProof<T> {
        Self::current::<T>(self, value, basis, record)
    }
}
