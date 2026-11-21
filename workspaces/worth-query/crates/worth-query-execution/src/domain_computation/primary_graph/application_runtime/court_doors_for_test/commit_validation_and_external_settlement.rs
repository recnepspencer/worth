//! Thin forwarding shims for `court_doors_for_test` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_attempt_provider_execution_phase_prepared_entry_custody_court_doors_for_test_doors {
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::carrier::CommitCarrier;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::CurrentCommitProgram;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::ReadmittedCommitProgram;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn validate_commit_program_for_test<
            Refusal,
            Program: CommitCarrier<Schema, Refusal = Refusal>,
        >(
            &self,
            program: Program,
        ) -> Result<ReadmittedCommitProgram<'_, Schema, Program>, Refusal> {
            self.inner
                .validate_commit_program_for_test::<Refusal, Program>(program)
        }
        #[doc(hidden)]
        pub fn validate_commit_program_in_advancement_for_test<
            'a,
            'phase,
            'request,
            Refusal,
            Program: CommitCarrier<Schema, Refusal = Refusal>,
        >(
            &'a self,
            phase: &'phase WorthQueryAdvancementPhase<'request>,
            program: Program,
        ) -> Result<CurrentCommitProgram<'a, 'phase, 'request, Schema, Program>, Refusal> {
            self.inner
                .validate_commit_program_in_advancement_for_test::<Refusal, Program>(phase, program)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_attempt_provider_execution_external_dispatch_court_doors_for_test_doors {
    use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_attempt::provider_execution::external_dispatch::*;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::application_aftermath::WorthQueryExternalEffectDispatch;
    use crate::domain_computation::application_aftermath::WorthQueryExternalEffectTransport;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema,
    {
        #[doc(hidden)]
        pub fn perform_committed_external_dispatch_for_test(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            phase: &WorthQueryAdvancementPhase<'_>,
            transport: &dyn WorthQueryExternalEffectTransport,
            committed: crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxObservation,
            request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        ) -> Result<WorthQueryExternalEffectDispatch, WorthQueryExternalDispatchPreparationDenial>
        {
            self.inner.perform_committed_external_dispatch_for_test(
                execution, phase, transport, committed, request,
            )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_attempt_provider_execution_elevation_lifecycle_installed_readmission_court_doors_for_test_doors {
    use crate::domain_computation::primary_graph::application_attempt::ReadmittedCommitProgram;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryElevationRequestOutcome;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryElevationRequestProgram;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn readmit_elevation_request_installation_for_test<Operation, Input, Scope>(
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
                .readmit_elevation_request_installation_for_test::<Operation, Input, Scope>(program)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_attempt_idempotency_resolution_external_settlement_court_doors_for_test_doors {
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::provider::WorthQueryInboundTerminalIndexDenial as Denial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitReceipt;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: worth_query_installation::facade::ApplicationSchema>
        WorthQueryPrimaryGraphApplicationRuntime<Schema>
    {
        #[doc(hidden)]
        pub fn resolve_guarded_workflow_external_settlement_for_test(
            &self,
            receipt: &WorthQueryApplicationCommitReceipt,
        ) -> Result<bool, Denial> {
            self.inner
                .resolve_guarded_workflow_external_settlement_for_test(receipt)
        }
    }
}
