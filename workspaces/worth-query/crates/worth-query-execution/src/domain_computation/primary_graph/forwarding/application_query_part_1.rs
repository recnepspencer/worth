//! Thin forwarding shims for `application_query` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_admission {
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
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
        pub(crate) fn admit_application_query<
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
            self.inner.admit_application_query::<Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>(execution, query, access, parameters, controls)
        }
        #[allow(clippy::too_many_arguments)]
        pub(crate) fn admit_governed_application_query<
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
            capability: crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess<
                Schema,
                Capability,
                Operation,
                Input,
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
        >
        where
            Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope>,
        {
            self.inner.admit_governed_application_query::<Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope, Capability, Operation, Input>(execution, query, access, capability, parameters, controls)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_admission_preparation {
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAccessContext;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAdmissionDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryControls;
    use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema,
    {
        pub(crate) fn prepare_application_query_source_readmission<
            'a,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >(
            &'a self,
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
            retained: &WorthQueryAdmittedApplicationQueryParameters,
            controls: WorthQueryApplicationQueryControls<'a, Schema>,
        ) -> Result<
            (
                WorthQueryAdmittedApplicationQueryParameters,
                WorthQueryApplicationQueryControls<'a, Schema>,
            ),
            WorthQueryApplicationQueryAdmissionDenial,
        > {
            self.inner.prepare_application_query_source_readmission::<Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>(query, access, retained, controls)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_one_shot {
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
        pub fn execute_application_query_one_shot<
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
            WorthQueryApplicationOneShotResult<Query, QueryResult>,
            WorthQueryApplicationOneShotDenial,
        >
        where
            QueryResult: WorthQueryApplicationProjection<Schema, Query>,
        {
            self.inner.execute_application_query_one_shot::<Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>(phase, plan)
        }
        pub(crate) fn execute_application_query_one_shot_with_request<
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
            self.inner.execute_application_query_one_shot_with_request::<Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>(execution, plan)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_readiness {
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryApplicationBasisIdentity;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAdmissionDenial;
    use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::readiness::WorthQueryPrimaryGraphApplicationReadinessSnapshot;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQuerySelectedProductOperation<'_, Schema>
    where
        Schema: ApplicationSchema,
    {
        pub fn inspect_application_readiness(
            &self,
        ) -> Result<
            WorthQueryPrimaryGraphApplicationReadinessSnapshot,
            WorthQueryApplicationQueryAdmissionDenial,
        > {
            self.inner.inspect_application_readiness()
        }
    }
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema,
    {
        pub fn application_basis_token(
            &self,
            basis: &WorthQueryApplicationBasisIdentity,
        ) -> Result<String, WorthQueryApplicationQueryAdmissionDenial> {
            self.inner.application_basis_token(basis)
        }
        pub fn application_commit_basis_token(
            &self,
            receipt: &WorthQueryApplicationCommitReceipt,
        ) -> Result<String, WorthQueryApplicationQueryAdmissionDenial> {
            self.inner.application_commit_basis_token(receipt)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_retained_read_runtime {
    use crate::basis::WorthQueryProductBranchAdmissionDenial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    use crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation;
    use std::sync::Arc;
    use worth_query_installation::facade::ApplicationSchema;
    impl<'runtime, Schema> WorthQuerySelectedProductOperation<'runtime, Schema>
    where
        Schema: ApplicationSchema,
    {
        pub fn retain_application_read(self) -> Arc<WorthQueryApplicationReadObservation> {
            self.inner.retain_application_read()
        }
    }
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema,
    {
        pub fn select_application_read_observation(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            observation: &WorthQueryApplicationReadObservation,
        ) -> Result<
            WorthQuerySelectedProductOperation<'_, Schema>,
            WorthQueryProductBranchAdmissionDenial,
        > {
            self.inner
                .select_application_read_observation(phase, observation)
                .map(|inner| WorthQuerySelectedProductOperation::from_inner(self, inner))
        }
        pub(crate) fn select_application_read_observation_with_request(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            observation: &WorthQueryApplicationReadObservation,
        ) -> Result<
            WorthQuerySelectedProductOperation<'_, Schema>,
            WorthQueryProductBranchAdmissionDenial,
        > {
            self.inner
                .select_application_read_observation_with_request(execution, observation)
                .map(|inner| WorthQuerySelectedProductOperation::from_inner(self, inner))
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_managed_index_receiver {
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::managed_index::denial::WorthQueryManagedIndexDenial as Denial;
    use crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan;
    use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationOneShotResult;
    use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationProjection;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
    use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
    use worth_query_declaration::facade::application_query::ApplicationQueryScopeBinding;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
    use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    type ResultValue<S, B> =
    <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
    type BoundPlan<'a, S, B> = WorthQueryAdmittedApplicationQueryPlan<
    'a, S, <B as ApplicationQueryBinding<S>>::Query,
    <<B as ApplicationQueryBinding<S>>::ParameterBinding as ApplicationStructuredValueBinding>::Value,
    ResultValue<S, B>, <B as ApplicationQueryBinding<S>>::Principal,
    <B as ApplicationQueryBinding<S>>::PrincipalIdentity,
    <<B as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope,
>;
    impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
        pub fn execute_application_query_binding_one_shot<B>(
            &self,
            phase: &WorthQueryAdvancementPhase<'_>,
            binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
            external: &WorthQueryAuthenticatedExternalPrincipal<S>,
            plan: BoundPlan<'_, S, B>,
        ) -> Result<WorthQueryApplicationOneShotResult<B::Query, ResultValue<S, B>>, Denial>
        where
            B: ApplicationQueryBinding<S>,
            ResultValue<S, B>: WorthQueryApplicationProjection<S, B::Query>,
        {
            self.inner
                .execute_application_query_binding_one_shot::<B>(phase, binding, external, plan)
        }
    }
}
