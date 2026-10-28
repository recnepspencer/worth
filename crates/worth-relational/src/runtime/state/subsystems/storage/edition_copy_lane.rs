use crate::runtime::state::subsystems::RuntimeInstrumentation;

/// The lane a partition-map copy is charged to.
///
/// Copying the authoritative map is legitimate work in more than one lane, and
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
/// The spine and the partitions behind it are separate costs and are tallied
/// separately. A spine copy is Theta(partitions) pointer work; copying one
/// partition out of structural sharing is Theta(that partition's slots), which
/// is the larger number whenever a partition holds more than a handful of
/// records. Charging only the spine would report the cheaper half of the work
/// and call the expensive half free.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PartitionCopyTally {
    spine: bool,
    partitions: usize,
    entity_slots: usize,
    relation_slots: usize,
}

impl PartitionCopyTally {
    /// Record that the map spine itself was copied. Idempotent: once the guard
    /// owns the spine outright, later mutations reuse it.
    pub(super) fn record_spine_copy(&mut self) {
        self.spine = true;
    }

    /// Record one partition state lifted out of structural sharing, together
    /// with the slots that copy carried.
    pub(super) fn record_partition_copy(&mut self, entity_slots: usize, relation_slots: usize) {
        self.partitions += 1;
        self.entity_slots += entity_slots;
        self.relation_slots += relation_slots;
    }

    const fn is_empty(&self) -> bool {
        !self.spine && self.partitions == 0
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
                if tally.spine {
                    counters.full_state_clones += 1;
                }
                counters.ordinary_partitions_copied_on_write += tally.partitions;
                counters.ordinary_partition_slots_copied_on_write +=
                    tally.entity_slots + tally.relation_slots;
            }
            Self::Reconstructive => {
                if tally.spine {
                    counters.reconstructive_state_clones += 1;
                }
                counters.reconstructive_partitions_materialized += tally.partitions;
            }
        });
    }
}
