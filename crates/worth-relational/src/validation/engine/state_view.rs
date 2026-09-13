use worth_foundational::facade::AuthoritativeRecordAspectState;

use crate::storage::overlay::PartitionAccess;
use crate::storage::substrate::EntityArena;
use crate::storage::substrate::HistoricalMetadata;

mod structural_adjacency;

#[derive(Clone, Copy)]
pub(crate) struct InvariantStateView<'state> {
    state: &'state dyn PartitionAccess,
    version_id: crate::identity::data::VersionId,
}

impl<'state> InvariantStateView<'state> {
    pub(crate) fn new(
        state: &'state dyn PartitionAccess,
        version_id: crate::identity::data::VersionId,
    ) -> Self {
        Self { state, version_id }
    }

    pub(crate) fn state(&self) -> &'state dyn PartitionAccess {
        self.state
    }

    pub(crate) fn version_id(&self) -> crate::identity::data::VersionId {
        self.version_id
    }

    pub(crate) fn touched_visible_entity_ids(
        &self,
    ) -> Option<Vec<crate::identity::data::EntityId>> {
        self.touched_visible_entity_ids_with_budget(|_| true)
    }

    pub(crate) fn touched_visible_entity_ids_with_budget(
        &self,
        mut charge: impl FnMut(usize) -> bool,
    ) -> Option<Vec<crate::identity::data::EntityId>> {
        let mut ids = Vec::new();
        let mut saw_any = false;
        let partition_ids = self.state.touched_partition_ids()?;
        for partition_id in partition_ids {
            let partition = self.state.get_partition(partition_id)?;
            let Some(slots) = self.state.touched_entity_slots(partition_id) else {
                continue;
            };
            if !charge(slots.len()) {
                return None;
            }
            saw_any = true;
            for slot in slots {
                let Some(metadata) = partition
                    .entity_arena
                    .metadata_history_at(slot)
                    .and_then(|history| self.visible_entity_metadata(history))
                else {
                    continue;
                };
                ids.push(crate::identity::data::EntityId::new(
                    partition_id,
                    slot as u64,
                    metadata.generation,
                ));
            }
        }
        if saw_any {
            Some(ids)
        } else {
            None
        }
    }

    pub(crate) fn entity_aspect_state(
        &self,
        entity_id: crate::identity::data::EntityId,
    ) -> Option<&'state AuthoritativeRecordAspectState> {
        let slot = entity_id.slot_index();
        let partition = self.entity_partition_for_slot(entity_id.partition_id, slot)?;
        if partition
            .entity_arena
            .get(&entity_id)
            .map(|slot_view| slot_view.generation())
            != Some(entity_id.generation_value())
        {
            return None;
        }
        partition
            .entity_arena
            .metadata_history_at(slot)
            .and_then(|history| self.visible_entity_metadata(history))
            .and_then(|metadata| metadata.authoritative_aspect_state.as_ref())
    }

    pub(crate) fn relation_aspect_state(
        &self,
        relation_id: crate::identity::data::RelationId,
    ) -> Option<&'state AuthoritativeRecordAspectState> {
        let slot = relation_id.slot_index();
        let partition = self.relation_partition_for_slot(relation_id.partition_id, slot)?;
        if partition
            .relation_arena
            .get(&relation_id)
            .map(|slot_view| slot_view.generation())
            != Some(relation_id.generation_value())
        {
            return None;
        }
        partition
            .relation_arena
            .metadata_history_at(slot)
            .and_then(|history| self.visible_relation_metadata(history))
            .and_then(|metadata| metadata.authoritative_aspect_state.as_ref())
    }

    pub(crate) fn entity_metadata(
        &self,
        entity_id: crate::identity::data::EntityId,
    ) -> Option<VisibleEntityMetadata> {
        let slot = entity_id.slot_index();
        let partition = self.entity_partition_for_slot(entity_id.partition_id, slot)?;
        if partition
            .entity_arena
            .get(&entity_id)
            .map(|slot_view| slot_view.generation())
            != Some(entity_id.generation_value())
        {
            return None;
        }
        self.entity_metadata_at(&partition.entity_arena, entity_id.partition_id, slot)
    }

    pub(crate) fn entity_visible_at_version(&self, arena: &EntityArena, slot: usize) -> bool {
        arena
            .metadata_history_at(slot)
            .and_then(|history| self.visible_entity_metadata(history))
            .is_some()
    }

    pub(crate) fn entity_slot_scan_count(
        &self,
        partition_id: crate::identity::data::PartitionId,
    ) -> Option<usize> {
        let visible_partition = self.state.get_partition(partition_id)?;
        let staged_slots = visible_partition.entity_arena.slot_count();
        let base_slots = self
            .state
            .base_partition(partition_id)
            .map(|partition| partition.entity_arena.slot_count())
            .unwrap_or(0);
        Some(staged_slots.max(base_slots))
    }

    pub(crate) fn entity_metadata_at(
        &self,
        arena: &'state EntityArena,
        partition_id: crate::identity::data::PartitionId,
        slot: usize,
    ) -> Option<VisibleEntityMetadata> {
        let metadata = arena
            .metadata_history_at(slot)
            .and_then(|history| self.visible_entity_metadata(history))?;
        Some(VisibleEntityMetadata {
            entity_id: crate::identity::data::EntityId::new(
                partition_id,
                slot as u64,
                metadata.generation,
            ),
            kind_id: metadata.kind_id,
        })
    }

    pub(crate) fn entity_metadata_for_slot(
        &self,
        partition_id: crate::identity::data::PartitionId,
        slot: usize,
    ) -> Option<VisibleEntityMetadata> {
        let partition = self.entity_partition_for_slot(partition_id, slot)?;
        self.entity_metadata_at(&partition.entity_arena, partition_id, slot)
    }

    pub(crate) fn relation_metadata(
        &self,
        relation_id: crate::identity::data::RelationId,
    ) -> Option<VisibleRelationMetadata> {
        let slot = relation_id.slot_index();
        let partition = self.relation_partition_for_slot(relation_id.partition_id, slot)?;
        if partition
            .relation_arena
            .get(&relation_id)
            .map(|slot_view| slot_view.generation())
            != Some(relation_id.generation_value())
        {
            return None;
        }
        self.relation_metadata_at(&partition.relation_arena, relation_id.partition_id, slot)
    }

    pub(crate) fn relation_metadata_at(
        &self,
        arena: &'state crate::storage::substrate::RelationArena,
        partition_id: crate::identity::data::PartitionId,
        slot: usize,
    ) -> Option<VisibleRelationMetadata> {
        let metadata = arena
            .metadata_history_at(slot)
            .and_then(|history| self.visible_relation_metadata(history))?;
        Some(VisibleRelationMetadata {
            relation_id: crate::identity::data::RelationId::new(
                partition_id,
                slot as u64,
                metadata.generation,
            ),
            kind_id: metadata.kind_id,
            source: metadata.endpoints.source,
            target: metadata.endpoints.target,
        })
    }

    pub(crate) fn relation_slot_scan_count(
        &self,
        partition_id: crate::identity::data::PartitionId,
    ) -> Option<usize> {
        let visible_partition = self.state.get_partition(partition_id)?;
        let staged_slots = visible_partition.relation_arena.slot_count();
        let base_slots = self
            .state
            .base_partition(partition_id)
            .map(|partition| partition.relation_arena.slot_count())
            .unwrap_or(0);
        Some(staged_slots.max(base_slots))
    }

    pub(crate) fn relation_metadata_for_slot(
        &self,
        partition_id: crate::identity::data::PartitionId,
        slot: usize,
    ) -> Option<VisibleRelationMetadata> {
        let partition = self.relation_partition_for_slot(partition_id, slot)?;
        self.relation_metadata_at(&partition.relation_arena, partition_id, slot)
    }

    pub(crate) fn touched_visible_relation_ids(
        &self,
    ) -> Option<Vec<crate::identity::data::RelationId>> {
        self.touched_visible_relation_ids_with_budget(|_| true)
    }

    pub(crate) fn touched_visible_relation_ids_with_budget(
        &self,
        mut charge: impl FnMut(usize) -> bool,
    ) -> Option<Vec<crate::identity::data::RelationId>> {
        let mut ids = Vec::new();
        let mut saw_any = false;
        let partition_ids = self.state.touched_partition_ids()?;
        for partition_id in partition_ids {
            let partition = self.state.get_partition(partition_id)?;
            let Some(slots) = self.state.touched_relation_slots(partition_id) else {
                continue;
            };
            if !charge(slots.len()) {
                return None;
            }
            saw_any = true;
            for slot in slots {
                let Some(metadata) =
                    self.relation_metadata_at(&partition.relation_arena, partition_id, slot)
                else {
                    continue;
                };
                ids.push(metadata.relation_id);
            }
        }
        if saw_any {
            Some(ids)
        } else {
            None
        }
    }

    fn visible_entity_metadata(
        &self,
        history: &'state [crate::storage::substrate::VersionedEntityMetadata],
    ) -> Option<&'state crate::storage::substrate::VersionedEntityMetadata> {
        let end = history.partition_point(|entry| entry.effective_at() <= self.version_id);
        history[..end].iter().rev().find(|entry| {
            entry.effective_at() <= self.version_id
                && entry
                    .retired_at()
                    .is_none_or(|retired| self.version_id < retired)
        })
    }

    fn visible_relation_metadata(
        &self,
        history: &'state [crate::storage::substrate::VersionedRelationMetadata],
    ) -> Option<&'state crate::storage::substrate::VersionedRelationMetadata> {
        let end = history.partition_point(|entry| entry.effective_at() <= self.version_id);
        history[..end].iter().rev().find(|entry| {
            entry.effective_at() <= self.version_id
                && entry
                    .retired_at()
                    .is_none_or(|retired| self.version_id < retired)
        })
    }

    fn entity_partition_for_slot(
        &self,
        partition_id: crate::identity::data::PartitionId,
        slot: usize,
    ) -> Option<&'state crate::storage::overlay::PartitionState> {
        let partition = self.state.get_partition(partition_id)?;
        if self.state.entity_slot_is_touched(partition_id, slot)
            || self.state.touched_entity_slots(partition_id).is_none()
        {
            return Some(partition);
        }
        self.state.base_partition(partition_id).or(Some(partition))
    }

    fn relation_partition_for_slot(
        &self,
        partition_id: crate::identity::data::PartitionId,
        slot: usize,
    ) -> Option<&'state crate::storage::overlay::PartitionState> {
        let partition = self.state.get_partition(partition_id)?;
        if self.state.relation_slot_is_touched(partition_id, slot)
            || (self.state.touched_relation_slots(partition_id).is_none()
                && partition.relation_arena.get_slot(slot).is_some())
            || (partition.relation_arena.get_slot(slot).is_some()
                && !partition.relation_overlay_is_sparse)
        {
            return Some(partition);
        }
        self.state.base_partition(partition_id).or(Some(partition))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct VisibleEntityMetadata {
    pub(crate) entity_id: crate::identity::data::EntityId,
    pub(crate) kind_id: crate::identity::data::KindId,
}

#[derive(Debug, Clone)]
pub(crate) struct VisibleRelationMetadata {
    pub(crate) relation_id: crate::identity::data::RelationId,
    pub(crate) kind_id: crate::identity::data::KindId,
    pub(crate) source: crate::identity::data::EntityId,
    pub(crate) target: crate::identity::data::EntityId,
}

#[cfg(test)]
mod tests;
