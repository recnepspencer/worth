//! Thin forwarding shims for `application_query` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_batch_planning {
    use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
    use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::batch::planning::WorthQueryApplicationQueryBatchReadPlan;
    use worth_query_execution_lineage::facade::primary_graph::application_query::batch::WorthQueryApplicationQueryBatchResourceDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::batch::WorthQueryApplicationQueryBatchAdmission;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: worth_query_declaration::facade::application_schema::ApplicationSchema>
        WorthQueryPrimaryGraphApplicationRuntime<Schema>
    {
        pub fn plan_application_query_batch_reads<B: ApplicationQueryBinding<Schema>>(
            &self,
            binding: &WorthQueryInstalledApplicationQueryBinding<Schema, B>,
            count: usize,
        ) -> Result<
            WorthQueryApplicationQueryBatchReadPlan,
            WorthQueryApplicationQueryBatchResourceDenial,
        > {
            self.inner
                .plan_application_query_batch_reads::<B>(binding, count)
        }
        pub fn plan_installed_query_batch_reads<Q, P, R, S>(
            &self,
            query: &worth_query_installation::facade::WorthQueryInstalledApplicationQuery<
                Schema,
                Q,
                P,
                R,
                S,
            >,
            count: usize,
            maximum_results: std::num::NonZeroUsize,
        ) -> Result<
            WorthQueryApplicationQueryBatchReadPlan,
            WorthQueryApplicationQueryBatchResourceDenial,
        > {
            self.inner
                .plan_installed_query_batch_reads::<Q, P, R, S>(query, count, maximum_results)
        }
        pub fn application_query_batch_from_installed_reads(
            &self,
            reads: &[WorthQueryApplicationQueryBatchReadPlan],
        ) -> Result<
            WorthQueryApplicationQueryBatchAdmission,
            WorthQueryApplicationQueryBatchResourceDenial,
        > {
            self.inner
                .application_query_batch_from_installed_reads(reads)
        }
        pub fn extend_application_query_batch_once(
            &self,
            batch: &WorthQueryApplicationQueryBatchAdmission,
            reads: &[WorthQueryApplicationQueryBatchReadPlan],
        ) -> Result<(), WorthQueryApplicationQueryBatchResourceDenial> {
            self.inner.extend_application_query_batch_once(batch, reads)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_live_lease_open {
    use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
    use worth_query_declaration::facade::application_query::ApplicationQueryLiveCauseBinding;
    use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::live::controls::WorthQueryApplicationLiveControls;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::live::outcome::WorthQueryApplicationLiveOpenDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::live::lease::WorthQueryApplicationLiveLease;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::live::whole_result::WorthQueryApplicationWholeResultLiveLease;
    use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationProjection;
    use crate::domain_computation::primary_graph::WorthQueryApplicationEntityIdentity;
    use crate::domain_computation::primary_graph::WorthQueryAuthenticatedPrincipal;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<'runtime, Schema> WorthQuerySelectedProductOperation<'runtime, Schema>
    where
        Schema: ApplicationSchema,
    {
        #[allow(clippy::too_many_arguments)]
        pub fn open_application_query_live<
            'principal,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
            Target,
            Binding,
        >(
            self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            query: WorthQueryInstalledApplicationQuery<
                Schema,
                Query,
                Parameters,
                QueryResult,
                Scope,
            >,
            principal: &'principal WorthQueryAuthenticatedPrincipal<
                Schema,
                Principal,
                PrincipalIdentity,
            >,
            scope: WorthQueryApplicationEntityIdentity<Schema, Scope>,
            parameters: ApplicationQueryParameterSet<Query>,
            controls: WorthQueryApplicationLiveControls,
        ) -> Result<
            WorthQueryApplicationLiveLease<
                'runtime,
                Schema,
                Query,
                Parameters,
                QueryResult,
                Principal,
                PrincipalIdentity,
                Scope,
                Target,
                Binding,
            >,
            WorthQueryApplicationLiveOpenDenial,
        >
        where
            QueryResult: WorthQueryApplicationProjection<Schema, Query>,
            Binding: ApplicationQueryLiveCauseBinding<Schema, Query, Scope, Target>,
        {
            self.inner.open_application_query_live::<Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope, Target, Binding>(phase, query, principal, scope, parameters, controls)
        }
        #[allow(clippy::too_many_arguments)]
        pub fn open_governed_application_query_live<
            'principal,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
            Target,
            Binding,
            Capability,
            Operation,
            Input,
        >(
            self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            query: WorthQueryInstalledApplicationQuery<
                Schema,
                Query,
                Parameters,
                QueryResult,
                Scope,
            >,
            principal: &'principal WorthQueryAuthenticatedPrincipal<
                Schema,
                Principal,
                PrincipalIdentity,
            >,
            scope: WorthQueryApplicationEntityIdentity<Schema, Scope>,
            capability: crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess<
                Schema,
                Capability,
                Operation,
                Input,
            >,
            parameters: ApplicationQueryParameterSet<Query>,
            controls: WorthQueryApplicationLiveControls,
        ) -> Result<
            WorthQueryApplicationLiveLease<
                'runtime,
                Schema,
                Query,
                Parameters,
                QueryResult,
                Principal,
                PrincipalIdentity,
                Scope,
                Target,
                Binding,
            >,
            WorthQueryApplicationLiveOpenDenial,
        >
        where
            QueryResult: WorthQueryApplicationProjection<Schema, Query>,
            Binding: ApplicationQueryLiveCauseBinding<Schema, Query, Scope, Target>,
            Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope>,
        {
            self.inner.open_governed_application_query_live::<Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope, Target, Binding, Capability, Operation, Input>(phase, query, principal, scope, capability, parameters, controls)
        }
        #[allow(clippy::too_many_arguments)]
        pub fn open_application_query_live_whole_result<
            'principal,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >(
            self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            query: WorthQueryInstalledApplicationQuery<
                Schema,
                Query,
                Parameters,
                QueryResult,
                Scope,
            >,
            principal: &'principal WorthQueryAuthenticatedPrincipal<
                Schema,
                Principal,
                PrincipalIdentity,
            >,
            scope: WorthQueryApplicationEntityIdentity<Schema, Scope>,
            parameters: ApplicationQueryParameterSet<Query>,
            controls: WorthQueryApplicationLiveControls,
        ) -> Result<
            WorthQueryApplicationWholeResultLiveLease<
                'runtime,
                Schema,
                Query,
                Parameters,
                QueryResult,
                Principal,
                PrincipalIdentity,
                Scope,
            >,
            WorthQueryApplicationLiveOpenDenial,
        >
        where
            QueryResult: WorthQueryApplicationProjection<Schema, Query>,
        {
            self.inner.open_application_query_live_whole_result::<Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>(phase, query, principal, scope, parameters, controls)
        }
        #[allow(clippy::too_many_arguments)]
        pub fn open_governed_application_query_live_whole_result<
            'principal,
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
            self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            query: WorthQueryInstalledApplicationQuery<
                Schema,
                Query,
                Parameters,
                QueryResult,
                Scope,
            >,
            principal: &'principal WorthQueryAuthenticatedPrincipal<
                Schema,
                Principal,
                PrincipalIdentity,
            >,
            scope: WorthQueryApplicationEntityIdentity<Schema, Scope>,
            capability: crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess<
                Schema,
                Capability,
                Operation,
                Input,
            >,
            parameters: ApplicationQueryParameterSet<Query>,
            controls: WorthQueryApplicationLiveControls,
        ) -> Result<
            WorthQueryApplicationWholeResultLiveLease<
                'runtime,
                Schema,
                Query,
                Parameters,
                QueryResult,
                Principal,
                PrincipalIdentity,
                Scope,
            >,
            WorthQueryApplicationLiveOpenDenial,
        >
        where
            QueryResult: WorthQueryApplicationProjection<Schema, Query>,
            Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope>,
        {
            self.inner.open_governed_application_query_live_whole_result::<Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope, Capability, Operation, Input>(phase, query, principal, scope, capability, parameters, controls)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_history_capture_plan {
    use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
    use worth_query_declaration::facade::application_query::ApplicationHistoryQueryBinding;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationHistoryQuery;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::history::SubjectPlan;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::history::SubjectScope;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::history::WorthQueryApplicationHistoryCaptureLimits;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::history::WorthQueryApplicationHistoryDenial as Denial;
    use crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::history::capture_plan::WorthQueryAdmittedApplicationHistoryCapture;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn admit_application_history_capture<'a, History>(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
            plan: SubjectPlan<'a, Schema, History>,
            limits: WorthQueryApplicationHistoryCaptureLimits,
        ) -> Result<WorthQueryAdmittedApplicationHistoryCapture<'a, Schema, History>, Denial>
        where
            History: ApplicationHistoryQueryBinding<Schema>,
        {
            self.inner
                .admit_application_history_capture::<History>(execution, history, plan, limits)
        }
        pub fn admit_governed_application_history_capture<
            'a,
            History,
            Capability,
            Operation,
            Input,
        >(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
            plan: SubjectPlan<'a, Schema, History>,
            capability: WorthQueryAdmittedApplicationCapabilityAccess<
                Schema,
                Capability,
                Operation,
                Input,
            >,
            limits: WorthQueryApplicationHistoryCaptureLimits,
        ) -> Result<WorthQueryAdmittedApplicationHistoryCapture<'a, Schema, History>, Denial>
        where
            History: ApplicationHistoryQueryBinding<Schema>,
            Input: ApplicationCapabilityRequest<
                Schema,
                Capability,
                Scope = SubjectScope<Schema, History>,
            >,
        {
            self.inner.admit_governed_application_history_capture::<History, Capability, Operation, Input>(execution, history, plan, capability, limits)
        }
    }
}
