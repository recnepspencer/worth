//! Thin forwarding shims for `court_doors_for_test` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_admission_preparation_court_doors_for_test_doors {
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAdmissionDenial;
    use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema,
    {
        #[doc(hidden)]
        pub fn validate_installed_query_for_test<Query, Parameters, QueryResult, Scope>(
            &self,
            query: &WorthQueryInstalledApplicationQuery<
                Schema,
                Query,
                Parameters,
                QueryResult,
                Scope,
            >,
        ) -> Result<(), WorthQueryApplicationQueryAdmissionDenial> {
            self.inner
                .validate_installed_query_for_test::<Query, Parameters, QueryResult, Scope>(query)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_basis_truth_view_admission_historical_court_doors_for_test_doors {
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::basis::historical_authority::WorthQueryApplicationHistoricalBasis;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::basis::historical_authority::WorthQueryApplicationHistoricalRead;
    use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAdmissionDenial;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema,
    {
        #[doc(hidden)]
        pub fn admit_application_historical_basis_for_test(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            read: WorthQueryApplicationHistoricalRead,
            request: &WorthQueryRequestScope,
        ) -> Result<
            WorthQueryApplicationHistoricalBasis<Schema>,
            WorthQueryApplicationQueryAdmissionDenial,
        > {
            self.inner
                .admit_application_historical_basis_for_test(execution, read, request)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_certification_controls_doors {
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    use worth_query_installation::facade::ApplicationSchema;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema + 'static,
    {
        #[doc(hidden)]
        pub fn fail_next_post_commit_snapshot_for_test(&self) {
            self.inner.fail_next_post_commit_snapshot_for_test()
        }
        #[doc(hidden)]
        pub fn release_workflow_instance_progress_for_test(&self) {
            self.inner.release_workflow_instance_progress_for_test()
        }
        #[doc(hidden)]
        pub fn release_workflow_compilation_for_test(&self) {
            self.inner.release_workflow_compilation_for_test()
        }
        #[doc(hidden)]
        pub fn delay_next_output_readiness_delivery_for_test(&self) {
            self.inner.delay_next_output_readiness_delivery_for_test()
        }
        #[doc(hidden)]
        pub fn fail_next_output_readiness_evaluation_for_test(&self) {
            self.inner.fail_next_output_readiness_evaluation_for_test()
        }
        #[doc(hidden)]
        pub fn press_next_ready_read_with_world_snapshots_for_test(&self) {
            self.inner
                .press_next_ready_read_with_world_snapshots_for_test()
        }
        #[doc(hidden)]
        pub fn press_next_readiness_with_world_snapshots_for_test(&self) {
            self.inner
                .press_next_readiness_with_world_snapshots_for_test()
        }
        #[doc(hidden)]
        pub fn cancel_after_next_output_comparison_for_test(
            &self,
            source: worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource,
        ) -> bool {
            self.inner
                .cancel_after_next_output_comparison_for_test(source)
        }
        #[doc(hidden)]
        pub fn output_readiness_attempt_count_for_test(&self) -> u64 {
            self.inner.output_readiness_attempt_count_for_test()
        }
        #[doc(hidden)]
        pub fn output_checkpoint_snapshot_state_for_test(&self) -> (usize, usize) {
            self.inner.output_checkpoint_snapshot_state_for_test()
        }
        #[doc(hidden)]
        pub fn prepared_required_output_source_count_for_test(&self) -> usize {
            self.inner.prepared_required_output_source_count_for_test()
        }
        #[doc(hidden)]
        pub fn retained_source_custody_count_for_test(&self) -> usize {
            self.inner.retained_source_custody_count_for_test()
        }
        #[doc(hidden)]
        pub fn primary_truth_snapshot_for_test(
            &self,
        ) -> Option<worth_runtime_bridge::facade::TruthSnapshotIdentity> {
            self.inner.primary_truth_snapshot_for_test()
        }
        #[doc(hidden)]
        pub fn relational_snapshot_state_for_test(
            &self,
            request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        ) -> (
            usize,
            worth_relational::facade::branch::RelationalBranchBasisDescriptor,
        ) {
            self.inner
                .relational_snapshot_state_for_test(request_execution)
        }
        #[doc(hidden)]
        pub fn attempt_workflow_node_field_update_for_test(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            definition: &crate::domain_computation::primary_graph::PublishedWorkflowDefinitionRef,
        ) -> Result<
            Result<(), worth_relational::facade::transactions::TransactionCommitError>,
            crate::domain_computation::primary_graph::WorthQueryForeignAdvancementPhase,
        > {
            self.inner
                .attempt_workflow_node_field_update_for_test(phase, definition)
        }
        #[doc(hidden)]
        pub fn attempt_workflow_existing_node_attachment_for_test(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            definition: &crate::domain_computation::primary_graph::PublishedWorkflowDefinitionRef,
        ) -> Result<
            Result<(), worth_relational::facade::transactions::TransactionCommitError>,
            crate::domain_computation::primary_graph::WorthQueryForeignAdvancementPhase,
        > {
            self.inner
                .attempt_workflow_existing_node_attachment_for_test(phase, definition)
        }
        pub fn provider_session_resource_count_for_test(&self) -> usize {
            self.inner.provider_session_resource_count_for_test()
        }
        pub fn application_attempt_work_for_test(&self) -> crate::domain_computation::primary_graph::provider::WorthQueryApplicationAttemptWorkSnapshot{
            self.inner.application_attempt_work_for_test()
        }
    }
}
