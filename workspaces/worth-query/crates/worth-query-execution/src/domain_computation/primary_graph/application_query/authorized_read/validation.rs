//! The same actual current READ/security checks before callbacks and payload reads.
use super::*;

pub(super) fn validate_current_read_at_basis<S: ApplicationSchema, Q, P, R, A, I, C>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    entity_resolution: &WorthQueryInstalledEntityResolutionContext,
    runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
    current: &worth_relational::facade::snapshots::SnapshotHandle,
    plan: &WorthQueryAdmittedApplicationQueryPlan<'_, S, Q, P, R, A, I, C>,
) -> Result<WorthQueryApplicationAuthorizationWorkEvidence, WorthQueryAuthorizedApplicationReadDenial>
{
    let authorization_work = validate_current_authorization(
        execution,
        application,
        entity_resolution,
        runtime,
        current,
        plan,
    )?
    .with_execution_security_product_resolution();
    entity_resolution
        .at_snapshot(
            runtime,
            plan.basis.snapshot_handle(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .and_then(|truth| truth.validate_entity_freshness(execution, plan.scope))
        .map_err(|denial| {
            freshness_denial(
                denial,
                WorthQueryAuthorizedApplicationReadDenial::StaleBasisScope,
            )
        })?;
    Ok(authorization_work)
}

/// No payload or Query completion proof is manufactured by this preflight.
/// All Native/security borrows end before the pure provider can run.
pub(in crate::domain_computation::primary_graph::application_query) fn validate_before_callback<
    S: ApplicationSchema,
    Q,
    P,
    R,
    A,
    I,
    C,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    plan: &WorthQueryAdmittedApplicationQueryPlan<'_, S, Q, P, R, A, I, C>,
) -> Result<(), WorthQueryAuthorizedApplicationReadDenial> {
    let security = admit_read_security(execution, application, &plan.security_product, &plan.basis, &plan.index_posture)
        .map_err(|cause| WorthQueryAuthorizedApplicationReadDenial::Authorization(
            crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenialKind::from(cause), plan.query.name(),
            )
        ))?;
    let graph = application
        .runtime
        .primary_graph()
        .ok_or(WorthQueryAuthorizedApplicationReadDenial::Session)?;
    let entity_resolution = graph.retain_entity_resolution_context();
    super::super::ApplicationQueryNativeAccess::acquire().with_runtime_mut(
        graph,
        execution,
        || WorthQueryAuthorizedApplicationReadDenial::Session,
        |runtime| {
            validate_current_read_at_basis(
                execution,
                application,
                &entity_resolution,
                runtime,
                security.snapshot_handle(),
                plan,
            )
            .map(|_| ())
        },
    )
}
