//! Install validated kind claims through the driver that owns their backing.
use super::*;

pub(super) fn install(
    partition: &mut PartitionState,
    expected: &BTreeMap<RelationId, RegionRelationProjection>,
    basis: AdjacencyKindBasis,
    request: Option<&request::RequestProjection<'_, '_, '_>>,
    checkpoint: &mut ProjectionCheckpoint<'_>,
    retain: &mut ProjectionRetention<'_>,
) -> Result<(), Cause> {
    match request {
        Some(request) => install_request(partition, expected, basis, request, checkpoint),
        None => install_controlled(partition, expected, basis, checkpoint, retain),
    }
}

fn install_request(
    partition: &mut PartitionState,
    expected: &BTreeMap<RelationId, RegionRelationProjection>,
    basis: AdjacencyKindBasis,
    funding: &request::RequestProjection<'_, '_, '_>,
    checkpoint: &mut ProjectionCheckpoint<'_>,
) -> Result<(), Cause> {
    let historical = matches!(basis, AdjacencyKindBasis::Historical);
    let request = funding.execution;
    for (incoming, table) in [
        (false, &mut partition.adjacency),
        (true, &mut partition.reverse_adjacency),
    ] {
        let original = table.clone();
        for (&slot, _) in original.iter() {
            checkpoint(1, 0)?;
            let adjacency = table
                .get_mut_for_request(slot, request)
                .map_err(|cause| funding.stop(cause))?
                .ok_or(Cause::IntegrityMismatch)?;
            if historical {
                adjacency.reset_historical_kind_buckets_with_custody(None);
            } else {
                adjacency.reset_kind_buckets();
            }
        }
        for entry in expected.values() {
            checkpoint(1, 0)?;
            let slots = if incoming {
                &entry.incoming
            } else {
                &entry.outgoing
            };
            for &slot in slots {
                checkpoint(1, 0)?;
                let adjacency = table
                    .get_mut_for_request(slot, request)
                    .map_err(|cause| funding.stop(cause))?
                    .ok_or(Cause::IntegrityMismatch)?;
                let result = if historical {
                    adjacency.index_historical_kind_for_request(entry.kind, entry.relation, request)
                } else {
                    adjacency.index_current_kind_for_request(entry.kind, entry.relation, request)
                };
                result.map_err(|cause| funding.stop(cause))?;
            }
        }
        if !historical {
            for (_, adjacency) in table.iter() {
                let mut ids = BTreeSet::new();
                for &id in adjacency.current_ids() {
                    checkpoint(1 + 11 * (ids.len() + 1).ilog2() as u64, 0)?;
                    ids.insert(id);
                }
                let mut indexed = BTreeSet::new();
                for (_, id) in adjacency.kind_memberships(basis) {
                    checkpoint(1 + 11 * (indexed.len() + 1).ilog2() as u64, 0)?;
                    indexed.insert(id);
                }
                checkpoint(ids.len() as u64, 0)?;
                if ids != indexed {
                    return Err(Cause::IntegrityMismatch);
                }
            }
        }
    }
    Ok(())
}

pub(super) fn install_controlled(
    partition: &mut PartitionState,
    expected: &BTreeMap<RelationId, RegionRelationProjection>,
    basis: AdjacencyKindBasis,
    checkpoint: &mut ProjectionCheckpoint<'_>,
    retain: &mut ProjectionRetention<'_>,
) -> Result<(), Cause> {
    let historical = matches!(basis, AdjacencyKindBasis::Historical);
    for (incoming, table) in [
        (false, &mut partition.adjacency),
        (true, &mut partition.reverse_adjacency),
    ] {
        let mut keys = Vec::new();
        for (&slot, _) in table.iter() {
            checkpoint(1, 2 * size_of::<usize>() as u64)?;
            keys.push(slot);
        }
        for slot in keys {
            let admission = table
                .adoption_allocation_bound(1)
                .ok_or(Cause::MemoryExhausted)?;
            let admission = admission.checked_add(256).ok_or(Cause::MemoryExhausted)?;
            checkpoint(table.navigation_work_bound() as u64, 0)?;
            let custody = retain(None, admission)?;
            let adjacency = table.get_mut(slot).ok_or(Cause::IntegrityMismatch)?;
            if historical {
                adjacency.reset_historical_kind_buckets_with_custody(custody);
            } else {
                adjacency.reset_kind_buckets();
            }
        }
        for entry in expected.values() {
            checkpoint(1, 0)?;
            let slots = if incoming {
                &entry.incoming
            } else {
                &entry.outgoing
            };
            for &slot in slots {
                checkpoint(table.navigation_work_bound() as u64, 0)?;
                let admission = table
                    .adoption_allocation_bound(1)
                    .and_then(|bytes| bytes.checked_add(256))
                    .ok_or(Cause::MemoryExhausted)?;
                let set = table.get(slot).ok_or(Cause::IntegrityMismatch)?;
                let admission = admission
                    .checked_add(
                        set.historical_kind_insertion_bound(entry.kind)
                            .ok_or(Cause::MemoryExhausted)?,
                    )
                    .ok_or(Cause::MemoryExhausted)?;
                checkpoint(128, 0)?;
                retain(set.historical_kind_custody(), admission)?;
                let adjacency = table.get_mut(slot).ok_or(Cause::IntegrityMismatch)?;
                if historical {
                    adjacency.index_historical_kind(entry.kind, entry.relation);
                } else {
                    adjacency.index_current_kind(entry.kind, entry.relation);
                }
            }
        }
        if !historical {
            for (_, adjacency) in table.iter() {
                let ids: BTreeSet<_> = adjacency.current_ids().iter().copied().collect();
                let indexed: BTreeSet<_> = adjacency
                    .checkpoint_kind_memberships(basis)
                    .into_iter()
                    .flat_map(|(_, ids)| ids)
                    .collect();
                if ids != indexed {
                    return Err(Cause::IntegrityMismatch);
                }
            }
        }
    }
    Ok(())
}
