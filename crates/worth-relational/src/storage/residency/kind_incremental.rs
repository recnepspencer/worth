//! Journal-scoped maintenance of admitted kind projections and their Merkle roots.
use super::kind_index::KindIndex;
use super::kind_projection::{ProjectionCheckpoint, ProjectionRetention};
use super::RelationalRegionKindCommitment;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::overlay::{PartitionMutationJournal, PartitionState};
use crate::storage::partition::AdjacencyKindBasis;
use crate::storage::substrate::StorageAllocationVisitor;

#[derive(Debug, Clone)]
pub(super) struct RegionKindIndexes {
    head: KindIndex,
    history: KindIndex,
}
impl RegionKindIndexes {
    pub(super) fn capture(partition: &PartitionState) -> Result<Self, Cause> {
        let mut checkpoint = |_, _| Ok(());
        Ok(Self {
            head: KindIndex::rebuild(&super::kind_projection::capture(partition, AdjacencyKindBasis::Current)?, &mut checkpoint)?,
            history: KindIndex::rebuild(&super::kind_projection::capture(partition, AdjacencyKindBasis::Historical)?, &mut checkpoint)?,
        })
    }
    pub(super) fn commitment(&self) -> RelationalRegionKindCommitment {
        RelationalRegionKindCommitment { head: self.head.digest(), history: self.history.digest() }
    }
    pub(super) fn allocation_bytes(&self) -> u64 { self.head.allocation_bytes() + self.history.allocation_bytes() }
    pub(super) fn visit_allocations(&self, visitor: &mut dyn StorageAllocationVisitor) {
        self.head.visit_allocations(visitor);
        self.history.visit_allocations(visitor);
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
    let mut indexes = prior.clone();
    for (incoming, slots) in [(false, &journal.adjacency_slots), (true, &journal.reverse_adjacency_slots)] {
        for &slot in slots {
            for basis in [AdjacencyKindBasis::Current, AdjacencyKindBasis::Historical] {
                let (current, old) = if incoming {
                    (&mut partition.reverse_adjacency, &previous.reverse_adjacency)
                } else { (&mut partition.adjacency, &previous.adjacency) };
                let empty = crate::storage::partition::AdjacencySet::new(&partition.adjacency_policy);
                let mut changes = Vec::new();
                current.get(slot).unwrap_or(&empty).visit_changed_kind_memberships(
                    old.get(slot), basis, checkpoint,
                    &mut |kind, relation, present| { changes.push((kind, relation, present)); Ok(()) },
                )?;
                for (kind, relation, present) in changes {
                    let historical = matches!(basis, AdjacencyKindBasis::Historical);
                    // Current entity slot initialization can replace its adjacency
                    // value. Local relation history still names this physical slot;
                    // peer claims have only the newly supplied root-local meaning.
                    if historical && !present && relation.partition_id == partition.partition_id
                        && current.get(slot).is_some()
                    {
                        retain_membership(current, slot, kind, relation, checkpoint, retain)?;
                        continue;
                    }
                    let index = if historical { &mut indexes.history } else { &mut indexes.head };
                    index.update(relation, kind, incoming, slot, present, checkpoint)?;
                }
            }
        }
    }
    // Owned historical meaning includes locally stored metadata even when a
    // record change emits no current incidence delta. Visit only newly supplied
    // history cells and the changed slot's current record, never its old prefix.
    for &slot in &journal.relation_slots {
        checkpoint(1, 0)?;
        let old = previous.relation_arena.metadata_history_at(slot);
        let history = partition.relation_arena.metadata_history_at(slot);
        let old_len = old.map_or(0, |values| values.len());
        let new_len = history.map_or(0, |values| values.len());
        if new_len < old_len { return Err(Cause::IntegrityMismatch); }
        let mut claims = Vec::new();
        if let Some(history) = history {
            for metadata in history.iter().skip(old_len) {
                checkpoint(1, 256)?;
                claims.push((metadata.generation, metadata.kind_id, metadata.endpoints.clone()));
            }
        }
        if let Some(view) = partition.relation_arena.get_slot(slot)
            && let (Some(kind), Some(endpoints)) = (view.kind_id(), view.extra().endpoints.as_ref())
        {
            checkpoint(1, 256)?;
            claims.push((view.generation(), kind, endpoints.clone()));
        }
        for (generation, kind, endpoints) in claims {
            let relation = crate::identity::data::RelationId::new(partition.partition_id, slot as u64, generation);
            for (incoming, endpoint) in [(false, endpoints.source), (true, endpoints.target)] {
                if endpoint.partition_id != partition.partition_id { continue; }
                let table = if incoming { &mut partition.reverse_adjacency } else { &mut partition.adjacency };
                if table.get(endpoint.slot_index()).is_some() {
                    retain_membership(table, endpoint.slot_index(), kind, relation, checkpoint, retain)?;
                    indexes.history.update(relation, kind, incoming, endpoint.slot_index(), true, checkpoint)?;
                }
            }
        }
    }
    Ok(indexes)
}

fn retain_membership(
    table: &mut crate::storage::partition::SparseAdjacencyTable,
    slot: usize,
    kind: crate::identity::data::KindId,
    relation: crate::identity::data::RelationId,
    checkpoint: &mut ProjectionCheckpoint<'_>,
    retain: &mut ProjectionRetention<'_>,
) -> Result<(), Cause> {
    checkpoint(table.navigation_work_bound() as u64, 0)?;
    let set = table.get(slot).ok_or(Cause::IntegrityMismatch)?;
    if set.historical_kind_ids(kind).iter().any(|id| *id == relation) { return Ok(()); }
    let bytes = table.adoption_allocation_bound(1)
        .and_then(|bytes| bytes.checked_add(set.historical_kind_insertion_bound(kind)?))
        .ok_or(Cause::MemoryExhausted)?;
    let custody = retain(set.historical_kind_custody(), bytes)?;
    let set = table.get_mut(slot).ok_or(Cause::IntegrityMismatch)?;
    set.set_historical_kind_custody(custody);
    set.index_historical_kind(kind, relation);
    Ok(())
}
