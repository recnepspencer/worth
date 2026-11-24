//! WORTH-governed live watch opening, revocation verification, and teardown.

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryParameterSet,
        ApplicationQueryScopeResolution,
    },
    application_schema::{
        ApplicationOperationMarkerIdentity, ApplicationSchema, ApplicationStructuredValueBinding,
    },
    authentication::WorthQueryExternalPrincipalIdentity,
};
use worth_query_execution::facade::{
    application_contribution::WorthQueryAdvancementPhase,
    primary_graph::{
        WorthQueryApplicationLiveControls, WorthQueryApplicationProjection,
        WorthQueryPrimaryGraphApplicationRuntime,
    },
};
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationCapability, WorthQueryInstalledApplicationQueryBinding,
};

use super::binding::WorthQuerySharedAccessProgram;
use super::denial::{AuthorizationRefusalReason, BudgetRefusalReason, SharedAccessDenial};
use super::types::{RemoteServerLimits, RemoteWatchPollOutcome};
use super::watch_drain::{
    admit_watch_and_read_caps, drain_lease_updates, open_live_lease_with_disclosure,
    read_initial_watch_rows, resolve_principal, resolve_scope_entity, IntentBinding, LiveLease,
    QueryMarker, QueryPrincipalId, QueryResult,
};

pub(super) struct ActiveRemoteWatch<'home, Schema, Program>
where
    Schema: ApplicationSchema + 'static,
    Program: WorthQuerySharedAccessProgram<Schema>,
    QueryResult<Schema, Program>:
        WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Program>>,
{
    pub(super) owner_principal: WorthQueryExternalPrincipalIdentity,
    pub(super) scope: Program::ScopeId,
    lease: LiveLease<'home, Schema, Program>,
}

impl<Schema, Program> ActiveRemoteWatch<'_, Schema, Program>
where
    Schema: ApplicationSchema + 'static,
    Program: WorthQuerySharedAccessProgram<Schema>,
    QueryResult<Schema, Program>:
        WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Program>>,
{
    pub(super) fn close(self) {
        let _ = self.lease.close();
    }
}

impl<'home, Schema, Program> ActiveRemoteWatch<'home, Schema, Program>
where
    Schema: ApplicationSchema + 'static,
    Program: WorthQuerySharedAccessProgram<Schema>,
    <IntentBinding<Schema, Program> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<Schema, QueryPrincipalId<Schema, Program>>,
    QueryResult<Schema, Program>:
        WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Program>> + Clone,
    <Program::ReadOperation as ApplicationOperationMarkerIdentity<Schema>>::InputBinding:
        ApplicationStructuredValueBinding<Value = Program::ReadCapabilityInput>,
    <Program::WatchOperation as ApplicationOperationMarkerIdentity<Schema>>::InputBinding:
        ApplicationStructuredValueBinding<Value = Program::WatchCapabilityInput>,
{
    pub(super) fn open(
        app: &'home WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        args: (
            Program::ScopeId,
            Program::QueryIntent,
            Program::QueryIntent,
            RemoteServerLimits,
        ),
    ) -> Result<(Self, Vec<Vec<u8>>), SharedAccessDenial> {
        let (scope, live_intent, initial_intent, limits) = args;
        let lease = open_governed_live_lease::<Schema, Program>(
            app,
            external,
            request_scope,
            (&scope, live_intent, limits),
        )?;
        let initial_rows = read_initial_watch_rows::<Schema, Program>(
            app,
            external,
            request_scope,
            (&scope, initial_intent, limits),
        )?;
        Ok((
            Self {
                owner_principal: external.identity().clone(),
                scope,
                lease,
            },
            initial_rows,
        ))
    }

    /// Re-verifies current WORTH watch capability before draining queued deliveries.
    pub(super) fn poll(
        &mut self,
        app: &'home WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        limits: RemoteServerLimits,
    ) -> Result<RemoteWatchPollOutcome<Vec<u8>>, SharedAccessDenial> {
        let binding = app
            .installed_schema()
            .installed_query_binding::<IntentBinding<Schema, Program>>()
            .map_err(|_| SharedAccessDenial::HostUnavailable)?;
        let installed_cap = app
            .installed_schema()
            .capability(Program::watch_capability(), Program::watch_operation())
            .map_err(|_| SharedAccessDenial::HostUnavailable)?;
        let watch_input = Program::build_watch_capability_input(&self.scope);
        let max_drain = limits.live_buffer_capacity().saturating_add(1);
        app.with_application_advancement(request_scope, |phase| {
            let selected = app
                .on_branch(app.current_world())
                .select(&phase)
                .map_err(|_| SharedAccessDenial::HostUnavailable)?;
            let principal = resolve_principal::<Schema, Program>(
                &selected,
                &phase,
                (&binding, external, request_scope),
            )?;
            selected
                .admit_capability_access(
                    &phase,
                    &principal,
                    &installed_cap,
                    watch_input,
                    request_scope,
                )
                .map_err(|_| {
                    SharedAccessDenial::AuthorizationRefused(
                        AuthorizationRefusalReason::WatchCapabilityRevoked,
                    )
                })?;
            drain_lease_updates::<Schema, Program>(
                &mut self.lease,
                &phase,
                &principal,
                (request_scope, max_drain),
            )
        })
        .map_err(|_| {
            SharedAccessDenial::BudgetRefused(BudgetRefusalReason::ExecutionResourceDenied)
        })?
    }
}

struct PreparedLiveOpen<Schema, Program>
where
    Schema: ApplicationSchema + 'static,
    Program: WorthQuerySharedAccessProgram<Schema>,
{
    binding: WorthQueryInstalledApplicationQueryBinding<Schema, IntentBinding<Schema, Program>>,
    watch_cap: WorthQueryInstalledApplicationCapability<
        Schema,
        Program::WatchCapability,
        Program::WatchOperation,
        Program::WatchCapabilityInput,
    >,
    read_cap: WorthQueryInstalledApplicationCapability<
        Schema,
        Program::ReadCapability,
        Program::ReadOperation,
        Program::ReadCapabilityInput,
    >,
    watch_input: Program::WatchCapabilityInput,
    read_input: Program::ReadCapabilityInput,
    scope_binding:
        <IntentBinding<Schema, Program> as ApplicationQueryBinding<Schema>>::ScopeBinding,
    parameters: ApplicationQueryParameterSet<QueryMarker<Schema, Program>>,
    controls: WorthQueryApplicationLiveControls,
}

fn prepare_live_open<Schema, Program>(
    app: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    request_scope: &WorthQueryRequestScope,
    args: (&Program::ScopeId, Program::QueryIntent, RemoteServerLimits),
) -> Result<PreparedLiveOpen<Schema, Program>, SharedAccessDenial>
where
    Schema: ApplicationSchema + 'static,
    Program: WorthQuerySharedAccessProgram<Schema>,
    <Program::ReadOperation as ApplicationOperationMarkerIdentity<Schema>>::InputBinding:
        ApplicationStructuredValueBinding<Value = Program::ReadCapabilityInput>,
    <Program::WatchOperation as ApplicationOperationMarkerIdentity<Schema>>::InputBinding:
        ApplicationStructuredValueBinding<Value = Program::WatchCapabilityInput>,
{
    let (scope_id, intent, limits) = args;
    let parameters = intent.parameters();
    let scope_binding = intent.into_scope();
    let binding = app
        .installed_schema()
        .installed_query_binding::<IntentBinding<Schema, Program>>()
        .map_err(|_| SharedAccessDenial::HostUnavailable)?;
    let watch_cap = app
        .installed_schema()
        .capability(Program::watch_capability(), Program::watch_operation())
        .map_err(|_| SharedAccessDenial::HostUnavailable)?;
    let read_cap = app
        .installed_schema()
        .capability(Program::read_capability(), Program::read_operation())
        .map_err(|_| SharedAccessDenial::HostUnavailable)?;
    let max_rows = (limits.max_rows_per_response() as usize)
        .min(binding.query().read_graph().maximum_result_count().max(1));
    let controls = WorthQueryApplicationLiveControls::bounded(
        request_scope.clone(),
        limits.live_buffer_capacity(),
        max_rows,
    )
    .map_err(|_| SharedAccessDenial::BudgetRefused(BudgetRefusalReason::RequestLimitExceeded))?;
    Ok(PreparedLiveOpen {
        binding,
        watch_cap,
        read_cap,
        watch_input: Program::build_watch_capability_input(scope_id),
        read_input: Program::build_read_capability_input(scope_id),
        scope_binding,
        parameters,
        controls,
    })
}

fn open_governed_live_lease<'home, Schema, Program>(
    app: &'home WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request_scope: &WorthQueryRequestScope,
    args: (&Program::ScopeId, Program::QueryIntent, RemoteServerLimits),
) -> Result<LiveLease<'home, Schema, Program>, SharedAccessDenial>
where
    Schema: ApplicationSchema + 'static,
    Program: WorthQuerySharedAccessProgram<Schema>,
    <IntentBinding<Schema, Program> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<Schema, QueryPrincipalId<Schema, Program>>,
    QueryResult<Schema, Program>:
        WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Program>> + Clone,
    <Program::ReadOperation as ApplicationOperationMarkerIdentity<Schema>>::InputBinding:
        ApplicationStructuredValueBinding<Value = Program::ReadCapabilityInput>,
    <Program::WatchOperation as ApplicationOperationMarkerIdentity<Schema>>::InputBinding:
        ApplicationStructuredValueBinding<Value = Program::WatchCapabilityInput>,
{
    let prepared = prepare_live_open::<Schema, Program>(app, request_scope, args)?;
    app.with_application_advancement(request_scope, |phase| {
        open_lease_in_phase::<Schema, Program>(app, &phase, (external, request_scope), prepared)
    })
    .map_err(|_| SharedAccessDenial::BudgetRefused(BudgetRefusalReason::ExecutionResourceDenied))?
}

fn open_lease_in_phase<'home, Schema, Program>(
    app: &'home WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
    caller: (
        &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        &WorthQueryRequestScope,
    ),
    prepared: PreparedLiveOpen<Schema, Program>,
) -> Result<LiveLease<'home, Schema, Program>, SharedAccessDenial>
where
    Schema: ApplicationSchema + 'static,
    Program: WorthQuerySharedAccessProgram<Schema>,
    <IntentBinding<Schema, Program> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<Schema, QueryPrincipalId<Schema, Program>>,
    QueryResult<Schema, Program>:
        WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Program>> + Clone,
    <Program::ReadOperation as ApplicationOperationMarkerIdentity<Schema>>::InputBinding:
        ApplicationStructuredValueBinding<Value = Program::ReadCapabilityInput>,
    <Program::WatchOperation as ApplicationOperationMarkerIdentity<Schema>>::InputBinding:
        ApplicationStructuredValueBinding<Value = Program::WatchCapabilityInput>,
{
    let (external, request_scope) = caller;
    let selected = app
        .on_branch(app.current_world())
        .select(phase)
        .map_err(|_| SharedAccessDenial::HostUnavailable)?;
    let principal = resolve_principal::<Schema, Program>(
        &selected,
        phase,
        (&prepared.binding, external, request_scope),
    )?;
    let disclosure = admit_watch_and_read_caps::<Schema, Program>(
        &selected,
        phase,
        (&principal, request_scope),
        (
            &prepared.watch_cap,
            &prepared.read_cap,
            prepared.watch_input,
            prepared.read_input,
        ),
    )?;
    let resolved_scope = resolve_scope_entity::<Schema, Program>(
        &selected,
        phase,
        (&principal, request_scope),
        prepared.scope_binding,
    )?;
    open_live_lease_with_disclosure::<Schema, Program>(
        selected,
        phase,
        (&principal, resolved_scope, disclosure),
        (prepared.binding, prepared.parameters, prepared.controls),
    )
}
