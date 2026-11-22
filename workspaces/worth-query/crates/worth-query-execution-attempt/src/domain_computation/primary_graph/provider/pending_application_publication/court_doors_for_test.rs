//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn reserve_application_publication_recovery_for_test(
        &self,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
        binding: Option<
            crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding,
        >,
    ) -> Result<WorthQueryApplicationPublicationRecoveryReservation, &'static str> {
        Self::reserve_application_publication_recovery(self, observation, binding)
    }
}
