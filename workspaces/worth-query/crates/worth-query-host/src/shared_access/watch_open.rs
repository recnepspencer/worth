//! Scope-bound governed live lease preparation and opening.

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
use super::denial::{BudgetRefusalReason, SharedAccessDenial};
use super::scope_binding::{clamped_row_limit_for_binding, BoundScopeIdentity};
use super::types::RemoteServerLimits;
use super::watch_drain::{
    admit_watch_and_read_caps, open_live_lease_with_disclosure, resolve_principal,
    resolve_scope_entity, IntentBinding, LiveLease, QueryMarker, QueryPrincipalId, QueryResult,
};

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

impl<Schema, Program> PreparedLiveOpen<Schema, Program>
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
    fn prepare(
        app: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        request_scope: &WorthQueryRequestScope,
        args: (&Program::ScopeId, Program::QueryIntent, RemoteServerLimits),
    ) -> Result<Self, SharedAccessDenial> {
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
        let max_rows = clamped_row_limit_for_binding::<Schema, Program>(&binding, limits)?.get();
        let controls = WorthQueryApplicationLiveControls::bounded(
            request_scope.clone(),
            limits.live_buffer_capacity(),
            max_rows,
        )
        .map_err(|_| {
            SharedAccessDenial::BudgetRefused(BudgetRefusalReason::RequestLimitExceeded)
        })?;
        Ok(Self {
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

    fn open_in_phase<'home>(
        self,
        app: &'home WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        phase: &WorthQueryAdvancementPhase<'_>,
        caller: (
            &WorthQueryAuthenticatedExternalPrincipal<Schema>,
            &WorthQueryRequestScope,
        ),
    ) -> Result<(LiveLease<'home, Schema, Program>, BoundScopeIdentity), SharedAccessDenial> {
        let (external, scope) = caller;
        let selected = app
            .on_branch(app.current_world())
            .select(phase)
            .map_err(|_| SharedAccessDenial::HostUnavailable)?;
        let principal = resolve_principal::<Schema, Program>(
            &selected,
            phase,
            (&self.binding, external, scope),
        )?;
        let resolved = resolve_scope_entity::<Schema, Program>(
            &selected,
            phase,
            (&principal, scope),
            self.scope_binding,
        )?;
        let bound = BoundScopeIdentity::from_resolved(&resolved);
        let caps = (
            &self.watch_cap,
            &self.read_cap,
            self.watch_input,
            self.read_input,
        );
        let disclosure = admit_watch_and_read_caps::<Schema, Program>(
            &selected,
            phase,
            (&principal, scope, &bound),
            caps,
        )?;
        let lease = open_live_lease_with_disclosure::<Schema, Program>(
            selected,
            phase,
            (&principal, resolved, disclosure),
            (self.binding, self.parameters, self.controls),
        )?;
        Ok((lease, bound))
    }
}

pub(super) fn open_governed_live_lease<'home, Schema, Program>(
    app: &'home WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request_scope: &WorthQueryRequestScope,
    args: (&Program::ScopeId, Program::QueryIntent, RemoteServerLimits),
) -> Result<(LiveLease<'home, Schema, Program>, BoundScopeIdentity), SharedAccessDenial>
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
    let prepared = PreparedLiveOpen::<Schema, Program>::prepare(app, request_scope, args)?;
    app.with_application_advancement(request_scope, |phase| {
        prepared.open_in_phase(app, &phase, (external, request_scope))
    })
    .map_err(|_| SharedAccessDenial::BudgetRefused(BudgetRefusalReason::ExecutionResourceDenied))?
}
