//! Root-authenticated relation projections, independent of serving caches.
use crate::identity::data::RelationId;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::overlay::PartitionState;
use crate::storage::partition::{AdjacencyKindBasis, RegionRelationProjection};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn capture(
    partition: &PartitionState,
    basis: AdjacencyKindBasis,
) -> Result<Vec<RegionRelationProjection>, Cause> {
    let mut projection = BTreeMap::<RelationId, RegionRelationProjection>::new();
    for (incoming, table) in [
        (false, &partition.adjacency),
        (true, &partition.reverse_adjacency),
    ] {
        for (&slot, adjacency) in table.iter() {
            for (kind, relations) in adjacency.checkpoint_kind_memberships(basis) {
                for relation in relations {
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
                }
            }
        }
    }
    Ok(projection.into_values().collect())
}

mod reconstruction;
pub(super) use reconstruction::rebuild;
pub(super) use reconstruction::rebuild_head_history;

pub(super) use reconstruction::canonical_history;

pub(crate) fn rebuild_region_history_kind_buckets(
    partition: &mut PartitionState,
) -> Result<(), Cause> {
    let projection = canonical_history(partition)?;
    rebuild(partition, &projection, AdjacencyKindBasis::Historical)
}
