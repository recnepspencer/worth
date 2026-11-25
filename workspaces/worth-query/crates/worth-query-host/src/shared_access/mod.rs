//! Single-owner Directory Home shared scope access and authenticated remote transport.
//!
//! Capability grants and revocations are committed directly by `worth-query-host`
//! through the owner's local `WorthQueryApplicationHome` into the Home's authoritative
//! WORTH graph, requiring a genuine published commit receipt with non-zero record
//! changes. Direct TCP transport is strictly loopback-only (`127.0.0.1` / `::1`);
//! cross-host use must tunnel over an authenticated encrypted transport or drive
//! `WorthQueryRemoteServer::handle_frame` over an encrypted channel.

mod auth;
mod binding;
mod capped_writer;
mod client;
mod denial;
mod framing;
mod hmac;
mod home_ops;
mod scope_binding;
mod server;
mod server_dispatch;
mod server_transport;
mod server_watch;
mod types;
mod watch_drain;
mod watch_open;

pub use auth::{RemoteAuthConfig, RemoteAuthKey, RemoteAuthToken, RemoteTokenIssueRequest};
pub use binding::{RemoteBinaryCodec, RemoteWatchContext, WorthQuerySharedAccessProgram};
pub use capped_writer::RemoteByteWriter;
pub use client::{WorthQueryRemoteClient, WorthQueryRemoteWatch};
pub use denial::{
    AuthenticationRefusalReason, AuthorizationRefusalReason, BudgetRefusalReason,
    ProtocolRefusalReason, SharedAccessDenial, WatchRefusalReason,
};
pub use server::WorthQueryRemoteServer;
pub use types::{
    RemoteClientLimits, RemoteServerLimits, RemoteWatchId, RemoteWatchPollOutcome, ScopeAccess,
    ScopeAccessGrant, ScopeGrantId, MIN_MAX_FRAME_BYTES,
};
