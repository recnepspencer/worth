//! Thin forwarding shims for `delegation_progression` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_authorization_delegation_progression_mod {
    use crate::domain_computation::authorization::operation_admission::capability_admission::preparation::observation::admitted_access::AdmittedCapabilityAccessRuntimeExt as _;
    use worth_query_declaration::facade::application_capability::ApplicationCapabilityDelegationRequest;
    use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
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
        Schema: ApplicationSchema,
    {
        #[allow(clippy::too_many_arguments)]
        pub fn authorize_capability_delegation<
            CommandCapability,
            TargetCapability,
            TargetOperation,
            TargetInput,
            Operation,
            Input,
        >(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, access: WorthQueryAdmittedApplicationCapabilityAccess<
                Schema,
                CommandCapability,
                Operation,
                Input,
            >, target: &WorthQueryInstalledApplicationCapability<
                Schema,
                TargetCapability,
                TargetOperation,
                TargetInput,
            >, operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>, preconditions: TypedMutationPreconditions<
                Schema,
                Operation,
                <Input as ApplicationCapabilityRequest<Schema, CommandCapability>>::Scope,
            >) -> Result<
            WorthQueryAdmittedApplicationOperation<
                Schema,
                Operation,
                Input,
                <Input as ApplicationCapabilityRequest<Schema, CommandCapability>>::Scope,
            >,
            WorthQueryOperationAuthorizationDenial,
        >
        where
            Operation:
                worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<
                    Schema,
                >,
            Input: ApplicationCapabilityRequest<Schema, CommandCapability>
        + ApplicationCapabilityDelegationRequest<Schema, TargetCapability>,{
            self.inner.authorize_capability_delegation::<CommandCapability, TargetCapability, TargetOperation, TargetInput, Operation, Input>(phase, access, target, operation, preconditions)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_authorization_delegation_progression_replay {
    use crate::domain_computation::authorization::operation_admission::capability_admission::preparation::observation::admitted_access::AdmittedCapabilityAccessRuntimeExt as _;
    use worth_query_declaration::facade::application_capability::ApplicationCapabilityDelegationRequest;
    use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
    use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
    use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationCapability;
    use worth_query_installation::facade::WorthQueryInstalledApplicationOperation;
    use worth_query_execution_query::facade::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess;
    use crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome;
    use crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding;
    use crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn replay_capability_delegation_after_fresh_denial<
            CommandCapability,
            TargetCapability,
            TargetOperation,
            TargetInput,
            Operation,
            Input,
        >(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            access: WorthQueryAdmittedApplicationCapabilityAccess<
                Schema,
                CommandCapability,
                Operation,
                Input,
            >,
            target: &WorthQueryInstalledApplicationCapability<
                Schema,
                TargetCapability,
                TargetOperation,
                TargetInput,
            >,
            operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
            preconditions: TypedMutationPreconditions<
                Schema,
                Operation,
                <Input as ApplicationCapabilityRequest<Schema, CommandCapability>>::Scope,
            >,
            idempotency: WorthQueryApplicationIdempotencyBinding,
        ) -> Result<
            Option<WorthQueryApplicationCommitOutcome>,
            WorthQueryApplicationIdempotencyResolutionDenial,
        >
        where
            Operation: ApplicationOperationMarkerIdentity<Schema>,
            Input: ApplicationCapabilityRequest<Schema, CommandCapability>
                + ApplicationCapabilityDelegationRequest<Schema, TargetCapability>
                + Clone
                + Send
                + Sync
                + 'static,
        {
            self.inner.replay_capability_delegation_after_fresh_denial::<CommandCapability, TargetCapability, TargetOperation, TargetInput, Operation, Input>(phase, access, target, operation, preconditions, idempotency)
        }
    }
}
