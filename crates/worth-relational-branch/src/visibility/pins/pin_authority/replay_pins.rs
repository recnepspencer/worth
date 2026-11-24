//! Replay pin effects require custody of every selected resident mirror cell.
use crate::storage::overlay::SnapshotPartitionPins;
use crate::storage::residency::{Full, RelationalResidencyLease};
use crate::visibility::snapshot_states::SnapshotState;

/// Move-only retention custody. Its constructor demands the mirror before
/// pinning, and release retains both its pin set and counted resident edition.
#[derive(Debug)]
pub struct ResidentReplayPins {
    pub state: SnapshotState,
    pub mirror: std::collections::BTreeMap<
        crate::identity::data::PartitionId,
        RelationalResidencyLease<Full>,
    >,
}

impl ResidentReplayPins {
    pub fn state(&self) -> &SnapshotState {
        &self.state
    }

    pub fn visit_partitions(
        &self,
        mut visit: impl FnMut(&SnapshotPartitionPins, &RelationalResidencyLease<Full>),
    ) {
        for (id, pins) in &self.state.pinned_partitions {
            // A branch-only record can be absent from the legacy main mirror.
            // Present cells were demanded before this custody was constructed.
            if let Some(resident) = self.mirror.get(id) {
                visit(pins, resident);
            }
        }
    }
}
