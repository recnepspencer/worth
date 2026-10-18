use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_operation::ApplicationEncodedInput,
    application_schema::{
        ApplicationOperationMarkerIdentity, ApplicationStructuredValueBinding,
        TypedMutationPreconditions,
    },
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
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        scope: &WorthQueryApplicationEntityIdentity<Schema, Scope>,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        preconditions: TypedMutationPreconditions<Schema, Operation, Scope>,
        request: &WorthQueryRequestScope,
    ) -> Result<
        WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        WorthQueryOperationAuthorizationDenial,
    > {
        self.authorize_borrowed_operation(
            phase,
            principal,
            scope,
            operation,
            &preconditions,
            request,
        )
    }

    pub fn authorize_borrowed_operation<Principal, PrincipalIdentity, Operation, Input, Scope>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        scope: &WorthQueryApplicationEntityIdentity<Schema, Scope>,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        preconditions: &TypedMutationPreconditions<Schema, Operation, Scope>,
        request: &WorthQueryRequestScope,
    ) -> Result<
        WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        WorthQueryOperationAuthorizationDenial,
    > {
        let execution = phase.execution_request_for(&self.application().product_runtime)
            .map_err(|_| WorthQueryOperationAuthorizationDenial::new(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::ForeignPhase), operation.operation()))?;
        self.authorize_operation_with_request(
            &execution,
            principal,
            scope,
            operation,
            preconditions,
            request,
        )
    }

    pub(in crate::domain_computation) fn authorize_operation_with_request<
        Principal,
        PrincipalIdentity,
        Operation,
        Input,
        Scope,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        scope: &WorthQueryApplicationEntityIdentity<Schema, Scope>,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        preconditions: impl std::borrow::Borrow<TypedMutationPreconditions<Schema, Operation, Scope>>,
        request: &WorthQueryRequestScope,
    ) -> Result<
        WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        WorthQueryOperationAuthorizationDenial,
    > {
        crate::domain_computation::authorization::progress_conventional_operation(
            execution,
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
phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        capability: &WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input>,
        input: Input,
        request: &WorthQueryRequestScope,
    ) -> Result<
        WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
        WorthQueryOperationAuthorizationDenial,
    >
    where
        Operation: worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<
                Schema,
            > + 'static,
        <Operation as worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<Schema>>::InputBinding:
            worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding<
                Value = Input,
            >,
        Input: ApplicationCapabilityRequest<Schema, Capability> + 'static,
    {
        let execution = phase.execution_request_for(&self.application().product_runtime)
            .map_err(|_| WorthQueryOperationAuthorizationDenial::new(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::ForeignPhase), "capability access"))?;
        self.admit_capability_access_with_request(&execution, principal, capability, input, request)
    }

    pub(in crate::domain_computation) fn admit_capability_access_with_request<Principal, PrincipalIdentity, Capability, Operation, Input>(
        &self,
execution: &worth_execution::ExecutionRequest<'_, '_>,
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        capability: &WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input>,
        input: Input,
        request: &WorthQueryRequestScope,
    ) -> Result<
        WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
        WorthQueryOperationAuthorizationDenial,
    >
    where
        Operation: worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<
                Schema,
            > + 'static,
        <Operation as worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<Schema>>::InputBinding:
            worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding<
                Value = Input,
            >,
        Input: ApplicationCapabilityRequest<Schema, Capability> + 'static,
    {
        crate::domain_computation::authorization::admit_capability_access(
            execution,
            self.application(),
            self.product(),
            principal,
            capability,
            input,
            request,
            None,
        )
    }

    /// Admits capability access for an input the request already encoded, so
    /// the admission governs the carrier's identity instead of encoding the
    /// input again. Only Publication's request entries drive it: they encode
    /// each request's input once and report that work themselves.
    pub fn admit_encoded_capability_access<
        Principal,
        PrincipalIdentity,
        Capability,
        Operation,
        Input,
    >(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        publication_access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        capability: &WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input>,
        input: ApplicationEncodedInput<Operation::InputBinding>,
        request: &WorthQueryRequestScope,
        workflow_actor: Option<&crate::domain_computation::primary_graph::application_attempt::WorthQueryWorkflowActorContext>,
    ) -> Result<
        WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
        WorthQueryOperationAuthorizationDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability> + 'static,
    {
        let execution = phase.execution_request_for(&self.application().product_runtime)
            .map_err(|_| WorthQueryOperationAuthorizationDenial::new(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::ForeignPhase), "encoded capability access"))?;
        self.admit_encoded_capability_access_with_request(
            &execution,
            publication_access,
            principal,
            capability,
            input,
            request,
            workflow_actor,
        )
    }

    pub(in crate::domain_computation) fn admit_encoded_capability_access_with_request<
        Principal,
        PrincipalIdentity,
        Capability,
        Operation,
        Input,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        capability: &WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input>,
        input: ApplicationEncodedInput<Operation::InputBinding>,
        request: &WorthQueryRequestScope,
        workflow_actor: Option<&crate::domain_computation::primary_graph::application_attempt::WorthQueryWorkflowActorContext>,
    ) -> Result<
        WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
        WorthQueryOperationAuthorizationDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability> + 'static,
    {
        crate::domain_computation::authorization::admit_encoded_capability_access(
            execution,
            self.application(),
            self.product(),
            principal,
            capability,
            input,
            request,
            workflow_actor,
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
phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
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
        Operation: worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<
                Schema,
            > + 'static,
        <Operation as worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<Schema>>::InputBinding:
            worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding<
                Value = Input,
            >,
        Input: ApplicationCapabilityRequest<Schema, Capability> + 'static,
    {
        let execution = phase.execution_request_for(&self.application().product_runtime)
            .map_err(|_| WorthQueryOperationAuthorizationDenial::new(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::ForeignPhase), "approved elevation access"))?;
        self.admit_approved_elevation_access_with_request(
            &execution, approved, principal, capability, input, request,
        )
    }

    pub(in crate::domain_computation) fn admit_approved_elevation_access_with_request<
        Principal,
        PrincipalIdentity,
        Capability,
        Operation,
        Input,
    >(
        &self,
execution: &worth_execution::ExecutionRequest<'_, '_>,
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
        Operation: worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<
                Schema,
            > + 'static,
        <Operation as worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<Schema>>::InputBinding:
            worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding<
                Value = Input,
            >,
        Input: ApplicationCapabilityRequest<Schema, Capability> + 'static,
    {
        crate::domain_computation::authorization::admit_capability_access(
            execution,
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
