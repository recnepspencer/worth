//! Exact selected read, with the carried Work reservation opened after auth.

use worth_query_declaration::facade::application_schema::ApplicationSchema;

use super::validate_selected_authorization_currentness;
use crate::domain_computation::primary_graph::application_query::read_execution::read_bounded_root_rows;
use crate::domain_computation::primary_graph::application_query::read_execution::OneShotReadWorkObservation;
use crate::domain_computation::primary_graph::application_query::read_execution::RawNonLiveKernelOutcome;
use crate::domain_computation::primary_graph::application_query::read_execution::WorthQueryApplicationReadExecutionDenial;

use crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationAuthorizationWorkEvidence;
use crate::domain_computation::primary_graph::product_operation::SelectedPermissionSecurityStop;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

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
    let super::WorthQueryQueryIndexPosture::SelectedInstalled(prepared) = &plan.index_posture
    else {
        return Err(SelectedAuthorizedReadStop::StaleSecurity);
    };
    let security = application
        .admit_matching_head_query_prepared_read_security_basis_with_native_access(
            execution,
            &plan.security_product,
            &plan.basis,
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
                let raw =
                    match read_bounded_root_rows(runtime, execution, layout, plan, Some(spent)) {
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
