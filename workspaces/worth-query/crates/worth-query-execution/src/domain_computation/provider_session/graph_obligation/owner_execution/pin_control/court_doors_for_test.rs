//! Sealed test doors over this module's private members.
use super::*;

impl PinControlGuardForTest {
    #[doc(hidden)]
    pub fn unavailable_snapshot_for_test(skip: usize, snapshot: SnapshotHandle) -> Self {
        Self::unavailable_snapshot(skip, snapshot)
    }
}
