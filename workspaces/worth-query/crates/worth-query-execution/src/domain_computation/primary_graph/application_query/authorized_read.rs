use crate::domain_computation::primary_graph::{
    application_query::{
        graph_read_plan_binding::WorthQueryQueryIndexPosture,
        read_execution::WorthQueryApplicationReadExecutionDenial,
        WorthQueryAdmittedApplicationQueryPlan, WorthQueryApplicationAuthorizationWorkEvidence,
    },
    WorthQueryInstalledEntityResolutionContext, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQueryPrincipalResolutionMode,
};
use worth_query_declaration::facade::application_schema::ApplicationSchema;

mod selected_currentness;
mod selected_read;
pub(in crate::domain_computation::primary_graph::application_query) use selected_currentness::{
    validate_selected_authorization_currentness, SelectedAuthorizationCurrentnessStop,
};
pub(in crate::domain_computation::primary_graph::application_query) use selected_read::{
    execute_selected_authorized_read, SelectedAuthorizedReadStop,
};

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
    let ((output, authorization_work), proof) = execute_read_with_security(
        application,
        &plan.security_product,
        &plan.basis,
        &plan.index_posture,
        &plan.graph_work,
        |denial| {
            WorthQueryAuthorizedApplicationReadDenial::Authorization(
                crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenial::new(
                    crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenialKind::ProductSecurityBasis(denial),
                    plan.query.name(),
                ),
            )
        },
        || WorthQueryAuthorizedApplicationReadDenial::Session,
        || {
            application
                .runtime
                .primary_graph()
                .ok_or(WorthQueryAuthorizedApplicationReadDenial::Session)
                .map(|graph| graph.retain_entity_resolution_context())
        },
        |runtime, layout, security, entity_resolution| {
            let authorization_work = validate_current_authorization(
                application,
                &entity_resolution,
                runtime,
                security,
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
        },
    )?;
    Ok((output, authorization_work, proof))
}

/// Keep the exact Product security and graph-session acceptance common to
/// ordinary and selected reads. Each caller supplies only its currentness
/// validation and read body; the selected caller retains its typed meter stop.
fn execute_read_with_security<Schema, Context, Output, Stop>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    security_product: &crate::basis::WorthQueryProductObservationLease,
    basis: &super::basis::WorthQueryApplicationQueryBasisCustody,
    index_posture: &WorthQueryQueryIndexPosture,
    graph_work: &crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession,
    security_denial: impl FnOnce(crate::basis::WorthQueryProductBranchAdmissionDenial) -> Stop,
    session_denial: impl Fn() -> Stop,
    prepare: impl FnOnce() -> Result<Context, Stop>,
    perform: impl FnOnce(
        &mut worth_relational::facade::runtime::RelationalRuntime,
        &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
        &worth_relational::facade::snapshots::SnapshotHandle,
        Context,
    ) -> Result<Output, Stop>,
) -> Result<
    (
        Output,
        crate::domain_computation::provider_session::WorthQuerySessionGraphReadProof,
    ),
    Stop,
>
where
    Schema: ApplicationSchema,
    Stop: From<crate::facade::primary_graph::WorthQueryHandleDenial>,
{
    let security = match index_posture {
        WorthQueryQueryIndexPosture::HistoricalAllPrimary => {
            application.admit_query_product_security_basis(security_product, basis)
        }
        WorthQueryQueryIndexPosture::SelectedInstalled(prepared) => {
            application.admit_query_prepared_read_security_basis(security_product, basis, prepared)
        }
    }
    .map_err(security_denial)?;
    let context = prepare()?;
    execute_graph_read_with_snapshot(
        graph_work,
        basis,
        security.snapshot_handle(),
        session_denial,
        context,
        perform,
    )
}

pub(super) fn execute_graph_read_with_snapshot<Context, Output, Stop>(
    graph_work: &crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession,
    basis: &super::basis::WorthQueryApplicationQueryBasisCustody,
    security_snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    session_denial: impl Fn() -> Stop,
    context: Context,
    perform: impl FnOnce(
        &mut worth_relational::facade::runtime::RelationalRuntime,
        &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
        &worth_relational::facade::snapshots::SnapshotHandle,
        Context,
    ) -> Result<Output, Stop>,
) -> Result<
    (
        Output,
        crate::domain_computation::provider_session::WorthQuerySessionGraphReadProof,
    ),
    Stop,
>
where
    Stop: From<crate::facade::primary_graph::WorthQueryHandleDenial>,
{
    let (read_outcome, proof) = graph_work
        .execute_query_read(basis.identity(), |runtime, layout| {
            perform(runtime, layout, security_snapshot, context)
        })
        .map_err(|denial| {
            denial
                .handle_denial()
                .map_or_else(&session_denial, Stop::from)
        })?;
    Ok((read_outcome?, proof))
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

impl From<crate::facade::primary_graph::WorthQueryHandleDenial>
    for WorthQueryAuthorizedApplicationReadDenial
{
    fn from(handle: crate::facade::primary_graph::WorthQueryHandleDenial) -> Self {
        Self::Authorization(crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenial::new(
            crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenialKind::Handle(handle), "application query read"))
    }
}
