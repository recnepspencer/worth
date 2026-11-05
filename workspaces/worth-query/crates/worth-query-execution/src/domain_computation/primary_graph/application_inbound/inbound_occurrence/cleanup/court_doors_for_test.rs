//! Sealed test doors over this module's private members.
use super::*;

pub trait CleanupDoorsOps<Schema: worth_query_installation::facade::ApplicationSchema>:
    crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema>
{
    #[doc(hidden)]
    fn retained_accepted_for_cleanup_test_for_test(
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

impl<Schema: worth_query_installation::facade::ApplicationSchema, HostRuntime: crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema> + ?Sized> CleanupDoorsOps<Schema> for HostRuntime {}

pub trait CleanupCleanupDoorsOps<Schema: worth_query_installation::facade::ApplicationSchema>:
    crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema>
{
    #[doc(hidden)]
    fn stale_publication_claim_for_cleanup_test_for_test(
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

impl<Schema: worth_query_installation::facade::ApplicationSchema, HostRuntime: crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema> + ?Sized> CleanupCleanupDoorsOps<Schema> for HostRuntime {}
