//! Adjacency buckets are accelerators, not truth. Verify their complete current
//! membership against admitted relation heads. Historical buckets are rebuilt
//! from retained relation metadata before a full serving lease is issued.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, OnceLock, Weak};

use super::{
    RelationalRegionResidency, RelationalResidencyDemandFailure as Failure, ResidentRegion,
};
use crate::identity::data::{KindId, PartitionId, RelationId};
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause,
};
use crate::storage::partition::AdjacencyKindBasis;

#[derive(Debug, Default)]
pub(crate) struct RelationalRegionKindAuthority {
    verified_heads: Mutex<BTreeSet<PartitionId>>,
    regions: OnceLock<BTreeMap<PartitionId, (usize, Weak<RelationalRegionResidency>)>>,
}

impl RelationalRegionKindAuthority {
    pub(super) fn require_cell(&self, cell: &RelationalRegionResidency) -> Result<(), Cause> {
        if let Some(regions) = self.regions.get() {
            let expected = regions
                .get(&cell.partition_id)
                .ok_or(Cause::IntegrityMismatch)?;
            if !std::ptr::eq(expected.1.as_ptr(), cell) {
                return Err(Cause::IntegrityMismatch);
            }
        }
        Ok(())
    }
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

    pub(super) fn verify_head(
        &self,
        region: &ResidentRegion,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<(), Failure> {
        let partition = &region.partition;
        let mut verified = self
            .verified_heads
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if verified.contains(&partition.partition_id) {
            return Ok(());
        }
        let refusal = |cause| unavailable(partition.partition_id, cause);
        for table in [&partition.adjacency, &partition.reverse_adjacency] {
            for (_, adjacency) in table.iter() {
                let mut expected = BTreeMap::<KindId, Vec<RelationId>>::new();
                for id in adjacency.current_ids().iter().copied() {
                    let owner;
                    let authority = if id.partition_id == partition.partition_id {
                        partition
                    } else {
                        owner = self
                            .cell(id.partition_id)
                            .map_err(refusal)?
                            .raw_head(execution, limit)?;
                        &owner.partition
                    };
                    let kind = authority
                        .relation_arena
                        .get(&id)
                        .and_then(|slot| slot.kind_id())
                        .ok_or_else(|| refusal(Cause::IntegrityMismatch))?;
                    expected.entry(kind).or_default().push(id);
                }
                let supplied = adjacency
                    .checkpoint_kind_memberships(AdjacencyKindBasis::Current)
                    .into_iter()
                    .filter(|(_, ids)| !ids.is_empty())
                    .collect::<BTreeMap<_, _>>();
                if expected != supplied {
                    return Err(refusal(Cause::IntegrityMismatch).into());
                }
            }
        }
        verified.insert(partition.partition_id);
        Ok(())
    }

    pub(super) fn admit_full(
        &self,
        region: &Arc<ResidentRegion>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<Arc<ResidentRegion>, Failure> {
        self.verify_head(region, execution, limit)?;
        let refusal = |cause| unavailable(region.partition.partition_id, cause);
        let mut authority =
            BTreeMap::from([(region.partition.partition_id, region.partition.clone())]);
        if let Some(regions) = self.regions.get() {
            for (&id, (relations, cell)) in regions {
                if id == region.partition.partition_id || *relations == 0 {
                    continue;
                }
                let cell = cell
                    .upgrade()
                    .ok_or_else(|| refusal(Cause::SourceUnavailable))?;
                let raw = cell.raw_full(execution, limit)?;
                authority.insert(id, raw.partition.clone());
            }
        }
        crate::storage::partition::rebuild_adjacency_kind_buckets(&mut authority)
            .map_err(|_| refusal(Cause::IntegrityMismatch))?;
        let partition = authority
            .remove(&region.partition.partition_id)
            .ok_or_else(|| refusal(Cause::Damaged))?;
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

fn unavailable(partition_id: PartitionId, cause: Cause) -> RelationalStorageUnavailable {
    RelationalStorageUnavailable {
        subject: RelationalStorageSubject::Region(partition_id),
        cause,
    }
}
