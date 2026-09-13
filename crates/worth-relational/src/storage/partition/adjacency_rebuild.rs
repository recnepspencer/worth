use std::collections::BTreeMap;

use crate::identity::data::{KindId, PartitionId, RelationId};

use crate::storage::overlay::PartitionState;

pub(crate) fn rebuild_adjacency_kind_buckets(
    partitions: &mut BTreeMap<PartitionId, PartitionState>,
) -> Result<(), String> {
    let mut relation_kinds = BTreeMap::<RelationId, KindId>::new();
    let mut historical = Vec::new();
    for (partition_id, partition) in partitions.iter() {
        for slot in partition.relation_arena.occupied_slots() {
            for metadata in partition
                .relation_arena
                .metadata_history_at(slot)
                .unwrap_or_default()
            {
                let relation_id = RelationId::new(*partition_id, slot as u64, metadata.generation);
                relation_kinds.insert(relation_id, metadata.kind_id);
                historical.push((
                    relation_id,
                    metadata.kind_id,
                    metadata.endpoints.source,
                    metadata.endpoints.target,
                    metadata.effective_at,
                    metadata.retired_at,
                ));
            }
            let Some(slot_view) = partition.relation_arena.get_slot(slot) else {
                continue;
            };
            let Some(kind_id) = slot_view.kind_id() else {
                continue;
            };
            let relation_id = RelationId::new(*partition_id, slot as u64, slot_view.generation());
            relation_kinds.insert(relation_id, kind_id);
            if let Some(endpoints) = slot_view.extra().endpoints.as_ref() {
                let effective_at = partition
                    .relation_arena
                    .created_at_for_slot(slot)
                    .ok_or_else(|| format!("relation slot {slot} has no creation version"))?;
                historical.push((
                    relation_id,
                    kind_id,
                    endpoints.source,
                    endpoints.target,
                    effective_at,
                    slot_view.retired_at(),
                ));
            }
        }
    }

    let mut current = Vec::new();
    for (partition_id, partition) in partitions.iter_mut() {
        for (&slot, adjacency) in partition.adjacency.iter_mut() {
            current.extend(
                adjacency
                    .as_slice()
                    .iter()
                    .copied()
                    .map(|relation_id| (*partition_id, slot, false, relation_id)),
            );
            adjacency.reset_kind_buckets();
        }
        for (&slot, adjacency) in partition.reverse_adjacency.iter_mut() {
            current.extend(
                adjacency
                    .as_slice()
                    .iter()
                    .copied()
                    .map(|relation_id| (*partition_id, slot, true, relation_id)),
            );
            adjacency.reset_kind_buckets();
        }
    }

    for (partition_id, slot, incoming, relation_id) in current {
        let kind_id = relation_kinds.get(&relation_id).copied().ok_or_else(|| {
            format!(
                "checkpoint adjacency references relation {:?} without retained kind authority",
                relation_id
            )
        })?;
        let partition = partitions.get_mut(&partition_id).ok_or_else(|| {
            format!(
                "checkpoint adjacency references missing partition {:?}",
                partition_id
            )
        })?;
        let adjacency = if incoming {
            partition.reverse_adjacency.get_mut(slot)
        } else {
            partition.adjacency.get_mut(slot)
        }
        .ok_or_else(|| format!("checkpoint adjacency references missing entity slot {slot}"))?;
        adjacency.index_current_kind(kind_id, relation_id);
    }

    for (relation_id, kind_id, source, target, effective_at, retired_at) in historical {
        if let Some(adjacency) = partitions
            .get_mut(&source.partition_id)
            .and_then(|partition| partition.adjacency.get_mut(source.slot_index()))
        {
            adjacency.index_historical_kind(kind_id, relation_id);
            adjacency.index_structural_revision(kind_id, effective_at);
            if let Some(retired_at) = retired_at {
                adjacency.index_structural_revision(kind_id, retired_at);
            }
        }
        if let Some(adjacency) = partitions
            .get_mut(&target.partition_id)
            .and_then(|partition| partition.reverse_adjacency.get_mut(target.slot_index()))
        {
            adjacency.index_historical_kind(kind_id, relation_id);
            adjacency.index_structural_revision(kind_id, effective_at);
            if let Some(retired_at) = retired_at {
                adjacency.index_structural_revision(kind_id, retired_at);
            }
        }
    }
    Ok(())
}
