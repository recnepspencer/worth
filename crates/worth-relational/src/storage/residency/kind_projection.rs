//! Root-authenticated relation projections, independent of serving caches.
use crate::identity::data::RelationId;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::overlay::PartitionState;
use crate::storage::partition::{AdjacencyKindBasis, RegionRelationProjection};
use crate::storage::substrate::maintenance::ReconstructionLane;
use std::collections::{BTreeMap, BTreeSet};

/// Conservative coexistence claims precede each map/set insertion. The callback
/// is erased so publication does not monomorphize the projection engine.
pub(super) type ProjectionCheckpoint<'a> = dyn FnMut(u64, u64) -> Result<(), Cause> + 'a;

pub(super) type ProjectionRetention<'a> = dyn FnMut(
        Option<&crate::storage::partition::HistoricalKindCustody>,
        u64,
    ) -> Result<Option<crate::storage::partition::HistoricalKindCustody>, Cause>
    + 'a;

#[cfg(test)]
pub(super) fn capture(
    lane: &ReconstructionLane<'_>,
    partition: &PartitionState,
    basis: AdjacencyKindBasis,
) -> Result<Vec<RegionRelationProjection>, Cause> {
    capture_controlled(lane, partition, basis, &mut |_, _| Ok(()))
}

pub(super) fn capture_controlled(
    _lane: &ReconstructionLane<'_>,
    partition: &PartitionState,
    basis: AdjacencyKindBasis,
    checkpoint: &mut ProjectionCheckpoint<'_>,
) -> Result<Vec<RegionRelationProjection>, Cause> {
    let mut projection = BTreeMap::<RelationId, RegionRelationProjection>::new();
    for (incoming, table) in [
        (false, &partition.adjacency),
        (true, &partition.reverse_adjacency),
    ] {
        for (&slot, adjacency) in table.iter() {
            checkpoint(1, 0)?;
            adjacency.visit_kind_memberships(basis, &mut |kind, relation| {
                checkpoint(1, 0)?;
                let Some(relation) = relation else {
                    return Ok(());
                };
                checkpoint(0, 4096)?;
                let entry =
                    projection
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
                if incoming {
                    entry.incoming.insert(slot);
                } else {
                    entry.outgoing.insert(slot);
                }
                Ok(())
            })?;
        }
    }
    let mut answer = Vec::new();
    for entry in projection.into_values() {
        checkpoint(1, 2 * size_of::<RegionRelationProjection>() as u64)?;
        answer.push(entry);
    }
    Ok(answer)
}

pub(super) mod full;
mod reconstruction;
#[cfg(test)]
pub(super) use reconstruction::rebuild;
pub(super) use reconstruction::{rebuild_for_request, rebuild_head_history_for_request};

#[cfg(test)]
pub(super) use reconstruction::canonical_history;
pub(super) use reconstruction::{canonical_history_controlled, rebuild_controlled};

#[cfg(test)]
pub(crate) fn rebuild_region_history_kind_buckets(
    lane: &ReconstructionLane<'_>,
    partition: &mut PartitionState,
) -> Result<(), Cause> {
    let projection = canonical_history(lane, partition)?;
    rebuild(lane, partition, &projection, AdjacencyKindBasis::Historical)
}
