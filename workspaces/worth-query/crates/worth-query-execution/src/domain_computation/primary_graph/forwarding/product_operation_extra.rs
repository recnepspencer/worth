//! Additional crate-internal forwarding shims for product-operation helpers.

use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial;
use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
use crate::domain_computation::primary_graph::product_operation::query::WorthQueryProductQueryControls;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationCapabilityAccess, WorthQueryAdmittedApplicationOperation,
    WorthQueryApplicationEntityIdentity, WorthQueryApplicationQueryAccessContext,
    WorthQueryApplicationQueryAdmissionDenial, WorthQueryApplicationQueryControls,
    WorthQueryAuthenticatedPrincipal, WorthQueryProductBranchAdmissionDenial,
};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_schema::TypedMutationPreconditions,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationCapability,
    WorthQueryInstalledApplicationOperation, WorthQueryInstalledApplicationQuery,
};

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(crate) fn admit_with_observation_room<Admitted>(
        &self,
        admit: impl FnMut() -> Result<Admitted, WorthQueryProductBranchAdmissionDenial>,
    ) -> Result<Admitted, WorthQueryProductBranchAdmissionDenial> {
        self.inner.admit_with_observation_room(admit)
    }
}

impl<'runtime, Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'runtime, Schema> {
    pub(crate) fn authorize_operation_with_request<
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
        self.inner.authorize_operation_with_request(
            execution,
            principal,
            scope,
            operation,
            preconditions,
            request,
        )
    }

    pub(crate) fn admit_capability_access_with_request<
        Principal,
        PrincipalIdentity,
        Capability,
        Operation,
        Input,
    >(
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
        self.inner
            .admit_capability_access_with_request(execution, principal, capability, input, request)
    }

    pub(crate) fn recover_retained_read_indexes<
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        retained: &WorthQuerySelectedProductOperation<'runtime, Schema>,
        query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
        access: &WorthQueryApplicationQueryAccessContext<
            '_,
            Schema,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
    ) -> Result<(), WorthQueryApplicationQueryAdmissionDenial> {
        self.inner
            .recover_retained_read_indexes(execution, retained.inner(), query, access)
    }

    pub(crate) fn retained_query_controls(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        retained: WorthQuerySelectedProductOperation<'runtime, Schema>,
        query_name: &str,
        controls: WorthQueryProductQueryControls<'runtime>,
    ) -> Result<
        (
            &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
            WorthQueryApplicationQueryControls<'runtime, Schema>,
        ),
        WorthQueryApplicationQueryAdmissionDenial,
    > {
        let application = self.application;
        self.inner
            .retained_query_controls(execution, retained.into_inner(), query_name, controls)
            .map(|(_, controls)| (application, controls))
    }
}
