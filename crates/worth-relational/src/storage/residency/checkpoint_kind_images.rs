//! Preserve authenticated projections across eager checkpoint image handoffs.
use super::kind_projection;
use crate::identity::data::PartitionId;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::overlay::PartitionState;
use crate::storage::partition::AdjacencyKindBasis;
use crate::storage::partition::RegionRelationProjection;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegionKinds {
    head: Vec<RegionRelationProjection>,
    history: Vec<RegionRelationProjection>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct CheckpointRegionKindImages(BTreeMap<PartitionId, RegionKinds>);

impl CheckpointRegionKindImages {
    pub(crate) fn capture<'a>(
        partitions: impl IntoIterator<Item = &'a PartitionState>,
    ) -> Result<Self, Cause> {
        let mut images = BTreeMap::new();
        for partition in partitions {
            let kinds = RegionKinds {
                head: kind_projection::capture(partition, AdjacencyKindBasis::Current)?,
                history: kind_projection::canonical_history(partition)?,
            };
            if images.insert(partition.partition_id, kinds).is_some() {
                return Err(Cause::Damaged);
            }
        }
        Ok(Self(images))
    }

    pub(crate) fn omit_deleted_current_records(
        &mut self,
        partitions: &[crate::durability::data::PartitionCheckpointImage],
    ) {
        let mut removed = std::collections::BTreeSet::new();
        for partition in partitions {
            for (index, life) in partition.relation_arena.lifecycle.iter().enumerate() {
                if *life == crate::storage::data::RecordLifecycleState::DeletedRetained {
                    let slot = partition
                        .relation_arena
                        .slots
                        .get(index)
                        .copied()
                        .unwrap_or(index as u64);
                    removed.insert(crate::identity::data::RelationId::new(
                        partition.partition_id,
                        slot,
                        partition.relation_arena.generations[index],
                    ));
                }
            }
        }
        for region in self.0.values_mut() {
            region
                .head
                .retain(|claim| !removed.contains(&claim.relation));
        }
    }
    pub(crate) fn restore(
        &self,
        partitions: &mut BTreeMap<PartitionId, PartitionState>,
    ) -> Result<(), Cause> {
        if self.0.len() != partitions.len() {
            return Err(Cause::Damaged);
        }
        for (id, partition) in partitions {
            let image = self.0.get(id).ok_or(Cause::Damaged)?;
            kind_projection::rebuild(partition, &image.head, AdjacencyKindBasis::Current)?;
            kind_projection::rebuild(partition, &image.history, AdjacencyKindBasis::Historical)?;
        }
        Ok(())
    }
}
