//! Persistent kind-to-region dispatch, derived only from changed membership keys.
use super::RegionRecordDirectory;
use crate::identity::data::{KindId, PartitionId};
use crate::storage::substrate::SharedMap;

#[derive(Debug, Clone, Default)]
pub(crate) struct KindRegionDirectory {
    entities: SharedMap<KindId, SharedMap<PartitionId, ()>>,
    relations: SharedMap<KindId, SharedMap<PartitionId, ()>>,
}
impl KindRegionDirectory {
    pub(crate) fn replaced(
        &mut self,
        partition: PartitionId,
        previous: Option<&RegionRecordDirectory>,
        current: Option<&RegionRecordDirectory>,
    ) {
        let empty = RegionRecordDirectory::empty(partition);
        let previous = previous.unwrap_or(&empty);
        let current = current.unwrap_or(&empty);
        for kind in current.changed_entity_kinds(previous) {
            let before = previous.entity_ids(kind).next().is_some();
            let after = current.entity_ids(kind).next().is_some();
            if before != after {
                update(&mut self.entities, kind, partition, after);
            }
        }
        for kind in current.changed_relation_kinds(previous) {
            let before = previous.relation_ids(kind).next().is_some();
            let after = current.relation_ids(kind).next().is_some();
            if before != after {
                update(&mut self.relations, kind, partition, after);
            }
        }
    }
    pub(crate) fn entity_regions(&self, kind: KindId) -> impl Iterator<Item = PartitionId> + '_ {
        self.entities
            .get(&kind)
            .into_iter()
            .flat_map(|regions| regions.keys().copied())
    }
    pub(crate) fn relation_regions(&self, kind: KindId) -> impl Iterator<Item = PartitionId> + '_ {
        self.relations
            .get(&kind)
            .into_iter()
            .flat_map(|regions| regions.keys().copied())
    }
}
fn update(
    directory: &mut SharedMap<KindId, SharedMap<PartitionId, ()>>,
    kind: KindId,
    partition: PartitionId,
    present: bool,
) {
    let mut regions = directory.get(&kind).cloned().unwrap_or_default();
    if present {
        regions.insert(partition, ());
    } else {
        regions.remove(&partition);
    }
    if regions.is_empty() {
        directory.remove(&kind);
    } else {
        directory.insert(kind, regions);
    }
}
