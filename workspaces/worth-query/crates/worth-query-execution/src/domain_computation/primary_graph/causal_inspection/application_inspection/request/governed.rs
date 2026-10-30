//! A governed inspection consumes genuine admitted capability access once.
use super::*;
use crate::domain_computation::authorization::{
    application_disclosure::prepare_funded_pending_disclosure,
    WorthQueryAdmittedApplicationCapabilityAccess,
};
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_query::ApplicationQueryScopeBinding,
};

impl<'o, Schema, Binding> WorthQueryReferenceOnlyInspectionRequest<'_, '_, '_, 'o, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationQueryBinding<Schema>,
{
    pub fn execute_governed<Capability, Operation, Input>(
        self,
        capability: WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
        >,
    ) -> Result<WorthQueryObservedApplicationInspection<'o>, WorthQueryApplicationInspectionDenial>
    where
        Input: ApplicationCapabilityRequest<
            Schema,
            Capability,
            Scope = <Binding::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope,
        >,
    {
        let runtime = self.0.runtime;
        let request = self.0.request;
        self.0
            .execute_with(|execution, installed, principal, scope, memory| {
                prepare_capability(
                    runtime, request, execution, installed, principal, scope, capability, memory,
                )
                .map(Some)
            })
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn prepare_capability<Schema, Binding, Capability, Operation, Input>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    request: &WorthQueryRequestScope,
    execution: &ExecutionRequest<'_, '_>,
    installed: &worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding<Schema, Binding>,
    principal: worth_relational::facade::identity::EntityId,
    scope: worth_relational::facade::identity::EntityId,
    capability: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
    memory: &mut Option<ExecutionMemoryReservation>,
) -> Result<crate::domain_computation::authorization::application_disclosure::WorthQueryPendingApplicationQueryGovernance, InspectionCause>
where
    Schema: ApplicationSchema,
    Binding: ApplicationQueryBinding<Schema>,
    Input: ApplicationCapabilityRequest<Schema, Capability,
        Scope = <Binding::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope>,
{
    if let Some(cause) = capability.request_scope().interruption() {
        return Err(InspectionCause::Admission(
            super::super::admission::InspectionAdmissionDenial::RequestControl(cause),
        ));
    }
    prepare_funded_pending_disclosure(
        runtime,
        installed.query().binding_identity(),
        principal,
        scope,
        capability,
        || {
            super::super::admission::require_request(execution, request)
                .map_err(InspectionCause::Admission)
        },
        || {
            InspectionCause::Admission(
                super::super::admission::InspectionAdmissionDenial::ForeignIssuedAccess,
            )
        },
        execution,
        memory,
        |cause| {
            InspectionCause::Admission(super::super::admission::InspectionAdmissionDenial::Storage(
                cause,
            ))
        },
    )
}
