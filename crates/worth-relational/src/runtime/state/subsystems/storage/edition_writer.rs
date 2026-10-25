use crate::storage::residency::RelationalRegionRetention;
use std::sync::RwLockWriteGuard;

use crate::identity::data::PartitionId;
use crate::storage::overlay::PartitionState;

use super::edition_copy_lane::{PartitionCopyTally, PartitionEditionCopyLane};
use super::partition_edition::{PartitionEdition, PartitionMap};
use crate::runtime::state::subsystems::RuntimeInstrumentation;

/// Exclusive authority to install the next edition of the record substrate.
///
/// The guard deliberately exposes no `DerefMut`. Every mutable entry point can
/// copy, and a copy that reaches Theta(partitions) or Theta(slots) must not
/// hide behind something that reads like a field access.
///
/// Two distinct copies can happen here and both are charged. The map spine is
/// copied at most once per guard, when a reader edition of the same map is
/// still outstanding. Each partition reached through that spine is copied out
/// of structural sharing the first time it is mutated while an observer still
/// holds it -- that copy is Theta(the partition's slots) and is the larger of
/// the two whenever a partition holds more than a handful of records. When
/// nothing observes the substrate, mutation happens in place and neither copy
/// occurs.
pub(crate) struct PartitionEditionWriter<'subsystem> {
    edition: RwLockWriteGuard<'subsystem, PartitionEdition>,
    instrumentation: &'subsystem RuntimeInstrumentation,
    lane: PartitionEditionCopyLane,
    copies: PartitionCopyTally,
}

impl<'subsystem> PartitionEditionWriter<'subsystem> {
    pub(super) fn new(
        edition: RwLockWriteGuard<'subsystem, PartitionEdition>,
        instrumentation: &'subsystem RuntimeInstrumentation,
        lane: PartitionEditionCopyLane,
    ) -> Self {
        Self {
            edition,
            instrumentation,
            lane,
            copies: PartitionCopyTally::default(),
        }
    }

    /// Exclusive access to the whole map, copying the spine once if observed.
    ///
    /// This buys ownership of the pointers, not of what they point at: the
    /// copied spine holds the same partition arcs the outstanding reader does.
    /// It is deliberately private: a caller holding `&mut PartitionMap` could
    /// reach `Arc::make_mut` on a partition itself, and that copy would be the
    /// one cost on this guard nobody counted.
    fn map_mut(&mut self) -> &mut PartitionMap {
        let (map, copied) = self.edition.map_mut();
        if copied {
            self.copies.record_spine_copy();
        }
        map
    }

    /// Exclusive access to one partition's authoritative state, copying that
    /// partition out of structural sharing only when a reader still holds the
    /// previous edition of it.
    pub(crate) fn partition_mut(
        &mut self,
        partition_id: PartitionId,
    ) -> Option<&mut PartitionState> {
        let Self {
            edition, copies, ..
        } = self;
        let (map, spine_copied) = edition.map_mut();
        if spine_copied {
            copies.record_spine_copy();
        }
        map.get_mut(&partition_id)
            .and_then(|partition| own_partition(copies, partition))
    }

    /// Exclusive access to every partition, copying each one that an observer
    /// still holds. This is Theta(all slots in the substrate) in the worst
    /// case, which is why the copies it performs are counted individually
    /// rather than summarized as one event.
    pub(crate) fn resident_partitions_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut PartitionState> + use<'_> {
        let Self {
            edition, copies, ..
        } = self;
        let (map, spine_copied) = edition.map_mut();
        if spine_copied {
            copies.record_spine_copy();
        }
        map.values_mut()
            .filter_map(move |partition| own_partition(copies, partition))
    }

    pub(crate) fn insert(&mut self, partition_id: PartitionId, partition: PartitionState) {
        self.map_mut()
            .insert(partition_id, RelationalRegionRetention::resident(partition));
    }

    pub(crate) fn remove(
        &mut self,
        partition_id: PartitionId,
    ) -> Option<RelationalRegionRetention> {
        self.map_mut().remove(&partition_id)
    }

    pub(crate) fn install(&mut self, partitions: PartitionMap) {
        *self.edition = PartitionEdition::new(partitions);
    }
}

impl Drop for PartitionEditionWriter<'_> {
    /// Settle once for the whole guard. Instrumentation is taken under a lock,
    /// so charging per copied partition would put a lock acquisition on every
    /// partition of a substrate-wide pass.
    fn drop(&mut self) {
        self.lane.settle(self.instrumentation, self.copies);
    }
}

/// Take sole ownership of one partition, tallying the copy if the arc was
/// shared.
///
/// The sharing test happens before the copy rather than being inferred from it,
/// so a partition the guard already owns outright is never charged for work
/// nobody performed.
fn own_partition<'partition>(
    copies: &mut PartitionCopyTally,
    partition: &'partition mut RelationalRegionRetention,
) -> Option<&'partition mut PartitionState> {
    let copied = !partition.is_uniquely_owned();
    let partition = partition.make_mut()?;
    if copied {
        copies.record_partition_copy(
            partition.entity_arena.slot_count(),
            partition.relation_arena.slot_count(),
        );
    }
    Some(partition)
}
