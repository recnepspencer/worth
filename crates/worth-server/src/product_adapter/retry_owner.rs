//! Product response replay and native application custody have different owners.
use super::{
    WorthServerProductOperationAuthorityRequirement, WorthServerProductOperationDeclaration,
};

/// Selects who resolves repeated runtime-backed product commands.
/// This is declaration meaning, never a caller-controlled retry override.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum WorthServerProductRetryOwner {
    /// Response-receipt replay for eligible runtime-backed mutations only.
    /// Reads always re-enter the adapter even when a retry key is supplied.
    #[default]
    ServerReceipt,
    /// Always re-enter the application under fresh Server admission and readiness.
    /// The application must retain its genuine operation, enforce original intent
    /// and concurrency, and return its actual completion or pending posture.
    /// Server neither caches these responses nor mints `PreviouslyCommitted` receipts.
    /// This does not install custody, grant permission or provide crash durability.
    ApplicationCustody,
}

impl WorthServerProductOperationDeclaration {
    pub(super) fn server_replays_response(&self) -> bool {
        self.retry_owner() == WorthServerProductRetryOwner::ServerReceipt
            && matches!(
                self.authority_requirement(),
                WorthServerProductOperationAuthorityRequirement::DraftMutation { .. }
            )
    }
}
