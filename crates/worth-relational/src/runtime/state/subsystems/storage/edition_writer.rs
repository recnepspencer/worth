use std::sync::{Arc, RwLockWriteGuard};

use crate::identity::data::PartitionId;
use crate::storage::overlay::PartitionState;

use super::edition_copy_lane::{PartitionCopyTally, PartitionEditionCopyLane};
use super::partition_edition::{PartitionEdition, PartitionMap};
use crate::runtime::state::subsystems::RuntimeInstrumentation;

pub(crate) enum TryPartitionMutationDenied<E> {
    Observed,
    SizeOverflow,
    Admission(E),
}

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
            .map(|partition| own_partition(copies, partition))
    }

    /// Borrow an unobserved spine and admit a necessary shallow partition owner
    /// before detaching it. The caller separately admits its selected value paths.
    pub(crate) fn try_partition_mut<E>(
        &mut self,
        partition_id: PartitionId,
        prepare: &mut impl FnMut(u64, u64) -> Result<(), E>,
    ) -> Result<Option<&mut PartitionState>, TryPartitionMutationDenied<E>> {
        let Self {
            edition, copies, ..
        } = self;
        let map = edition
            .unobserved_map_mut()
            .ok_or(TryPartitionMutationDenied::Observed)?;
        // The standard map exposes no search height. Its entry count bounds
        // comparisons without depending on private B-tree node geometry. This
        // conservative admission belongs only to explicit global maintenance.
        prepare(map.len() as u64, 0).map_err(TryPartitionMutationDenied::Admission)?;
        let Some(partition) = map.get_mut(&partition_id) else {
            return Ok(None);
        };
        if Arc::get_mut(partition).is_none() {
            let bytes = std::alloc::Layout::new::<[usize; 2]>()
                .extend(std::alloc::Layout::new::<PartitionState>())
                .map_err(|_| TryPartitionMutationDenied::SizeOverflow)?
                .0
                .pad_to_align()
                .size() as u64;
            prepare(1, bytes).map_err(TryPartitionMutationDenied::Admission)?;
        }
        Ok(Some(own_partition(copies, partition)))
    }

    /// Exclusive access to every partition, copying each one that an observer
    /// still holds. This is Theta(all slots in the substrate) in the worst
    /// case, which is why the copies it performs are counted individually
    /// rather than summarized as one event.
    pub(crate) fn partitions_mut(&mut self) -> impl Iterator<Item = &mut PartitionState> + use<'_> {
        let Self {
            edition, copies, ..
        } = self;
        let (map, spine_copied) = edition.map_mut();
        if spine_copied {
            copies.record_spine_copy();
        }
        map.values_mut()
            .map(move |partition| own_partition(copies, partition))
    }

    pub(crate) fn insert(&mut self, partition_id: PartitionId, partition: PartitionState) {
        self.map_mut().insert(partition_id, Arc::new(partition));
    }

    pub(crate) fn remove(&mut self, partition_id: PartitionId) -> Option<Arc<PartitionState>> {
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
    partition: &'partition mut Arc<PartitionState>,
) -> &'partition mut PartitionState {
    if Arc::get_mut(partition).is_none() {
        copies.record_partition_copy(
            partition.entity_arena.slot_count(),
            partition.relation_arena.slot_count(),
        );
    }
    Arc::make_mut(partition)
}
