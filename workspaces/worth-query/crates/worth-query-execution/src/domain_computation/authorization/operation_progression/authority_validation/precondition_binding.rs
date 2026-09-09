//! Operation-precondition binding phase owner.

use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_schema::TypedMutationPreconditions,
};
use worth_query_installation::facade::ApplicationSchema;

use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial;
use crate::domain_computation::primary_graph::WorthQueryBoundMutationPreconditions;

use super::{ValidatedCapabilityOperation, ValidatedConventionalOperation};

mod transition;
pub(in crate::domain_computation) use transition::admit_capability_access;
pub use transition::WorthQueryAdmittedApplicationCapabilityAccess;
pub use transition::WorthQueryAdmittedApplicationOperation;
pub(in crate::domain_computation) use transition::WorthQueryOperationAdmissionIdentity;
pub(in crate::domain_computation::authorization::operation_progression) use transition::{
    transition_capability_operation, transition_conventional_operation,
};
pub(in crate::domain_computation::authorization) use transition::{
    WorthQueryCapabilityContextKey, WorthQueryCurrentCapabilityObservation,
    WorthQueryDelegationResolvedRequest, WorthQueryExactCapabilityObservationContext,
    WorthQueryResolvedCapabilityRequest,
};

pub(in crate::domain_computation::authorization::operation_progression) struct PreconditionBoundConventionalOperation<
    'a,
    Schema,
    Principal,
    PrincipalIdentity,
    Operation,
    Input,
    Scope,
> {
    validated: ValidatedConventionalOperation<
        'a,
        Schema,
        Principal,
        PrincipalIdentity,
        Operation,
        Input,
        Scope,
    >,
    preconditions: WorthQueryBoundMutationPreconditions,
}

pub(in crate::domain_computation::authorization::operation_progression) fn bind_conventional_preconditions<
    Schema,
    Principal,
    PrincipalIdentity,
    Operation,
    Input,
    Scope,
>(
    validated: ValidatedConventionalOperation<
        '_,
        Schema,
        Principal,
        PrincipalIdentity,
        Operation,
        Input,
        Scope,
    >,
    preconditions: TypedMutationPreconditions<Schema, Operation, Scope>,
) -> Result<
    PreconditionBoundConventionalOperation<
        '_,
        Schema,
        Principal,
        PrincipalIdentity,
        Operation,
        Input,
        Scope,
    >,
    WorthQueryOperationAuthorizationDenial,
>
where
    Schema: ApplicationSchema,
{
    let runtime = validated.runtime;
    let scope = validated.scope;
    let operation = validated.operation;
    let graph = runtime.runtime.primary_graph().ok_or_else(|| {
        WorthQueryOperationAuthorizationDenial::new(
            crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::ForeignRuntime,
            operation.operation(),
        )
    })?;
    let preconditions = crate::domain_computation::primary_graph::bind_mutation_preconditions(
        preconditions,
        operation.contracts(),
        scope.entity_name(),
        scope.entity_id(),
        graph.layout(),
    )
    .map_err(|()| {
        WorthQueryOperationAuthorizationDenial::new(
            crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::MutationPreconditionRejected,
            operation.operation(),
        )
    })?;
    Ok(PreconditionBoundConventionalOperation {
        validated,
        preconditions,
    })
}

pub(in crate::domain_computation::authorization::operation_progression) struct PreconditionBoundCapabilityOperation<
    'a,
    Schema,
    Capability,
    Operation,
    Input,
> where
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    validated: ValidatedCapabilityOperation<'a, Schema, Capability, Operation, Input>,
    preconditions: WorthQueryBoundMutationPreconditions,
}

pub(in crate::domain_computation::authorization::operation_progression) fn bind_capability_preconditions<
    Schema,
    Capability,
    Operation,
    Input,
>(
    validated: ValidatedCapabilityOperation<'_, Schema, Capability, Operation, Input>,
    preconditions: TypedMutationPreconditions<
        Schema,
        Operation,
        <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
    >,
) -> Result<
    PreconditionBoundCapabilityOperation<'_, Schema, Capability, Operation, Input>,
    WorthQueryOperationAuthorizationDenial,
>
where
    Schema: ApplicationSchema,
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    let preconditions = validated.access.bind_operation_preconditions(
        validated.runtime,
        validated.operation,
        preconditions,
    )?;
    Ok(PreconditionBoundCapabilityOperation {
        validated,
        preconditions,
    })
}
