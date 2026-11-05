//! Sealed test doors over this module's private members.
use super::*;

pub trait ExternalDispatchDoorsOps<Schema>:
    crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>
where
    Schema: ApplicationSchema,
{
    #[doc(hidden)]
    fn perform_committed_external_dispatch_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,
        transport: &dyn WorthQueryExternalEffectTransport,
        committed: crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxObservation,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<WorthQueryExternalEffectDispatch, WorthQueryExternalDispatchPreparationDenial> {
        Self::perform_committed_external_dispatch(
            self, execution, phase, transport, committed, request,
        )
    }
}

impl<Schema, HostRuntime: crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema> + ?Sized> ExternalDispatchDoorsOps<Schema> for HostRuntime where
    Schema: ApplicationSchema, {}
