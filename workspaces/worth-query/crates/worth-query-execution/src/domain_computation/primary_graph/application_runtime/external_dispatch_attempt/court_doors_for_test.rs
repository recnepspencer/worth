//! Sealed test doors over this module's private members.
use super::*;

impl<Schema: worth_query_installation::facade::ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn admit_external_dispatch_attempt_for_test(
        &self, committed: WorthQueryCommittedDispatchOutboxObservation) -> Result<WorthQueryAdmittedExternalDispatchAttempt, WorthQueryExternalDispatchAdmissionDenial> {
        Self::admit_external_dispatch_attempt(self, committed)
    }
}
