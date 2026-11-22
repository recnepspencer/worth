//! Thin forwarding shims for `entity_resolution` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_entity_resolution_product {
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    use worth_query_execution_query::facade::domain_computation::primary_graph::entity_resolution::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::entity_resolution::WorthQueryApplicationEntityIdentity;
    use worth_query_execution_query::facade::domain_computation::primary_graph::entity_resolution::WorthQueryEntityResolutionDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::entity_resolution::WorthQueryPrincipalResolutionMode;
    use worth_query_execution_query::facade::domain_computation::primary_graph::entity_resolution::WorthQueryRequestScope;
    impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
        pub fn resolve_entity<Aspect, Entity, Field, Value>(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            field: Field,
            value: Value,
            request: &WorthQueryRequestScope,
            mode: WorthQueryPrincipalResolutionMode,
        ) -> Result<
            WorthQueryApplicationEntityIdentity<Schema, Entity>,
            WorthQueryEntityResolutionDenial,
        >
        where
            Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
                Entity = Entity,
                Aspect = Aspect,
                Value = Value,
            >,
        {
            self.inner
                .resolve_entity::<Aspect, Entity, Field, Value>(phase, field, value, request, mode)
        }
        pub(crate) fn resolve_entity_with_request<Aspect, Entity, Field, Value>(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            field: Field,
            value: Value,
            request: &WorthQueryRequestScope,
            mode: WorthQueryPrincipalResolutionMode,
        ) -> Result<
            WorthQueryApplicationEntityIdentity<Schema, Entity>,
            WorthQueryEntityResolutionDenial,
        >
        where
            Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
                Entity = Entity,
                Aspect = Aspect,
                Value = Value,
            >,
        {
            self.inner
                .resolve_entity_with_request::<Aspect, Entity, Field, Value>(
                    execution, field, value, request, mode,
                )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_entity_resolution_product_issued_scope {
    use worth_query_declaration::facade::application_schema::fragment::RecordSelection;
    use worth_query_execution_query::facade::domain_computation::primary_graph::entity_resolution::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::entity_resolution::WorthQueryEntityResolutionDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::entity_resolution::WorthQueryRequestScope;
    use crate::domain_computation::primary_graph::application_query::WorthQueryObservedScopeSelector;
    use worth_query_execution_query::facade::domain_computation::primary_graph::entity_resolution::product::issued_scope::WorthQueryIssuedSelectedScope;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<'runtime, Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'runtime, Schema> {
        pub(crate) fn resolve_query_scope_issued_with_native_access<
            'selected,
            Field: RecordSelection,
        >(
            &'selected self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            field: Field,
            value: Field::Value,
            request: &WorthQueryRequestScope,
            native: &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess<'_>,
        ) -> Result<
            WorthQueryIssuedSelectedScope<'selected, 'runtime, Schema, Field::Entity>,
            WorthQueryEntityResolutionDenial,
        > {
            self.inner
                .resolve_query_scope_issued_with_native_access::<Field>(
                    execution, field, value, request, native,
                )
        }
        pub(crate) fn resolve_retained_query_scope_issued<'selected, Field: RecordSelection>(
            &'selected self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            field: Field,
            selector: &WorthQueryObservedScopeSelector,
            request: &WorthQueryRequestScope,
        ) -> Result<
            WorthQueryIssuedSelectedScope<'selected, 'runtime, Schema, Field::Entity>,
            WorthQueryEntityResolutionDenial,
        > {
            self.inner
                .resolve_retained_query_scope_issued::<Field>(execution, field, selector, request)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_entity_resolution_product_retained_scope {
    use crate::domain_computation::primary_graph::application_query::WorthQueryObservedScopeSelector;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    use worth_query_declaration::facade::application_schema::fragment::RecordSelection;
    use worth_query_execution_query::facade::domain_computation::primary_graph::entity_resolution::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::entity_resolution::WorthQueryApplicationEntityIdentity;
    use worth_query_execution_query::facade::domain_computation::primary_graph::entity_resolution::WorthQueryEntityResolutionDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::entity_resolution::WorthQueryRequestScope;
    impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
        pub(crate) fn resolve_retained_query_scope<Field: RecordSelection>(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            field: Field,
            selector: &WorthQueryObservedScopeSelector,
            request: &WorthQueryRequestScope,
        ) -> Result<
            WorthQueryApplicationEntityIdentity<Schema, Field::Entity>,
            WorthQueryEntityResolutionDenial,
        > {
            self.inner
                .resolve_retained_query_scope::<Field>(execution, field, selector, request)
        }
    }
}
