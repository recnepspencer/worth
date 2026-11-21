use super::*;
use worth_query_execution_runtime::facade::ports::ProductBranchInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductRuntimeBranchLifecycleInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductRuntimeInternalPort as _;

impl<Schema, Capability, Operation, Input>
    AdmittedCapabilityAccessRuntimeExt<Schema, Capability, Operation, Input>
    for WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>
where
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    fn refuse_stale_incarnation(
        &self,
        receiver: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    ) -> Result<(), crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial>
    where
        Schema: ApplicationSchema,
    {
        match self.runtime_authority_for(receiver.runtime.product_runtime.owner_ref().owner_identity()) {
            Ok(_) | Err(worth_proof::HomeIncarnationDenial::ForeignHome) => Ok(()),
            Err(worth_proof::HomeIncarnationDenial::StaleIncarnation) => Err(
                crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial::new(
                    crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(
                        crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind::StaleIncarnation,
                    ),
                    self.operation(),
                ),
            ),
        }
    }

    fn read_product(
        &self,
        receiver: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    ) -> Option<crate::basis::WorthQueryProductBranchLease>
    where
        Schema: ApplicationSchema,
    {
        self.runtime_authority_for(
            receiver
                .runtime
                .product_runtime
                .owner_ref()
                .owner_identity(),
        )
        .ok()?;
        receiver
            .runtime
            .product_runtime
            .admit_product_occurrence(self.product.occurrence())
            .ok()
    }

    fn validate_workflow_read_authority(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    ) -> Result<(), crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial>
    where
        Schema: ApplicationSchema,
    {
        runtime.validate_capability_read_access(execution, self)
    }

    fn validate_operation_authority(
        &self,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        operation: &worth_query_installation::facade::WorthQueryInstalledApplicationOperation<
            Schema,
            Operation,
            Input,
        >,
        progression: crate::domain_computation::authorization::operation_progression::WorthQueryCapabilityOperationProgression,
    ) -> Result<(), crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial>
    where
        Schema: ApplicationSchema,
        Operation: ApplicationOperationMarkerIdentity<Schema>,
    {
        operation_progression::validate_capability_operation_authority(
            runtime,
            self,
            operation,
            progression,
        )
    }

    fn bind_operation_preconditions(
        &self,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        operation: &worth_query_installation::facade::WorthQueryInstalledApplicationOperation<
            Schema,
            Operation,
            Input,
        >,
        preconditions: impl std::borrow::Borrow<
            worth_query_declaration::facade::application_schema::TypedMutationPreconditions<
                Schema,
                Operation,
                <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
            >,
        >,
    ) -> Result<
        crate::domain_computation::primary_graph::WorthQueryBoundMutationPreconditions,
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    >
    where
        Schema: ApplicationSchema,
    {
        operation_progression::bind_capability_operation_preconditions(
            runtime,
            self,
            operation,
            preconditions,
        )
    }

    fn transition_operation(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        operation: &worth_query_installation::facade::WorthQueryInstalledApplicationOperation<
            Schema,
            Operation,
            Input,
        >,
        preconditions: crate::domain_computation::primary_graph::WorthQueryBoundMutationPreconditions,
    ) -> Result<
        crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    >
    where
        Schema: ApplicationSchema,
    {
        operation_progression::transition_capability_operation(
            execution,
            runtime,
            self,
            operation,
            preconditions,
        )
    }

    fn with_exact_observation<Output>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        observe: impl FnOnce(
            &exact_observation::WorthQueryExactCapabilityObservation<'_, Schema>,
        ) -> Output,
    ) -> Result<
        Option<Output>,
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    >
    where
        Schema: ApplicationSchema,
    {
        exact_observation::with_exact_observation(execution, self, runtime, observe)
    }
}
