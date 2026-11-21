//! Thin forwarding shims for `court_doors_for_test` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_observed_source_prepared_expectation_court_doors_for_test_doors {
    use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
    use worth_query_execution_lineage::facade::primary_graph::application_query::observed_source::WorthQueryObservedSource;
    use worth_query_execution_lineage::facade::primary_graph::application_query::observed_source::WorthQuerySourceExpectationDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::observed_source::prepared_expectation::PreparedObservedSourceExpectation;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema>
    WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema, {
        #[doc(hidden)]
        pub fn prepare_application_source_expectation_for_test<Binding, Scope>(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, admission: &WorthQueryAdmittedApplicationOperation<
                Schema,
                Binding::Operation,
                Binding::Input,
                Scope,
            >, source: WorthQueryObservedSource<
                <Binding::SourceExpectation as worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation<Schema>>::Query,
            >, input: &Binding::Input) -> Result<PreparedObservedSourceExpectation, WorthQuerySourceExpectationDenial>
        where
            Binding: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
                Schema,
            >, {
            self.inner.prepare_application_source_expectation_for_test::<Binding, Scope>(phase, admission, source, input)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_one_shot_court_doors_for_test_doors {
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::one_shot::denial::WorthQueryApplicationOneShotDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryApplicationProjection;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::one_shot::WorthQueryApplicationOneShotResult;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema, {
        #[doc(hidden)]
        pub fn execute_application_query_one_shot_with_request_for_test<
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, plan: WorthQueryAdmittedApplicationQueryPlan<
                '_,
                Schema,
                Query,
                Parameters,
                QueryResult,
                Principal,
                PrincipalIdentity,
                Scope,
            >) -> Result<
            WorthQueryApplicationOneShotResult<Query, QueryResult>,
            WorthQueryApplicationOneShotDenial,
        >
        where
            QueryResult: WorthQueryApplicationProjection<Schema, Query>, {
            self.inner.execute_application_query_one_shot_with_request_for_test::<Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>(execution, plan)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_one_shot_correction_disclosure_court_doors_for_test_doors {
    use worth_relational::facade::identity::EntityId;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::one_shot::denial::WorthQueryApplicationOneShotDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn execute_correction_disclosure_for_test<
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, plan: WorthQueryAdmittedApplicationQueryPlan<
                '_,
                Schema,
                Query,
                Parameters,
                QueryResult,
                Principal,
                PrincipalIdentity,
                Scope,
            >, original_subject: EntityId) -> Result<bool, WorthQueryApplicationOneShotDenial> {
            self.inner.execute_correction_disclosure_for_test::<Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>(execution, plan, original_subject)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_admission_court_doors_for_test_doors {
    use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAccessContext;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAdmissionDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryControls;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema, {
        #[doc(hidden)]
        pub fn admit_application_query_for_test<
            'a,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >(&'a self, execution: &worth_execution::ExecutionRequest<'_, '_>, query: &'a WorthQueryInstalledApplicationQuery<
                Schema,
                Query,
                Parameters,
                QueryResult,
                Scope,
            >, access: &WorthQueryApplicationQueryAccessContext<
                'a,
                Schema,
                Principal,
                PrincipalIdentity,
                Scope,
            >, parameters: ApplicationQueryParameterSet<Query>, controls: WorthQueryApplicationQueryControls<'a, Schema>) -> Result<
            WorthQueryAdmittedApplicationQueryPlan<
                'a,
                Schema,
                Query,
                Parameters,
                QueryResult,
                Principal,
                PrincipalIdentity,
                Scope,
            >,
            WorthQueryApplicationQueryAdmissionDenial,
        > {
            self.inner.admit_application_query_for_test::<Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>(execution, query, access, parameters, controls)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_admission_permission_fresh_access_court_doors_for_test_doors {
    use crate::domain_computation::primary_graph::WorthQueryIssuedSelectedPrincipal;
    use crate::domain_computation::primary_graph::WorthQueryIssuedSelectedScope;
    use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
    use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
    use worth_query_declaration::facade::application_query::ApplicationQueryScopeBinding;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;
    use crate::domain_computation::primary_graph::WorthQueryObservedSource;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAccessContext;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::admission::permission::fresh::FreshQueryPermissionStop;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    type Scope<Schema, Binding> =
    <<Binding as ApplicationQueryBinding<Schema>>::ScopeBinding as ApplicationQueryScopeBinding<
        Schema,
    >>::Scope;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        #[allow(clippy::too_many_arguments)]
        pub fn with_fresh_retained_query_identities_for_test<'a, 'selected, 'runtime, Binding, R, E>(&'a self, execution: &worth_execution::ExecutionRequest<'_, '_>, installed: &'a WorthQueryInstalledApplicationQueryBinding<Schema, Binding>, validated_principal: Option<
                &worth_query_installation::facade::WorthQueryValidatedPrincipalBinding,
            >, external: &WorthQueryAuthenticatedExternalPrincipal<Schema>, observed: &WorthQueryObservedSource<impl Sized>, selected: &'selected WorthQuerySelectedProductOperation<'runtime, Schema>, request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope, inspect: impl for<'fresh> FnOnce(
                WorthQueryApplicationQueryAccessContext<
                    'fresh,
                    Schema,
                    Binding::Principal,
                    Binding::PrincipalIdentity,
                    Scope<Schema, Binding>,
                >,
                &'fresh WorthQueryIssuedSelectedPrincipal<
                    'selected,
                    'runtime,
                    Schema,
                    Binding::Principal,
                    Binding::PrincipalIdentity,
                >,
                &'fresh WorthQueryIssuedSelectedScope<
                    'selected,
                    'runtime,
                    Schema,
                    Scope<Schema, Binding>,
                >,
            ) -> Result<R, FreshQueryPermissionStop<E>>) -> Result<R, FreshQueryPermissionStop<E>>
        where
            Binding: ApplicationQueryBinding<Schema>, {
            self.inner.with_fresh_retained_query_identities_for_test::<Binding, R, E>(execution, installed, validated_principal, external, observed, selected.inner(), request, inspect)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_admission_preparation_court_doors_for_test_doors {
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAdmissionDenial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema, {
        #[doc(hidden)]
        pub fn validate_installed_query_for_test<Query, Parameters, QueryResult, Scope>(&self, query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>) -> Result<(), WorthQueryApplicationQueryAdmissionDenial> {
            self.inner.validate_installed_query_for_test::<Query, Parameters, QueryResult, Scope>(query)
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
    Schema: ApplicationSchema, {
        #[doc(hidden)]
        pub fn admit_application_historical_basis_for_test(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, read: WorthQueryApplicationHistoricalRead, request: &WorthQueryRequestScope) -> Result<
            WorthQueryApplicationHistoricalBasis<Schema>,
            WorthQueryApplicationQueryAdmissionDenial,
        > {
            self.inner.admit_application_historical_basis_for_test(execution, read, request)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_certification_controls_doors {
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static, {
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
            self.inner.press_next_ready_read_with_world_snapshots_for_test()
        }
        #[doc(hidden)]
        pub fn press_next_readiness_with_world_snapshots_for_test(&self) {
            self.inner.press_next_readiness_with_world_snapshots_for_test()
        }
        #[doc(hidden)]
        pub fn cancel_after_next_output_comparison_for_test(&self, source: worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource) -> bool {
            self.inner.cancel_after_next_output_comparison_for_test(source)
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
        pub fn primary_truth_snapshot_for_test(&self) -> Option<worth_runtime_bridge::facade::TruthSnapshotIdentity> {
            self.inner.primary_truth_snapshot_for_test()
        }
        #[doc(hidden)]
        pub fn relational_snapshot_state_for_test(&self, request_execution: &worth_execution::ExecutionRequest<'_, '_>) -> (
            usize,
            worth_relational::facade::branch::RelationalBranchBasisDescriptor,
        ) {
            self.inner.relational_snapshot_state_for_test(request_execution)
        }
        #[doc(hidden)]
        pub fn attempt_workflow_node_field_update_for_test(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, definition: &crate::domain_computation::primary_graph::PublishedWorkflowDefinitionRef) -> Result<
            Result<(), worth_relational::facade::transactions::TransactionCommitError>,
            crate::domain_computation::primary_graph::WorthQueryForeignAdvancementPhase,
        > {
            self.inner.attempt_workflow_node_field_update_for_test(phase, definition)
        }
        #[doc(hidden)]
        pub fn attempt_workflow_existing_node_attachment_for_test(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, definition: &crate::domain_computation::primary_graph::PublishedWorkflowDefinitionRef) -> Result<
            Result<(), worth_relational::facade::transactions::TransactionCommitError>,
            crate::domain_computation::primary_graph::WorthQueryForeignAdvancementPhase,
        > {
            self.inner.attempt_workflow_existing_node_attachment_for_test(phase, definition)
        }
        pub fn provider_session_resource_count_for_test(&self) -> usize {
            self.inner.provider_session_resource_count_for_test()
        }
        pub fn application_attempt_work_for_test(&self) -> crate::domain_computation::primary_graph::provider::WorthQueryApplicationAttemptWorkSnapshot {
            self.inner.application_attempt_work_for_test()
        }
    }
}
