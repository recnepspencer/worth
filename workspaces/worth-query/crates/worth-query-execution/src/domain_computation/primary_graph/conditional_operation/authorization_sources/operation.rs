//! Authorization sources a conditional operation uses to admit its operation
//! each time a due wake re-enters it.

use crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation;
use crate::domain_computation::primary_graph::{
    WorthQueryAuthenticatedPrincipal, WorthQueryOperationAuthorizationDenial,
};
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_schema::{ApplicationSchema, TypedMutationPreconditions},
};
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationCapability, WorthQueryInstalledApplicationOperation,
};

/// How a conditional operation authorizes its operation each time a due wake
/// re-enters it.
///
/// Use [`WorthQueryPublicTemporalOperationAuthorization`] for an operation
/// that requires no capability or
/// [`WorthQueryGovernedTemporalOperationAuthorization`] for one governed by a
/// capability. Authorization runs at admission, before any effect.
pub trait WorthQueryTemporalOperationAuthorization<Schema, Operation, Input, Scope>:
    Send + Sync + 'static
where
    Schema: ApplicationSchema,
{
    fn authorize<Principal, PrincipalIdentity>(
        &self,
        product: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
            '_,
            Schema,
        >,
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        scope: &crate::domain_computation::primary_graph::WorthQueryApplicationEntityIdentity<
            Schema,
            Scope,
        >,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        input: &Input,
        preconditions: TypedMutationPreconditions<Schema, Operation, Scope>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<
        WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        WorthQueryOperationAuthorizationDenial,
    >;
}

/// Authorizes an operation that requires no capability, after validating its
/// input.
#[derive(Default)]
pub struct WorthQueryPublicTemporalOperationAuthorization;

impl<Schema, Operation, Input, Scope>
    WorthQueryTemporalOperationAuthorization<Schema, Operation, Input, Scope>
    for WorthQueryPublicTemporalOperationAuthorization
where
    Schema: ApplicationSchema,
    Operation:
        worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<
                Schema,
            > + 'static,
    Operation::InputBinding:
        worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding<
            Value = Input,
        >,
{
    fn authorize<Principal, PrincipalIdentity>(
        &self,
        product: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
            '_,
            Schema,
        >,
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        scope: &crate::domain_computation::primary_graph::WorthQueryApplicationEntityIdentity<
            Schema,
            Scope,
        >,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        input: &Input,
        preconditions: TypedMutationPreconditions<Schema, Operation, Scope>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<
        WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        WorthQueryOperationAuthorizationDenial,
    > {
        super::super::input_validation::validate_operation_input::<Schema, Operation, Input>(
            input, operation,
        )?;
        product.authorize_operation(principal, scope, operation, preconditions, request)
    }
}

/// Authorizes an operation through a fresh capability authorization, using the
/// operation input as the capability request.
pub struct WorthQueryGovernedTemporalOperationAuthorization<Schema, Capability, Operation, Input> {
    capability: WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input>,
}

impl<Schema, Capability, Operation, Input>
    WorthQueryGovernedTemporalOperationAuthorization<Schema, Capability, Operation, Input>
{
    pub fn new(
        capability: WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input>,
    ) -> Self {
        Self { capability }
    }
}

impl<Schema, Capability, Operation, Input, Scope>
    WorthQueryTemporalOperationAuthorization<Schema, Operation, Input, Scope>
    for WorthQueryGovernedTemporalOperationAuthorization<Schema, Capability, Operation, Input>
where
    Schema: ApplicationSchema,
    Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope>
        + Clone
        + Send
        + Sync
        + 'static,
    Capability: Send + Sync + 'static,
    Operation:
        worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<
                Schema,
            > + Send
            + Sync
            + 'static,
    <Operation as worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<Schema>>::InputBinding:
        worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding<
            Value = Input,
        >,
{
    fn authorize<Principal, PrincipalIdentity>(
        &self,
        product: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
            '_,
            Schema,
        >,
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        _scope: &crate::domain_computation::primary_graph::WorthQueryApplicationEntityIdentity<
            Schema,
            Scope,
        >,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        input: &Input,
        preconditions: TypedMutationPreconditions<Schema, Operation, Scope>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<
        WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        WorthQueryOperationAuthorizationDenial,
    > {
        let capability =
            product.admit_capability_access(principal, &self.capability, input.clone(), request)?;
        product
            .application()
            .authorize_capability_operation(capability, operation, preconditions)
    }
}
