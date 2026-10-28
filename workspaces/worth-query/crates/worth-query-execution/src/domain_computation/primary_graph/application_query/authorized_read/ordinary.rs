//! One current-security and exact-truth owner for ordinary and history reads.
use super::*;

pub(in crate::domain_computation::primary_graph::application_query) fn execute_authorized_read<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
    Output,
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
    read: impl FnOnce(
        &worth_relational::facade::runtime::RelationalRuntime,
        &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
        &WorthQueryAdmittedApplicationQueryPlan<
            '_,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
    ) -> Result<Output, WorthQueryApplicationReadExecutionDenial>,
) -> Result<
    (
        Output,
        WorthQueryApplicationAuthorizationWorkEvidence,
        crate::domain_computation::provider_session::WorthQuerySessionGraphReadProof,
    ),
    WorthQueryAuthorizedApplicationReadDenial,
>
where
    Schema: ApplicationSchema,
{
    execute_authorized_read_with_access(
        execution,
        application,
        plan,
        |runtime, layout, _security, plan| {
            read(runtime, layout, plan).map_err(WorthQueryAuthorizedApplicationReadDenial::Read)
        },
    )
}

pub(in crate::domain_computation::primary_graph::application_query) fn execute_authorized_read_with_access<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
    Output,
    Stop,
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
    read: impl FnOnce(
        &worth_relational::facade::runtime::RelationalRuntime,
        &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
        &worth_relational::facade::snapshots::SnapshotHandle,
        &WorthQueryAdmittedApplicationQueryPlan<
            '_,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
    ) -> Result<Output, Stop>,
) -> Result<
    (
        Output,
        WorthQueryApplicationAuthorizationWorkEvidence,
        crate::domain_computation::provider_session::WorthQuerySessionGraphReadProof,
    ),
    Stop,
>
where
    Schema: ApplicationSchema,
    Stop: From<WorthQueryAuthorizedApplicationReadDenial>,
{
    let ((output, authorization_work), proof) = execute_read_with_security(
        execution,
        application,
        &plan.security_product,
        &plan.basis,
        &plan.index_posture,
        &plan.graph_work,
        |denial| {
            Stop::from(WorthQueryAuthorizedApplicationReadDenial::Authorization(
                crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenial::new(
                    crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenialKind::from(denial),
                    plan.query.name(),
                ),
            ))
        },
        || Stop::from(WorthQueryAuthorizedApplicationReadDenial::Session),
        || {
            application
                .runtime
                .primary_graph()
                .ok_or_else(|| Stop::from(WorthQueryAuthorizedApplicationReadDenial::Session))
                .map(|graph| graph.retain_entity_resolution_context())
        },
        |runtime, layout, security, entity_resolution| {
            let authorization_work = validate_current_authorization(
                execution,
                application,
                &entity_resolution,
                runtime,
                security,
                plan,
            )
            .map_err(Stop::from)?;
            let authorization_work =
                authorization_work.with_execution_security_product_resolution();
            entity_resolution
                .at_snapshot(
                    runtime,
                    plan.basis.snapshot_handle(),
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .and_then(|truth| truth.validate_entity_freshness(execution, plan.scope))
                .map_err(|denial| {
                    Stop::from(freshness_denial(
                        denial,
                        WorthQueryAuthorizedApplicationReadDenial::StaleBasisScope,
                    ))
                })?;
            let output = read(runtime, layout, security, plan)?;
            Ok((output, authorization_work))
        },
    )?;
    Ok((output, authorization_work, proof))
}
