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
        self.replace_controlled(partition, previous, current, None)
            .expect("configuration directory replacement has no request refusal");
    }

    pub(crate) fn replaced_for_request(
        &mut self,
        partition: PartitionId,
        previous: Option<&RegionRecordDirectory>,
        current: Option<&RegionRecordDirectory>,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
        self.replace_controlled(partition, previous, current, Some(request))
    }

    fn replace_controlled(
        &mut self,
        partition: PartitionId,
        previous: Option<&RegionRecordDirectory>,
        current: Option<&RegionRecordDirectory>,
        request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
    ) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
        let empty = RegionRecordDirectory::empty(partition);
        let previous = previous.unwrap_or(&empty);
        let current = current.unwrap_or(&empty);
        for kind in current.changed_entity_kinds(previous) {
            let before = previous.entity_ids(kind).next().is_some();
            let after = current.entity_ids(kind).next().is_some();
            if before != after {
                update(&mut self.entities, kind, partition, after, request)?;
            }
        }
        for kind in current.changed_relation_kinds(previous) {
            let before = previous.relation_ids(kind).next().is_some();
            let after = current.relation_ids(kind).next().is_some();
            if before != after {
                update(&mut self.relations, kind, partition, after, request)?;
            }
        }
        Ok(())
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
    request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
    let mut regions = directory.get(&kind).cloned().unwrap_or_default();
    if present {
        if let Some(request) = request {
            regions.insert_for_request(partition, (), request)?;
        } else {
            regions.insert(partition, ());
        }
    } else {
        if let Some(request) = request {
            regions.remove_for_request(&partition, request)?;
        } else {
            regions.remove(&partition);
        }
    }
    if regions.is_empty() {
        if let Some(request) = request {
            directory.remove_for_request(&kind, request)?;
        } else {
            directory.remove(&kind);
        }
    } else {
        if let Some(request) = request {
            directory.insert_for_request(kind, regions, request)?;
        } else {
            directory.insert(kind, regions);
        }
    }
    Ok(())
}
