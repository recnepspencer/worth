//! Thin forwarding shims for `application_attempt` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_attempt_provider_execution_entry_application_commit {
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcome;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationEffectProgram;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn compare_and_commit_application<Operation, Input, Scope>(&self, program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>, idempotency: WorthQueryApplicationIdempotencyBinding, allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>) -> WorthQueryApplicationCommitOutcome
        where
            Operation: 'static,
            Input: Clone + Send + Sync + 'static, {
            self.inner.compare_and_commit_application::<Operation, Input, Scope>(program, idempotency, allocation_policy)
        }
        #[doc(hidden)]
        pub fn compare_and_commit_application_in_advancement<Operation, Input, Scope>(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>, idempotency: WorthQueryApplicationIdempotencyBinding, allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>) -> WorthQueryApplicationCommitOutcome
        where
            Operation: 'static,
            Input: Clone + Send + Sync + 'static, {
            self.inner.compare_and_commit_application_in_advancement::<Operation, Input, Scope>(phase, program, idempotency, allocation_policy)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_attempt_provider_execution_entry_output_source {
    use crate::domain_computation::primary_graph::application_attempt::effect_program::candidate_planning::EffectProgramCandidatePlanningExt as _;
    use crate::domain_computation::application_aftermath::ApplicationCommitCausality;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcome;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationEffectProgram;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding;
    use crate::domain_computation::primary_graph::product_operation::document_history::HistoryProducerInvocation;
    use crate::domain_computation::primary_graph::program_occurrence::WorthQueryPresentedProgram;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[allow(clippy::too_many_arguments)]
        pub fn compare_and_commit_application_for_required_output_source<Operation, Input, Scope>(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, phase: &WorthQueryAdvancementPhase<'_>, presented: &WorthQueryPresentedProgram<'_>, program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>, idempotency: WorthQueryApplicationIdempotencyBinding, causality: ApplicationCommitCausality, allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>) -> WorthQueryApplicationCommitOutcome
        where
            Input: Clone + Send + Sync + 'static, {
            self.inner.compare_and_commit_application_for_required_output_source::<Operation, Input, Scope>(execution, phase, presented, program, idempotency, causality, allocation_policy)
        }
        #[allow(clippy::too_many_arguments)]
        pub(crate) fn compare_and_commit_application_in_edit_envelope<
            Operation,
            Input,
            Scope,
        >(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, phase: &WorthQueryAdvancementPhase<'_>, presented: &WorthQueryPresentedProgram<'_>, program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>, idempotency: WorthQueryApplicationIdempotencyBinding, unseen: &crate::domain_computation::primary_graph::application_contribution::authored_edit::UnseenAuthoredEnvelope<'_>, causality: ApplicationCommitCausality, allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>) -> WorthQueryApplicationCommitOutcome
        where
            Input: Clone + Send + Sync + 'static, {
            self.inner.compare_and_commit_application_in_edit_envelope_with_outer::<Operation, Input, Scope>(execution, phase, presented, program, idempotency, unseen, causality, allocation_policy, Some(self))
        }
        pub fn compare_and_commit_application_for_program_output_producer<Operation, Input, Scope>(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, phase: &WorthQueryAdvancementPhase<'_>, program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>, idempotency: WorthQueryApplicationIdempotencyBinding, selected_program: Option<(
                &worth_query_declaration::facade::application_program::ApplicationProgramIdentity,
                &worth_query_declaration::facade::application_program::ApplicationProgramRevision,
            )>, history: Option<HistoryProducerInvocation<'_>>) -> WorthQueryApplicationCommitOutcome
        where
            Operation: 'static,
            Input: Clone + Send + Sync + 'static, {
            self.inner.compare_and_commit_application_for_program_output_producer::<Operation, Input, Scope>(execution, phase, program, idempotency, selected_program, history)
        }
        #[allow(clippy::too_many_arguments)]
        pub fn compare_and_commit_application_for_program_action<Operation, Input, Scope>(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, phase: &WorthQueryAdvancementPhase<'_>, presented: &WorthQueryPresentedProgram, program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>, idempotency: WorthQueryApplicationIdempotencyBinding, causality: ApplicationCommitCausality, allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>) -> WorthQueryApplicationCommitOutcome
        where
            Operation: 'static,
            Input: Clone + Send + Sync + 'static, {
            self.inner.compare_and_commit_application_for_program_action::<Operation, Input, Scope>(execution, phase, presented, program, idempotency, causality, allocation_policy)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_attempt_provider_execution_external_dispatch_cleanup_dispatch {
    use crate::domain_computation::primary_graph::WorthQueryApplicationManagedCleanupReceipt as Receipt;
    use crate::domain_computation::primary_graph::WorthQueryCleanupDispatchDenial as Denial;
    use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_attempt::provider_execution::external_dispatch::*;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::application_aftermath::WorthQueryExternalEffectDispatch;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn resume_cleanup_original_dispatch<'a>(&self, phase: &WorthQueryAdvancementPhase<'_>, receipt: &'a mut Receipt, request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope) -> Result<Option<&'a WorthQueryExternalEffectDispatch>, Denial> {
            self.inner.resume_cleanup_original_dispatch(phase, receipt, request)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_attempt_provider_execution_elevation_lifecycle_elevation_approval {
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryElevationApprovalOutcome;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryElevationApprovalProgram;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema, {
        pub fn compare_and_commit_elevation_approval<Operation, Input, Scope>(&self, program: WorthQueryElevationApprovalProgram<Schema, Operation, Input, Scope>, idempotency: WorthQueryApplicationIdempotencyBinding) -> WorthQueryElevationApprovalOutcome
        where
            Operation: 'static,
            Input: Clone + Send + Sync + 'static, {
            self.inner.compare_and_commit_elevation_approval::<Operation, Input, Scope>(program, idempotency)
        }
        pub(crate) fn compare_and_commit_elevation_approval_for_program<
            Operation,
            Input,
            Scope,
        >(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, phase: &WorthQueryAdvancementPhase<'_>, program: WorthQueryElevationApprovalProgram<Schema, Operation, Input, Scope>, idempotency: WorthQueryApplicationIdempotencyBinding, direct_denial: Option<WorthQueryApplicationCommitDenial>) -> WorthQueryElevationApprovalOutcome
        where
            Input: Clone + Send + Sync + 'static, {
            self.inner.compare_and_commit_elevation_approval_for_program::<Operation, Input, Scope>(execution, phase, program, idempotency, direct_denial)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_attempt_provider_execution_elevation_lifecycle_elevation_close {
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryElevationCloseOutcome;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryElevationCloseProgram;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema, {
        pub fn compare_and_commit_elevation_close<Operation, Input, Scope>(&self, program: WorthQueryElevationCloseProgram<Schema, Operation, Input, Scope>, idempotency: WorthQueryApplicationIdempotencyBinding) -> WorthQueryElevationCloseOutcome
        where
            Operation: 'static,
            Input: Clone + Send + Sync + 'static, {
            self.inner.compare_and_commit_elevation_close::<Operation, Input, Scope>(program, idempotency)
        }
        pub(crate) fn compare_and_commit_elevation_close_for_program<
            Operation,
            Input,
            Scope,
        >(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, phase: &WorthQueryAdvancementPhase<'_>, program: WorthQueryElevationCloseProgram<Schema, Operation, Input, Scope>, idempotency: WorthQueryApplicationIdempotencyBinding, direct_denial: Option<WorthQueryApplicationCommitDenial>) -> WorthQueryElevationCloseOutcome
        where
            Input: Clone + Send + Sync + 'static, {
            self.inner.compare_and_commit_elevation_close_for_program::<Operation, Input, Scope>(execution, phase, program, idempotency, direct_denial)
        }
    }
}

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
    Schema: ApplicationSchema, {
        pub fn compare_and_commit_elevation_request<Operation, Input, Scope>(&self, program: WorthQueryElevationRequestProgram<Schema, Operation, Input, Scope>, idempotency: WorthQueryApplicationIdempotencyBinding) -> WorthQueryElevationRequestOutcome
        where
            Operation: 'static,
            Input: Clone + Send + Sync + 'static, {
            self.inner.compare_and_commit_elevation_request::<Operation, Input, Scope>(program, idempotency)
        }
        pub(crate) fn compare_and_commit_elevation_request_for_program<
            Operation,
            Input,
            Scope,
        >(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, phase: &WorthQueryAdvancementPhase<'_>, program: WorthQueryElevationRequestProgram<Schema, Operation, Input, Scope>, idempotency: WorthQueryApplicationIdempotencyBinding) -> WorthQueryElevationRequestOutcome
        where
            Input: Clone + Send + Sync + 'static, {
            self.inner.compare_and_commit_elevation_request_for_program::<Operation, Input, Scope>(execution, phase, program, idempotency)
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
        pub(crate) fn readmit_elevation_request_installation<
            Operation,
            Input,
            Scope,
        >(&self, program: WorthQueryElevationRequestProgram<Schema, Operation, Input, Scope>) -> Result<
            ReadmittedCommitProgram<
                '_,
                Schema,
                WorthQueryElevationRequestProgram<Schema, Operation, Input, Scope>,
            >,
            WorthQueryElevationRequestOutcome,
        > {
            self.inner.readmit_elevation_request_installation::<Operation, Input, Scope>(program)
        }
        pub(crate) fn readmit_elevation_approval_installation<
            Operation,
            Input,
            Scope,
        >(&self, program: WorthQueryElevationApprovalProgram<Schema, Operation, Input, Scope>) -> Result<
            ReadmittedCommitProgram<
                '_,
                Schema,
                WorthQueryElevationApprovalProgram<Schema, Operation, Input, Scope>,
            >,
            WorthQueryElevationApprovalOutcome,
        > {
            self.inner.readmit_elevation_approval_installation::<Operation, Input, Scope>(program)
        }
        pub(crate) fn readmit_elevation_close_installation<
            Operation,
            Input,
            Scope,
        >(&self, program: WorthQueryElevationCloseProgram<Schema, Operation, Input, Scope>) -> Result<
            ReadmittedCommitProgram<
                '_,
                Schema,
                WorthQueryElevationCloseProgram<Schema, Operation, Input, Scope>,
            >,
            WorthQueryElevationCloseOutcome,
        > {
            self.inner.readmit_elevation_close_installation::<Operation, Input, Scope>(program)
        }
        pub(crate) fn readmit_mandatory_review_installation<
            Operation,
            Input,
            Scope,
        >(&self, program: WorthQueryMandatoryReviewProgram<Schema, Operation, Input, Scope>) -> Result<
            ReadmittedCommitProgram<
                '_,
                Schema,
                WorthQueryMandatoryReviewProgram<Schema, Operation, Input, Scope>,
            >,
            WorthQueryMandatoryReviewOutcome,
        > {
            self.inner.readmit_mandatory_review_installation::<Operation, Input, Scope>(program)
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
    Schema: ApplicationSchema, {
        pub fn compare_and_commit_mandatory_review<Operation, Input, Scope>(&self, program: WorthQueryMandatoryReviewProgram<Schema, Operation, Input, Scope>, idempotency: WorthQueryApplicationIdempotencyBinding) -> WorthQueryMandatoryReviewOutcome
        where
            Operation: 'static,
            Input: Clone + Send + Sync + 'static, {
            self.inner.compare_and_commit_mandatory_review::<Operation, Input, Scope>(program, idempotency)
        }
        pub(crate) fn compare_and_commit_mandatory_review_for_program<
            Operation,
            Input,
            Scope,
        >(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, phase: &WorthQueryAdvancementPhase<'_>, program: WorthQueryMandatoryReviewProgram<Schema, Operation, Input, Scope>, idempotency: WorthQueryApplicationIdempotencyBinding, direct_denial: Option<WorthQueryApplicationCommitDenial>) -> WorthQueryMandatoryReviewOutcome
        where
            Input: Clone + Send + Sync + 'static, {
            self.inner.compare_and_commit_mandatory_review_for_program::<Operation, Input, Scope>(execution, phase, program, idempotency, direct_denial)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_attempt_idempotency_resolution_external_settlement {
    use crate::domain_computation::primary_graph::provider::WorthQueryInboundTerminalIndexDenial as Denial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: worth_query_installation::facade::ApplicationSchema>
    WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub(crate) fn resolve_correction_external_settlement(&self, occurrence: &crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphCommittedApplication) -> Result<bool, Denial> {
            self.inner.resolve_correction_external_settlement(occurrence)
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
    WorthQueryPrimaryGraphApplicationRuntime<S> {
        pub fn fragment_candidate<
            'a,
            F: worth_query_declaration::facade::application_schema::fragment::Fragment,
            O,
            I,
            Scope,
        >(&'a self, candidate: &'a mut Builder<S, O, I, Scope>, marker: F) -> Result<
            crate::domain_computation::primary_graph::fragment_view::View<
                'a,
                F,
                crate::domain_computation::primary_graph::fragment_view::Writing<'a>,
            >,
            FragmentDenial,
        > {
            self.inner.fragment_candidate::<F, O, I, Scope>(candidate, marker)
        }
    }
}
