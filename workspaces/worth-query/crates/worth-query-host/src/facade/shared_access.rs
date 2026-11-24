//! Public facade for single-owner Directory Home shared scope access and
//! authenticated remote read/watch delegation.
//!
//! Only the owner's `WorthQueryApplicationHome` may issue or revoke durable
//! scope-access grants and host remote requests. `ScopeAccessGrant` cannot be
//! forged by callers:
//!
//! ```compile_fail,E0451
//! use worth_query_host::facade::shared_access::ScopeAccessGrant;
//! fn forge_grant() -> ScopeAccessGrant {
//!     ScopeAccessGrant {
//!         id: todo!(),
//!         scope_key: String::new(),
//!         principal: todo!(),
//!         access: todo!(),
//!         home_identity: None,
//!         home_incarnation: None,
//!     }
//! }
//! ```
//!
//! Public value types preserve `Send + Sync + UnwindSafe + RefUnwindSafe`:
//!
//! ```
//! use std::panic::{RefUnwindSafe, UnwindSafe};
//! use worth_query_host::facade::shared_access::{
//!     RemoteAuthConfig, RemoteAuthKey, RemoteAuthToken, RemoteClientLimits,
//!     RemoteServerLimits, RemoteWatchId, RemoteWatchPollOutcome, ScopeAccess,
//!     ScopeAccessGrant, ScopeGrantId, SharedAccessDenial,
//! };
//! fn assert_value_traits<T: Send + Sync + UnwindSafe + RefUnwindSafe>() {}
//! assert_value_traits::<ScopeAccess>();
//! assert_value_traits::<ScopeGrantId>();
//! assert_value_traits::<ScopeAccessGrant>();
//! assert_value_traits::<RemoteWatchId>();
//! assert_value_traits::<RemoteServerLimits>();
//! assert_value_traits::<RemoteClientLimits>();
//! assert_value_traits::<RemoteAuthKey>();
//! assert_value_traits::<RemoteAuthConfig>();
//! assert_value_traits::<RemoteAuthToken>();
//! assert_value_traits::<SharedAccessDenial>();
//! assert_value_traits::<RemoteWatchPollOutcome<u64>>();
//! ```

pub use crate::shared_access::{
    AuthenticationRefusalReason, AuthorizationRefusalReason, BudgetRefusalReason,
    ProtocolRefusalReason, RemoteAuthConfig, RemoteAuthKey, RemoteAuthToken, RemoteBinaryCodec,
    RemoteClientLimits, RemoteQueryContext, RemoteServerLimits, RemoteTokenIssueRequest,
    RemoteWatchContext, RemoteWatchId, RemoteWatchPollOutcome, ScopeAccess, ScopeAccessGrant,
    ScopeGrantId, SharedAccessDenial, WatchRefusalReason, WorthQueryRemoteClient,
    WorthQueryRemoteServer, WorthQueryRemoteWatch, WorthQuerySharedAccessProgram,
};
