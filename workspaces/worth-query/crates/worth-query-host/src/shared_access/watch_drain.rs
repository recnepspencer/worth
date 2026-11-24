//! Governed live lease draining and initial watch snapshot reads.

use std::num::NonZeroUsize;

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::{
    application_query::{
        ApplicationLiveQueryIntent, ApplicationQueryBinding, ApplicationQueryIntent,
        ApplicationQueryParameterSet, ApplicationQueryScopeBinding,
        ApplicationQueryScopeResolution,
    },
    application_schema::{
        ApplicationOperationMarkerIdentity, ApplicationSchema, ApplicationStructuredValueBinding,
    },
};
use worth_query_execution::facade::{
    application_contribution::WorthQueryAdvancementPhase,
    primary_graph::{
        WorthQueryAdmittedApplicationCapabilityAccess, WorthQueryApplicationEntityIdentity,
        WorthQueryApplicationLiveControls, WorthQueryApplicationLiveLease,
        WorthQueryApplicationLiveOutcome, WorthQueryApplicationProjection,
        WorthQueryAuthenticatedPrincipal, WorthQueryPrimaryGraphApplicationRuntime,
        WorthQueryPrincipalResolutionMode, WorthQuerySelectedProductOperation,
    },
};
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationCapability, WorthQueryInstalledApplicationQueryBinding,
};
use worth_query_publication::facade::application_entry::WorthQueryApplicationRequestExt;

use super::binding::{RemoteBinaryCodec, WorthQuerySharedAccessProgram};
use super::denial::{
    AuthorizationRefusalReason, BudgetRefusalReason, SharedAccessDenial, WatchRefusalReason,
};
use super::server_dispatch::map_query_denial;
use super::types::{RemoteServerLimits, RemoteWatchPollOutcome};

pub(super) type IntentBinding<S, P> =
    <<P as WorthQuerySharedAccessProgram<S>>::QueryIntent as ApplicationQueryIntent<S>>::Binding;
pub(super) type QueryMarker<S, P> = <IntentBinding<S, P> as ApplicationQueryBinding<S>>::Query;
pub(super) type QueryParams<S, P> = <<IntentBinding<S, P> as ApplicationQueryBinding<S>>::ParameterBinding as ApplicationStructuredValueBinding>::Value;
pub(super) type QueryResult<S, P> = <<IntentBinding<S, P> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
pub(super) type QueryPrincipal<S, P> =
    <IntentBinding<S, P> as ApplicationQueryBinding<S>>::Principal;
pub(super) type QueryPrincipalId<S, P> =
    <IntentBinding<S, P> as ApplicationQueryBinding<S>>::PrincipalIdentity;
pub(super) type QueryScope<S, P> = <<IntentBinding<S, P> as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope;
pub(super) type QueryTarget<S, P> =
    <<P as WorthQuerySharedAccessProgram<S>>::QueryIntent as ApplicationLiveQueryIntent<S>>::Target;
pub(super) type QueryLiveCause<S, P> =
    <<P as WorthQuerySharedAccessProgram<S>>::QueryIntent as ApplicationLiveQueryIntent<S>>::LiveCause;

pub(super) type LiveLease<'home, S, P> = WorthQueryApplicationLiveLease<
    'home,
    S,
    QueryMarker<S, P>,
    QueryParams<S, P>,
    QueryResult<S, P>,
    QueryPrincipal<S, P>,
    QueryPrincipalId<S, P>,
    QueryScope<S, P>,
    QueryTarget<S, P>,
    QueryLiveCause<S, P>,
>;

pub(super) fn clamped_query_row_limit<Schema, Program>(
    app: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    limits: RemoteServerLimits,
) -> Result<NonZeroUsize, SharedAccessDenial>
where
    Schema: ApplicationSchema + 'static,
    Program: WorthQuerySharedAccessProgram<Schema>,
{
    let binding = app
        .installed_schema()
        .installed_query_binding::<IntentBinding<Schema, Program>>()
        .map_err(|_| SharedAccessDenial::HostUnavailable)?;
    let declared_max = binding.query().read_graph().maximum_result_count().max(1);
    let clamped = (limits.max_rows_per_response() as usize).min(declared_max);
    NonZeroUsize::new(clamped).ok_or(SharedAccessDenial::BudgetRefused(
        BudgetRefusalReason::RequestLimitExceeded,
    ))
}

pub(super) fn read_initial_watch_rows<Schema, Program>(
    app: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request_scope: &WorthQueryRequestScope,
    args: (&Program::ScopeId, Program::QueryIntent, RemoteServerLimits),
) -> Result<Vec<Vec<u8>>, SharedAccessDenial>
where
    Schema: ApplicationSchema + 'static,
    Program: WorthQuerySharedAccessProgram<Schema>,
    <IntentBinding<Schema, Program> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<Schema, QueryPrincipalId<Schema, Program>>,
    QueryResult<Schema, Program>:
        WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Program>> + Clone,
    <Program::ReadOperation as ApplicationOperationMarkerIdentity<Schema>>::InputBinding:
        ApplicationStructuredValueBinding<Value = Program::ReadCapabilityInput>,
{
    let (scope_id, intent, limits) = args;
    let max_rows = clamped_query_row_limit::<Schema, Program>(app, limits)?;
    let result = app
        .request(external, request_scope)
        .query(intent)
        .limit_results(max_rows)
        .execute_governed(
            Program::read_capability(),
            Program::read_operation(),
            Program::build_read_capability_input(scope_id),
        )
        .map_err(map_query_denial)?;
    Ok(result
        .rows()
        .iter()
        .cloned()
        .map(|v| Program::project_row(v).encode_remote())
        .collect())
}

pub(super) fn drain_lease_updates<Schema, Program>(
    lease: &mut LiveLease<'_, Schema, Program>,
    phase: &WorthQueryAdvancementPhase<'_>,
    principal: &WorthQueryAuthenticatedPrincipal<
        Schema,
        QueryPrincipal<Schema, Program>,
        QueryPrincipalId<Schema, Program>,
    >,
    scope_and_cap: (&WorthQueryRequestScope, usize),
) -> Result<RemoteWatchPollOutcome<Vec<u8>>, SharedAccessDenial>
where
    Schema: ApplicationSchema + 'static,
    Program: WorthQuerySharedAccessProgram<Schema>,
    QueryResult<Schema, Program>:
        WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Program>> + Clone,
{
    let (request_scope, max_drain) = scope_and_cap;
    let mut latest: Option<Vec<Vec<u8>>> = None;
    for _ in 0..max_drain {
        match lease.next(phase, principal, request_scope) {
            WorthQueryApplicationLiveOutcome::Delivered(update) => {
                let row = Program::project_row(update.result().clone()).encode_remote();
                latest = Some(vec![row]);
            }
            WorthQueryApplicationLiveOutcome::Pending => break,
            other => return Err(map_terminal_live_outcome(other)),
        }
    }
    Ok(match latest {
        Some(rows) => RemoteWatchPollOutcome::Delivered(rows),
        None => RemoteWatchPollOutcome::Pending,
    })
}

fn map_terminal_live_outcome<Q, R>(
    outcome: WorthQueryApplicationLiveOutcome<Q, R>,
) -> SharedAccessDenial {
    match outcome {
        WorthQueryApplicationLiveOutcome::AuthorizationDenied(_) => {
            SharedAccessDenial::AuthorizationRefused(
                AuthorizationRefusalReason::WatchCapabilityRevoked,
            )
        }
        WorthQueryApplicationLiveOutcome::StalePrincipal => {
            SharedAccessDenial::AuthorizationRefused(AuthorizationRefusalReason::StalePrincipal)
        }
        WorthQueryApplicationLiveOutcome::StaleScope => {
            SharedAccessDenial::AuthorizationRefused(AuthorizationRefusalReason::StaleScope)
        }
        WorthQueryApplicationLiveOutcome::Overflow(_) => {
            SharedAccessDenial::WatchRefused(WatchRefusalReason::WatchOverflow)
        }
        WorthQueryApplicationLiveOutcome::Closed => {
            SharedAccessDenial::WatchRefused(WatchRefusalReason::WatchClosed)
        }
        WorthQueryApplicationLiveOutcome::ExecutionRequest(_)
        | WorthQueryApplicationLiveOutcome::Cancelled
        | WorthQueryApplicationLiveOutcome::DeadlineExceeded => {
            SharedAccessDenial::BudgetRefused(BudgetRefusalReason::ExecutionResourceDenied)
        }
        WorthQueryApplicationLiveOutcome::ProjectionDenied(_)
        | WorthQueryApplicationLiveOutcome::CauseDenied(_)
        | WorthQueryApplicationLiveOutcome::Unavailable
        | WorthQueryApplicationLiveOutcome::Delivered(_)
        | WorthQueryApplicationLiveOutcome::Pending => SharedAccessDenial::HostUnavailable,
    }
}

pub(super) type WatchCaps<'a, Schema, Program> = (
    &'a WorthQueryInstalledApplicationCapability<
        Schema,
        <Program as WorthQuerySharedAccessProgram<Schema>>::WatchCapability,
        <Program as WorthQuerySharedAccessProgram<Schema>>::WatchOperation,
        <Program as WorthQuerySharedAccessProgram<Schema>>::WatchCapabilityInput,
    >,
    &'a WorthQueryInstalledApplicationCapability<
        Schema,
        <Program as WorthQuerySharedAccessProgram<Schema>>::ReadCapability,
        <Program as WorthQuerySharedAccessProgram<Schema>>::ReadOperation,
        <Program as WorthQuerySharedAccessProgram<Schema>>::ReadCapabilityInput,
    >,
    <Program as WorthQuerySharedAccessProgram<Schema>>::WatchCapabilityInput,
    <Program as WorthQuerySharedAccessProgram<Schema>>::ReadCapabilityInput,
);

pub(super) fn admit_watch_and_read_caps<Schema, Program>(
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
    caller: (
        &WorthQueryAuthenticatedPrincipal<
            Schema,
            QueryPrincipal<Schema, Program>,
            QueryPrincipalId<Schema, Program>,
        >,
        &WorthQueryRequestScope,
    ),
    caps: WatchCaps<'_, Schema, Program>,
) -> Result<
    WorthQueryAdmittedApplicationCapabilityAccess<
        Schema,
        Program::ReadCapability,
        Program::ReadOperation,
        Program::ReadCapabilityInput,
    >,
    SharedAccessDenial,
>
where
    Schema: ApplicationSchema + 'static,
    Program: WorthQuerySharedAccessProgram<Schema>,
    <Program::ReadOperation as ApplicationOperationMarkerIdentity<Schema>>::InputBinding:
        ApplicationStructuredValueBinding<Value = Program::ReadCapabilityInput>,
    <Program::WatchOperation as ApplicationOperationMarkerIdentity<Schema>>::InputBinding:
        ApplicationStructuredValueBinding<Value = Program::WatchCapabilityInput>,
{
    let (principal, request_scope) = caller;
    let (watch_cap, read_cap, watch_input, read_input) = caps;
    selected
        .admit_capability_access(phase, principal, watch_cap, watch_input, request_scope)
        .map_err(|_| {
            SharedAccessDenial::AuthorizationRefused(AuthorizationRefusalReason::CapabilityDenied)
        })?;
    selected
        .admit_capability_access(phase, principal, read_cap, read_input, request_scope)
        .map_err(|_| {
            SharedAccessDenial::AuthorizationRefused(AuthorizationRefusalReason::CapabilityDenied)
        })
}

pub(super) fn resolve_principal<Schema, Program>(
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
    args: (
        &WorthQueryInstalledApplicationQueryBinding<Schema, IntentBinding<Schema, Program>>,
        &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        &WorthQueryRequestScope,
    ),
) -> Result<
    WorthQueryAuthenticatedPrincipal<
        Schema,
        QueryPrincipal<Schema, Program>,
        QueryPrincipalId<Schema, Program>,
    >,
    SharedAccessDenial,
>
where
    Schema: ApplicationSchema + 'static,
    Program: WorthQuerySharedAccessProgram<Schema>,
{
    let (binding, external, request_scope) = args;
    selected
        .resolve_authenticated_principal(
            phase,
            binding.principal_binding(),
            external,
            request_scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .map_err(|_| {
            SharedAccessDenial::AuthorizationRefused(
                AuthorizationRefusalReason::PrincipalResolutionDenied,
            )
        })
}

pub(super) fn resolve_scope_entity<Schema, Program>(
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
    caller: (
        &WorthQueryAuthenticatedPrincipal<
            Schema,
            QueryPrincipal<Schema, Program>,
            QueryPrincipalId<Schema, Program>,
        >,
        &WorthQueryRequestScope,
    ),
    scope_binding: <IntentBinding<Schema, Program> as ApplicationQueryBinding<Schema>>::ScopeBinding,
) -> Result<
    WorthQueryApplicationEntityIdentity<Schema, QueryScope<Schema, Program>>,
    SharedAccessDenial,
>
where
    Schema: ApplicationSchema + 'static,
    Program: WorthQuerySharedAccessProgram<Schema>,
    <IntentBinding<Schema, Program> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<Schema, QueryPrincipalId<Schema, Program>>,
{
    let (principal, request_scope) = caller;
    let (scope_field, scope_value) = scope_binding.into_field_parts(principal.principal_identity());
    selected
        .resolve_entity(
            phase,
            scope_field,
            scope_value,
            request_scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .map_err(|_| {
            SharedAccessDenial::AuthorizationRefused(AuthorizationRefusalReason::ScopeNotFound)
        })
}

pub(super) type OpenLeaseAccess<'a, Schema, Program> = (
    &'a WorthQueryAuthenticatedPrincipal<
        Schema,
        QueryPrincipal<Schema, Program>,
        QueryPrincipalId<Schema, Program>,
    >,
    WorthQueryApplicationEntityIdentity<Schema, QueryScope<Schema, Program>>,
    WorthQueryAdmittedApplicationCapabilityAccess<
        Schema,
        <Program as WorthQuerySharedAccessProgram<Schema>>::ReadCapability,
        <Program as WorthQuerySharedAccessProgram<Schema>>::ReadOperation,
        <Program as WorthQuerySharedAccessProgram<Schema>>::ReadCapabilityInput,
    >,
);

pub(super) type OpenLeasePlan<Schema, Program> = (
    WorthQueryInstalledApplicationQueryBinding<Schema, IntentBinding<Schema, Program>>,
    ApplicationQueryParameterSet<QueryMarker<Schema, Program>>,
    WorthQueryApplicationLiveControls,
);

pub(super) fn open_live_lease_with_disclosure<'home, Schema, Program>(
    selected: WorthQuerySelectedProductOperation<'home, Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
    access: OpenLeaseAccess<'_, Schema, Program>,
    plan: OpenLeasePlan<Schema, Program>,
) -> Result<LiveLease<'home, Schema, Program>, SharedAccessDenial>
where
    Schema: ApplicationSchema + 'static,
    Program: WorthQuerySharedAccessProgram<Schema>,
    QueryResult<Schema, Program>:
        WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Program>> + Clone,
{
    let (principal, resolved_scope, disclosure) = access;
    let (binding, parameters, controls) = plan;
    selected
        .open_governed_application_query_live::<
            QueryMarker<Schema, Program>,
            QueryParams<Schema, Program>,
            QueryResult<Schema, Program>,
            QueryPrincipal<Schema, Program>,
            QueryPrincipalId<Schema, Program>,
            QueryScope<Schema, Program>,
            QueryTarget<Schema, Program>,
            QueryLiveCause<Schema, Program>,
            Program::ReadCapability,
            Program::ReadOperation,
            Program::ReadCapabilityInput,
        >(phase, binding.into_query(), principal, resolved_scope, disclosure, parameters, controls)
        .map_err(|_| {
            SharedAccessDenial::AuthorizationRefused(AuthorizationRefusalReason::CapabilityDenied)
        })
}
