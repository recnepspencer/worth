//! Sealed test doors over this module's private members.
use super::*;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn has_retained_installed_transport_completion_for_test(
        &self,
        correlation: &ExternalEffectCorrelationIdentity,
    ) -> bool {
        Self::has_retained_installed_transport_completion(self, correlation)
    }
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn record_installed_transport_completion_for_test(
        &self,
        evidence: InstalledTransportCompletion,
        in_flight: OutstandingDispatchInFlightLease,
    ) -> Result<(), InstalledTransportCustodyDenial> {
        Self::record_installed_transport_completion(self, evidence, in_flight)
    }
}
