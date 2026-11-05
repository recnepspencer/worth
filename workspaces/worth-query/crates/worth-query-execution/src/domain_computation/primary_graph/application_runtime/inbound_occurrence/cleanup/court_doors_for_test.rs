//! Sealed test doors over this module's private members.
use super::*;

impl<Schema: worth_query_installation::facade::ApplicationSchema>
    WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    #[doc(hidden)]
    pub fn retained_accepted_for_cleanup_test_for_test(
        &self,
        correlation: &crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity,
    ) -> Option<
        std::sync::Arc<
            crate::domain_computation::application_aftermath::WorthQueryAcceptedInboundOccurrence,
        >,
    > {
        Self::retained_accepted_for_cleanup_test(self, correlation)
    }
}

impl<Schema: worth_query_installation::facade::ApplicationSchema>
    WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    #[doc(hidden)]
    pub fn stale_publication_claim_for_cleanup_test_for_test(
        &self,
        accepted: &std::sync::Arc<
            crate::domain_computation::application_aftermath::WorthQueryAcceptedInboundOccurrence,
        >,
    ) -> (
        crate::domain_computation::application_aftermath::WorthQueryInboundPublicationClaim,
        bool,
    ) {
        Self::stale_publication_claim_for_cleanup_test(self, accepted)
    }
}
