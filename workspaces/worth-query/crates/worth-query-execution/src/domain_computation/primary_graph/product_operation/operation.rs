use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_schema::TypedMutationPreconditions, portable_identity::WorthQueryPortableType,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationCapability,
    WorthQueryInstalledApplicationOperation,
};

use super::WorthQuerySelectedProductOperation;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationCapabilityAccess, WorthQueryAdmittedApplicationOperation,
    WorthQueryApplicationEntityIdentity, WorthQueryApprovedElevation,
    WorthQueryAuthenticatedPrincipal, WorthQueryOperationAuthorizationDenial,
};

impl<'runtime, Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'runtime, Schema> {
    pub fn authorize_operation<Principal, PrincipalIdentity, Operation, Input, Scope>(
        &self,
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        scope: &WorthQueryApplicationEntityIdentity<Schema, Scope>,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        preconditions: TypedMutationPreconditions<Schema, Operation, Scope>,
        request: &WorthQueryRequestScope,
    ) -> Result<
        WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        WorthQueryOperationAuthorizationDenial,
    > {
        crate::domain_computation::authorization::progress_conventional_operation(
            self.application(),
            self.product(),
            principal,
            scope,
            operation,
            preconditions,
            request,
        )
    }

    pub fn admit_capability_access<Principal, PrincipalIdentity, Capability, Operation, Input>(
        &self,
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        capability: &WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input>,
        input: Input,
        request: &WorthQueryRequestScope,
    ) -> Result<
        WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
        WorthQueryOperationAuthorizationDenial,
    >
    where
        Operation: 'static,
        Input: ApplicationCapabilityRequest<Schema, Capability> + WorthQueryPortableType + 'static,
    {
        crate::domain_computation::authorization::admit_capability_access(
            self.application(),
            self.product(),
            principal,
            capability,
            input,
            request,
            None,
        )
    }

    pub fn admit_approved_elevation_access<
        Principal,
        PrincipalIdentity,
        Capability,
        Operation,
        Input,
    >(
        &self,
        approved: &WorthQueryApprovedElevation,
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        capability: &WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input>,
        input: Input,
        request: &WorthQueryRequestScope,
    ) -> Result<
        WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
        WorthQueryOperationAuthorizationDenial,
    >
    where
        Operation: 'static,
        Input: ApplicationCapabilityRequest<Schema, Capability> + WorthQueryPortableType + 'static,
    {
        crate::domain_computation::authorization::admit_capability_access(
            self.application(),
            self.product(),
            principal,
            capability,
            input,
            request,
            Some(approved),
        )
    }
}
