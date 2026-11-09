use crate::runtime::state::subsystems::RuntimeInstrumentation;

/// The lane an observed partition payload copy is charged to.
///
/// Copying an observed partition is legitimate work in more than one lane, and
/// the lanes must never share a counter: an ordinary-lane zero is only
/// interpretable when reconstructive copies are accounted somewhere else
/// instead of being laundered into silence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PartitionEditionCopyLane {
    /// Ordinary reads, writes, and settlement. A copy here means a write
    /// observed an outstanding reader edition.
    Ordinary,
    /// Runtime forks, checkpoint capture, and recovery rebuilds, which
    /// materialize whole state by contract.
    Reconstructive,
}

/// Copy-on-write actually performed under one writer guard.
///
/// Persistent map roots share their spine; only changed paths are copied.
/// This tally records actual partition payload detachment, separately by lane.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PartitionCopyTally {
    partitions: usize,
    entity_slots: usize,
    relation_slots: usize,
}

impl PartitionCopyTally {
    /// Record one partition state lifted out of structural sharing, together
    /// with the slots that copy carried.
    pub(super) fn record_partition_copy(&mut self, entity_slots: usize, relation_slots: usize) {
        self.partitions += 1;
        self.entity_slots += entity_slots;
        self.relation_slots += relation_slots;
    }

    const fn is_empty(&self) -> bool {
        self.partitions == 0
    }
}

impl PartitionEditionCopyLane {
    /// Charge everything one writer guard copied, in a single settlement.
    ///
    /// Instrumentation is taken under a lock, so a guard that touched a
    /// thousand partitions must still settle once rather than a thousand times.
    pub(super) fn settle(
        self,
        instrumentation: &RuntimeInstrumentation,
        tally: PartitionCopyTally,
    ) {
        if tally.is_empty() {
            return;
        }
        instrumentation.count(|counters| match self {
            Self::Ordinary => {
                counters.ordinary_partitions_copied_on_write += tally.partitions;
                counters.ordinary_partition_slots_copied_on_write +=
                    tally.entity_slots + tally.relation_slots;
            }
            Self::Reconstructive => {
                counters.reconstructive_partitions_materialized += tally.partitions;
            }
        });
    }
}
