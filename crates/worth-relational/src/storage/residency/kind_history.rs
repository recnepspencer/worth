//! Historical incidence is supported by carried prefix facts and resident rows.
use super::kind_incremental::RegionKindIndexes;
use super::kind_projection::{ProjectionCheckpoint, ProjectionRetention};
use crate::identity::data::{KindId, RelationId};
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::overlay::{PartitionMutationJournal, PartitionState};
use crate::storage::substrate::{
    RecordHistoryPrefix, RelationEndpoints, SharedColumn, SharedMap, StorageAllocationVisitor,
};

type Coordinate = (RelationId, bool, usize);
type Edit = (KindId, RelationId, bool, usize, bool);

/// Counts are acceleration, not digest material. Dropping a row removes a
/// membership only when no retained row or authenticated prefix supports it.
#[derive(Debug, Clone, Default)]
pub(super) struct HistorySupports(SharedMap<Coordinate, (KindId, u64)>);
impl HistorySupports {
    pub(super) fn capture(
        partition: &PartitionState,
        checkpoint: &mut ProjectionCheckpoint<'_>,
    ) -> Result<Self, Cause> {
        let mut result = Self::default();
        for slot in partition.relation_arena.occupied_slots_iter() {
            checkpoint(1, 0)?;
            let mut edits = Vec::new();
            prefix_edits(partition, slot, true, &mut edits, checkpoint)?;
            for metadata in partition
                .relation_arena
                .metadata_history_at(slot)
                .into_iter()
                .flatten()
            {
                checkpoint(1, 0)?;
                row_edits(
                    partition,
                    RelationId::new(partition.partition_id, slot as u64, metadata.generation),
                    metadata.kind_id,
                    &metadata.endpoints,
                    true,
                    &mut edits,
                    checkpoint,
                )?;
            }
            extra_edits(partition, slot, true, &mut edits, checkpoint)?;
            for (kind, relation, incoming, endpoint, add) in edits {
                result.change(kind, relation, incoming, endpoint, add, checkpoint)?;
            }
        }
        Ok(result)
    }
    fn change(
        &mut self,
        kind: KindId,
        relation: RelationId,
        incoming: bool,
        slot: usize,
        add: bool,
        checkpoint: &mut ProjectionCheckpoint<'_>,
    ) -> Result<Option<bool>, Cause> {
        let key = (relation, incoming, slot);
        checkpoint(self.0.navigation_work_bound() as u64, 0)?;
        let old = self.0.get(&key).copied().unwrap_or((kind, 0));
        if old.0 != kind {
            return Err(Cause::IntegrityMismatch);
        }
        let next = if add {
            old.1.checked_add(1)
        } else {
            old.1.checked_sub(1)
        }
        .ok_or(Cause::IntegrityMismatch)?;
        checkpoint(
            self.0.navigation_work_bound() as u64,
            self.0
                .adoption_allocation_bound()
                .and_then(|n| n.checked_add(256))
                .ok_or(Cause::MemoryExhausted)?,
        )?;
        if next == 0 {
            self.0.remove(&key);
        } else {
            self.0.insert(key, (kind, next));
        }
        Ok(if old.1 == 0 {
            Some(true)
        } else if next == 0 {
            Some(false)
        } else {
            None
        })
    }
    pub(super) fn allocation_bytes(&self) -> u64 {
        self.0.allocation_bytes()
    }
    pub(super) fn visit_allocations(&self, visitor: &mut dyn StorageAllocationVisitor) {
        self.0.visit_allocations(
            false,
            visitor,
            &mut crate::storage::substrate::visit_inline_value,
        );
    }
}

/// Cost: changed row cells and any newly imported prefix opening, plus lookup
/// paths. An unchanged history frontier is never walked, even when rows shrink.
pub(super) fn apply(
    partition: &mut PartitionState,
    previous: &PartitionState,
    indexes: &mut RegionKindIndexes,
    journal: &PartitionMutationJournal,
    checkpoint: &mut ProjectionCheckpoint<'_>,
    retain: &mut ProjectionRetention<'_>,
) -> Result<(), Cause> {
    for &slot in &journal.relation_slots {
        checkpoint(1, 0)?;
        let mut edits = Vec::new();
        if !same_prefix(prefix(partition, slot), prefix(previous, slot)) {
            // cost-lane: import. A different authenticated cold opening is an
            // explicit source-basis replacement; resident suffixes remain delta.
            prefix_edits(previous, slot, false, &mut edits, checkpoint)?;
            prefix_edits(partition, slot, true, &mut edits, checkpoint)?;
        }
        let empty = SharedColumn::default();
        let old = previous
            .relation_arena
            .metadata_history_at(slot)
            .unwrap_or(&empty);
        let new = partition
            .relation_arena
            .metadata_history_at(slot)
            .unwrap_or(&empty);
        let mut failure = None;
        new.visit_changed_values(old, &mut |_, current, prior| {
            if failure.is_some() {
                return;
            }
            let semantic = |row: &crate::storage::substrate::VersionedRelationMetadata| {
                (row.generation, row.kind_id, row.endpoints.clone())
            };
            if current.map(semantic) == prior.map(semantic) {
                return;
            }
            let result = (|| {
                checkpoint(1, 0)?;
                for (row, add, source) in [(prior, false, previous), (current, true, &*partition)] {
                    if let Some(row) = row {
                        row_edits(
                            source,
                            RelationId::new(source.partition_id, slot as u64, row.generation),
                            row.kind_id,
                            &row.endpoints,
                            add,
                            &mut edits,
                            checkpoint,
                        )?;
                    }
                }
                Ok(())
            })();
            if let Err(cause) = result {
                failure = Some(cause);
            }
        });
        if let Some(cause) = failure {
            return Err(cause);
        }
        let extra = |source: &PartitionState| {
            source.relation_arena.get_slot(slot).and_then(|view| {
                Some((
                    view.generation(),
                    view.kind_id()?,
                    view.extra().endpoints.clone()?,
                ))
            })
        };
        if extra(previous) != extra(partition) {
            extra_edits(previous, slot, false, &mut edits, checkpoint)?;
            extra_edits(partition, slot, true, &mut edits, checkpoint)?;
        }
        for (kind, relation, incoming, endpoint, add) in edits {
            let Some(present) = indexes
                .supports
                .change(kind, relation, incoming, endpoint, add, checkpoint)?
            else {
                continue;
            };
            indexes
                .history
                .update(relation, kind, incoming, endpoint, present, checkpoint)?;
            let table = if incoming {
                &mut partition.reverse_adjacency
            } else {
                &mut partition.adjacency
            };
            if present {
                super::kind_incremental::retain_membership(
                    table, endpoint, kind, relation, checkpoint, retain,
                )?;
            } else if let Some(set) = table.get_mut(endpoint) {
                set.remove_historical_kind(kind, relation);
            }
        }
    }
    Ok(())
}

fn prefix(partition: &PartitionState, slot: usize) -> Option<&RecordHistoryPrefix> {
    let physical = partition.relation_arena.physical_index(slot)?;
    partition.relation_arena.history_prefixes[physical].as_ref()
}
fn same_prefix(
    current: Option<&RecordHistoryPrefix>,
    previous: Option<&RecordHistoryPrefix>,
) -> bool {
    match (current, previous) {
        (None, None) => true,
        (Some(current), Some(previous)) => {
            current.member == previous.member
                && current.row_index == previous.row_index
                && current.partition_id == previous.partition_id
                && current.history_opening.id() == previous.history_opening.id()
        }
        _ => false,
    }
}
fn prefix_edits(
    partition: &PartitionState,
    slot: usize,
    add: bool,
    edits: &mut Vec<Edit>,
    checkpoint: &mut ProjectionCheckpoint<'_>,
) -> Result<(), Cause> {
    if let Some(prefix) = prefix(partition, slot) {
        for claim in &prefix.history_opening.historical_relations {
            if claim.relation.partition_id != partition.partition_id
                || claim.relation.slot_index() != slot
            {
                return Err(Cause::IntegrityMismatch);
            }
            for (incoming, slots) in [(false, &claim.outgoing), (true, &claim.incoming)] {
                for &endpoint in slots {
                    let table = if incoming {
                        &partition.reverse_adjacency
                    } else {
                        &partition.adjacency
                    };
                    checkpoint(table.navigation_work_bound() as u64, 256)?;
                    if table.get(endpoint).is_some() {
                        edits.push((claim.kind, claim.relation, incoming, endpoint, add));
                    }
                }
            }
        }
    }
    Ok(())
}
fn extra_edits(
    partition: &PartitionState,
    slot: usize,
    add: bool,
    edits: &mut Vec<Edit>,
    checkpoint: &mut ProjectionCheckpoint<'_>,
) -> Result<(), Cause> {
    if let Some(view) = partition.relation_arena.get_slot(slot)
        && let (Some(kind), Some(endpoints)) = (view.kind_id(), view.extra().endpoints.as_ref())
    {
        row_edits(
            partition,
            RelationId::new(partition.partition_id, slot as u64, view.generation()),
            kind,
            endpoints,
            add,
            edits,
            checkpoint,
        )?;
    }
    Ok(())
}
fn row_edits(
    partition: &PartitionState,
    relation: RelationId,
    kind: KindId,
    endpoints: &RelationEndpoints,
    add: bool,
    edits: &mut Vec<Edit>,
    checkpoint: &mut ProjectionCheckpoint<'_>,
) -> Result<(), Cause> {
    for (incoming, endpoint) in [(false, endpoints.source), (true, endpoints.target)] {
        let table = if incoming {
            &partition.reverse_adjacency
        } else {
            &partition.adjacency
        };
        if endpoint.partition_id == partition.partition_id
            && table.get(endpoint.slot_index()).is_some()
        {
            checkpoint(1, 256)?;
            edits.push((kind, relation, incoming, endpoint.slot_index(), add));
        }
    }
    Ok(())
}
