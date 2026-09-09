use crate::domain_computation::primary_graph::{
    application_query::{
        read_execution::WorthQueryApplicationReadExecutionDenial,
        WorthQueryAdmittedApplicationQueryPlan, WorthQueryApplicationAuthorizationWorkEvidence,
    },
    WorthQueryInstalledEntityResolutionContext, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQueryPrincipalResolutionMode,
};
use worth_query_declaration::facade::application_schema::ApplicationSchema;

pub(super) enum WorthQueryAuthorizedApplicationReadDenial {
    StalePrincipal,
    StaleScope,
    StaleBasisScope,
    Authorization(crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenial),
    Read(WorthQueryApplicationReadExecutionDenial),
    Session,
}

pub(super) fn execute_authorized_read<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
    Output,
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
    let security = application.admit_product_security_basis(plan.basis.product())
        .map_err(|denial| WorthQueryAuthorizedApplicationReadDenial::Authorization(
            crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenialKind::ProductSecurityBasis(denial),
                plan.query.name(),
            ),
        ))?;
    let basis = plan.basis.identity();
    let entity_resolution = application
        .runtime
        .primary_graph()
        .ok_or(WorthQueryAuthorizedApplicationReadDenial::Session)?
        .retain_entity_resolution_context();
    let (read_outcome, proof) = plan
        .graph_work
        .execute_query_read(basis, |runtime, layout| {
            let authorization_work = validate_current_authorization(
                application,
                &entity_resolution,
                runtime,
                security.snapshot_handle(),
                plan,
            )?;
            let authorization_work =
                authorization_work.with_execution_security_product_resolution();
            entity_resolution
                .at_snapshot(
                    runtime,
                    plan.basis.snapshot_handle(),
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .and_then(|truth| truth.validate_entity_freshness(plan.scope))
                .map_err(|_| WorthQueryAuthorizedApplicationReadDenial::StaleBasisScope)?;
            let output = read(runtime, layout, plan)
                .map_err(WorthQueryAuthorizedApplicationReadDenial::Read)?;
            Ok((output, authorization_work))
        })
        .map_err(|_| WorthQueryAuthorizedApplicationReadDenial::Session)?;
    let (output, authorization_work) = read_outcome?;
    Ok((output, authorization_work, proof))
}

fn validate_current_authorization<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    entity_resolution: &WorthQueryInstalledEntityResolutionContext,
    runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
    current: &worth_relational::facade::snapshots::SnapshotHandle,
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
) -> Result<WorthQueryApplicationAuthorizationWorkEvidence, WorthQueryAuthorizedApplicationReadDenial>
where
    Schema: ApplicationSchema,
{
    plan.authorization
        .validate_currentness_in(runtime, current, application.authorization.bridge())
        .map_err(|kind| match kind {
            crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenialKind::StalePrincipal => {
                WorthQueryAuthorizedApplicationReadDenial::StalePrincipal
            }
            kind => WorthQueryAuthorizedApplicationReadDenial::Authorization(
                crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenial::new(
                    kind,
                    plan.query.name(),
                ),
            ),
        })?;
    if let Some(authorization) = plan.governance.authorization() {
        authorization
            .validate_currentness_in(runtime, current, application.authorization.bridge())
            .map_err(|kind| {
                WorthQueryAuthorizedApplicationReadDenial::Authorization(
                    crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenial::new(
                        kind,
                        plan.query.name(),
                    ),
                )
            })?;
    }
    if !plan.governance.computation_matches(
        &plan.graph_work,
        application.runtime.authority_identity(),
        plan.query.identity(),
        plan.parameters.identity(),
        plan.principal.principal_entity_id(),
        plan.scope.entity_id(),
    ) {
        return Err(WorthQueryAuthorizedApplicationReadDenial::Authorization(
            crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenial::inconsistent(
                plan.query.name(),
            ),
        ));
    }
    entity_resolution
        .at_snapshot(
            runtime,
            current,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .and_then(|truth| truth.validate_entity_freshness(plan.scope))
        .map_err(|_| WorthQueryAuthorizedApplicationReadDenial::StaleScope)?;
    Ok(plan.authorization_work)
}

pub(super) fn refresh_governed_authorization<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    plan: &mut WorthQueryAdmittedApplicationQueryPlan<
        '_,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >,
) -> Result<(), WorthQueryAuthorizedApplicationReadDenial>
where
    Schema: ApplicationSchema,
{
    let (governance, graph_work) = (&mut plan.governance, &mut plan.graph_work);
    let Some(authorization) = governance.authorization_mut() else {
        return Ok(());
    };
    application
        .refresh_capability_authorization_for_graph_work(authorization, graph_work)
        .map_err(WorthQueryAuthorizedApplicationReadDenial::Authorization)
}
