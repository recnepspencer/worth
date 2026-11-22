//! Thin forwarding shims for `resolution` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_resolution_selected_product {
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_execution_lineage::facade::primary_graph::resolution::WorthQueryPrincipalResolutionMode;
    use worth_query_execution_query::facade::domain_computation::primary_graph::WorthQueryAuthenticatedPrincipal;
    use worth_query_execution_query::facade::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenial;
    use worth_query_installation::facade::ApplicationIdentityScalarValueBinding;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledPrincipalBinding;
    impl<Schema> WorthQuerySelectedProductOperation<'_, Schema>
    where
        Schema: ApplicationSchema,
    {
        pub fn resolve_authenticated_principal<
            Binding,
            Mapping,
            Principal,
            PrincipalIdentity,
            PrincipalIdentityBinding,
        >(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            installed_binding: &WorthQueryInstalledPrincipalBinding<
                Schema,
                Binding,
                Mapping,
                Principal,
                PrincipalIdentity,
                PrincipalIdentityBinding,
            >,
            external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
            scope: &WorthQueryRequestScope,
            mode: WorthQueryPrincipalResolutionMode,
        ) -> Result<
            WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
            WorthQueryPrincipalResolutionDenial,
        >
        where
            PrincipalIdentityBinding:
                ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
            PrincipalIdentity: 'static,
        {
            self.inner.resolve_authenticated_principal::<Binding, Mapping, Principal, PrincipalIdentity, PrincipalIdentityBinding>(phase, installed_binding, external, scope, mode)
        }
        pub(crate) fn resolve_authenticated_principal_with_request<
            Binding,
            Mapping,
            Principal,
            PrincipalIdentity,
            PrincipalIdentityBinding,
        >(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            installed_binding: &WorthQueryInstalledPrincipalBinding<
                Schema,
                Binding,
                Mapping,
                Principal,
                PrincipalIdentity,
                PrincipalIdentityBinding,
            >,
            external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
            scope: &WorthQueryRequestScope,
            mode: WorthQueryPrincipalResolutionMode,
        ) -> Result<
            WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
            WorthQueryPrincipalResolutionDenial,
        >
        where
            PrincipalIdentityBinding:
                ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
            PrincipalIdentity: 'static,
        {
            self.inner.resolve_authenticated_principal_with_request::<Binding, Mapping, Principal, PrincipalIdentity, PrincipalIdentityBinding>(execution, installed_binding, external, scope, mode)
        }
        pub fn validate_authenticated_principal<Principal, PrincipalIdentity>(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
            scope: &WorthQueryRequestScope,
        ) -> Result<(), WorthQueryPrincipalResolutionDenial> {
            self.inner
                .validate_authenticated_principal::<Principal, PrincipalIdentity>(
                    phase, principal, scope,
                )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_resolution_selected_product_admitted {
    use worth_query_execution_query::facade::domain_computation::primary_graph::resolution::selected_product::admitted::issued_principal::WorthQueryIssuedSelectedPrincipal;
    use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_installation::facade::ApplicationIdentityScalarValueBinding;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledPrincipalBinding;
    use worth_query_execution_query::facade::domain_computation::primary_graph::WorthQueryAuthenticatedPrincipal;
    use worth_query_execution_query::facade::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenial;
    use worth_query_execution_lineage::facade::primary_graph::resolution::WorthQueryPrincipalResolutionMode;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<'runtime, Schema> WorthQuerySelectedProductOperation<'runtime, Schema>
    where
        Schema: ApplicationSchema,
    {
        pub(crate) fn resolve_authenticated_principal_issued<
            'selected,
            Binding,
            Mapping,
            Principal,
            PrincipalIdentity,
            PrincipalIdentityBinding,
        >(
            &'selected self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            installed_binding: &WorthQueryInstalledPrincipalBinding<
                Schema,
                Binding,
                Mapping,
                Principal,
                PrincipalIdentity,
                PrincipalIdentityBinding,
            >,
            external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
            request: &WorthQueryRequestScope,
            mode: WorthQueryPrincipalResolutionMode,
            validated: Option<
                &worth_query_installation::facade::WorthQueryValidatedPrincipalBinding,
            >,
        ) -> Result<
            WorthQueryIssuedSelectedPrincipal<
                'selected,
                'runtime,
                Schema,
                Principal,
                PrincipalIdentity,
            >,
            WorthQueryPrincipalResolutionDenial,
        >
        where
            PrincipalIdentityBinding:
                ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
            PrincipalIdentity: 'static,
        {
            self.inner.resolve_authenticated_principal_issued::<Binding, Mapping, Principal, PrincipalIdentity, PrincipalIdentityBinding>(execution, installed_binding, external, request, mode, validated)
        }
        pub(crate) fn resolve_authenticated_principal_admitted<
            Binding,
            Mapping,
            Principal,
            PrincipalIdentity,
            PrincipalIdentityBinding,
        >(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            installed_binding: &WorthQueryInstalledPrincipalBinding<
                Schema,
                Binding,
                Mapping,
                Principal,
                PrincipalIdentity,
                PrincipalIdentityBinding,
            >,
            external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
            scope: &WorthQueryRequestScope,
            mode: WorthQueryPrincipalResolutionMode,
        ) -> Result<
            WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
            WorthQueryPrincipalResolutionDenial,
        >
        where
            PrincipalIdentityBinding:
                ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
            PrincipalIdentity: 'static,
        {
            self.inner.resolve_authenticated_principal_admitted::<Binding, Mapping, Principal, PrincipalIdentity, PrincipalIdentityBinding>(execution, installed_binding, external, scope, mode)
        }
    }
}
