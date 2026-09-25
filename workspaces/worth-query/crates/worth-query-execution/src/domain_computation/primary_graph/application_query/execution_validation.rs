use worth_query_admission::facade::authenticated_principal::{
    WorthQueryRequestInterruption, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_schema::ApplicationSchema;

use super::{WorthQueryAdmittedApplicationQueryControls, WorthQueryAdmittedApplicationQueryPlan};
use crate::domain_computation::primary_graph::{
    WorthQueryAuthenticatedPrincipal, WorthQueryPrimaryGraphApplicationRuntime,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum WorthQueryApplicationQueryExecutionValidationDenial {
    ForeignPlan,
    StaleInstalledQuery,
    StalePrincipal,
    Cancelled,
    DeadlineExceeded,
    ExpiredBasis,
    BasisUnavailable,
}

pub(super) fn validate_execution_plan<
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
) -> Result<(), WorthQueryApplicationQueryExecutionValidationDenial>
where
    Schema: ApplicationSchema,
{
    if plan.runtime_authority != application.runtime.authority_identity() {
        return Err(WorthQueryApplicationQueryExecutionValidationDenial::ForeignPlan);
    }
    if !application.installed_schema_is_current() {
        return Err(WorthQueryApplicationQueryExecutionValidationDenial::StaleInstalledQuery);
    }
    application
        .installed_schema
        .validate_installed_query(plan.query)
        .map_err(|_| WorthQueryApplicationQueryExecutionValidationDenial::StaleInstalledQuery)
}

pub(super) fn validate_execution_lifetimes<Schema, Principal, PrincipalIdentity>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    controls: &WorthQueryAdmittedApplicationQueryControls<'_>,
    principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
) -> Result<(), WorthQueryApplicationQueryExecutionValidationDenial> {
    validate_request(controls.request_scope())?;
    if controls.basis_is_expired() {
        return Err(WorthQueryApplicationQueryExecutionValidationDenial::ExpiredBasis);
    }
    if application.authentication_is_expired(principal.valid_until()) {
        return Err(WorthQueryApplicationQueryExecutionValidationDenial::StalePrincipal);
    }
    Ok(())
}

pub(super) fn validate_live_basis(
    live: bool,
) -> Result<(), WorthQueryApplicationQueryExecutionValidationDenial> {
    live.then_some(())
        .ok_or(WorthQueryApplicationQueryExecutionValidationDenial::BasisUnavailable)
}

pub(super) fn validate_request(
    request: &WorthQueryRequestScope,
) -> Result<(), WorthQueryApplicationQueryExecutionValidationDenial> {
    match request.interruption() {
        Some(WorthQueryRequestInterruption::Cancelled) => {
            Err(WorthQueryApplicationQueryExecutionValidationDenial::Cancelled)
        }
        Some(WorthQueryRequestInterruption::DeadlineExceeded) => {
            Err(WorthQueryApplicationQueryExecutionValidationDenial::DeadlineExceeded)
        }
        None => Ok(()),
    }
}
