//! Thin forwarding shims for `court_doors_for_test` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_observed_source_prepared_expectation_court_doors_for_test_doors {
    use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
    use worth_query_execution_lineage::facade::primary_graph::application_query::observed_source::WorthQueryObservedSource;
    use worth_query_execution_lineage::facade::primary_graph::application_query::observed_source::WorthQuerySourceExpectationDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::observed_source::prepared_expectation::PreparedObservedSourceExpectation;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
    {
        #[doc(hidden)]
        pub fn prepare_application_source_expectation_for_test<Binding, Scope>(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            admission: &WorthQueryAdmittedApplicationOperation<
                Schema,
                Binding::Operation,
                Binding::Input,
                Scope,
            >,
            source: WorthQueryObservedSource<
                <Binding::SourceExpectation as worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation<Schema>>::Query,
            >,
            input: &Binding::Input,
        ) -> Result<PreparedObservedSourceExpectation, WorthQuerySourceExpectationDenial>
        where
            Binding:
                worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
                    Schema,
                >,
        {
            self.inner
                .prepare_application_source_expectation_for_test::<Binding, Scope>(
                    phase, admission, source, input,
                )
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
        Schema: ApplicationSchema,
    {
        #[doc(hidden)]
        pub fn execute_application_query_one_shot_with_request_for_test<
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            plan: WorthQueryAdmittedApplicationQueryPlan<
                '_,
                Schema,
                Query,
                Parameters,
                QueryResult,
                Principal,
                PrincipalIdentity,
                Scope,
            >,
        ) -> Result<
            WorthQueryApplicationOneShotResult<Query, QueryResult>,
            WorthQueryApplicationOneShotDenial,
        >
        where
            QueryResult: WorthQueryApplicationProjection<Schema, Query>,
        {
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
        >(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            plan: WorthQueryAdmittedApplicationQueryPlan<
                '_,
                Schema,
                Query,
                Parameters,
                QueryResult,
                Principal,
                PrincipalIdentity,
                Scope,
            >,
            original_subject: EntityId,
        ) -> Result<bool, WorthQueryApplicationOneShotDenial> {
            self.inner.execute_correction_disclosure_for_test::<Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>(execution, plan, original_subject)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_admission_court_doors_for_test_doors {
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAccessContext;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAdmissionDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryControls;
    use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema,
    {
        #[doc(hidden)]
        pub fn admit_application_query_for_test<
            'a,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >(
            &'a self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            query: &'a WorthQueryInstalledApplicationQuery<
                Schema,
                Query,
                Parameters,
                QueryResult,
                Scope,
            >,
            access: &WorthQueryApplicationQueryAccessContext<
                'a,
                Schema,
                Principal,
                PrincipalIdentity,
                Scope,
            >,
            parameters: ApplicationQueryParameterSet<Query>,
            controls: WorthQueryApplicationQueryControls<'a, Schema>,
        ) -> Result<
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
        pub fn with_fresh_retained_query_identities_for_test<
            'a,
            'selected,
            'runtime,
            Binding,
            R,
            E,
        >(
            &'a self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            installed: &'a WorthQueryInstalledApplicationQueryBinding<Schema, Binding>,
            validated_principal: Option<
                &worth_query_installation::facade::WorthQueryValidatedPrincipalBinding,
            >,
            external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
            observed: &WorthQueryObservedSource<impl Sized>,
            selected: &'selected WorthQuerySelectedProductOperation<'runtime, Schema>,
            request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
            inspect: impl for<'fresh> FnOnce(
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
            ) -> Result<R, FreshQueryPermissionStop<E>>,
        ) -> Result<R, FreshQueryPermissionStop<E>>
        where
            Binding: ApplicationQueryBinding<Schema>,
        {
            self.inner
                .with_fresh_retained_query_identities_for_test::<Binding, R, E>(
                    execution,
                    installed,
                    validated_principal,
                    external,
                    observed,
                    selected.inner(),
                    request,
                    inspect,
                )
        }
    }
}
