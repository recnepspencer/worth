//! Named reconstruction-lane entry point for taking a working copy of
//! `LineageState`. Naming and group tables are shared copy-on-write via
//! [`LineageState::fork_detached`].

use super::LineageState;

impl LineageState {
    /// A reconstruction lane's working copy of the lineage state; the lane
    /// token keeps it off every ordinary open, commit and read path, and
    /// naming/group tables share copy-on-write (`O(resident)`).
    pub fn duplicate_for_reconstruction(
        &self,
        _lane: &crate::contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    ) -> Self {
        self.fork_detached()
    }
}
