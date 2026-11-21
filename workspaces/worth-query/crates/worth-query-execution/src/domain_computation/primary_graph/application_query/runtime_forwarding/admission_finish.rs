//! Thin forwarding shims for `application_query` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_admission_work_limit {
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn resolve_application_query_limits(
            &self,
            limits: worth_query_installation::facade::WorthQueryInstalledApplicationQueryLimits,
        ) -> worth_query_installation::facade::WorthQueryResolvedApplicationQueryLimits {
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
            parameters: WorthQueryAdmittedApplicationQueryParameters,
            controls: WorthQueryApplicationQueryControls<'a, Schema>,
            pending_governance: Option<WorthQueryPendingApplicationQueryGovernance>,
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
            self.inner.finish_application_query_admission::<Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>(execution, query, access, parameters, controls, pending_governance)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_admission_finish_prepared_permission {
    use crate::domain_computation::primary_graph::application_query::admission::permission::PreparedApplicationQueryPermission;
    use crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan;
    use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAdmissionDenial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub(crate) fn finish_prepared_application_query_permission<
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
            permission: PreparedApplicationQueryPermission<
                'a,
                Schema,
                Query,
                Parameters,
                QueryResult,
                Principal,
                PrincipalIdentity,
                Scope,
            >,
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
        >(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            query: &WorthQueryInstalledApplicationQuery<
                Schema,
                Query,
                Parameters,
                QueryResult,
                Scope,
            >,
            root: &SelectedIssuedAccessRoot<'_, Schema, Principal, PrincipalIdentity, Scope>,
            session: &WorthQueryPreparedProductReadSessionIdentity,
        ) -> Result<ReadAuthorization, WorthQueryApplicationQueryAdmissionDenial> {
            self.inner.observe_issued_query_read_authorization::<Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>(execution, query, root, session)
        }
    }
}
