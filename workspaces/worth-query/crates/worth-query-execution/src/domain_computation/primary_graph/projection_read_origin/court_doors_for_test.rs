//! Sealed test doors over this module's private members.
use super::*;

impl ProjectionReadOrigin {
    #[doc(hidden)]
    pub fn committed_snapshot_for_test(&self) -> Option<&SnapshotHandle> {
        Self::committed_snapshot(self)
    }
}
