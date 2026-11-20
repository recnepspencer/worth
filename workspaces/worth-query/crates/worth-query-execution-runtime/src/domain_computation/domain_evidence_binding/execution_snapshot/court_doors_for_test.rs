//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryBoundExecutionSnapshotIdentity {
    #[doc(hidden)]
    pub fn capture_for_test(identity: Arc<str>) -> Self {
        Self::capture(identity)
    }
}
