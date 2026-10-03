//! Selected-root validation before a Fresh read can reserve kernel Work.

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_relational::facade::mvcc::CompanionPreflightStop;

use super::super::WorthQueryAdmittedApplicationQueryPlan;
use crate::domain_computation::primary_graph::{
    InvalidationEditAdmission, WorthQueryPrimaryGraphApplicationRuntime,
};

pub(in crate::domain_computation::primary_graph::application_query) enum SelectedAuthorizationCurrentnessStop
{
    Admission(CompanionPreflightStop),
    StaleIssuedAccess,
    StaleDecision,
    UnsupportedGovernance,
    Interrupted(WorthQueryRequestInterruption),
}

/// A selected permission proves the immutable principal and scope on this
/// precise native root. Mutable Bridge, Signal and Native decision evidence
/// remains subject to its own admitted currentness check at read time.
pub(in crate::domain_computation::primary_graph::application_query) fn validate_selected_authorization_currentness<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    security: &worth_relational::facade::snapshots::SnapshotHandle,
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
    admission: &mut InvalidationEditAdmission,
) -> Result<(), SelectedAuthorizationCurrentnessStop>
where
    Schema: ApplicationSchema,
{
    // The selected-access option, governance posture and final request
    // interruption are Query wrapper reads outside their nested owners.
    admission
        .charge_external_work(3)
        .map_err(SelectedAuthorizationCurrentnessStop::Admission)?;
    let issued = plan
        .selected_access
        .as_ref()
        .ok_or(SelectedAuthorizationCurrentnessStop::StaleIssuedAccess)?;
    let exact = issued
        .matches(
            plan.principal,
            plan.scope,
            &plan.security_product,
            security,
            plan.basis.snapshot_handle(),
            plan.controls.request_scope(),
            admission,
        )
        .map_err(SelectedAuthorizationCurrentnessStop::Admission)?;
    if !exact {
        return Err(SelectedAuthorizationCurrentnessStop::StaleIssuedAccess);
    }
    if plan.governance.authorization().is_some() {
        return Err(SelectedAuthorizationCurrentnessStop::UnsupportedGovernance);
    }
    let current = plan
        .authorization
        .conventional_decisions_current_admitted(
            runtime,
            security,
            application.authorization.bridge(),
            admission,
        )
        .map_err(SelectedAuthorizationCurrentnessStop::Admission)?;
    if !current {
        return Err(SelectedAuthorizationCurrentnessStop::StaleDecision);
    }
    // The request may have been cancelled while owner currentness inspected
    // Native and Signal state. The kernel has not reserved or run yet.
    if let Some(interruption) = plan.controls.request_scope().interruption() {
        return Err(SelectedAuthorizationCurrentnessStop::Interrupted(
            interruption,
        ));
    }
    Ok(())
}
