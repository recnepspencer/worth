//! Adjacency buckets are accelerators, not truth. Verify their complete current
//! membership against admitted relation heads. Historical buckets are rebuilt
//! from retained relation metadata before a full serving lease is issued.
use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock, Weak};

use super::{RelationalRegionResidency, ResidentRegion};
use crate::identity::data::{KindId, PartitionId, RelationId};
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::partition::AdjacencyKindBasis;

#[derive(Debug, Default)]
pub(crate) struct RelationalRegionKindAuthority {
    regions: OnceLock<BTreeMap<PartitionId, (usize, Weak<RelationalRegionResidency>)>>,
}

impl RelationalRegionKindAuthority {
    pub(crate) fn bind(
        &self,
        regions: impl IntoIterator<Item = (PartitionId, usize, Arc<RelationalRegionResidency>)>,
    ) -> Result<(), Cause> {
        let mut directory = BTreeMap::new();
        for (id, relations, cell) in regions {
            if directory
                .insert(id, (relations, Arc::downgrade(&cell)))
                .is_some()
            {
                return Err(Cause::Damaged);
            }
        }
        self.regions.set(directory).map_err(|_| Cause::Damaged)
    }

    pub(super) fn verify_head(&self, region: &ResidentRegion, limit: u64) -> Result<(), Cause> {
        let partition = &region.partition;
        for table in [&partition.adjacency, &partition.reverse_adjacency] {
            for (_, adjacency) in table.iter() {
                let mut expected = BTreeMap::<KindId, Vec<RelationId>>::new();
                for id in adjacency.current_ids().iter().copied() {
                    let owner;
                    let authority = if id.partition_id == partition.partition_id {
                        partition
                    } else {
                        owner = self.cell(id.partition_id)?.raw_head(limit)?;
                        &owner.partition
                    };
                    let kind = authority
                        .relation_arena
                        .get(&id)
                        .and_then(|slot| slot.kind_id())
                        .ok_or(Cause::IntegrityMismatch)?;
                    expected.entry(kind).or_default().push(id);
                }
                let supplied = adjacency
                    .checkpoint_kind_memberships(AdjacencyKindBasis::Current)
                    .into_iter()
                    .filter(|(_, ids)| !ids.is_empty())
                    .collect::<BTreeMap<_, _>>();
                if expected != supplied {
                    return Err(Cause::IntegrityMismatch);
                }
            }
        }
        Ok(())
    }

    pub(super) fn admit_full(
        &self,
        region: &Arc<ResidentRegion>,
        limit: u64,
    ) -> Result<Arc<ResidentRegion>, Cause> {
        self.verify_head(region, limit)?;
        let mut authority =
            BTreeMap::from([(region.partition.partition_id, region.partition.clone())]);
        if let Some(regions) = self.regions.get() {
            for (&id, (relations, cell)) in regions {
                if id == region.partition.partition_id || *relations == 0 {
                    continue;
                }
                let cell = cell.upgrade().ok_or(Cause::SourceUnavailable)?;
                let raw = cell.raw_full(limit)?;
                authority.insert(id, raw.partition.clone());
            }
        }
        crate::storage::partition::rebuild_adjacency_kind_buckets(&mut authority)
            .map_err(|_| Cause::IntegrityMismatch)?;
        let partition = authority
            .remove(&region.partition.partition_id)
            .ok_or(Cause::Damaged)?;
        Ok(Arc::new(ResidentRegion {
            partition,
            commitment: region.commitment.clone(),
        }))
    }

    fn cell(&self, id: PartitionId) -> Result<Arc<RelationalRegionResidency>, Cause> {
        self.regions
            .get()
            .and_then(|regions| regions.get(&id))
            .and_then(|(_, cell)| cell.upgrade())
            .ok_or(Cause::SourceUnavailable)
    }
}
