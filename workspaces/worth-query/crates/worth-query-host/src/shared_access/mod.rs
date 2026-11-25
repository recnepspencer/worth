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
mod field_codec;
mod framing;
mod grant_vocabulary;
mod hmac;
mod home_ops;
mod program_macro;
mod scope_access_action_macro;
mod scope_access_contract_macro;
mod scope_access_macro;
mod scope_access_schema_macro;
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
pub use field_codec::{RemoteBinaryField, RemoteByteReader};
pub use grant_vocabulary::{
    scope_grant_value, ScopeAccessBinding, ScopeGrantAction, ScopeGrantActionBinding,
    ScopeGrantDisclosure, ScopeGrantDisclosureBinding, ScopeGrantPurpose, ScopeGrantPurposeBinding,
    ScopeGrantValue,
};
pub use program_macro::{
    scope_grant_idempotency_key, scope_revoke_idempotency_key, WORTH_QUERY_SHARED_GRANT_KEY_TAG,
    WORTH_QUERY_SHARED_REVOKE_KEY_TAG, WORTH_QUERY_SHARED_SEQUENCE_MASK,
};
pub use scope_access_macro::WorthQueryScopeGrantRefusal;
pub use server::WorthQueryRemoteServer;
pub use types::{
    RemoteClientLimits, RemoteServerLimits, RemoteWatchId, RemoteWatchPollOutcome, ScopeAccess,
    ScopeAccessGrant, ScopeGrantId, MIN_MAX_FRAME_BYTES,
};
