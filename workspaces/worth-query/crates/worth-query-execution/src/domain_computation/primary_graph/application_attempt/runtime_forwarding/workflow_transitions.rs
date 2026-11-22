//! Thin forwarding shims for `application_attempt` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_attempt_workflow_transition_program_preparation {
    use worth_query_declaration::facade::application_capability::ApplicationCapabilityMarkerIdentity;
    use worth_query_declaration::facade::application_program::ApplicationWorkflowSpec;
    use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationWorkflowSpec;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::workflow_transition_program::PreparedWorkflowAdvance;
    use crate::domain_computation::primary_graph::PublishedWorkflowInstanceRef;
    use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::workflow_transition_program::preparation::WorkflowTransitionPreparationDenial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQuerySelectedProductOperation<'_, Schema>
    where
        Schema: ApplicationSchema,
    {
        pub(crate) fn prepare_workflow_advance<Capability, Operation, Input, Scope, Spec>(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
            instance: PublishedWorkflowInstanceRef,
            admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        ) -> Result<
            PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
            WorkflowTransitionPreparationDenial,
        >
        where
            Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
            Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
            Spec: ApplicationWorkflowSpec<Schema = Schema>,
        {
            self.inner
                .prepare_workflow_advance::<Capability, Operation, Input, Scope, Spec>(
                    execution, installed, instance, admission,
                )
        }
        pub(crate) fn prepare_workflow_navigate_back<Capability, Operation, Input, Scope, Spec>(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
            instance: PublishedWorkflowInstanceRef,
            admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        ) -> Result<
            PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
            WorkflowTransitionPreparationDenial,
        >
        where
            Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
            Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
            Spec: ApplicationWorkflowSpec<Schema = Schema>,
        {
            self.inner
                .prepare_workflow_navigate_back::<Capability, Operation, Input, Scope, Spec>(
                    execution, installed, instance, admission,
                )
        }
        pub(crate) fn prepare_workflow_collect_assessment<
            Capability,
            Operation,
            Input,
            Scope,
            Spec,
        >(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
            instance: PublishedWorkflowInstanceRef,
            node_path: String,
            admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        ) -> Result<
            PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
            WorkflowTransitionPreparationDenial,
        >
        where
            Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
            Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
            Spec: ApplicationWorkflowSpec<Schema = Schema>,
        {
            self.inner
                .prepare_workflow_collect_assessment::<Capability, Operation, Input, Scope, Spec>(
                    execution, installed, instance, node_path, admission,
                )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_attempt_workflow_transition_program_publication_commit {
    use crate::domain_computation::primary_graph::application_attempt::workflow_transition_program::publication::transition_replay::PreparedWorkflowAdvanceTransitionReplayExt as _;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::workflow_transition_program::publication::*;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema,
    {
        pub(crate) fn compare_and_commit_workflow_advance<Operation: 'static, Input, Scope>(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            phase: &WorthQueryAdvancementPhase<'_>,
            prepared: PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
            idempotency: worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding,
        ) -> WorkflowProgressOutcome
        where
            Input: Clone + Send + Sync + 'static,
        {
            self.inner
                .compare_and_commit_workflow_advance::<Operation, Input, Scope>(
                    execution,
                    phase,
                    prepared,
                    idempotency,
                )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_attempt_workflow_transition_program_preparation_observation {
    use worth_query_declaration::facade::application_program::ApplicationWorkflowSpec;
    use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationWorkflowSpec;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::workflow_transition_program::PreparedWorkflowAdvance;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::workflow_transition_program::WorkflowTransitionRequestKind;
    use crate::domain_computation::primary_graph::PublishedWorkflowInstanceRef;
    use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::workflow_transition_program::preparation::WorkflowTransitionPreparationDenial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<S: ApplicationSchema> WorthQuerySelectedProductOperation<'_, S> {
        pub(crate) fn prepare_local_workflow_transition<O, I, Scope, Spec>(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            installed: &WorthQueryInstalledApplicationWorkflowSpec<S, Spec>,
            instance: PublishedWorkflowInstanceRef,
            admission: WorthQueryAdmittedApplicationOperation<S, O, I, Scope>,
            request_kind: WorkflowTransitionRequestKind,
        ) -> Result<PreparedWorkflowAdvance<S, O, I, Scope>, WorkflowTransitionPreparationDenial>
        where
            O: ApplicationOperationMarkerIdentity<S> + 'static,
            Spec: ApplicationWorkflowSpec<Schema = S>,
        {
            self.inner
                .prepare_local_workflow_transition::<O, I, Scope, Spec>(
                    execution,
                    installed,
                    instance,
                    admission,
                    request_kind,
                )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_attempt_read_set_admission {
    use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_attempt::read_set::*;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAttemptDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryProjectedApplicationMutation;
    use crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation;
    use worth_query_execution_lineage::facade::primary_graph::invariant_projection::WorthQueryApplicationOperationInvariantProjectionSnapshot;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema,
    {
        pub fn begin_application_read_attempt<'request, 'authority, Operation, Input, Scope>(
            &self,
            execution: &worth_execution::ExecutionRequest<'request, 'authority>,
            admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        ) -> Result<
            WorthQueryApplicationReadAttempt<'request, 'authority, Schema, Operation, Input, Scope>,
            WorthQueryApplicationAttemptDenial,
        > {
            self.inner
                .begin_application_read_attempt::<Operation, Input, Scope>(execution, admission)
        }
        pub fn begin_projected_application_read_attempt<'scope, Operation, Input, Scope>(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'scope>,
            admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
            projection: WorthQueryApplicationOperationInvariantProjectionSnapshot<
                Schema,
                Operation,
            >,
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
            self.inner
                .begin_projected_application_read_attempt::<Operation, Input, Scope>(
                    phase,
                    admission,
                    projection,
                    allocation_policy,
                )
        }
        pub(crate) fn begin_projected_application_read_attempt_with_request<
            'request,
            'authority,
            Operation,
            Input,
            Scope,
        >(
            &self,
            execution: &worth_execution::ExecutionRequest<'request, 'authority>,
            admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
            projection: WorthQueryApplicationOperationInvariantProjectionSnapshot<
                Schema,
                Operation,
            >,
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
            self.inner
                .begin_projected_application_read_attempt_with_request::<Operation, Input, Scope>(
                    execution,
                    admission,
                    projection,
                    allocation_policy,
                )
        }

        pub fn durable_dispatch_custody(
            &self,
            correlation_token: &[u8; 32],
        ) -> Option<
            crate::domain_computation::application_aftermath::dispatch_journal::QueryDispatchCustody,
        >{
            self.inner.durable_dispatch_custody(correlation_token)
        }
    }
}
