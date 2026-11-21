//! Operation-precondition binding phase owner.

use crate::domain_computation::authorization::operation_admission::capability_admission::preparation::observation::admitted_access::AdmittedCapabilityAccessRuntimeExt as _;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_schema::TypedMutationPreconditions,
};
use worth_query_installation::facade::ApplicationSchema;

use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial;
use crate::domain_computation::primary_graph::WorthQueryBoundMutationPreconditions;

use super::{ValidatedCapabilityOperation, ValidatedConventionalOperation};

pub mod transition;
pub use transition::WorthQueryAdmittedApplicationCapabilityAccess;
pub use transition::WorthQueryAdmittedApplicationOperation;
pub use transition::WorthQueryOperationAdmissionIdentity;
pub use transition::{admit_capability_access, admit_encoded_capability_access};
pub use transition::{
    authorize_applied_control_mutation_on_selected, authorize_current_control_mutation_on_selected,
    authorize_historical_control_mutation_on_selected, authorize_public_mutation_on_selected,
    SelectedConventionalAdmissionStop,
};
pub use transition::{transition_capability_operation, transition_conventional_operation};
pub use transition::{
    WorthQueryCapabilityContextKey, WorthQueryCurrentCapabilityObservation,
    WorthQueryDelegationResolvedRequest, WorthQueryExactCapabilityObservationContext,
    WorthQueryResolvedCapabilityRequest,
};

pub struct PreconditionBoundConventionalOperation<
    'a,
    Schema,
    Principal,
    PrincipalIdentity,
    Operation,
    Input,
    Scope,
> {
    pub validated: ValidatedConventionalOperation<
        'a,
        Schema,
        Principal,
        PrincipalIdentity,
        Operation,
        Input,
        Scope,
    >,
    pub preconditions: WorthQueryBoundMutationPreconditions,
}

pub fn bind_conventional_preconditions<
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
    preconditions: impl std::borrow::Borrow<TypedMutationPreconditions<Schema, Operation, Scope>>,
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
            crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::ForeignRuntime),
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
            crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::MutationPreconditionRejected),
            operation.operation(),
        )
    })?;
    Ok(PreconditionBoundConventionalOperation {
        validated,
        preconditions,
    })
}

pub struct PreconditionBoundCapabilityOperation<'a, Schema, Capability, Operation, Input>
where
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    validated: ValidatedCapabilityOperation<'a, Schema, Capability, Operation, Input>,
    preconditions: WorthQueryBoundMutationPreconditions,
}

pub fn bind_capability_preconditions<Schema, Capability, Operation, Input>(
    validated: ValidatedCapabilityOperation<'_, Schema, Capability, Operation, Input>,
    preconditions: impl std::borrow::Borrow<
        TypedMutationPreconditions<
            Schema,
            Operation,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
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

pub use transition::{BrandTag, ReadCompletionAdmission, WorthQueryAdmittedOperation};
