//! Thin forwarding shims for `operation_admission` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_authorization_operation_admission_capability_admission_preparation_observation_admitted_access_operation_progression {
    use crate::domain_computation::authorization::operation_admission::capability_admission::preparation::observation::admitted_access::AdmittedCapabilityAccessRuntimeExt as _;
    use crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess;
    use crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation;
    use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial;
    use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
    use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
    use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationOperation;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema,
    {
        pub fn authorize_capability_operation<Capability, Operation, Input>(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            access: WorthQueryAdmittedApplicationCapabilityAccess<
                Schema,
                Capability,
                Operation,
                Input,
            >,
            operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
            preconditions: TypedMutationPreconditions<
                Schema,
                Operation,
                <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
            >,
        ) -> Result<
            WorthQueryAdmittedApplicationOperation<
                Schema,
                Operation,
                Input,
                <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
            >,
            WorthQueryOperationAuthorizationDenial,
        >
        where
            Operation: ApplicationOperationMarkerIdentity<Schema>,
            Input: ApplicationCapabilityRequest<Schema, Capability>,
        {
            self.inner
                .authorize_capability_operation::<Capability, Operation, Input>(
                    phase,
                    access,
                    operation,
                    preconditions,
                )
        }
        pub(crate) fn authorize_capability_operation_with_request<Capability, Operation, Input>(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            access: WorthQueryAdmittedApplicationCapabilityAccess<
                Schema,
                Capability,
                Operation,
                Input,
            >,
            operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
            preconditions: impl std::borrow::Borrow<
                TypedMutationPreconditions<
                    Schema,
                    Operation,
                    <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
                >,
            >,
        ) -> Result<
            WorthQueryAdmittedApplicationOperation<
                Schema,
                Operation,
                Input,
                <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
            >,
            WorthQueryOperationAuthorizationDenial,
        >
        where
            Operation: ApplicationOperationMarkerIdentity<Schema>,
            Input: ApplicationCapabilityRequest<Schema, Capability>,
        {
            self.inner
                .authorize_capability_operation_with_request::<Capability, Operation, Input>(
                    execution,
                    access,
                    operation,
                    preconditions,
                )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_authorization_operation_admission_capability_admission_preparation_observation_admitted_access_operation_progression_borrowed_capability {
    use crate::domain_computation::authorization::operation_admission::capability_admission::preparation::observation::admitted_access::AdmittedCapabilityAccessRuntimeExt as _;
    use crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess;
    use crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation;
    use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial;
    use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
    use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
    use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationOperation;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn authorize_borrowed_capability_operation<Capability, Operation, Input>(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            access: WorthQueryAdmittedApplicationCapabilityAccess<
                Schema,
                Capability,
                Operation,
                Input,
            >,
            operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
            preconditions: &TypedMutationPreconditions<
                Schema,
                Operation,
                <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
            >,
        ) -> Result<
            WorthQueryAdmittedApplicationOperation<
                Schema,
                Operation,
                Input,
                <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
            >,
            WorthQueryOperationAuthorizationDenial,
        >
        where
            Operation: ApplicationOperationMarkerIdentity<Schema>,
            Input: ApplicationCapabilityRequest<Schema, Capability>,
        {
            self.inner
                .authorize_borrowed_capability_operation::<Capability, Operation, Input>(
                    phase,
                    access,
                    operation,
                    preconditions,
                )
        }
    }
}
