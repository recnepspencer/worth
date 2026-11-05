//! Sealed test doors over this module's private members.
use super::*;

pub trait CustodyDoorsOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema>
{
    #[doc(hidden)]
    fn has_retained_installed_transport_completion_for_test(
        &self,
        correlation: &ExternalEffectCorrelationIdentity,
    ) -> bool {
        Self::has_retained_installed_transport_completion(self, correlation)
    }
}

impl<Schema: ApplicationSchema, HostRuntime: crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema> + ?Sized> CustodyDoorsOps<Schema> for HostRuntime {}

pub trait CustodyCustodyDoorsOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema>
{
    #[doc(hidden)]
    fn record_installed_transport_completion_for_test(
        &self,
        evidence: InstalledTransportCompletion,
        in_flight: OutstandingDispatchInFlightLease,
    ) -> Result<(), InstalledTransportCustodyDenial> {
        Self::record_installed_transport_completion(self, evidence, in_flight)
    }
}

impl<Schema: ApplicationSchema, HostRuntime: crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema> + ?Sized> CustodyCustodyDoorsOps<Schema> for HostRuntime {}
