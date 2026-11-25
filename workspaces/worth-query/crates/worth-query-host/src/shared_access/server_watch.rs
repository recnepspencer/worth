//! WORTH-governed live watch lifecycle, scope-bound renewal, and teardown.

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
    },
    application_schema::{
        ApplicationOperationMarkerIdentity, ApplicationSchema, ApplicationStructuredValueBinding,
    },
    authentication::WorthQueryExternalPrincipalIdentity,
};
use worth_query_execution::facade::{
    application_contribution::WorthQueryAdvancementPhase,
    primary_graph::{WorthQueryApplicationProjection, WorthQueryPrimaryGraphApplicationRuntime},
};
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationCapability, WorthQueryInstalledApplicationQueryBinding,
};

use super::auth::current_unix_secs;
use super::binding::WorthQuerySharedAccessProgram;
use super::capped_writer::RemoteByteWriter;
use super::denial::{
    AuthenticationRefusalReason, AuthorizationRefusalReason, BudgetRefusalReason,
    SharedAccessDenial,
};
use super::scope_binding::{verify_admitted_scope_entity, BoundScopeIdentity};
use super::types::{RemoteServerLimits, RemoteWatchId};
use super::watch_drain::{
    drain_lease_into_payload, resolve_principal, resolve_scope_entity, write_initial_watch_rows,
    IntentBinding, LiveLease, QueryMarker, QueryPrincipalId, QueryResult,
};
use super::watch_open::open_governed_live_lease;

pub(super) struct ActiveRemoteWatch<'home, Schema, Program>
where
    Schema: ApplicationSchema + 'static,
    Program: WorthQuerySharedAccessProgram<Schema>,
    QueryResult<Schema, Program>:
        WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Program>>,
{
    pub(super) owner_principal: WorthQueryExternalPrincipalIdentity,
    pub(super) scope: Program::ScopeId,
    pub(super) connection_id: Option<u64>,
    pub(super) token_expires_at_unix_secs: u64,
    bound_scope: BoundScopeIdentity,
    query_payload: Program::QueryPayload,
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

    pub(super) const fn is_expired_at(&self, now_unix_secs: u64) -> bool {
        self.token_expires_at_unix_secs <= now_unix_secs
    }
}

pub(super) struct WatchOpenArgs<Schema, Program>
where
    Schema: ApplicationSchema + 'static,
    Program: WorthQuerySharedAccessProgram<Schema>,
{
    pub(super) watch_id: RemoteWatchId,
    pub(super) connection_id: Option<u64>,
    pub(super) token_expires_at_unix_secs: u64,
    pub(super) scope: Program::ScopeId,
    pub(super) query_payload: Program::QueryPayload,
    pub(super) live_intent: Program::QueryIntent,
    pub(super) initial_intent: Program::QueryIntent,
    pub(super) limits: RemoteServerLimits,
    pub(super) _schema: std::marker::PhantomData<Schema>,
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
    /// Opens a scope-bound governed live lease and streams the WatchOpenResponse payload
    /// into a pre-capped writer. Rolls back the lease immediately if response encoding fails.
    pub(super) fn open(
        app: &'home WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        args: WatchOpenArgs<Schema, Program>,
    ) -> Result<(Self, Vec<u8>), SharedAccessDenial> {
        let (lease, bound_scope) = open_governed_live_lease::<Schema, Program>(
            app,
            external,
            request_scope,
            (&args.scope, args.live_intent, args.limits),
        )?;
        let mut writer = RemoteByteWriter::new(args.limits.max_frame_bytes());
        let encoded = writer
            .write_u64(args.watch_id.incarnation())
            .and_then(|()| writer.write_u64(args.watch_id.sequence()))
            .and_then(|()| {
                write_initial_watch_rows::<Schema, Program>(
                    app,
                    (external, request_scope),
                    (&args.scope, args.initial_intent, args.limits),
                    &mut writer,
                )
            });
        if let Err(denial) = encoded {
            let _ = lease.close();
            return Err(denial);
        }
        let active = Self {
            owner_principal: external.identity().clone(),
            scope: args.scope,
            connection_id: args.connection_id,
            token_expires_at_unix_secs: args.token_expires_at_unix_secs,
            bound_scope,
            query_payload: args.query_payload,
            lease,
        };
        Ok((active, writer.into_bytes()))
    }

    /// Re-verifies token expiry, bound scope entity identity, and current WORTH watch
    /// capability before draining queued deliveries directly into a pre-capped payload.
    pub(super) fn poll(
        &mut self,
        app: &'home WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        limits: RemoteServerLimits,
    ) -> Result<Vec<u8>, SharedAccessDenial> {
        if self.is_expired_at(current_unix_secs()?) {
            return Err(SharedAccessDenial::AuthenticationRefused(
                AuthenticationRefusalReason::TokenExpired,
            ));
        }
        let prepared = self.prepare_poll(app)?;
        app.with_application_advancement(request_scope, |phase| {
            self.poll_in_phase(app, &phase, (external, request_scope, limits), prepared)
        })
        .map_err(|_| {
            SharedAccessDenial::BudgetRefused(BudgetRefusalReason::ExecutionResourceDenied)
        })?
    }

    fn prepare_poll(
        &self,
        app: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    ) -> Result<PreparedWatchPoll<Schema, Program>, SharedAccessDenial> {
        let binding = app
            .installed_schema()
            .installed_query_binding::<IntentBinding<Schema, Program>>()
            .map_err(|_| SharedAccessDenial::HostUnavailable)?;
        let installed_cap = app
            .installed_schema()
            .capability(Program::watch_capability(), Program::watch_operation())
            .map_err(|_| SharedAccessDenial::HostUnavailable)?;
        let watch_input = Program::build_watch_capability_input(&self.scope);
        let scope_binding =
            Program::build_query_intent(&self.scope, &self.query_payload)?.into_scope();
        Ok(PreparedWatchPoll {
            binding,
            installed_cap,
            watch_input,
            scope_binding,
        })
    }

    fn poll_in_phase(
        &mut self,
        app: &'home WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        phase: &WorthQueryAdvancementPhase<'_>,
        caller: (
            &WorthQueryAuthenticatedExternalPrincipal<Schema>,
            &WorthQueryRequestScope,
            RemoteServerLimits,
        ),
        prepared: PreparedWatchPoll<Schema, Program>,
    ) -> Result<Vec<u8>, SharedAccessDenial> {
        let (external, request_scope, limits) = caller;
        let selected = app
            .on_branch(app.current_world())
            .select(phase)
            .map_err(|_| SharedAccessDenial::HostUnavailable)?;
        let principal = resolve_principal::<Schema, Program>(
            &selected,
            phase,
            (&prepared.binding, external, request_scope),
        )?;
        let current_scope = resolve_scope_entity::<Schema, Program>(
            &selected,
            phase,
            (&principal, request_scope),
            prepared.scope_binding,
        )?;
        if current_scope.entity_id() != self.bound_scope.entity_id() {
            return Err(SharedAccessDenial::AuthorizationRefused(
                AuthorizationRefusalReason::StaleScope,
            ));
        }
        let watch_admitted = selected
            .admit_capability_access(
                phase,
                &principal,
                &prepared.installed_cap,
                prepared.watch_input,
                request_scope,
            )
            .map_err(|_| {
                SharedAccessDenial::AuthorizationRefused(
                    AuthorizationRefusalReason::WatchCapabilityRevoked,
                )
            })?;
        verify_admitted_scope_entity(
            &watch_admitted,
            &self.bound_scope,
            AuthorizationRefusalReason::StaleScope,
        )?;
        let drain_args = (
            request_scope,
            limits.live_buffer_capacity().saturating_add(1),
            limits.max_frame_bytes(),
        );
        drain_lease_into_payload::<Schema, Program>(&mut self.lease, phase, &principal, drain_args)
    }
}

struct PreparedWatchPoll<Schema, Program>
where
    Schema: ApplicationSchema + 'static,
    Program: WorthQuerySharedAccessProgram<Schema>,
{
    binding: WorthQueryInstalledApplicationQueryBinding<Schema, IntentBinding<Schema, Program>>,
    installed_cap: WorthQueryInstalledApplicationCapability<
        Schema,
        Program::WatchCapability,
        Program::WatchOperation,
        Program::WatchCapabilityInput,
    >,
    watch_input: Program::WatchCapabilityInput,
    scope_binding:
        <IntentBinding<Schema, Program> as ApplicationQueryBinding<Schema>>::ScopeBinding,
}
