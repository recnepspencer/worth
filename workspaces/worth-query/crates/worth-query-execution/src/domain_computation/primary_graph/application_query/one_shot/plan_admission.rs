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
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
    validate_one_shot_plan_with_native_access(
        execution,
        application,
        plan,
        &mut super::super::ApplicationQueryNativeAccess::acquire(),
    )
}

pub(in crate::domain_computation::primary_graph::application_query) fn validate_one_shot_plan_with_native_access<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
    native: &mut super::super::ApplicationQueryNativeAccess<'_>,
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
    let graph = application.runtime.primary_graph().ok_or_else(|| {
        denial(
            WorthQueryApplicationOneShotDenialKind::ForeignPlan,
            plan.query.name(),
            plan.query.name(),
        )
    })?;
    let live = native.with_runtime_mut(
        graph,
        execution,
        || {
            denial(
                WorthQueryApplicationOneShotDenialKind::ForeignPlan,
                plan.query.name(),
                plan.query.name(),
            )
        },
        |runtime| Ok(plan.basis.is_live_at(runtime)),
    )?;
    if !live {
        return Err(denial(
            WorthQueryApplicationOneShotDenialKind::BasisUnavailable,
            plan.query.name(),
            plan.query.name(),
        ));
    }
    Ok(())
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
