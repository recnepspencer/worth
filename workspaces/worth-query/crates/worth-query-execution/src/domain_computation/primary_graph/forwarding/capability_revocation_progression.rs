//! Thin forwarding shims for `capability_revocation_progression` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_authorization_capability_revocation_progression {
    use worth_query_execution_attempt::facade::domain_computation::authorization::capability_revocation_progression::*;
    use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
    use worth_query_declaration::facade::application_capability::ApplicationCapabilityRevocationRequest;
    use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationCapability;
    use worth_query_installation::facade::WorthQueryInstalledApplicationOperation;
    use worth_query_execution_query::facade::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess;
    use worth_query_execution_query::facade::domain_computation::authorization::WorthQueryAdmittedApplicationOperation;
    use worth_query_execution_query::facade::domain_computation::authorization::WorthQueryOperationAuthorizationDenial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema, {
        pub fn authorize_capability_revocation<Capability, Operation, Input>(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, access: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>, capability: &WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input>, operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>, preconditions: TypedMutationPreconditions<
                Schema,
                Operation,
                <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
            >) -> Result<
            WorthQueryAdmittedApplicationOperation<
                Schema,
                Operation,
                Input,
                <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
            >,
            WorthQueryOperationAuthorizationDenial,
        >
        where
            Operation:
                worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<
                    Schema,
                >,
            Input: ApplicationCapabilityRequest<Schema, Capability>
                + ApplicationCapabilityRevocationRequest<Schema, Capability>, {
            self.inner.authorize_capability_revocation::<Capability, Operation, Input>(phase, access, capability, operation, preconditions)
        }
    }
}
