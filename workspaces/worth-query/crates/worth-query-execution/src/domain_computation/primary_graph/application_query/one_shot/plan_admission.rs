//! Owner-side plan, request, and lifetime admission for one-shot reads.

use super::*;

pub(in crate::domain_computation::primary_graph::application_query) fn validate_one_shot_plan<
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
    if !plan.basis.is_live() {
        return Err(denial(
            WorthQueryApplicationOneShotDenialKind::BasisUnavailable,
            plan.query.name(),
            plan.query.name(),
        ));
    }
    Ok(())
}

pub(in crate::domain_computation::primary_graph::application_query) fn reserve_one_shot_result_buffer<
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
    super::super::resource_lifecycle::WorthQueryApplicationResultBufferReservation,
    WorthQueryApplicationOneShotDenial,
>
where
    Schema: ApplicationSchema,
{
    reserve_one_shot_result_buffer_in_batch(application, plan, None)
}

pub(in crate::domain_computation::primary_graph::application_query) fn validate_authentication_lifetime<
    Schema,
    Principal,
    PrincipalIdentity,
>(
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

pub(in crate::domain_computation::primary_graph::application_query) fn validate_basis_lifetime(
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

pub(in crate::domain_computation::primary_graph::application_query) fn validate_plan_owner<
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

pub(in crate::domain_computation::primary_graph::application_query) fn admit_request(
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

pub(in crate::domain_computation::primary_graph::application_query) fn reserve_one_shot_result_buffer_in_batch<
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
    batch: Option<&super::super::WorthQueryApplicationQueryBatchAdmission>,
) -> Result<
    super::super::resource_lifecycle::WorthQueryApplicationResultBufferReservation,
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
        Some(batch) => application
            .result_buffers
            .reserve_in_batch(batch.inline_result_bytes(bytes), batch),
        None => application.result_buffers.reserve(bytes),
    })
}
