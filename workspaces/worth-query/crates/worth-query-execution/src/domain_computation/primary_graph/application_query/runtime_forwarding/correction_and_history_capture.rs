//! Thin forwarding shims for `application_query` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_one_shot_batch {
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryBatchAdmission;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::one_shot::batch::result::WorthQueryApplicationBatchResult;
    use worth_query_execution_lineage::facade::primary_graph::application_query::one_shot::WorthQueryApplicationBatchReadDenial;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryApplicationProjection;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn execute_application_query_one_shot_in_batch<
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
            batch: &WorthQueryApplicationQueryBatchAdmission,
        ) -> Result<
            WorthQueryApplicationBatchResult<Query, QueryResult>,
            WorthQueryApplicationBatchReadDenial,
        >
        where
            QueryResult: WorthQueryApplicationProjection<Schema, Query>,
        {
            self.inner.execute_application_query_one_shot_in_batch::<Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>(phase, plan, batch)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_one_shot_correction_disclosure {
    use worth_relational::facade::identity::EntityId;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::one_shot::denial::WorthQueryApplicationOneShotDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub(crate) fn execute_correction_disclosure<
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
            self.inner.execute_correction_disclosure::<Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>(execution, plan, original_subject)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_one_shot_history_capture {
    use worth_query_declaration::facade::application_query::ApplicationHistoryQueryBinding;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use crate::domain_computation::primary_graph::application_query::history::SubjectQuery;
    use crate::domain_computation::primary_graph::application_query::history::SubjectResult;
    use crate::domain_computation::primary_graph::application_query::history::WorthQueryAdmittedApplicationHistoryCapture;
    use crate::domain_computation::primary_graph::application_query::history::WorthQueryApplicationHistoryDenial as Denial;
    use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationProjection;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::one_shot::history_capture::WorthQueryApplicationHistoryCaptureResult;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn execute_application_history_capture<History>(
            &self,
            phase: &WorthQueryAdvancementPhase<'_>,
            capture: WorthQueryAdmittedApplicationHistoryCapture<'_, Schema, History>,
        ) -> Result<WorthQueryApplicationHistoryCaptureResult<Schema, History>, Denial>
        where
            History: ApplicationHistoryQueryBinding<Schema>,
            SubjectResult<Schema, History>:
                WorthQueryApplicationProjection<Schema, SubjectQuery<Schema, History>>,
        {
            self.inner
                .execute_application_history_capture::<History>(phase, capture)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_admission_correction_disclosure {
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAccessContext;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAdmissionDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryControls;
    use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub(crate) fn admit_correction_disclosure<
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
            retained: &WorthQueryAdmittedApplicationQueryParameters,
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
            self.inner.admit_correction_disclosure::<Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>(execution, query, access, retained, controls)
        }
    }
}
