use worth_query_admission::facade::authenticated_principal::{
    WorthQueryRequestInterruption, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_relational::facade::mvcc::CompanionPreflightStop;

mod admitted;
mod batch;
pub use batch::WorthQueryApplicationBatchReadDenial;
mod custody_work;
mod denial;
mod outcome;
mod result;
mod validation;
use validation::{
    admit_request, validate_authentication_lifetime, validate_basis_lifetime,
    validate_one_shot_plan,
};

use denial::{authorization_denial, denial};
pub use denial::{WorthQueryApplicationOneShotDenial, WorthQueryApplicationOneShotDenialKind};

use super::authorized_read::{
    execute_authorized_read, execute_selected_authorized_read, refresh_governed_authorization,
    SelectedAuthorizationCurrentnessStop, SelectedAuthorizedReadStop,
    WorthQueryAuthorizedApplicationReadDenial,
};
use super::read_execution::{
    read_bounded_root_rows, OneShotReadWorkObservation,
    WorthQueryApplicationReadExecutionDenialKind,
};
use super::{
    WorthQueryAdmittedApplicationQueryControls, WorthQueryAdmittedApplicationQueryPlan,
    WorthQueryApplicationProjection, WorthQueryApplicationQueryAccessReceipt,
};
use crate::domain_computation::primary_graph::{
    output_lineage::invalidation::{InvalidationEditAdmission, ReservedExternalWork},
    WorthQueryAuthenticatedPrincipal, WorthQueryPrimaryGraphApplicationRuntime,
};
use outcome::finalize_one_shot;

/// The complete result of a one-shot query read: projected rows, the observed
/// sources and result set, and the access receipt.
///
/// The observed sources can later be bound as a mutation's source expectation.
pub struct WorthQueryApplicationOneShotResult<Query, QueryResult> {
    rows: Vec<QueryResult>,
    observed_sources: Vec<super::WorthQueryObservedSource<Query>>,
    result_set_observation: super::WorthQueryObservedResultSet<Query>,
    request_affinity: super::admitted_result::WorthQueryApplicationQueryRequestAffinity,
    receipt: WorthQueryApplicationQueryAccessReceipt,
}

/// The admitted read keeps its original semantic denial and accounts for
/// newly performed Query execution work on either outcome.
pub(in crate::domain_computation::primary_graph) enum WorthQueryAdmittedOneShotStop {
    Admission(CompanionPreflightStop),
    WorkUnavailable,
    WorkCounterOverflow,
    WorkAccountingMismatch,
    Execution(WorthQueryApplicationOneShotDenial),
}

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
        self.execute_application_query_one_shot_core(plan, None, None, None)
    }

    fn execute_application_query_one_shot_core<
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &self,
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
        spent: Option<&OneShotReadWorkObservation>,
        batch: Option<&super::WorthQueryApplicationQueryBatchAdmission>,
        maximum_work: Option<usize>,
    ) -> Result<
        WorthQueryApplicationOneShotResult<Query, QueryResult>,
        WorthQueryApplicationOneShotDenial,
    >
    where
        QueryResult: WorthQueryApplicationProjection<Schema, Query>,
    {
        validate_one_shot_plan(self, &plan)?;
        refresh_governed_authorization(self, &mut plan)
            .map_err(|read| map_authorized_read_denial(read, plan.query.name()))?;
        let result_buffer = reserve_one_shot_result_buffer_in_batch(self, &plan, batch)?;
        let (raw, authorization_work, read_proof) =
            execute_authorized_read(self, &plan, |runtime, graph, plan| {
                read_bounded_root_rows(
                    runtime,
                    graph,
                    plan,
                    result_buffer,
                    spent,
                    maximum_work.unwrap_or(plan.controls.maximum_work().get()),
                )
            })
            .map_err(|read| map_authorized_read_denial(read, plan.query.name()))?;
        finalize_one_shot(self, plan, raw, authorization_work, read_proof, spent)
    }
}

fn reserve_one_shot_result_buffer<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    plan: &WorthQueryAdmittedApplicationQueryPlan<
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
    super::resource_lifecycle::WorthQueryApplicationResultBufferReservation,
    WorthQueryApplicationOneShotDenial,
>
where
    Schema: ApplicationSchema,
{
    reserve_one_shot_result_buffer_in_batch(application, plan, None)
}

fn reserve_one_shot_result_buffer_in_batch<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    plan: &WorthQueryAdmittedApplicationQueryPlan<
        '_,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >,
    batch: Option<&super::WorthQueryApplicationQueryBatchAdmission>,
) -> Result<
    super::resource_lifecycle::WorthQueryApplicationResultBufferReservation,
    WorthQueryApplicationOneShotDenial,
>
where
    Schema: ApplicationSchema,
{
    application.runtime.primary_graph().ok_or_else(|| {
        denial(
            WorthQueryApplicationOneShotDenialKind::StaleInstalledQuery,
            plan.query.name(),
            plan.query.name(),
        )
    })?;
    let bytes = plan
        .graph_read_plan()
        .budget_check()
        .max_inline_result_bytes();
    Ok(match batch {
        Some(batch) => application.result_buffers.reserve_in_batch(bytes, batch),
        None => application.result_buffers.reserve(bytes),
    })
}

fn map_authorized_read_denial(
    denial_value: WorthQueryAuthorizedApplicationReadDenial,
    query: &str,
) -> WorthQueryApplicationOneShotDenial {
    let (kind, subject) = match denial_value {
        WorthQueryAuthorizedApplicationReadDenial::StalePrincipal => (
            WorthQueryApplicationOneShotDenialKind::StalePrincipal,
            query.to_string(),
        ),
        WorthQueryAuthorizedApplicationReadDenial::StaleScope
        | WorthQueryAuthorizedApplicationReadDenial::StaleBasisScope => (
            WorthQueryApplicationOneShotDenialKind::StaleScope,
            query.to_string(),
        ),
        WorthQueryAuthorizedApplicationReadDenial::Authorization(denial) => {
            return authorization_denial(denial, query);
        }
        WorthQueryAuthorizedApplicationReadDenial::Read(read) => {
            let kind = match read.kind() {
                WorthQueryApplicationReadExecutionDenialKind::Cancelled => {
                    WorthQueryApplicationOneShotDenialKind::Cancelled
                }
                WorthQueryApplicationReadExecutionDenialKind::DeadlineExceeded => {
                    WorthQueryApplicationOneShotDenialKind::DeadlineExceeded
                }
                WorthQueryApplicationReadExecutionDenialKind::PredicateIndexUnavailable => {
                    WorthQueryApplicationOneShotDenialKind::PredicateIndexUnavailable
                }
                WorthQueryApplicationReadExecutionDenialKind::PredicateLookupOverflow => {
                    WorthQueryApplicationOneShotDenialKind::PredicateLookupOverflow
                }
                WorthQueryApplicationReadExecutionDenialKind::ResultLimitExceeded => {
                    WorthQueryApplicationOneShotDenialKind::ResultLimitExceeded
                }
                WorthQueryApplicationReadExecutionDenialKind::CardinalityMismatch => {
                    WorthQueryApplicationOneShotDenialKind::CardinalityMismatch
                }
                WorthQueryApplicationReadExecutionDenialKind::ProjectionUnavailable => {
                    WorthQueryApplicationOneShotDenialKind::ProjectionUnavailable
                }
                WorthQueryApplicationReadExecutionDenialKind::ResultBufferLimitExceeded => {
                    WorthQueryApplicationOneShotDenialKind::ResultBufferLimitExceeded
                }
                WorthQueryApplicationReadExecutionDenialKind::TargetIdentityIndexUnavailable
                | WorthQueryApplicationReadExecutionDenialKind::TargetIdentityLookupOverflow
                | WorthQueryApplicationReadExecutionDenialKind::TargetIdentityNotFound => {
                    WorthQueryApplicationOneShotDenialKind::ProjectionUnavailable
                }
                WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded => {
                    WorthQueryApplicationOneShotDenialKind::WorkLimitExceeded
                }
                WorthQueryApplicationReadExecutionDenialKind::TraversalUnavailable
                | WorthQueryApplicationReadExecutionDenialKind::ContinuationIndexUnavailable
                | WorthQueryApplicationReadExecutionDenialKind::ContinuationBoundaryRejected
                | WorthQueryApplicationReadExecutionDenialKind::ContinuationGenerationChanged
                | WorthQueryApplicationReadExecutionDenialKind::ContinuationPageWidthInvalid => {
                    WorthQueryApplicationOneShotDenialKind::TraversalUnavailable
                }
            };
            return denial(kind, query, read.into_subject());
        }
        WorthQueryAuthorizedApplicationReadDenial::Session => (
            WorthQueryApplicationOneShotDenialKind::ForeignPlan,
            query.to_string(),
        ),
    };
    denial(kind, query, subject)
}
