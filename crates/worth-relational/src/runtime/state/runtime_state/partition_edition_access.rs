use crate::identity::data::PartitionId;
use crate::runtime::state::subsystems::{
    PartitionEdition, PartitionEditionCopyLane, PartitionEditionWriter, RuntimeSubsystem,
    StorageSubsystem,
};
use crate::storage::overlay::PartitionState;

use super::RelationalRuntimeState;

impl RelationalRuntimeState {
    /// Pin one structurally shared edition of the record substrate.
    ///
    /// The storage lock is released before this returns, so the edition may be
    /// held across record materialization, re-entrant substrate lookups,
    /// derived projection work, callbacks, branch waiting, durability I/O, and
    /// test pauses. A read-only traversal must pin exactly one edition and
    /// resolve every lookup against it; a read-modify-read loop must drop its
    /// edition before each write, or the write copies what the loop observes.
    pub(crate) fn acquire_partition_edition(&self) -> PartitionEdition {
        self.services
            .instrumentation
            .count(|counters| counters.partition_editions_acquired += 1);
        self.partitions.acquire()
    }

    /// Exclusive authority to install the next ordinary edition.
    ///
    /// Ordinary mutation copies the map spine only while a reader edition is
    /// outstanding, and copies an individual partition only when a reader still
    /// observes that partition's previous state.
    pub(crate) fn edit_partitions(&self) -> PartitionEditionWriter<'_> {
        self.partitions.edit(
            &self.services.instrumentation,
            PartitionEditionCopyLane::Ordinary,
        )
    }

    /// Exclusive authority to install a whole rebuilt substrate.
    ///
    /// Recovery, checkpoint restore, and fork materialization rebuild state by
    /// contract; their copies are reconstructive and must never be charged to
    /// the ordinary lane.
    pub(crate) fn rebuild_partitions(&self) -> PartitionEditionWriter<'_> {
        self.partitions.edit(
            &self.services.instrumentation,
            PartitionEditionCopyLane::Reconstructive,
        )
    }

    /// Share a fork's immutable substrate; each demanded writer pays for its
    /// own copy and pin-counter detachment at the editing boundary.
    pub(crate) fn fork_partitions(&self) -> StorageSubsystem {
        RuntimeSubsystem::fork(&self.partitions)
    }

    pub(crate) fn install_rebuilt_partitions(
        &self,
        partitions: impl IntoIterator<Item = (PartitionId, PartitionState)>,
    ) {
        self.partitions.install_owned(
            &self.services.instrumentation,
            PartitionEditionCopyLane::Reconstructive,
            partitions,
        );
    }
}
