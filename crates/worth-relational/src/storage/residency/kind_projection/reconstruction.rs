use super::*;
use crate::identity::data::KindId;

// Numeric projection keys take one comparison unit. Include both node search
// and rearrangement; never use the node's unused allocation slots as work.
fn ordered_lookup_work(entries: usize) -> Result<u64, Cause> {
    let levels = entries
        .checked_add(1)
        .ok_or(Cause::MemoryExhausted)?
        .ilog2() as u64
        + 1;
    Ok(22 * levels)
}

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
    rebuild_using(partition, projection, basis, None, checkpoint, retain)
}
fn rebuild_using(
    partition: &mut PartitionState,
    projection: &[RegionRelationProjection],
    basis: AdjacencyKindBasis,
    request: Option<&request::RequestProjection<'_, '_, '_>>,
    checkpoint: &mut ProjectionCheckpoint<'_>,
    retain: &mut ProjectionRetention<'_>,
) -> Result<(), Cause> {
    let mut expected = BTreeMap::<RelationId, RegionRelationProjection>::new();
    for entry in projection {
        let cloned = clone_claim(entry, checkpoint)?;
        checkpoint(
            ordered_lookup_work(expected.len())? + size_of::<RelationId>() as u64,
            0,
        )?;
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
            checkpoint(
                1 + 2 * size_of::<RelationId>() as u64,
                2 * size_of::<RelationId>() as u64,
            )?;
            if id.partition_id == partition.partition_id {
                owned.push(*id);
            }
        }
        for id in owned {
            checkpoint(ordered_lookup_work(expected.len())?, 0)?;
            expected.remove(&id);
        }
        project_owned_history(partition, &mut expected, checkpoint)?;
        if !claims_match(&expected, projection, checkpoint)? {
            return Err(Cause::IntegrityMismatch);
        }
    }
    installation::install(partition, &expected, basis, request, checkpoint, retain)?;
    if !historical {
        for entry in expected
            .values()
            .filter(|entry| entry.relation.partition_id == partition.partition_id)
        {
            checkpoint(
                ordered_lookup_work(partition.relation_arena.slot_count())?,
                0,
            )?;
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
            checkpoint(
                ordered_lookup_work(local.len())? + size_of::<RegionRelationProjection>() as u64,
                0,
            )?;
            checkpoint(
                (entry.outgoing.len() as u64)
                    .checked_add(entry.incoming.len() as u64)
                    .ok_or(Cause::MemoryExhausted)?,
                0,
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
fn rebuild_head_using(
    partition: &mut PartitionState,
    projection: &[RegionRelationProjection],
    request: Option<&request::RequestProjection<'_, '_, '_>>,
    checkpoint: &mut ProjectionCheckpoint<'_>,
) -> Result<(), Cause> {
    let mut expected = BTreeMap::new();
    for entry in projection {
        let cloned = clone_claim(entry, checkpoint)?;
        checkpoint(
            ordered_lookup_work(expected.len())? + size_of::<RelationId>() as u64,
            0,
        )?;
        if expected.insert(entry.relation, cloned).is_some() {
            return Err(Cause::Damaged);
        }
    }
    let mut known = BTreeMap::new();
    project_owned_history(partition, &mut known, checkpoint)?;
    for (relation, claim) in known {
        checkpoint(ordered_lookup_work(expected.len())?, 0)?;
        let supplied = expected.get(&relation).ok_or(Cause::IntegrityMismatch)?;
        for (left, right) in [
            (&claim.outgoing, &supplied.outgoing),
            (&claim.incoming, &supplied.incoming),
        ] {
            // BTreeSet may choose a merge or ordered searches. Fund either
            // from actual endpoint cardinalities, not backing capacity.
            let searches = (left.len() as u64)
                .checked_mul(ordered_lookup_work(right.len())?)
                .ok_or(Cause::MemoryExhausted)?;
            let merge = (left.len() as u64)
                .checked_add(right.len() as u64)
                .ok_or(Cause::MemoryExhausted)?;
            checkpoint(searches.max(merge), 0)?;
        }
        if claim.kind != supplied.kind
            || !claim.outgoing.is_subset(&supplied.outgoing)
            || !claim.incoming.is_subset(&supplied.incoming)
        {
            return Err(Cause::IntegrityMismatch);
        }
    }
    installation::install(
        partition,
        &expected,
        AdjacencyKindBasis::Historical,
        request,
        checkpoint,
        &mut |_, _| Ok(None),
    )
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

mod installation;
mod request;
pub(in crate::storage::residency) use request::{
    rebuild_for_request, rebuild_head_history_for_request,
};
