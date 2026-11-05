//! Exact selected read, with the carried Work reservation opened after auth.

use crate::domain_computation::primary_graph::product_operation::MatchingHeadScopedOps;
use worth_query_declaration::facade::application_schema::ApplicationSchema;

use super::validate_selected_authorization_currentness;
use crate::domain_computation::primary_graph::application_query::read_execution::read_bounded_root_rows;
use crate::domain_computation::primary_graph::application_query::read_execution::OneShotReadWorkObservation;
use crate::domain_computation::primary_graph::application_query::read_execution::RawNonLiveKernelOutcome;
use crate::domain_computation::primary_graph::application_query::read_execution::WorthQueryApplicationReadExecutionDenial;

use crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationAuthorizationWorkEvidence;
use crate::domain_computation::primary_graph::product_operation::SelectedPermissionSecurityStop;

pub(in crate::domain_computation::primary_graph::application_query) enum SelectedAuthorizedReadStop
{
    Currentness(super::SelectedAuthorizationCurrentnessStop),
    Security(SelectedPermissionSecurityStop),
    StaleSecurity,
    Session,

    Read(WorthQueryApplicationReadExecutionDenial),
}

pub(in crate::domain_computation::primary_graph::application_query) fn execute_selected_authorized_read_with_native_access<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    application: &dyn crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>,
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

    spent: &OneShotReadWorkObservation,
    native: &mut super::super::ApplicationQueryNativeAccess<'_>,
) -> Result<
    (
        RawNonLiveKernelOutcome,
        WorthQueryApplicationAuthorizationWorkEvidence,
        crate::domain_computation::provider_session::WorthQuerySessionGraphReadProof,
    ),
    SelectedAuthorizedReadStop,
>
where
    Schema: ApplicationSchema,
{
    execute_selected_authorized_body_with_native_access(
        execution,
        application,
        plan,
        native,
        |runtime, layout, plan| {
            read_bounded_root_rows(runtime, execution, layout, plan, Some(spent))
        },
    )
}

/// The existing selected security/read session surrounds both Native row reads
/// and an exact maintained Source observation. A callback supplies read mechanics,
/// never an admission, invariant verdict or publication effect.
pub(in crate::domain_computation::primary_graph::application_query) fn execute_selected_authorized_body_with_native_access<
    Schema: ApplicationSchema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
    Output,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    application: &dyn crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>,
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
    SelectedAuthorizedReadStop,
> {
    let super::WorthQueryQueryIndexPosture::SelectedInstalled(prepared) = &plan.index_posture
    else {
        return Err(SelectedAuthorizedReadStop::StaleSecurity);
    };
    let security = application
        .admit_matching_head_query_prepared_read_security_basis_with_native_access(
            execution,
            &plan.security_product,
            plan.security_basis.as_ref().unwrap_or(&plan.basis),
            prepared,
            native,
        )
        .map_err(SelectedAuthorizedReadStop::Security)?
        .ok_or(SelectedAuthorizedReadStop::StaleSecurity)?;
    let ((raw, authorization_work), proof) =
        super::execute_graph_read_with_snapshot_with_native_access(
            execution,
            &plan.graph_work,
            &plan.basis,
            security.snapshot_handle(),
            || SelectedAuthorizedReadStop::Session,
            (),
            native,
            |runtime, layout, security, ()| {
                validate_selected_authorization_currentness(
                    execution,
                    application,
                    runtime,
                    security,
                    plan,
                )
                .map_err(SelectedAuthorizedReadStop::Currentness)?;
                let raw = match read(runtime, layout, plan) {
                    Ok(raw) => raw,
                    Err(denial) => {
                        return Err(SelectedAuthorizedReadStop::Read(denial));
                    }
                };
                Ok((
                    raw,
                    plan.authorization_work
                        .with_execution_security_product_resolution(),
                ))
            },
        )?;
    Ok((raw, authorization_work, proof))
}
