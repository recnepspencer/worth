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
};
use worth_query_execution::facade::primary_graph::WorthQueryApplicationProjection;

use crate::application_home::WorthQueryApplicationHome;

use super::auth::RemoteAuthConfig;
use super::binding::WorthQuerySharedAccessProgram;
use super::denial::{BudgetRefusalReason, SharedAccessDenial};
use super::framing::{decode_frame_slice, encode_frame, encode_refusal_payload, FrameKind};
use super::server_dispatch::RemoteDispatcher;
use super::server_transport::RemoteTcpPool;
use super::server_watch::ActiveRemoteWatch;
use super::types::RemoteWatchId;
use super::watch_drain::{IntentBinding, QueryMarker, QueryPrincipalId, QueryResult};

/// Single-owner authenticated remote server bound exclusively to one Directory Home.
///
/// Never spawns background threads or opens a second Directory Home. All live
/// governed watches stay on the owner's thread and are closed deterministically
/// when `close` or `Drop` runs.
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

    pub fn active_watch_count(&self) -> usize {
        self.watches.len()
    }

    /// Processes one complete framed binary request and returns the framed binary response.
    pub fn handle_frame(&mut self, request_frame: &[u8]) -> Vec<u8> {
        let max_bytes = self.auth_config.limits().max_frame_bytes();
        let outcome = decode_frame_slice(request_frame, max_bytes)
            .and_then(|(kind, payload)| self.dispatch_payload(kind, payload));
        self.encode_outcome(outcome)
    }

    /// Accepts ready connections and services all immediately readable framed requests.
    pub fn serve_pending(&mut self) -> Result<usize, SharedAccessDenial> {
        let limits = self.auth_config.limits();
        let mut dispatcher = RemoteDispatcher {
            home: self.home,
            auth_config: &self.auth_config,
            home_identity: self.home_identity,
            home_incarnation: self.home_incarnation,
            watches: &mut self.watches,
            next_watch_id: &mut self.next_watch_id,
        };
        self.tcp
            .serve_pending(limits, |kind, payload| dispatcher.dispatch(kind, payload))
    }

    /// Polls until at least one request is serviced or `timeout` elapses.
    pub fn serve_once(&mut self, timeout: Duration) -> Result<bool, SharedAccessDenial> {
        let deadline = Instant::now() + timeout;
        loop {
            if self.serve_pending()? > 0 {
                return Ok(true);
            }
            if Instant::now() >= deadline {
                return Ok(false);
            }
            std::thread::sleep(Duration::from_millis(1));
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
        dispatcher.dispatch(kind, payload)
    }

    fn encode_outcome(&self, outcome: Result<(FrameKind, Vec<u8>), SharedAccessDenial>) -> Vec<u8> {
        let max_bytes = self.auth_config.limits().max_frame_bytes();
        match outcome {
            Ok((kind, payload)) => match encode_frame(kind, &payload, max_bytes) {
                Ok(frame) => frame,
                Err(_) => {
                    let refusal =
                        SharedAccessDenial::BudgetRefused(BudgetRefusalReason::ResponseTooLarge {
                            bytes: u32::try_from(payload.len()).unwrap_or(u32::MAX),
                            maximum: max_bytes,
                        });
                    encode_frame(
                        FrameKind::RefusalResponse,
                        &encode_refusal_payload(&refusal),
                        max_bytes,
                    )
                    .expect("refusal frame fits within configured bound")
                }
            },
            Err(denial) => encode_frame(
                FrameKind::RefusalResponse,
                &encode_refusal_payload(&denial),
                max_bytes,
            )
            .expect("refusal frame fits within configured bound"),
        }
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
