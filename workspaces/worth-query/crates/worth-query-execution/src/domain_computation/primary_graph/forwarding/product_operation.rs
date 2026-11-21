//! Thin forwarding shims for `product_operation` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_product_operation_operation {
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
    use worth_query_declaration::facade::application_operation::ApplicationEncodedInput;
    use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
    use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
    use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationCapability;
    use worth_query_installation::facade::WorthQueryInstalledApplicationOperation;
    use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationCapabilityAccess;
    use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
    use crate::domain_computation::primary_graph::WorthQueryApplicationEntityIdentity;
    use crate::domain_computation::primary_graph::WorthQueryApprovedElevation;
    use crate::domain_computation::primary_graph::WorthQueryAuthenticatedPrincipal;
    use crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<'runtime, Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'runtime, Schema> {
        pub fn authorize_operation<Principal, PrincipalIdentity, Operation, Input, Scope>(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>, scope: &WorthQueryApplicationEntityIdentity<Schema, Scope>, operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>, preconditions: TypedMutationPreconditions<Schema, Operation, Scope>, request: &WorthQueryRequestScope) -> Result<
            WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
            WorthQueryOperationAuthorizationDenial,
        > {
            self.inner.authorize_operation::<Principal, PrincipalIdentity, Operation, Input, Scope>(phase, principal, scope, operation, preconditions, request)
        }
        pub fn authorize_borrowed_operation<Principal, PrincipalIdentity, Operation, Input, Scope>(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>, scope: &WorthQueryApplicationEntityIdentity<Schema, Scope>, operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>, preconditions: &TypedMutationPreconditions<Schema, Operation, Scope>, request: &WorthQueryRequestScope) -> Result<
            WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
            WorthQueryOperationAuthorizationDenial,
        > {
            self.inner.authorize_borrowed_operation::<Principal, PrincipalIdentity, Operation, Input, Scope>(phase, principal, scope, operation, preconditions, request)
        }
        pub fn admit_capability_access<Principal, PrincipalIdentity, Capability, Operation, Input>(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>, capability: &WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input>, input: Input, request: &WorthQueryRequestScope) -> Result<
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
            Input: ApplicationCapabilityRequest<Schema, Capability> + 'static, {
            self.inner.admit_capability_access::<Principal, PrincipalIdentity, Capability, Operation, Input>(phase, principal, capability, input, request)
        }
        #[allow(clippy::too_many_arguments)]
        pub fn admit_encoded_capability_access<
            Principal,
            PrincipalIdentity,
            Capability,
            Operation,
            Input,
        >(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, publication_access: &crate::publication_boundary::WorthQueryProgramPublicationAccess, principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>, capability: &WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input>, input: ApplicationEncodedInput<Operation::InputBinding>, request: &WorthQueryRequestScope, workflow_actor: Option<&crate::domain_computation::primary_graph::application_attempt::WorthQueryWorkflowActorContext>) -> Result<
            WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
            WorthQueryOperationAuthorizationDenial,
        >
        where
            Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
            Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
            Input: ApplicationCapabilityRequest<Schema, Capability> + 'static, {
            self.inner.admit_encoded_capability_access::<Principal, PrincipalIdentity, Capability, Operation, Input>(phase, publication_access, principal, capability, input, request, workflow_actor)
        }
        pub fn admit_approved_elevation_access<
            Principal,
            PrincipalIdentity,
            Capability,
            Operation,
            Input,
        >(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, approved: &WorthQueryApprovedElevation, principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>, capability: &WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input>, input: Input, request: &WorthQueryRequestScope) -> Result<
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
            Input: ApplicationCapabilityRequest<Schema, Capability> + 'static, {
            self.inner.admit_approved_elevation_access::<Principal, PrincipalIdentity, Capability, Operation, Input>(phase, approved, principal, capability, input, request)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_product_operation_security_basis {
    use crate::basis::WorthQueryProductBranchAdmissionDenial;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::product_operation::security_basis::WorthQueryProductObservationSource;
    use worth_query_execution_query::facade::domain_computation::primary_graph::product_operation::security_basis::WorthQueryProductSecurityBasis;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub(crate) fn admit_product_security_basis(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, product: &impl WorthQueryProductObservationSource) -> Result<WorthQueryProductSecurityBasis<'static>, WorthQueryProductBranchAdmissionDenial> {
            self.inner.admit_product_security_basis(execution, product)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_product_operation_shared_query_basis {
    use crate::basis::WorthQueryProductObservationLease;
    use crate::domain_computation::primary_graph::application_query::resource_lifecycle::WorthQueryApplicationBasisLease;
    use worth_relational::facade::mvcc::CompanionPreflightStop;
    use worth_query_execution_lineage::facade::primary_graph::product_operation::SelectedQueryBasisRetentionStop;
    use worth_query_execution_query::facade::domain_computation::primary_graph::product_operation::shared_query_basis::SharedSelectedProductOperation;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<'runtime, Schema> WorthQuerySelectedProductOperation<'runtime, Schema> {
        pub(crate) fn prepare_shared_query_basis(self) -> Result<SharedSelectedProductOperation<'runtime, Schema>, (Self, CompanionPreflightStop)> {
            let application = self.application;
            self.inner.prepare_shared_query_basis()
                .map_err(|(inner, stop)| (Self::from_inner(application, inner), stop))
        }
    }
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub(crate) fn retain_selected_query_basis_admitted(&self, shared: &SharedSelectedProductOperation<'_, Schema>) -> Result<
            (
                WorthQueryProductObservationLease,
                WorthQueryApplicationBasisLease,
            ),
            SelectedQueryBasisRetentionStop,
        > {
            self.inner.retain_selected_query_basis_admitted(shared)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_product_operation_query_retained {
    use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
    use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;
    use worth_query_execution_query::facade::domain_computation::primary_graph::product_operation::query::WorthQueryProductQueryControls;
    use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationQueryPlan;
    use crate::domain_computation::primary_graph::WorthQueryApplicationQueryAccessContext;
    use crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<'runtime, Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'runtime, Schema> {
        #[allow(clippy::too_many_arguments)]
        pub fn admit_retained_governed_application_query<
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
            Capability,
            Operation,
            Input,
        >(self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, retained: WorthQuerySelectedProductOperation<'runtime, Schema>, query: &'runtime WorthQueryInstalledApplicationQuery<
                Schema,
                Query,
                Parameters,
                QueryResult,
                Scope,
            >, access: &WorthQueryApplicationQueryAccessContext<
                'runtime,
                Schema,
                Principal,
                PrincipalIdentity,
                Scope,
            >, capability: crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess<
                Schema,
                Capability,
                Operation,
                Input,
            >, parameters: ApplicationQueryParameterSet<Query>, controls: WorthQueryProductQueryControls<'runtime>) -> Result<
            WorthQueryAdmittedApplicationQueryPlan<
                'runtime,
                Schema,
                Query,
                Parameters,
                QueryResult,
                Principal,
                PrincipalIdentity,
                Scope,
            >,
            WorthQueryApplicationQueryAdmissionDenial,
        >
        where
            Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope>, {
            self.inner.admit_retained_governed_application_query::<Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope, Capability, Operation, Input>(phase, retained.into_inner(), query, access, capability, parameters, controls)
        }
    }
}
