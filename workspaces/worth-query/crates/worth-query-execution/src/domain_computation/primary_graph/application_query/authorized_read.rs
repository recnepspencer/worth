use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
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

mod freshness_denial;
use freshness_denial::freshness_denial;
mod selected_currentness;
mod selected_read;
pub(in crate::domain_computation::primary_graph::application_query) use selected_currentness::{
    validate_selected_authorization_currentness, SelectedAuthorizationCurrentnessStop,
};
pub(in crate::domain_computation::primary_graph::application_query) use selected_read::{
    execute_selected_authorized_read_with_native_access, SelectedAuthorizedReadStop,
};

pub(super) enum WorthQueryAuthorizedApplicationReadDenial {
    StalePrincipal,
    StaleScope,
    StaleBasisScope,
    Authorization(crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenial),
    Read(WorthQueryApplicationReadExecutionDenial),
    Session,
}

mod validation;
pub(in crate::domain_computation::primary_graph::application_query) use validation::validate_before_callback;
mod ordinary;
pub(super) use ordinary::{execute_authorized_read, execute_authorized_read_with_access};

/// Keep the exact Product security and graph-session acceptance common to
/// ordinary and selected reads. Each caller supplies only its currentness
/// validation and read body; the selected caller retains its typed meter stop.
fn execute_read_with_security<Schema, Context, Output, Stop>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
{
    let security = admit_read_security(
        execution,
        application,
        security_product,
        basis,
        index_posture,
    )
    .map_err(security_denial)?;
    let context = prepare()?;
    execute_graph_read_with_snapshot(
        execution,
        graph_work,
        basis,
        security.snapshot_handle(),
        session_denial,
        context,
        perform,
    )
}

fn admit_read_security<'basis, Schema: ApplicationSchema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    security_product: &crate::basis::WorthQueryProductObservationLease,
    basis: &'basis super::basis::WorthQueryApplicationQueryBasisCustody,
    index_posture: &WorthQueryQueryIndexPosture,
) -> Result<
    crate::domain_computation::primary_graph::product_operation::WorthQueryProductSecurityBasis<
        'basis,
    >,
    crate::basis::WorthQueryProductBranchAdmissionDenial,
> {
    match index_posture {
        WorthQueryQueryIndexPosture::HistoricalAllPrimary => {
            application.admit_query_product_security_basis(execution, security_product, basis)
        }
        WorthQueryQueryIndexPosture::SelectedInstalled(prepared) => application
            .admit_query_prepared_read_security_basis(execution, security_product, basis, prepared),
    }
}

pub(super) fn execute_graph_read_with_snapshot<Context, Output, Stop>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
> {
    execute_graph_read_with_snapshot_with_native_access(
        execution,
        graph_work,
        basis,
        security_snapshot,
        session_denial,
        context,
        &mut super::ApplicationQueryNativeAccess::acquire(),
        perform,
    )
}

pub(super) fn execute_graph_read_with_snapshot_with_native_access<Context, Output, Stop>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    graph_work: &crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession,
    basis: &super::basis::WorthQueryApplicationQueryBasisCustody,
    security_snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    session_denial: impl Fn() -> Stop,
    context: Context,
    native: &mut super::ApplicationQueryNativeAccess<'_>,
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
> {
    let (read_outcome, proof) = native
        .execute_query_read(
            execution,
            graph_work,
            basis.identity(),
            |runtime, layout| perform(runtime, layout, security_snapshot, context),
        )
        .map_err(|_| session_denial())?;
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
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
        .validate_currentness_in(execution, runtime, current, application.authorization.bridge())
        .map_err(|kind| match kind {
            crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::StalePrincipal) => {
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
            .validate_currentness_in(execution, runtime, current, application.authorization.bridge())
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
        .and_then(|truth| truth.validate_entity_freshness(execution, plan.scope))
        .map_err(|denial| {
            freshness_denial(
                denial,
                WorthQueryAuthorizedApplicationReadDenial::StaleScope,
            )
        })?;
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
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
        .refresh_capability_authorization_for_graph_work(execution, authorization, graph_work)
        .map_err(WorthQueryAuthorizedApplicationReadDenial::Authorization)
}
