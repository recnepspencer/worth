use super::*;
use crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use {
    super::super::WorthQueryApplicationQueryBatchAdmission,
    crate::domain_computation::primary_graph::vocabulary::application_query::batch::WorthQueryApplicationQueryBatchResourceDenial,
};

mod result;
use crate::domain_computation::primary_graph::vocabulary::application_query::one_shot_denial::authorization_denial;
use crate::domain_computation::primary_graph::vocabulary::application_query::one_shot_denial::WorthQueryApplicationOneShotDenial;
pub use result::WorthQueryApplicationBatchResult;

/// A batch read preserves the ordinary execution cause and distinguishes an
/// aggregate admission refusal. It carries no partially completed result.
#[derive(Debug)]
pub enum WorthQueryApplicationBatchReadDenial {
    Resource(WorthQueryApplicationQueryBatchResourceDenial),
    Execution(WorthQueryApplicationOneShotDenial),
}

pub trait BatchOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>
{
    fn execute_application_query_one_shot_in_batch<
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
        let request = match phase.execution_request_for(self.product_runtime()) {
            Ok(request) => request,
            Err(_) => return Err(WorthQueryApplicationBatchReadDenial::Execution(authorization_denial(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial::new(crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::ForeignPhase), "phase-custody"), plan.query.name()))),
        };
        let execution = &request;
        self.execute_application_query_one_shot_in_batch_with_request(execution, plan, batch)
    }
}

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                Schema,
            > + ?Sized,
    > BatchOps<Schema> for HostRuntime
{
}

pub(in crate::domain_computation) trait BatchScopedOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>
{
    fn execute_application_query_one_shot_in_batch_with_request<
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
        batch: &WorthQueryApplicationQueryBatchAdmission,
    ) -> Result<
        WorthQueryApplicationBatchResult<Query, QueryResult>,
        WorthQueryApplicationBatchReadDenial,
    >
    where
        QueryResult: WorthQueryApplicationProjection<Schema, Query>,
    {
        self.execute_application_query_one_shot_in_batch_metered(execution, plan, batch, None)
    }
}

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                Schema,
            > + ?Sized,
    > BatchScopedOps<Schema> for HostRuntime
{
}

pub(in crate::domain_computation::primary_graph::application_query) trait OneShotBatchScopedOps<
    Schema: ApplicationSchema,
>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>
{
    fn execute_application_query_one_shot_in_batch_metered<
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        mut plan: WorthQueryAdmittedApplicationQueryPlan<
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
        charge: Option<
            &dyn Fn(
                usize,
                &str,
            ) -> Result<
                (),
                crate::domain_computation::primary_graph::vocabulary::application_query::read_execution_denial::WorthQueryApplicationReadExecutionDenial,
            >,
        >,
    ) -> Result<
        WorthQueryApplicationBatchResult<Query, QueryResult>,
        WorthQueryApplicationBatchReadDenial,
    >
    where
        QueryResult: WorthQueryApplicationProjection<Schema, Query>,
    {
        use WorthQueryApplicationBatchReadDenial::{Execution, Resource};
        let item = match plan.planned_batch_item.take() {
            Some(item) if !item.belongs_to(batch) => {
                return Err(Resource(
                    WorthQueryApplicationQueryBatchResourceDenial::ForeignPlan,
                ))
            }
            Some(item) => item,
            None => batch
                .take_planned_read(
                    plan.runtime_authority.as_u64(),
                    plan.query.binding_identity(),
                    plan.query.identity(),
                    plan.controls.maximum_result_count().get(),
                )
                .map_err(Resource)?
                .expect("a registered read consumes one slot"),
        };
        let spent = match charge {
            Some(charge) => OneShotReadWorkObservation::with_charge(charge),
            None => OneShotReadWorkObservation::new(),
        };
        self.execute_application_query_one_shot_core(execution, plan, Some(&spent))
            .map(|result| WorthQueryApplicationBatchResult::issued(result, item))
            .map_err(Execution)
    }
}

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                Schema,
            > + ?Sized,
    > OneShotBatchScopedOps<Schema> for HostRuntime
{
}
