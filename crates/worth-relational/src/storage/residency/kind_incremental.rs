//! Journal-scoped maintenance of admitted kind projections and their Merkle roots.
use super::kind_index::KindIndex;
use super::kind_projection::{ProjectionCheckpoint, ProjectionRetention};
use super::RelationalRegionKindCommitment;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::overlay::{PartitionMutationJournal, PartitionState};
use crate::storage::partition::AdjacencyKindBasis;
use crate::storage::substrate::maintenance::ReconstructionLane;
use crate::storage::substrate::StorageAllocationVisitor;

#[derive(Debug, Clone)]
pub(super) struct RegionKindIndexes {
    pub(super) head: KindIndex,
    pub(super) history: KindIndex,
    pub(super) supports: super::kind_history::HistorySupports,
}
impl RegionKindIndexes {
    pub(super) fn capture(
        lane: &ReconstructionLane<'_>,
        partition: &PartitionState,
    ) -> Result<Self, Cause> {
        Self::capture_controlled(lane, partition, &mut |_, _| Ok(()))
    }
    pub(super) fn capture_controlled(
        lane: &ReconstructionLane<'_>,
        partition: &PartitionState,
        checkpoint: &mut ProjectionCheckpoint<'_>,
    ) -> Result<Self, Cause> {
        Ok(Self {
            supports: super::kind_history::HistorySupports::capture(lane, partition, checkpoint)?,
            head: KindIndex::rebuild(
                lane,
                &super::kind_projection::capture_controlled(
                    lane,
                    partition,
                    AdjacencyKindBasis::Current,
                    checkpoint,
                )?,
                checkpoint,
            )?,
            history: KindIndex::rebuild(
                lane,
                &super::kind_projection::capture_controlled(
                    lane,
                    partition,
                    AdjacencyKindBasis::Historical,
                    checkpoint,
                )?,
                checkpoint,
            )?,
        })
    }
    pub(super) fn commitment(&self) -> RelationalRegionKindCommitment {
        RelationalRegionKindCommitment {
            head: self.head.digest(),
            history: self.history.digest(),
        }
    }
    pub(super) fn allocation_bytes(&self) -> u64 {
        self.head.allocation_bytes()
            + self.history.allocation_bytes()
            + self.supports.allocation_bytes()
    }
    pub(super) fn visit_allocations(&self, visitor: &mut dyn StorageAllocationVisitor) {
        self.head.visit_allocations(visitor);
        self.history.visit_allocations(visitor);
        self.supports.visit_allocations(visitor);
    }
}

pub(super) fn update_value(
    partition: &mut PartitionState,
    previous: &PartitionState,
    prior: &RegionKindIndexes,
    journal: &PartitionMutationJournal,
    checkpoint: &mut ProjectionCheckpoint<'_>,
    retain: &mut ProjectionRetention<'_>,
) -> Result<RegionKindIndexes, Cause> {
    // These traversal stacks are reused across journal slots. Reserve their
    // coexistence once, rather than accumulating one reservation per bucket.
    checkpoint(0, 65_536)?;
    let mut indexes = prior.clone();
    for (incoming, slots) in [
        (false, &journal.adjacency_slots),
        (true, &journal.reverse_adjacency_slots),
    ] {
        for &slot in slots {
            for basis in [AdjacencyKindBasis::Current, AdjacencyKindBasis::Historical] {
                let (current, old) = if incoming {
                    (
                        &mut partition.reverse_adjacency,
                        &previous.reverse_adjacency,
                    )
                } else {
                    (&mut partition.adjacency, &previous.adjacency)
                };
                let empty =
                    crate::storage::partition::AdjacencySet::new(&partition.adjacency_policy);
                let mut changes = Vec::new();
                current
                    .get(slot)
                    .unwrap_or(&empty)
                    .visit_changed_kind_memberships(
                        old.get(slot),
                        basis,
                        checkpoint,
                        &mut |kind, relation, present| {
                            changes.push((kind, relation, present));
                            Ok(())
                        },
                    )?;
                for (kind, relation, present) in changes {
                    let historical = matches!(basis, AdjacencyKindBasis::Historical);
                    // Current entity slot initialization can replace its adjacency
                    // value. Local relation history still names this physical slot;
                    // peer claims have only the newly supplied root-local meaning.
                    if historical
                        && !present
                        && relation.partition_id == partition.partition_id
                        && current.get(slot).is_some()
                    {
                        retain_membership(current, slot, kind, relation, checkpoint, retain)?;
                        continue;
                    }
                    if historical && relation.partition_id == partition.partition_id {
                        continue;
                    }
                    let index = if historical {
                        &mut indexes.history
                    } else {
                        &mut indexes.head
                    };
                    index.update(relation, kind, incoming, slot, present, checkpoint)?;
                }
            }
        }
    }
    super::kind_history::apply(
        partition,
        previous,
        &mut indexes,
        journal,
        checkpoint,
        retain,
    )?;
    Ok(indexes)
}

pub(super) fn retain_membership(
    table: &mut crate::storage::partition::SparseAdjacencyTable,
    slot: usize,
    kind: crate::identity::data::KindId,
    relation: crate::identity::data::RelationId,
    checkpoint: &mut ProjectionCheckpoint<'_>,
    retain: &mut ProjectionRetention<'_>,
) -> Result<(), Cause> {
    checkpoint(table.navigation_work_bound() as u64, 0)?;
    let set = table.get(slot).ok_or(Cause::IntegrityMismatch)?;
    if set.contains_historical_kind(kind, relation) {
        return Ok(());
    }
    let bytes = table
        .adoption_allocation_bound(1)
        .and_then(|bytes| bytes.checked_add(set.historical_kind_insertion_bound(kind)?))
        .ok_or(Cause::MemoryExhausted)?;
    let custody = retain(set.historical_kind_custody(), bytes)?;
    let set = table.get_mut(slot).ok_or(Cause::IntegrityMismatch)?;
    set.set_historical_kind_custody(custody);
    set.index_historical_kind(kind, relation);
    Ok(())
}

#[cfg(test)]
mod tests;
