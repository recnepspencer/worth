//! Thin forwarding shims for `application_attempt` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_attempt_provider_execution_elevation_lifecycle_elevation_request {
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryElevationRequestOutcome;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryElevationRequestProgram;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema,
    {
        pub fn compare_and_commit_elevation_request<Operation, Input, Scope>(
            &self,
            program: WorthQueryElevationRequestProgram<Schema, Operation, Input, Scope>,
            idempotency: WorthQueryApplicationIdempotencyBinding,
        ) -> WorthQueryElevationRequestOutcome
        where
            Operation: 'static,
            Input: Clone + Send + Sync + 'static,
        {
            self.inner
                .compare_and_commit_elevation_request::<Operation, Input, Scope>(
                    program,
                    idempotency,
                )
        }
        pub(crate) fn compare_and_commit_elevation_request_for_program<Operation, Input, Scope>(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            phase: &WorthQueryAdvancementPhase<'_>,
            program: WorthQueryElevationRequestProgram<Schema, Operation, Input, Scope>,
            idempotency: WorthQueryApplicationIdempotencyBinding,
        ) -> WorthQueryElevationRequestOutcome
        where
            Input: Clone + Send + Sync + 'static,
        {
            self.inner
                .compare_and_commit_elevation_request_for_program::<Operation, Input, Scope>(
                    execution,
                    phase,
                    program,
                    idempotency,
                )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_attempt_provider_execution_elevation_lifecycle_installed_readmission {
    use crate::domain_computation::primary_graph::application_attempt::ReadmittedCommitProgram;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryElevationApprovalOutcome;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryElevationApprovalProgram;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryElevationCloseOutcome;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryElevationCloseProgram;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryElevationRequestOutcome;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryElevationRequestProgram;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryMandatoryReviewOutcome;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryMandatoryReviewProgram;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub(crate) fn readmit_elevation_request_installation<Operation, Input, Scope>(
            &self,
            program: WorthQueryElevationRequestProgram<Schema, Operation, Input, Scope>,
        ) -> Result<
            ReadmittedCommitProgram<
                '_,
                Schema,
                WorthQueryElevationRequestProgram<Schema, Operation, Input, Scope>,
            >,
            WorthQueryElevationRequestOutcome,
        > {
            self.inner
                .readmit_elevation_request_installation::<Operation, Input, Scope>(program)
        }
        pub(crate) fn readmit_elevation_approval_installation<Operation, Input, Scope>(
            &self,
            program: WorthQueryElevationApprovalProgram<Schema, Operation, Input, Scope>,
        ) -> Result<
            ReadmittedCommitProgram<
                '_,
                Schema,
                WorthQueryElevationApprovalProgram<Schema, Operation, Input, Scope>,
            >,
            WorthQueryElevationApprovalOutcome,
        > {
            self.inner
                .readmit_elevation_approval_installation::<Operation, Input, Scope>(program)
        }
        pub(crate) fn readmit_elevation_close_installation<Operation, Input, Scope>(
            &self,
            program: WorthQueryElevationCloseProgram<Schema, Operation, Input, Scope>,
        ) -> Result<
            ReadmittedCommitProgram<
                '_,
                Schema,
                WorthQueryElevationCloseProgram<Schema, Operation, Input, Scope>,
            >,
            WorthQueryElevationCloseOutcome,
        > {
            self.inner
                .readmit_elevation_close_installation::<Operation, Input, Scope>(program)
        }
        pub(crate) fn readmit_mandatory_review_installation<Operation, Input, Scope>(
            &self,
            program: WorthQueryMandatoryReviewProgram<Schema, Operation, Input, Scope>,
        ) -> Result<
            ReadmittedCommitProgram<
                '_,
                Schema,
                WorthQueryMandatoryReviewProgram<Schema, Operation, Input, Scope>,
            >,
            WorthQueryMandatoryReviewOutcome,
        > {
            self.inner
                .readmit_mandatory_review_installation::<Operation, Input, Scope>(program)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_attempt_provider_execution_elevation_lifecycle_mandatory_review {
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryMandatoryReviewOutcome;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryMandatoryReviewProgram;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema,
    {
        pub fn compare_and_commit_mandatory_review<Operation, Input, Scope>(
            &self,
            program: WorthQueryMandatoryReviewProgram<Schema, Operation, Input, Scope>,
            idempotency: WorthQueryApplicationIdempotencyBinding,
        ) -> WorthQueryMandatoryReviewOutcome
        where
            Operation: 'static,
            Input: Clone + Send + Sync + 'static,
        {
            self.inner
                .compare_and_commit_mandatory_review::<Operation, Input, Scope>(
                    program,
                    idempotency,
                )
        }
        pub(crate) fn compare_and_commit_mandatory_review_for_program<Operation, Input, Scope>(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            phase: &WorthQueryAdvancementPhase<'_>,
            program: WorthQueryMandatoryReviewProgram<Schema, Operation, Input, Scope>,
            idempotency: WorthQueryApplicationIdempotencyBinding,
            direct_denial: Option<WorthQueryApplicationCommitDenial>,
        ) -> WorthQueryMandatoryReviewOutcome
        where
            Input: Clone + Send + Sync + 'static,
        {
            self.inner
                .compare_and_commit_mandatory_review_for_program::<Operation, Input, Scope>(
                    execution,
                    phase,
                    program,
                    idempotency,
                    direct_denial,
                )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_attempt_idempotency_resolution_external_settlement {
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    use crate::domain_computation::primary_graph::provider::WorthQueryInboundTerminalIndexDenial as Denial;
    impl<Schema: worth_query_installation::facade::ApplicationSchema>
        WorthQueryPrimaryGraphApplicationRuntime<Schema>
    {
        pub(crate) fn resolve_correction_external_settlement(
            &self,
            occurrence: &crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphCommittedApplication,
        ) -> Result<bool, Denial> {
            self.inner
                .resolve_correction_external_settlement(occurrence)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_attempt_effect_program_fragment {
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::effect_program::WorthQueryApplicationEffectProgramBuilder as Builder;
    use crate::domain_computation::primary_graph::fragment_view::FragmentDenial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<S: worth_query_installation::facade::ApplicationSchema>
        WorthQueryPrimaryGraphApplicationRuntime<S>
    {
        pub fn fragment_candidate<
            'a,
            F: worth_query_declaration::facade::application_schema::fragment::Fragment,
            O,
            I,
            Scope,
        >(
            &'a self,
            candidate: &'a mut Builder<S, O, I, Scope>,
            marker: F,
        ) -> Result<
            crate::domain_computation::primary_graph::fragment_view::View<
                'a,
                F,
                crate::domain_computation::primary_graph::fragment_view::Writing<'a>,
            >,
            FragmentDenial,
        > {
            self.inner
                .fragment_candidate::<F, O, I, Scope>(candidate, marker)
        }
    }
}
