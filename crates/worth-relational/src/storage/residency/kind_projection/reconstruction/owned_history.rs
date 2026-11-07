//! Reconstruct local historical relation claims from authenticated record rows.
use super::*;

pub(super) fn add_owned(
    partition: &PartitionState,
    target: &mut BTreeMap<RelationId, RegionRelationProjection>,
    relation: RelationId,
    kind: KindId,
    endpoints: &crate::storage::substrate::RelationEndpoints,
    checkpoint: &mut ProjectionCheckpoint<'_>,
) -> Result<(), Cause> {
    checkpoint(1, 4096)?;
    let entry = target
        .entry(relation)
        .or_insert_with(|| RegionRelationProjection {
            relation,
            kind,
            outgoing: BTreeSet::new(),
            incoming: BTreeSet::new(),
        });
    if entry.kind != kind {
        return Err(Cause::IntegrityMismatch);
    }
    if endpoints.source.partition_id == partition.partition_id
        && partition
            .adjacency
            .get(endpoints.source.slot_index())
            .is_some()
    {
        entry.outgoing.insert(endpoints.source.slot_index());
    }
    if endpoints.target.partition_id == partition.partition_id
        && partition
            .reverse_adjacency
            .get(endpoints.target.slot_index())
            .is_some()
    {
        entry.incoming.insert(endpoints.target.slot_index());
    }
    if entry.outgoing.is_empty() && entry.incoming.is_empty() {
        target.remove(&relation);
    }
    Ok(())
}

pub(super) fn project_owned_history(
    _lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    partition: &PartitionState,
    expected: &mut BTreeMap<RelationId, RegionRelationProjection>,
    checkpoint: &mut ProjectionCheckpoint<'_>,
) -> Result<(), Cause> {
    for slot in partition.relation_arena.occupied_slots_iter() {
        checkpoint(1, 0)?;
        project_prefix(partition, slot, expected, checkpoint)?;
        for metadata in partition
            .relation_arena
            .metadata_history_at(slot)
            .into_iter()
            .flatten()
        {
            add_owned(
                partition,
                expected,
                RelationId::new(partition.partition_id, slot as u64, metadata.generation),
                metadata.kind_id,
                &metadata.endpoints,
                checkpoint,
            )?;
        }
        if let Some(view) = partition.relation_arena.get_slot(slot)
            && let (Some(kind), Some(endpoints)) = (view.kind_id(), view.extra().endpoints.as_ref())
        {
            add_owned(
                partition,
                expected,
                RelationId::new(partition.partition_id, slot as u64, view.generation()),
                kind,
                endpoints,
                checkpoint,
            )?;
        }
    }
    Ok(())
}

fn project_prefix(
    partition: &PartitionState,
    slot: usize,
    expected: &mut BTreeMap<RelationId, RegionRelationProjection>,
    checkpoint: &mut ProjectionCheckpoint<'_>,
) -> Result<(), Cause> {
    let physical = partition
        .relation_arena
        .physical_index(slot)
        .ok_or(Cause::IntegrityMismatch)?;
    let Some(prefix) = &partition.relation_arena.history_prefixes[physical] else {
        return Ok(());
    };
    for claim in &prefix.history_opening.historical_relations {
        checkpoint(1, 4096)?;
        if claim.relation.partition_id != partition.partition_id
            || claim.relation.slot_index() != slot
        {
            return Err(Cause::IntegrityMismatch);
        }
        let entry = expected
            .entry(claim.relation)
            .or_insert_with(|| RegionRelationProjection {
                relation: claim.relation,
                kind: claim.kind,
                outgoing: BTreeSet::new(),
                incoming: BTreeSet::new(),
            });
        if entry.kind != claim.kind {
            return Err(Cause::IntegrityMismatch);
        }
        for &slot in &claim.outgoing {
            checkpoint(partition.adjacency.navigation_work_bound() as u64, 256)?;
            if partition.adjacency.get(slot).is_some() {
                entry.outgoing.insert(slot);
            }
        }
        for &slot in &claim.incoming {
            checkpoint(
                partition.reverse_adjacency.navigation_work_bound() as u64,
                256,
            )?;
            if partition.reverse_adjacency.get(slot).is_some() {
                entry.incoming.insert(slot);
            }
        }
        if entry.outgoing.is_empty() && entry.incoming.is_empty() {
            expected.remove(&claim.relation);
        }
    }
    Ok(())
}
