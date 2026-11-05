//! The runtime's consumer methods over `one_shot` in `domain_computation::primary_graph::application_query`; each only calls its subsystem.
use crate::domain_computation::primary_graph::application_query::one_shot::*;
use crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationHistoryCapture;
use crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationBatchResult;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationOneShotResult;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationProjection;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryBatchAdmission;
use crate::domain_computation::primary_graph::vocabulary::application_query::history_denial::WorthQueryApplicationHistoryDenial;
use crate::domain_computation::primary_graph::vocabulary::application_query::one_shot_denial::WorthQueryApplicationOneShotDenial;
use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;
use worth_query_declaration::facade::application_query::ApplicationHistoryQueryBinding;
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_installation::facade::ApplicationSchema;

impl<Schema: ApplicationSchema>
    super::super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema>
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
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
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
        <Self as crate::domain_computation::primary_graph::application_query::OneShotOps<Schema>>::execute_application_query_one_shot(self, phase, plan)
    }

    pub fn execute_application_query_one_shot_in_batch<
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
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
        <Self as crate::domain_computation::primary_graph::application_query::BatchOps<Schema>>::execute_application_query_one_shot_in_batch(self, phase, plan, batch)
    }

    pub fn execute_application_history_capture<History>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        capture: WorthQueryAdmittedApplicationHistoryCapture<'_, Schema, History>,
    ) -> Result<WorthQueryApplicationHistoryCaptureResult<Schema, History>, WorthQueryApplicationHistoryDenial>
    where
        History: ApplicationHistoryQueryBinding<Schema>,
        <<<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value:
    WorthQueryApplicationProjection<Schema, <<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding<Schema>>::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::HistoryCaptureOps<
            Schema,
        >>::execute_application_history_capture(self, phase, capture)
    }
}
