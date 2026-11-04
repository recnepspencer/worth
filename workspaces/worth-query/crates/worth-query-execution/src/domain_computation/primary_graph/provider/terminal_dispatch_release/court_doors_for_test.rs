//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn begin_external_dispatch_in_flight_for_test(
        &self,
        original: &WorthQueryCommittedDispatchOutboxObservation,
    ) -> Result<Option<OutstandingDispatchInFlightLease>, WorthQueryOutstandingInFlightDenial> {
        Self::begin_external_dispatch_in_flight(self, original)
    }
}
