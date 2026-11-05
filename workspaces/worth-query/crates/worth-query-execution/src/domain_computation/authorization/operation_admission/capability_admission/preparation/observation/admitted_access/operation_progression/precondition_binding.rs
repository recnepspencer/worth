use crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_schema::TypedMutationPreconditions,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationOperation,
};

use super::super::WorthQueryAdmittedApplicationCapabilityAccess;
use crate::domain_computation::primary_graph::{
    bind_mutation_preconditions, WorthQueryBoundMutationPreconditions,
};
use {
    crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind,
};

pub(super) fn bind_progression_preconditions<Schema, Capability, Operation, Input>(
    runtime: &dyn crate::domain_computation::authorization::host::AuthorizationHost<Schema>,
    access: &WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
    operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
    preconditions: impl std::borrow::Borrow<
        TypedMutationPreconditions<
            Schema,
            Operation,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
    >,
) -> Result<WorthQueryBoundMutationPreconditions, WorthQueryOperationAuthorizationDenial>
where
    Schema: ApplicationSchema,
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    let graph = runtime.primary_graph().ok_or_else(|| {
        denial(
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::ForeignRuntime,
            ),
            access.operation.as_ref(),
        )
    })?;
    bind_mutation_preconditions(
        preconditions,
        operation.contracts(),
        access.resource_entity_name(),
        access.resource_entity_id(),
        graph.layout(),
    )
    .map_err(|()| {
        denial(
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::MutationPreconditionRejected,
            ),
            operation.operation(),
        )
    })
}

fn denial(
    kind: WorthQueryOperationAuthorizationDenialKind,
    subject: impl Into<String>,
) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(kind, subject)
}
