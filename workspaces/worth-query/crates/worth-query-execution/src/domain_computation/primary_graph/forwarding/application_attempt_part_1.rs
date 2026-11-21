//! Thin forwarding shims for `application_attempt` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_attempt_elevation_approval_replay {
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyResolutionDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryElevationApprovalOutcome;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryRequestedElevation;
    use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn resolve_admitted_elevation_approval_replay<Operation, Input, Scope>(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            admission: &mut WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
            idempotency: WorthQueryApplicationIdempotencyBinding,
        ) -> Result<
            Option<WorthQueryElevationApprovalOutcome>,
            (
                WorthQueryApplicationIdempotencyResolutionDenial,
                WorthQueryRequestedElevation,
            ),
        >
        where
            Input: Clone + Send + Sync + 'static,
        {
            self.inner
                .resolve_admitted_elevation_approval_replay::<Operation, Input, Scope>(
                    phase,
                    admission,
                    idempotency,
                )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_attempt_elevation_close_replay {
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyResolutionDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApprovedElevation;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryElevationCloseOutcome;
    use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn resolve_admitted_elevation_close_replay<Operation, Input, Scope>(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            admission: &mut WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
            idempotency: WorthQueryApplicationIdempotencyBinding,
        ) -> Result<
            Option<WorthQueryElevationCloseOutcome>,
            (
                WorthQueryApplicationIdempotencyResolutionDenial,
                WorthQueryApprovedElevation,
            ),
        >
        where
            Input: Clone + Send + Sync + 'static,
        {
            self.inner
                .resolve_admitted_elevation_close_replay::<Operation, Input, Scope>(
                    phase,
                    admission,
                    idempotency,
                )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_attempt_idempotency_resolution {
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::idempotency_resolution::denial::WorthQueryApplicationIdempotencyResolutionDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitReceipt;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding;
    use crate::domain_computation::application_aftermath::WorthQueryAdmittedIdempotencyRead;
    use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
    use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_attempt::idempotency_resolution_types::WorthQueryGuardedWorkflowOperationCustody;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema,
    {
        pub fn resolve_admitted_guarded_workflow_operation_custody<Operation, Input, Scope>(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
            binding: WorthQueryApplicationIdempotencyBinding,
            transition_identity: &[u8; 32],
            completed: Option<&WorthQueryApplicationCommitReceipt>,
        ) -> Result<
            WorthQueryGuardedWorkflowOperationCustody,
            WorthQueryApplicationIdempotencyResolutionDenial,
        >
        where
            Input: Clone + Send + Sync + 'static,
        {
            self.inner
                .resolve_admitted_guarded_workflow_operation_custody::<Operation, Input, Scope>(
                    execution,
                    admission,
                    binding,
                    transition_identity,
                    completed,
                )
        }
        pub fn resolve_admitted_application_idempotency<Operation, Input, Scope>(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
            binding: WorthQueryApplicationIdempotencyBinding,
        ) -> Result<
            WorthQueryAdmittedIdempotencyRead,
            WorthQueryApplicationIdempotencyResolutionDenial,
        >
        where
            Input: Clone + Send + Sync + 'static,
        {
            self.inner
                .resolve_admitted_application_idempotency::<Operation, Input, Scope>(
                    phase, admission, binding,
                )
        }
        pub(crate) fn resolve_admitted_application_idempotencies<Operation, Input, Scope>(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
            bindings: impl IntoIterator<Item = WorthQueryApplicationIdempotencyBinding>,
        ) -> Result<
            Vec<WorthQueryAdmittedIdempotencyRead>,
            WorthQueryApplicationIdempotencyResolutionDenial,
        >
        where
            Input: Clone + Send + Sync + 'static,
        {
            self.inner
                .resolve_admitted_application_idempotencies::<Operation, Input, Scope>(
                    execution, admission, bindings,
                )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_attempt_mandatory_review_replay {
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyResolutionDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryMandatoryReview;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryMandatoryReviewOutcome;
    use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn resolve_admitted_mandatory_review_replay<Operation, Input, Scope>(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            admission: &mut WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
            idempotency: WorthQueryApplicationIdempotencyBinding,
        ) -> Result<
            Option<WorthQueryMandatoryReviewOutcome>,
            (
                WorthQueryApplicationIdempotencyResolutionDenial,
                WorthQueryMandatoryReview,
            ),
        >
        where
            Input: Clone + Send + Sync + 'static,
        {
            self.inner
                .resolve_admitted_mandatory_review_replay::<Operation, Input, Scope>(
                    phase,
                    admission,
                    idempotency,
                )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_attempt_retained_commit {
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcome;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationEffectProgram;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::retained_commit::WorthQueryApplicationRetainedCommitOutcome;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema,
    {
        #[doc(hidden)]
        pub fn compare_and_commit_application_retained_in_advancement<Operation, Input, Scope>(
            &self,
            phase: &worth_query_execution_query::facade::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
            idempotency: WorthQueryApplicationIdempotencyBinding,
            allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        ) -> WorthQueryApplicationRetainedCommitOutcome
        where
            Operation: 'static,
            Input: Clone + Send + Sync + 'static,
        {
            self.inner
                .compare_and_commit_application_retained_in_advancement::<Operation, Input, Scope>(
                    phase,
                    program,
                    idempotency,
                    allocation_policy,
                )
        }
        pub(crate) fn retained_commit_outcome(
            &self,
            outcome: WorthQueryApplicationCommitOutcome,
        ) -> WorthQueryApplicationRetainedCommitOutcome {
            self.inner.retained_commit_outcome(outcome)
        }
    }
}
