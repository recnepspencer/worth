//! Governed live lease draining and capped row streaming.

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

use super::binding::WorthQuerySharedAccessProgram;
use super::capped_writer::{write_encoded_rows, RemoteByteWriter};
use super::denial::{
    AuthorizationRefusalReason, BudgetRefusalReason, SharedAccessDenial, WatchRefusalReason,
};
use super::scope_binding::{
    clamped_query_row_limit, verify_admitted_scope_entity, BoundScopeIdentity,
};
use super::server_dispatch::map_query_denial;
use super::types::RemoteServerLimits;

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

/// Executes the initial governed read and streams its rows directly into `writer`.
pub(super) fn write_initial_watch_rows<Schema, Program>(
    app: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    caller: (
        &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        &WorthQueryRequestScope,
    ),
    args: (&Program::ScopeId, Program::QueryIntent, RemoteServerLimits),
    writer: &mut RemoteByteWriter,
) -> Result<(), SharedAccessDenial>
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
    let (external, request_scope) = caller;
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
    let rows = result.rows();
    write_encoded_rows(
        writer,
        rows.len(),
        rows.iter().cloned().map(Program::project_row),
    )
}

/// Drains live lease updates and encodes the response directly into a pre-capped writer.
pub(super) fn drain_lease_into_payload<Schema, Program>(
    lease: &mut LiveLease<'_, Schema, Program>,
    phase: &WorthQueryAdvancementPhase<'_>,
    principal: &WorthQueryAuthenticatedPrincipal<
        Schema,
        QueryPrincipal<Schema, Program>,
        QueryPrincipalId<Schema, Program>,
    >,
    args: (&WorthQueryRequestScope, usize, u32),
) -> Result<Vec<u8>, SharedAccessDenial>
where
    Schema: ApplicationSchema + 'static,
    Program: WorthQuerySharedAccessProgram<Schema>,
    QueryResult<Schema, Program>:
        WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Program>> + Clone,
{
    let (request_scope, max_drain, max_frame_bytes) = args;
    let mut latest: Option<QueryResult<Schema, Program>> = None;
    for _ in 0..max_drain {
        match lease.next(phase, principal, request_scope) {
            WorthQueryApplicationLiveOutcome::Delivered(update) => {
                latest = Some(update.result().clone());
            }
            WorthQueryApplicationLiveOutcome::Pending => break,
            other => return Err(map_terminal_live_outcome(other)),
        }
    }
    let mut writer = RemoteByteWriter::new(max_frame_bytes);
    match latest {
        Some(value) => {
            writer.write_u8(1)?;
            write_encoded_rows(&mut writer, 1, std::iter::once(Program::project_row(value)))?;
        }
        None => {
            writer.write_u8(0)?;
        }
    }
    Ok(writer.into_bytes())
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
        &BoundScopeIdentity,
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
    let (principal, request_scope, bound_scope) = caller;
    let (watch_cap, read_cap, watch_input, read_input) = caps;
    let watch_admitted = selected
        .admit_capability_access(phase, principal, watch_cap, watch_input, request_scope)
        .map_err(|_| {
            SharedAccessDenial::AuthorizationRefused(AuthorizationRefusalReason::CapabilityDenied)
        })?;
    verify_admitted_scope_entity(
        &watch_admitted,
        bound_scope,
        AuthorizationRefusalReason::CapabilityDenied,
    )?;
    let read_admitted = selected
        .admit_capability_access(phase, principal, read_cap, read_input, request_scope)
        .map_err(|_| {
            SharedAccessDenial::AuthorizationRefused(AuthorizationRefusalReason::CapabilityDenied)
        })?;
    verify_admitted_scope_entity(
        &read_admitted,
        bound_scope,
        AuthorizationRefusalReason::CapabilityDenied,
    )?;
    Ok(read_admitted)
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
