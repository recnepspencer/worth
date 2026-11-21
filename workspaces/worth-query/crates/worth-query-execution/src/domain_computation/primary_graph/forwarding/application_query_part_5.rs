//! Thin forwarding shims for `application_query` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_admission_work_limit {
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema>
    WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn resolve_application_query_limits(&self, limits: worth_query_installation::facade::WorthQueryInstalledApplicationQueryLimits) -> worth_query_installation::facade::WorthQueryResolvedApplicationQueryLimits {
            self.inner.resolve_application_query_limits(limits)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_admission_finish_ordinary {
    use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::disclosure::WorthQueryPendingApplicationQueryGovernance;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAccessContext;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAdmissionDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryControls;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub(crate) fn finish_application_query_admission<
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
            >, parameters: WorthQueryAdmittedApplicationQueryParameters, controls: WorthQueryApplicationQueryControls<'a, Schema>, pending_governance: Option<WorthQueryPendingApplicationQueryGovernance>) -> Result<
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
            self.inner.finish_application_query_admission::<Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>(execution, query, access, parameters, controls, pending_governance)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_admission_finish_prepared_permission {
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use crate::domain_computation::primary_graph::application_query::admission::permission::PreparedApplicationQueryPermission;
    use crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan;
    use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAdmissionDenial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub(crate) fn finish_prepared_application_query_permission<
            'a,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >(&'a self, execution: &worth_execution::ExecutionRequest<'_, '_>, permission: PreparedApplicationQueryPermission<
                'a,
                Schema,
                Query,
                Parameters,
                QueryResult,
                Principal,
                PrincipalIdentity,
                Scope,
            >) -> Result<
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
            self.inner.finish_prepared_application_query_permission::<Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>(execution, permission)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_admission_permission_read_authorization {
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryApplicationAuthorizationWorkEvidence;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAdmissionDenial;
    use crate::domain_computation::authorization::WorthQueryRetainedAuthorizationDecisionFacts;
    use crate::domain_computation::provider_session::WorthQueryPreparedProductReadSessionIdentity;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::admission::permission::selected_access::SelectedIssuedAccessRoot;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    type ReadAuthorization = (
    WorthQueryRetainedAuthorizationDecisionFacts,
    WorthQueryApplicationAuthorizationWorkEvidence,
);
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub(crate) fn observe_issued_query_read_authorization<
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>, root: &SelectedIssuedAccessRoot<'_, Schema, Principal, PrincipalIdentity, Scope>, session: &WorthQueryPreparedProductReadSessionIdentity) -> Result<ReadAuthorization, WorthQueryApplicationQueryAdmissionDenial> {
            self.inner.observe_issued_query_read_authorization::<Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>(execution, query, root, session)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_admission_permission_source_parameters {
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAccessContext;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAdmissionDenial;
    use crate::domain_computation::primary_graph::WorthQueryIssuedSelectedPrincipal;
    use crate::domain_computation::primary_graph::WorthQueryIssuedSelectedScope;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::admission::permission::PreparedApplicationQueryPermission;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[allow(clippy::too_many_arguments)]
        pub(crate) fn prepare_application_query_permission_with_parameters<
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
            >, access: WorthQueryApplicationQueryAccessContext<
                'a,
                Schema,
                Principal,
                PrincipalIdentity,
                Scope,
            >, source_query_identity: &worth_query_installation::facade::WorthQueryInstalledApplicationQueryIdentity, parameters: worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters, controls: worth_query_execution_query::facade::domain_computation::primary_graph::application_query::admission::WorthQueryApplicationQueryControls<'a, Schema>, selected: &'a WorthQuerySelectedProductOperation<'_, Schema>, truth: Option<&'a WorthQuerySelectedProductOperation<'_, Schema>>, request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope, issued_principal: &'a WorthQueryIssuedSelectedPrincipal<
                '_,
                '_,
                Schema,
                Principal,
                PrincipalIdentity,
            >, issued_scope: &'a WorthQueryIssuedSelectedScope<'_, '_, Schema, Scope>, native: &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess<'_>) -> Result<
            PreparedApplicationQueryPermission<
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
            self.inner.prepare_application_query_permission_with_parameters::<Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>(execution, query, access, source_query_identity, parameters, controls, selected.inner(), truth.map(|selected| selected.inner()), request, issued_principal, issued_scope, native)
        }
        #[allow(clippy::too_many_arguments)]
        pub(crate) fn prepare_historical_source_query_permission_with_parameters<
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
            >, access: WorthQueryApplicationQueryAccessContext<
                'a,
                Schema,
                Principal,
                PrincipalIdentity,
                Scope,
            >, source_query_identity: &worth_query_installation::facade::WorthQueryInstalledApplicationQueryIdentity, parameters: worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters, controls: worth_query_execution_query::facade::domain_computation::primary_graph::application_query::admission::WorthQueryApplicationQueryControls<'a, Schema>, selected: &'a WorthQuerySelectedProductOperation<'_, Schema>, truth: Option<&'a WorthQuerySelectedProductOperation<'_, Schema>>, request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope, issued_principal: &'a WorthQueryIssuedSelectedPrincipal<
                '_,
                '_,
                Schema,
                Principal,
                PrincipalIdentity,
            >, history: crate::domain_computation::primary_graph::product_operation::document_history::HistoryProducerInvocation<'a>, native: &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess<'_>) -> Result<
            PreparedApplicationQueryPermission<
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
            self.inner.prepare_historical_source_query_permission_with_parameters::<Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>(execution, query, access, source_query_identity, parameters, controls, selected.inner(), truth.map(|selected| selected.inner()), request, issued_principal, history, native)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_admission_permission_fresh_access {
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
        #[allow(clippy::too_many_arguments)]
        pub(crate) fn with_fresh_retained_query_identities<
            'a,
            'selected,
            'runtime,
            Binding,
            R,
            E,
        >(&'a self, execution: &worth_execution::ExecutionRequest<'_, '_>, installed: &'a WorthQueryInstalledApplicationQueryBinding<Schema, Binding>, validated_principal: Option<
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
            self.inner.with_fresh_retained_query_identities::<Binding, R, E>(execution, installed, validated_principal, external, observed, selected.inner(), request, inspect)
        }
        #[allow(clippy::too_many_arguments)]
        pub(crate) fn with_fresh_inspection_scope_identities<
            'a,
            'selected,
            'runtime,
            Binding,
            R,
            E,
        >(&'a self, execution: &worth_execution::ExecutionRequest<'_, '_>, installed: &'a WorthQueryInstalledApplicationQueryBinding<Schema, Binding>, validated_principal: Option<
                &worth_query_installation::facade::WorthQueryValidatedPrincipalBinding,
            >, external: &WorthQueryAuthenticatedExternalPrincipal<Schema>, selector: &crate::domain_computation::primary_graph::WorthQueryObservedScopeSelector, selected: &'selected WorthQuerySelectedProductOperation<'runtime, Schema>, request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope, inspect: impl for<'fresh> FnOnce(
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
            self.inner.with_fresh_inspection_scope_identities::<Binding, R, E>(execution, installed, validated_principal, external, selector, selected.inner(), request, inspect)
        }
    }
}
