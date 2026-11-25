//! Authenticated remote request dispatch over a single-owner Directory Home.

use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::time::Instant;

use worth_foundational::facade::{HomeIdentity, HomeIncarnation};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAdmissionClock, WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_declaration::facade::{
    application_query::{ApplicationQueryBinding, ApplicationQueryScopeResolution},
    application_schema::{
        ApplicationOperationMarkerIdentity, ApplicationSchemaComposition,
        ApplicationStructuredValueBinding,
    },
    authentication::WorthQueryExternalPrincipalIdentity,
};
use worth_query_execution::facade::primary_graph::WorthQueryApplicationProjection;
use worth_query_publication::facade::application_entry::{
    WorthQueryApplicationRequestExt, WorthQueryApplicationRequestQueryDenial,
    WorthQueryApplicationRequestQueryDenialKind,
};

use crate::application_home::WorthQueryApplicationHome;

use super::auth::{
    authenticate_remote_token, current_unix_secs, RemoteAuthConfig, RemoteAuthTarget,
};
use super::binding::{RemoteBinaryCodec, WorthQuerySharedAccessProgram};
use super::capped_writer::{
    decode_scope_request, decode_watch_id_request, write_encoded_rows, RemoteByteWriter,
};
use super::denial::{
    AuthorizationRefusalReason, BudgetRefusalReason, ProtocolRefusalReason, SharedAccessDenial,
    WatchRefusalReason,
};
use super::framing::FrameKind;
use super::scope_binding::clamped_query_row_limit;
use super::server_watch::{ActiveRemoteWatch, WatchOpenArgs};
use super::types::RemoteWatchId;
use super::watch_drain::{IntentBinding, QueryMarker, QueryPrincipalId, QueryResult};

pub(super) struct RemoteDispatcher<'a, 'home, Schema, Program>
where
    Schema: ApplicationSchemaComposition + 'static,
    Program: WorthQuerySharedAccessProgram<Schema>,
    QueryResult<Schema, Program>:
        WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Program>>,
{
    pub(super) home: &'home WorthQueryApplicationHome<Schema, Program>,
    pub(super) auth_config: &'a RemoteAuthConfig,
    pub(super) home_identity: HomeIdentity,
    pub(super) home_incarnation: HomeIncarnation,
    pub(super) watches: &'a mut BTreeMap<RemoteWatchId, ActiveRemoteWatch<'home, Schema, Program>>,
    pub(super) next_watch_id: &'a mut u64,
}

impl<'home, Schema, Program> RemoteDispatcher<'_, 'home, Schema, Program>
where
    Schema: ApplicationSchemaComposition + 'static,
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
    pub(super) fn dispatch(
        &mut self,
        conn_id: Option<u64>,
        kind: FrameKind,
        payload: &[u8],
    ) -> Result<(FrameKind, Vec<u8>), SharedAccessDenial> {
        reap_expired_watches(self.watches);
        match kind {
            FrameKind::QueryRequest => self.handle_query(payload),
            FrameKind::WatchOpenRequest => self.handle_watch_open(conn_id, payload),
            FrameKind::WatchPollRequest => self.handle_watch_poll(payload),
            FrameKind::WatchCloseRequest => self.handle_watch_close(payload),
            _ => Err(SharedAccessDenial::ProtocolRefused(
                ProtocolRefusalReason::UnknownMessageKind,
            )),
        }
    }

    pub(super) fn reclaim_connection(&mut self, conn_id: u64) {
        reclaim_connection_watches(self.watches, conn_id);
    }

    fn auth_target(&self) -> RemoteAuthTarget<'_> {
        RemoteAuthTarget {
            config: self.auth_config,
            home_identity: self.home_identity,
            incarnation: self.home_incarnation,
        }
    }

    fn request_scope(&self) -> (WorthQueryCancellationSource, WorthQueryRequestScope) {
        let cancellation = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(
            Some(Instant::now() + self.auth_config.limits().request_timeout()),
            cancellation.token(),
            WorthQueryAdmissionClock::system(),
        );
        (cancellation, scope)
    }

    fn handle_query(&self, payload: &[u8]) -> Result<(FrameKind, Vec<u8>), SharedAccessDenial> {
        let limits = self.auth_config.limits();
        let (token, scope_key, query_bytes) =
            decode_scope_request(payload, limits.max_frame_bytes())?;
        let (_cancel, request_scope) = self.request_scope();
        let app = self.home.runtime().runtime();
        let auth = authenticate_remote_token(
            app.installed_schema(),
            &self.auth_target(),
            &token,
            &request_scope,
        )?;
        let scope = Program::parse_scope_key(scope_key)?;
        let query_payload = Program::QueryPayload::decode_remote(query_bytes)?;
        let intent = Program::build_query_intent(&scope, &query_payload)?;
        let max_rows = clamped_query_row_limit::<Schema, Program>(app, limits)?;
        let published = app
            .request(&auth.external, &request_scope)
            .query(intent)
            .limit_results(max_rows)
            .execute_governed(
                Program::read_capability(),
                Program::read_operation(),
                Program::build_read_capability_input(&scope),
            )
            .map_err(map_query_denial)?;
        let mut writer = RemoteByteWriter::new(limits.max_frame_bytes());
        let rows = published.rows();
        write_encoded_rows(
            &mut writer,
            rows.len(),
            rows.iter().cloned().map(Program::project_row),
        )?;
        Ok((FrameKind::QueryResponse, writer.into_bytes()))
    }

    fn handle_watch_open(
        &mut self,
        conn_id: Option<u64>,
        payload: &[u8],
    ) -> Result<(FrameKind, Vec<u8>), SharedAccessDenial> {
        let limits = self.auth_config.limits();
        let (token, scope_key, query_bytes) =
            decode_scope_request(payload, limits.max_frame_bytes())?;
        let (_cancel, request_scope) = self.request_scope();
        let app = self.home.runtime().runtime();
        let auth = authenticate_remote_token(
            app.installed_schema(),
            &self.auth_target(),
            &token,
            &request_scope,
        )?;
        self.check_watch_capacity(auth.external.identity())?;
        let scope = Program::parse_scope_key(scope_key)?;
        let query_payload = Program::QueryPayload::decode_remote(query_bytes)?;
        let live_intent = Program::build_query_intent(&scope, &query_payload)?;
        let initial_intent = Program::build_query_intent(&scope, &query_payload)?;
        let watch_id = RemoteWatchId::new(self.home_incarnation.get(), *self.next_watch_id);
        let (active, response_payload) = ActiveRemoteWatch::open(
            app,
            &auth.external,
            &request_scope,
            WatchOpenArgs {
                watch_id,
                connection_id: conn_id,
                token_expires_at_unix_secs: auth.expires_at_unix_secs,
                scope,
                query_payload,
                live_intent,
                initial_intent,
                limits,
                _schema: PhantomData,
            },
        )?;
        *self.next_watch_id = self.next_watch_id.saturating_add(1);
        self.watches.insert(watch_id, active);
        Ok((FrameKind::WatchOpenResponse, response_payload))
    }

    fn handle_watch_poll(
        &mut self,
        payload: &[u8],
    ) -> Result<(FrameKind, Vec<u8>), SharedAccessDenial> {
        let limits = self.auth_config.limits();
        let (token, watch_id) = decode_watch_id_request(payload)?;
        let (_cancel, request_scope) = self.request_scope();
        let app = self.home.runtime().runtime();
        let auth = authenticate_remote_token(
            app.installed_schema(),
            &self.auth_target(),
            &token,
            &request_scope,
        )?;
        let mut watch = self.take_owned_watch(watch_id, auth.external.identity())?;
        watch.token_expires_at_unix_secs = auth.expires_at_unix_secs;
        match watch.poll(app, &auth.external, &request_scope, limits) {
            Ok(response_payload) => {
                self.watches.insert(watch_id, watch);
                Ok((FrameKind::WatchPollResponse, response_payload))
            }
            Err(denial) => {
                watch.close();
                Err(denial)
            }
        }
    }

    fn handle_watch_close(
        &mut self,
        payload: &[u8],
    ) -> Result<(FrameKind, Vec<u8>), SharedAccessDenial> {
        let (token, watch_id) = decode_watch_id_request(payload)?;
        let (_cancel, request_scope) = self.request_scope();
        let app = self.home.runtime().runtime();
        let auth = authenticate_remote_token(
            app.installed_schema(),
            &self.auth_target(),
            &token,
            &request_scope,
        )?;
        let watch = self.take_owned_watch(watch_id, auth.external.identity())?;
        watch.close();
        Ok((FrameKind::WatchCloseResponse, Vec::new()))
    }

    fn take_owned_watch(
        &mut self,
        watch_id: RemoteWatchId,
        principal: &WorthQueryExternalPrincipalIdentity,
    ) -> Result<ActiveRemoteWatch<'home, Schema, Program>, SharedAccessDenial> {
        if watch_id.incarnation() != self.home_incarnation.get() {
            return Err(SharedAccessDenial::WatchRefused(
                WatchRefusalReason::WatchNotFound,
            ));
        }
        let Some(watch) = self.watches.remove(&watch_id) else {
            return Err(SharedAccessDenial::WatchRefused(
                WatchRefusalReason::WatchNotFound,
            ));
        };
        if &watch.owner_principal != principal {
            self.watches.insert(watch_id, watch);
            return Err(SharedAccessDenial::WatchRefused(
                WatchRefusalReason::WatchNotFound,
            ));
        }
        Ok(watch)
    }

    fn check_watch_capacity(
        &self,
        principal: &WorthQueryExternalPrincipalIdentity,
    ) -> Result<(), SharedAccessDenial> {
        let limits = self.auth_config.limits();
        let active_total = u32::try_from(self.watches.len()).unwrap_or(u32::MAX);
        if active_total >= limits.max_active_watches() {
            return Err(SharedAccessDenial::BudgetRefused(
                BudgetRefusalReason::WatchCapacityExhausted {
                    active: active_total,
                    maximum: limits.max_active_watches(),
                },
            ));
        }
        let per_principal = u32::try_from(
            self.watches
                .values()
                .filter(|watch| &watch.owner_principal == principal)
                .count(),
        )
        .unwrap_or(u32::MAX);
        if per_principal >= limits.max_watches_per_principal() {
            return Err(SharedAccessDenial::BudgetRefused(
                BudgetRefusalReason::WatchCapacityExhausted {
                    active: per_principal,
                    maximum: limits.max_watches_per_principal(),
                },
            ));
        }
        Ok(())
    }
}

pub(super) fn reap_expired_watches<Schema, Program>(
    watches: &mut BTreeMap<RemoteWatchId, ActiveRemoteWatch<'_, Schema, Program>>,
) -> usize
where
    Schema: ApplicationSchemaComposition + 'static,
    Program: WorthQuerySharedAccessProgram<Schema>,
    QueryResult<Schema, Program>:
        WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Program>>,
{
    let Ok(now_secs) = current_unix_secs() else {
        return 0;
    };
    let expired: Vec<RemoteWatchId> = watches
        .iter()
        .filter(|(_, watch)| watch.is_expired_at(now_secs))
        .map(|(&id, _)| id)
        .collect();
    let mut reclaimed = 0_usize;
    for id in expired {
        if let Some(watch) = watches.remove(&id) {
            watch.close();
            reclaimed += 1;
        }
    }
    reclaimed
}

pub(super) fn reclaim_connection_watches<Schema, Program>(
    watches: &mut BTreeMap<RemoteWatchId, ActiveRemoteWatch<'_, Schema, Program>>,
    conn_id: u64,
) where
    Schema: ApplicationSchemaComposition + 'static,
    Program: WorthQuerySharedAccessProgram<Schema>,
    QueryResult<Schema, Program>:
        WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Program>>,
{
    let owned: Vec<RemoteWatchId> = watches
        .iter()
        .filter(|(_, watch)| watch.connection_id == Some(conn_id))
        .map(|(&id, _)| id)
        .collect();
    for id in owned {
        if let Some(watch) = watches.remove(&id) {
            watch.close();
        }
    }
}

pub(super) fn map_query_denial(
    denial: WorthQueryApplicationRequestQueryDenial,
) -> SharedAccessDenial {
    match denial.kind() {
        WorthQueryApplicationRequestQueryDenialKind::CapabilityAdmission => {
            SharedAccessDenial::AuthorizationRefused(AuthorizationRefusalReason::CapabilityDenied)
        }
        WorthQueryApplicationRequestQueryDenialKind::PrincipalResolution => {
            SharedAccessDenial::AuthorizationRefused(
                AuthorizationRefusalReason::PrincipalResolutionDenied,
            )
        }
        WorthQueryApplicationRequestQueryDenialKind::ScopeResolution => {
            SharedAccessDenial::AuthorizationRefused(AuthorizationRefusalReason::ScopeNotFound)
        }
        WorthQueryApplicationRequestQueryDenialKind::Limit => {
            SharedAccessDenial::BudgetRefused(BudgetRefusalReason::RequestLimitExceeded)
        }
        WorthQueryApplicationRequestQueryDenialKind::ExecutionRequest(_) => {
            SharedAccessDenial::BudgetRefused(BudgetRefusalReason::ExecutionResourceDenied)
        }
        WorthQueryApplicationRequestQueryDenialKind::InputPreparation => {
            SharedAccessDenial::ProtocolRefused(ProtocolRefusalReason::MalformedPayload)
        }
        WorthQueryApplicationRequestQueryDenialKind::Admission
        | WorthQueryApplicationRequestQueryDenialKind::Execution
        | WorthQueryApplicationRequestQueryDenialKind::HistoryOutput
        | WorthQueryApplicationRequestQueryDenialKind::BindingInstallation
        | WorthQueryApplicationRequestQueryDenialKind::ProductSelection
        | WorthQueryApplicationRequestQueryDenialKind::OutputSettlement
        | WorthQueryApplicationRequestQueryDenialKind::ManagedIndex
        | WorthQueryApplicationRequestQueryDenialKind::ContinuationExecution
        | WorthQueryApplicationRequestQueryDenialKind::RequestMode
        | WorthQueryApplicationRequestQueryDenialKind::CapabilityInstallation => {
            SharedAccessDenial::HostUnavailable
        }
    }
}
