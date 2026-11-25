//! Single-owner authenticated loopback TCP and framed binary remote host endpoint.

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use worth_foundational::facade::{HomeIdentity, HomeIncarnation};
use worth_query_declaration::facade::{
    application_query::{ApplicationQueryBinding, ApplicationQueryScopeResolution},
    application_schema::{
        ApplicationOperationMarkerIdentity, ApplicationSchemaComposition,
        ApplicationStructuredValueBinding,
    },
    authentication::WorthQueryExternalPrincipalIdentity,
};
use worth_query_execution::facade::primary_graph::WorthQueryApplicationProjection;

use crate::application_home::WorthQueryApplicationHome;

use super::auth::RemoteAuthConfig;
use super::binding::WorthQuerySharedAccessProgram;
use super::denial::SharedAccessDenial;
use super::framing::{decode_frame_slice, FrameKind};
use super::server_dispatch::{reap_expired_watches, RemoteDispatcher};
use super::server_transport::{encode_dispatch_outcome, RemoteTcpPool};
use super::server_watch::ActiveRemoteWatch;
use super::types::RemoteWatchId;
use super::watch_drain::{IntentBinding, QueryMarker, QueryPrincipalId, QueryResult};

/// Single-owner authenticated remote server bound exclusively to one Directory Home.
///
/// Never spawns background threads or opens a second Directory Home. Direct TCP
/// binding is restricted strictly to loopback addresses (`127.0.0.1` / `::1`);
/// cross-host deployments must tunnel loopback over an authenticated encrypted
/// transport (TLS/mTLS, Tailscale, or SSH) or drive `handle_frame` over an
/// encrypted channel. All live governed watches stay on the owner's thread and
/// are reclaimed on disconnect, token expiry, `close`, or `Drop`.
pub struct WorthQueryRemoteServer<'home, Schema, Program>
where
    Schema: ApplicationSchemaComposition + 'static,
    Program: WorthQuerySharedAccessProgram<Schema>,
    QueryResult<Schema, Program>:
        WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Program>>,
{
    home: &'home WorthQueryApplicationHome<Schema, Program>,
    tcp: RemoteTcpPool,
    home_identity: HomeIdentity,
    home_incarnation: HomeIncarnation,
    auth_config: RemoteAuthConfig,
    watches: BTreeMap<RemoteWatchId, ActiveRemoteWatch<'home, Schema, Program>>,
    next_watch_id: u64,
}

impl<'home, Schema, Program> WorthQueryRemoteServer<'home, Schema, Program>
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
    pub(super) fn bind(
        home: &'home WorthQueryApplicationHome<Schema, Program>,
        bind_addr: SocketAddr,
        auth_config: RemoteAuthConfig,
    ) -> Result<Self, SharedAccessDenial> {
        let home_identity = home
            .home_identity()
            .ok_or(SharedAccessDenial::DirectoryHomeRequired)?;
        let home_incarnation = home
            .home_incarnation()
            .ok_or(SharedAccessDenial::DirectoryHomeRequired)?;
        let tcp = RemoteTcpPool::bind(bind_addr)?;
        Ok(Self {
            home,
            tcp,
            home_identity,
            home_incarnation,
            auth_config,
            watches: BTreeMap::new(),
            next_watch_id: 1,
        })
    }

    pub const fn local_addr(&self) -> SocketAddr {
        self.tcp.local_addr()
    }

    pub const fn home_identity(&self) -> HomeIdentity {
        self.home_identity
    }

    pub const fn home_incarnation(&self) -> HomeIncarnation {
        self.home_incarnation
    }

    pub fn active_connection_count(&self) -> usize {
        self.tcp.connection_count()
    }

    /// Read-only count of currently live server-owned watches.
    ///
    /// Never mutates or reaps watches; drive expiry via `serve_pending`,
    /// `serve_once`, or `reap_expired_watches`.
    pub fn active_watch_count(&self) -> usize {
        self.watches.len()
    }

    /// Read-only snapshot of live server-owned watches `(id, principal, scope)`.
    ///
    /// Never mutates or reaps watches; drive expiry via `serve_pending`,
    /// `serve_once`, or `reap_expired_watches`.
    pub fn active_watches(
        &self,
    ) -> Vec<(
        RemoteWatchId,
        WorthQueryExternalPrincipalIdentity,
        Program::ScopeId,
    )> {
        self.watches
            .iter()
            .map(|(&id, watch)| (id, watch.owner_principal.clone(), watch.scope.clone()))
            .collect()
    }

    /// Reclaims all live watches whose bearer token has expired and returns the count closed.
    pub fn reap_expired_watches(&mut self) -> usize {
        reap_expired_watches(&mut self.watches)
    }

    /// Processes one complete framed binary request and returns the framed binary response.
    pub fn handle_frame(&mut self, request_frame: &[u8]) -> Vec<u8> {
        let max_bytes = self.auth_config.limits().max_frame_bytes();
        let outcome = decode_frame_slice(request_frame, max_bytes)
            .and_then(|(kind, payload)| self.dispatch_payload(None, kind, payload));
        encode_dispatch_outcome(outcome, max_bytes)
    }

    /// Accepts bounded ready connections, reaps expired watches and stalled peers,
    /// and services immediately available framed requests without blocking.
    pub fn serve_pending(&mut self) -> Result<usize, SharedAccessDenial> {
        reap_expired_watches(&mut self.watches);
        let limits = self.auth_config.limits();
        let mut dispatcher = RemoteDispatcher {
            home: self.home,
            auth_config: &self.auth_config,
            home_identity: self.home_identity,
            home_incarnation: self.home_incarnation,
            watches: &mut self.watches,
            next_watch_id: &mut self.next_watch_id,
        };
        let (served, dropped) = self.tcp.serve_pending(limits, |conn_id, kind, payload| {
            dispatcher.dispatch(Some(conn_id), kind, payload)
        })?;
        for conn_id in dropped {
            dispatcher.reclaim_connection(conn_id);
        }
        Ok(served)
    }

    /// Polls until at least one request is serviced or `timeout` elapses, never blocking
    /// on partial/trickle frames past `timeout`.
    pub fn serve_once(&mut self, timeout: Duration) -> Result<bool, SharedAccessDenial> {
        let deadline = Instant::now() + timeout;
        loop {
            if self.serve_pending()? > 0 {
                return Ok(true);
            }
            let now = Instant::now();
            if now >= deadline {
                return Ok(false);
            }
            let remaining = deadline.saturating_duration_since(now);
            std::thread::sleep(Duration::from_millis(1).min(remaining));
        }
    }

    /// Closes the listener, drops connections, and cleanly releases all live watches.
    pub fn close(mut self) {
        self.tcp.clear();
        for (_, watch) in std::mem::take(&mut self.watches) {
            watch.close();
        }
    }

    fn dispatch_payload(
        &mut self,
        conn_id: Option<u64>,
        kind: FrameKind,
        payload: &[u8],
    ) -> Result<(FrameKind, Vec<u8>), SharedAccessDenial> {
        let mut dispatcher = RemoteDispatcher {
            home: self.home,
            auth_config: &self.auth_config,
            home_identity: self.home_identity,
            home_incarnation: self.home_incarnation,
            watches: &mut self.watches,
            next_watch_id: &mut self.next_watch_id,
        };
        dispatcher.dispatch(conn_id, kind, payload)
    }
}

impl<Schema, Program> Drop for WorthQueryRemoteServer<'_, Schema, Program>
where
    Schema: ApplicationSchemaComposition + 'static,
    Program: WorthQuerySharedAccessProgram<Schema>,
    QueryResult<Schema, Program>:
        WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Program>>,
{
    fn drop(&mut self) {
        self.tcp.clear();
        for (_, watch) in std::mem::take(&mut self.watches) {
            watch.close();
        }
    }
}
