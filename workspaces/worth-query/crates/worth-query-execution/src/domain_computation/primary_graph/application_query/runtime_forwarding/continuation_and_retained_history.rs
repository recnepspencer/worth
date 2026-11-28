//! Thin forwarding shims for `application_query` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_continuation_mod {
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::continuation::denial::WorthQueryApplicationContinuationDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryApplicationProjection;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::continuation::WorthQueryApplicationContinuationPageResult;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema,
    {
        pub fn execute_application_query_continuation_page<
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
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
            WorthQueryApplicationContinuationPageResult<
                Schema,
                Query,
                Parameters,
                QueryResult,
                Scope,
            >,
            WorthQueryApplicationContinuationDenial,
        >
        where
            QueryResult: WorthQueryApplicationProjection<Schema, Query>,
        {
            self.inner.execute_application_query_continuation_page::<Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>(phase, plan)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_continuation_readmission {
    use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
    use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::continuation::authority::WorthQueryApplicationQueryContinuation;
    use crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan;
    use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAccessContext;
    use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAdmissionDenial;
    use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryResumeControls;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema,
    {
        pub fn readmit_application_query_continuation<
            'a,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >(
            &'a self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
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
            continuation: WorthQueryApplicationQueryContinuation<
                Schema,
                Query,
                Parameters,
                QueryResult,
                Scope,
            >,
            controls: WorthQueryApplicationQueryResumeControls<'a>,
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
            self.inner.readmit_application_query_continuation::<Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>(phase, query, access, parameters, continuation, controls)
        }
        #[allow(clippy::too_many_arguments)]
        pub fn readmit_governed_application_query_continuation<
            'a,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
            Capability,
            Operation,
            Input,
        >(
            &'a self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
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
            capability: crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess<
                Schema,
                Capability,
                Operation,
                Input,
            >,
            parameters: ApplicationQueryParameterSet<Query>,
            continuation: WorthQueryApplicationQueryContinuation<
                Schema,
                Query,
                Parameters,
                QueryResult,
                Scope,
            >,
            controls: WorthQueryApplicationQueryResumeControls<'a>,
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
        >
        where
            Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope>,
        {
            self.inner.readmit_governed_application_query_continuation::<Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope, Capability, Operation, Input>(phase, query, access, capability, parameters, continuation, controls)
        }
    }
}
