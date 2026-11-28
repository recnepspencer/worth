//! Thin forwarding shims for `application_query` methods onto `worth-query-execution-query`.


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
        pub(crate) fn with_fresh_retained_query_identities<'a, 'selected, 'runtime, Binding, R, E>(
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
                .with_fresh_retained_query_identities::<Binding, R, E>(
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
        #[allow(clippy::too_many_arguments)]
        pub(crate) fn with_fresh_inspection_scope_identities<
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
            selector: &crate::domain_computation::primary_graph::WorthQueryObservedScopeSelector,
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
                .with_fresh_inspection_scope_identities::<Binding, R, E>(
                    execution,
                    installed,
                    validated_principal,
                    external,
                    selector,
                    selected.inner(),
                    request,
                    inspect,
                )
        }
    }
}
