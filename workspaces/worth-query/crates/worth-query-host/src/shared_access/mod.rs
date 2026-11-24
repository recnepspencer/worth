//! Single-owner Directory Home shared scope access and authenticated remote transport.
//!
//! Capability grants and revocations are committed through the owner's local
//! `WorthQueryApplicationHome` into the Home's authoritative WORTH graph.
//! Remote collaborators connect through an authenticated, bounded binary-framed
//! endpoint whose read and watch executions go strictly through WORTH's own
//! principal resolution, capability admission, and governed live-lease renewal.

mod auth;
mod binding;
mod client;
mod denial;
mod framing;
mod hmac;
mod home_ops;
mod server;
mod server_dispatch;
mod server_transport;
mod server_watch;
mod types;
mod watch_drain;

pub use auth::{RemoteAuthConfig, RemoteAuthKey, RemoteAuthToken, RemoteTokenIssueRequest};
pub use binding::{
    RemoteBinaryCodec, RemoteQueryContext, RemoteWatchContext, WorthQuerySharedAccessProgram,
};
pub use client::{WorthQueryRemoteClient, WorthQueryRemoteWatch};
pub use denial::{
    AuthenticationRefusalReason, AuthorizationRefusalReason, BudgetRefusalReason,
    ProtocolRefusalReason, SharedAccessDenial, WatchRefusalReason,
};
pub use server::WorthQueryRemoteServer;
pub use types::{
    RemoteClientLimits, RemoteServerLimits, RemoteWatchId, RemoteWatchPollOutcome, ScopeAccess,
    ScopeAccessGrant, ScopeGrantId,
};
