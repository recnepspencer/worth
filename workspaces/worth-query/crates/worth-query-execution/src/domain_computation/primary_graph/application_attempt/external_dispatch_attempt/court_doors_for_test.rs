//! Sealed test doors over this module's private members.
use super::*;

pub trait ExternalDispatchAttemptDoorsOps<
    Schema: worth_query_installation::facade::ApplicationSchema,
>:
    crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>
{
    #[doc(hidden)]
    fn admit_external_dispatch_attempt_for_test(
        &self,
        committed: WorthQueryCommittedDispatchOutboxObservation,
    ) -> Result<WorthQueryAdmittedExternalDispatchAttempt, WorthQueryExternalDispatchAdmissionDenial>
    {
        Self::admit_external_dispatch_attempt(self, committed)
    }
}

impl<Schema: worth_query_installation::facade::ApplicationSchema, HostRuntime: crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema> + ?Sized> ExternalDispatchAttemptDoorsOps<Schema> for HostRuntime {}
