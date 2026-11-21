use super::*;
use worth_query_execution_attempt::facade::ports::{
    AdmittedOperationExecutionPort as _, ApplicationReadAttemptAdmissionPort as _,
    WorthQueryReadAttemptRuntimeContext,
};

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub(super) fn read_attempt_runtime_context(&self) -> WorthQueryReadAttemptRuntimeContext<'_> {
        let primary_graph = self.runtime.primary_graph().map(|graph| {
            (
                Arc::clone(&graph.layout),
                graph.retain_entity_resolution_context(),
            )
        });
        WorthQueryReadAttemptRuntimeContext {
            world_owner: self.runtime.product_runtime.owner.owner_identity(),
            runtime_authority: self.runtime.authority_identity(),
            binding_identity: self.installed_schema.binding_identity(),
            installed_operations: &self.runtime.installed_operations,
            primary_graph,
            value_decode: super::super::super::value_decode::ValueDecodeState::new(
                self.installed_schema.value_bindings().clone(),
            ),
        }
    }

    pub fn begin_application_read_attempt<'request, 'authority, Operation, Input, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'request, 'authority>,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<
        WorthQueryApplicationReadAttempt<'request, 'authority, Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    > {
        let context = self.read_attempt_runtime_context();
        WorthQueryApplicationReadAttempt::admit_ordinary_read_attempt(execution, context, admission)
    }

    /// Consumes a projection lease typed for this exact admitted operation.
    ///
    /// A same-schema projection for another operation cannot enter the
    /// application read-set progression:
    pub fn begin_projected_application_read_attempt<'scope, Operation, Input, Scope>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'scope>,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        projection: WorthQueryApplicationOperationInvariantProjectionSnapshot<Schema, Operation>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryApplicationReadAttempt<
            'scope,
            'scope,
            Schema,
            Operation,
            Input,
            Scope,
            WorthQueryProjectedApplicationMutation,
        >,
        WorthQueryApplicationAttemptDenial,
    > {
        let phase_request = match phase.execution_request_for(&self.runtime.product_runtime) {
            Ok(request) => request,
            Err(_) => {
                return Err(
                    crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::new(
                        crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::ForeignPhase,
                        "application-read-phase",
                    ),
                )
            }
        };
        let execution = &phase_request;
        self.begin_projected_application_read_attempt_with_request(
            execution,
            admission,
            projection,
            allocation_policy,
        )
    }

    pub fn begin_projected_application_read_attempt_with_request<
        'request,
        'authority,
        Operation,
        Input,
        Scope,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'request, 'authority>,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        projection: WorthQueryApplicationOperationInvariantProjectionSnapshot<Schema, Operation>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryApplicationReadAttempt<
            'request,
            'authority,
            Schema,
            Operation,
            Input,
            Scope,
            WorthQueryProjectedApplicationMutation,
        >,
        WorthQueryApplicationAttemptDenial,
    > {
        let context = self.read_attempt_runtime_context();
        WorthQueryApplicationReadAttempt::admit_projected_snapshot_read_attempt(
            execution,
            context,
            admission,
            projection,
            allocation_policy,
        )
    }
}
