//! Thin forwarding shims for `admission` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_authorization_admission_resource_observation {
    use crate::domain_computation::authorization::WorthQueryAuthorizationDecisionFact;
    use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial;
    use crate::domain_computation::primary_graph::WorthQueryApplicationQueryAccessContext;
    use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
    use worth_query_declaration::facade::application_query::ApplicationQueryScopeBinding;
    use worth_query_declaration::facade::application_resource::ApplicationResourceAction;
    use worth_query_declaration::facade::application_resource::ApplicationResourceBinding;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationResource;
    use worth_query_execution_attempt::facade::domain_computation::authorization::admission::*;
    use worth_query_execution_query::facade::domain_computation::authorization::admission::validation::*;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[allow(clippy::too_many_arguments)]
        pub(crate) fn observe_resource_authorization<Binding>(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            session_identity: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
            relational: &worth_relational::facade::runtime::RelationalRuntime,
            snapshot: worth_relational::facade::snapshots::SnapshotHandle,
            resource: &WorthQueryInstalledApplicationResource<Schema, Binding>,
            action: ApplicationResourceAction,
            access: &WorthQueryApplicationQueryAccessContext<'_, Schema,
                <Binding::Access as ApplicationQueryBinding<Schema>>::Principal,
                <Binding::Access as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
                <<Binding::Access as ApplicationQueryBinding<Schema>>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope,
            >,
        ) -> Result<Vec<WorthQueryAuthorizationDecisionFact>, WorthQueryOperationAuthorizationDenial>
        where
            Binding: ApplicationResourceBinding<Schema>,
        {
            self.inner.observe_resource_authorization::<Binding>(
                execution,
                session_identity,
                relational,
                snapshot,
                resource,
                action,
                access,
            )
        }
    }
}
