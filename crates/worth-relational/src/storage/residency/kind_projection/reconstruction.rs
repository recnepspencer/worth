use super::*;
use crate::identity::data::KindId;

/// Owned relations are reconstructed from record metadata. Peer relations use
/// only the projection authenticated by this region's canonical root leaf.
pub(in crate::storage::residency) fn rebuild(
    partition: &mut PartitionState,
    projection: &[RegionRelationProjection],
    basis: AdjacencyKindBasis,
) -> Result<(), Cause> {
    rebuild_controlled(
        partition,
        projection,
        basis,
        &mut |_, _| Ok(()),
        &mut |_, _| Ok(None),
    )
}

pub(in crate::storage::residency) fn rebuild_controlled(
    partition: &mut PartitionState,
    projection: &[RegionRelationProjection],
    basis: AdjacencyKindBasis,
    checkpoint: &mut ProjectionCheckpoint<'_>,
    retain: &mut ProjectionRetention<'_>,
) -> Result<(), Cause> {
    let mut expected = BTreeMap::<RelationId, RegionRelationProjection>::new();
    for entry in projection {
        let cloned = clone_claim(entry, checkpoint)?;
        if expected.insert(entry.relation, cloned).is_some() {
            return Err(Cause::Damaged);
        }
    }
    let historical = matches!(basis, AdjacencyKindBasis::Historical);
    if historical {
        // Discard every supplied claim about locally owned history, and derive
        // it again from the authenticated arena, including retired generations.
        let mut owned = Vec::new();
        for id in expected.keys() {
            checkpoint(1, 2 * size_of::<RelationId>() as u64)?;
            if id.partition_id == partition.partition_id {
                owned.push(*id);
            }
        }
        for id in owned {
            checkpoint(1, 0)?;
            expected.remove(&id);
        }
        project_owned_history(partition, &mut expected, checkpoint)?;
        if !claims_match(&expected, projection, checkpoint)? {
            return Err(Cause::IntegrityMismatch);
        }
    }
    install_projection(partition, &expected, basis, checkpoint, retain)?;
    if !historical {
        for entry in expected
            .values()
            .filter(|entry| entry.relation.partition_id == partition.partition_id)
        {
            let view = partition
                .relation_arena
                .get_slot(entry.relation.slot_index())
                .ok_or(Cause::Damaged)?;
            if view.generation() != entry.relation.generation.0
                || view.kind_id() != Some(entry.kind)
            {
                return Err(Cause::IntegrityMismatch);
            }
            let mut local = BTreeMap::new();
            add_owned(
                partition,
                &mut local,
                entry.relation,
                entry.kind,
                view.extra().endpoints.as_ref().ok_or(Cause::Damaged)?,
                checkpoint,
            )?;
            if local.get(&entry.relation) != Some(entry) {
                return Err(Cause::IntegrityMismatch);
            }
        }
    }
    Ok(())
}

/// The digest is authenticated by the caller. Head can corroborate current
/// owned claims; Full must later reconstruct all owned historical claims.
pub(in crate::storage::residency) fn rebuild_head_history(
    partition: &mut PartitionState,
    projection: &[RegionRelationProjection],
) -> Result<(), Cause> {
    let checkpoint: &mut ProjectionCheckpoint<'_> = &mut |_, _| Ok(());
    let mut expected = BTreeMap::new();
    for entry in projection {
        let cloned = clone_claim(entry, checkpoint)?;
        if expected.insert(entry.relation, cloned).is_some() {
            return Err(Cause::Damaged);
        }
    }
    let mut known = BTreeMap::new();
    project_owned_history(partition, &mut known, checkpoint)?;
    for (relation, claim) in known {
        let supplied = expected.get(&relation).ok_or(Cause::IntegrityMismatch)?;
        if claim.kind != supplied.kind
            || !claim.outgoing.is_subset(&supplied.outgoing)
            || !claim.incoming.is_subset(&supplied.incoming)
        {
            return Err(Cause::IntegrityMismatch);
        }
    }
    install_projection(
        partition,
        &expected,
        AdjacencyKindBasis::Historical,
        checkpoint,
        &mut |_, _| Ok(None),
    )
}

fn install_projection(
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

pub(in crate::storage::residency) fn canonical_history(
    partition: &PartitionState,
) -> Result<Vec<RegionRelationProjection>, Cause> {
    canonical_history_controlled(partition, &mut |_, _| Ok(()))
}

pub(in crate::storage::residency) fn canonical_history_controlled(
    partition: &PartitionState,
    checkpoint: &mut ProjectionCheckpoint<'_>,
) -> Result<Vec<RegionRelationProjection>, Cause> {
    let mut expected = BTreeMap::new();
    for entry in capture_controlled(partition, AdjacencyKindBasis::Historical, checkpoint)? {
        checkpoint(1, 4096)?;
        if entry.relation.partition_id != partition.partition_id {
            expected.insert(entry.relation, entry);
        }
    }
    project_owned_history(partition, &mut expected, checkpoint)?;
    let mut answer = Vec::new();
    for entry in expected.into_values() {
        checkpoint(1, 2 * size_of::<RegionRelationProjection>() as u64)?;
        answer.push(entry);
    }
    Ok(answer)
}

mod owned_history;
use owned_history::{add_owned, project_owned_history};
mod claim_copy;
use claim_copy::{claims_match, clone_claim};
