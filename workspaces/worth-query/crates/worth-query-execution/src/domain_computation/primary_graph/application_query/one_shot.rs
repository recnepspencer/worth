use worth_query_admission::facade::authenticated_principal::{
    WorthQueryRequestInterruption, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_relational::facade::mvcc::CompanionPreflightStop;

mod admitted;
mod custody_work;
mod denial;
mod outcome;
mod result;

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
        self.execute_application_query_one_shot_core(plan, None)
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
        let result_buffer = reserve_one_shot_result_buffer(self, &plan)?;
        let (raw, authorization_work, read_proof) =
            execute_authorized_read(self, &plan, |runtime, graph, plan| {
                read_bounded_root_rows(
                    runtime,
                    graph,
                    plan,
                    result_buffer,
                    spent,
                    plan.controls.maximum_work().get(),
                )
            })
            .map_err(|read| map_authorized_read_denial(read, plan.query.name()))?;
        finalize_one_shot(self, plan, raw, authorization_work, read_proof, spent)
    }
}

fn validate_one_shot_plan<
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
) -> Result<(), WorthQueryApplicationOneShotDenial>
where
    Schema: ApplicationSchema,
{
    validate_plan_owner(application, plan)?;
    if plan.controls.lane()
        != worth_query_admission::facade::application_query::WorthQueryApplicationQueryLane::OneShot
    {
        return Err(denial(
            WorthQueryApplicationOneShotDenialKind::ForeignPlan,
            plan.query.name(),
            plan.query.name(),
        ));
    }
    let request = plan.controls.request_scope();
    admit_request(request, plan.query.name())?;
    validate_basis_lifetime(&plan.controls, plan.query.name())?;
    validate_authentication_lifetime(application, plan.principal, plan.query.name())?;
    if !plan.basis.is_live().map_err(|handle| denial(
        WorthQueryApplicationOneShotDenialKind::Authorization(
            crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenialKind::Handle(handle)),
        plan.query.name(), plan.query.name()))? {
        return Err(denial(
            WorthQueryApplicationOneShotDenialKind::BasisUnavailable,
            plan.query.name(),
            plan.query.name(),
        ));
    }
    Ok(())
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
    application.runtime.primary_graph().ok_or_else(|| {
        denial(
            WorthQueryApplicationOneShotDenialKind::StaleInstalledQuery,
            plan.query.name(),
            plan.query.name(),
        )
    })?;
    Ok(application.result_buffers.reserve(
        plan.graph_read_plan()
            .budget_check()
            .max_inline_result_bytes(),
    ))
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

fn validate_authentication_lifetime<Schema, Principal, PrincipalIdentity>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    subject: &str,
) -> Result<(), WorthQueryApplicationOneShotDenial> {
    if application.authentication_is_expired(principal.valid_until()) {
        Err(denial(
            WorthQueryApplicationOneShotDenialKind::StalePrincipal,
            subject,
            subject,
        ))
    } else {
        Ok(())
    }
}

fn validate_basis_lifetime(
    controls: &WorthQueryAdmittedApplicationQueryControls<'_>,
    subject: &str,
) -> Result<(), WorthQueryApplicationOneShotDenial> {
    if controls.basis_is_expired() {
        Err(denial(
            WorthQueryApplicationOneShotDenialKind::ExpiredBasis,
            subject,
            subject,
        ))
    } else {
        Ok(())
    }
}

fn validate_plan_owner<
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
) -> Result<(), WorthQueryApplicationOneShotDenial>
where
    Schema: ApplicationSchema,
{
    if plan.runtime_authority != application.runtime.authority_identity() {
        return Err(denial(
            WorthQueryApplicationOneShotDenialKind::ForeignPlan,
            plan.query.name(),
            plan.query.name(),
        ));
    }
    if !application.installed_schema_is_current() {
        return Err(denial(
            WorthQueryApplicationOneShotDenialKind::StaleInstalledQuery,
            plan.query.name(),
            plan.query.name(),
        ));
    }
    application
        .installed_schema
        .validate_installed_query(plan.query)
        .map_err(|_| {
            denial(
                WorthQueryApplicationOneShotDenialKind::StaleInstalledQuery,
                plan.query.name(),
                plan.query.name(),
            )
        })
}

fn admit_request(
    request: &WorthQueryRequestScope,
    subject: &str,
) -> Result<(), WorthQueryApplicationOneShotDenial> {
    match request.interruption() {
        Some(WorthQueryRequestInterruption::Cancelled) => Err(denial(
            WorthQueryApplicationOneShotDenialKind::Cancelled,
            subject,
            subject,
        )),
        Some(WorthQueryRequestInterruption::DeadlineExceeded) => Err(denial(
            WorthQueryApplicationOneShotDenialKind::DeadlineExceeded,
            subject,
            subject,
        )),
        None => Ok(()),
    }
}
