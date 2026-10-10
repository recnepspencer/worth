use super::super::super::{
    outcome::{WorthQueryApplicationLiveOpenDenial, WorthQueryApplicationLiveOpenDenialKind},
    scope_identity::read_scope_identity,
};
use super::super::validation::{open_denial, open_read_denial};
use crate::domain_computation::primary_graph::{
    application_query::authorized_read::{execute_authorized_read, refresh_governed_authorization},
    WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_declaration::facade::application_schema::ApplicationSchema;
pub(super) struct WorthQueryApplicationLiveInitialRead {
    pub(super) governance: crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
    pub(super) scope_identity: worth_foundational::facade::AspectValue,
    pub(super) graph_work: crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession,
    pub(super) read_proof: crate::domain_computation::provider_session::WorthQuerySessionGraphReadProof,
    pub(super) initial_read_work: crate::domain_computation::provider_session::WorthQueryObservedGraphReadWork,
    pub(super) basis_release: super::super::super::super::WorthQueryApplicationBasisReleaseReceipt,
}

pub(super) fn execute_live_initial_read<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    mut plan: crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan<
        '_, Schema, Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope,
    >,
    subject: &str,
) -> Result<WorthQueryApplicationLiveInitialRead, WorthQueryApplicationLiveOpenDenial>
where
    Schema: ApplicationSchema,
{
    refresh_governed_authorization(application, &mut plan)
        .map_err(|denial| open_read_denial(denial, subject))?;
    application.runtime.primary_graph().ok_or_else(|| {
        open_denial(
            WorthQueryApplicationLiveOpenDenialKind::ScopeIdentityUnavailable,
            subject,
        )
    })?;
    let ((scope_identity, initial_read_work), _, read_proof) =
        execute_authorized_read(application, &plan, read_scope_identity)
            .map_err(|denial| open_read_denial(denial, subject))?;
    let governance = plan.take_governance();
    let basis_release = plan.basis.release();
    if !basis_release.released() {
        return Err(open_denial(
            WorthQueryApplicationLiveOpenDenialKind::BasisReleaseFailed,
            subject,
        ));
    }
    Ok(WorthQueryApplicationLiveInitialRead {
        governance,
        scope_identity,
        graph_work: plan.graph_work,
        read_proof,
        initial_read_work,
        basis_release,
    })
}
