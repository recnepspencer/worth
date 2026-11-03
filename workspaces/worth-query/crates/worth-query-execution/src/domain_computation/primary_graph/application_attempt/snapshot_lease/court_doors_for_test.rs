//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationSnapshotLease {
    #[doc(hidden)]
    pub fn handle_for_test(&self) -> &WorthQueryPrimaryGraphIntegrationHandle {
        Self::handle(self)
    }
}

impl WorthQueryApplicationSnapshotLease {
    #[doc(hidden)]
    pub fn snapshot_for_test(&self) -> &SnapshotHandle {
        Self::snapshot(self)
    }
}
