use crate::storage::overlay::PartitionAccess;
use crate::storage::substrate::EntityArena;
use crate::storage::substrate::HistoricalMetadata;
use std::sync::Arc;

mod aspect_state;
pub(crate) mod slot_resolution;
mod structural_adjacency;

#[derive(Clone)]
pub(crate) struct InvariantStateView<'state> {
    state: &'state dyn PartitionAccess,
    version_id: crate::identity::data::VersionId,
    candidate_inputs: Option<(
        std::sync::Arc<super::input_preparation::SharedCandidateInputs>,
        super::input_preparation::CandidateInputBasis,
    )>,
}

impl<'state> InvariantStateView<'state> {
    pub(crate) fn new(
        state: &'state dyn PartitionAccess,
        version_id: crate::identity::data::VersionId,
    ) -> Self {
        Self {
            state,
            version_id,
            candidate_inputs: None,
        }
    }

    pub(crate) fn with_candidate_inputs(
        mut self,
        inputs: Option<std::sync::Arc<super::input_preparation::SharedCandidateInputs>>,
        basis: super::input_preparation::CandidateInputBasis,
    ) -> Self {
        self.candidate_inputs = inputs.map(|inputs| (inputs, basis));
        self
    }

    pub(crate) fn candidate_inputs(
        &self,
    ) -> Option<&(
        std::sync::Arc<super::input_preparation::SharedCandidateInputs>,
        super::input_preparation::CandidateInputBasis,
    )> {
        self.candidate_inputs.as_ref()
    }

    pub(crate) fn state(&self) -> &'state dyn PartitionAccess {
        self.state
    }

    pub(crate) fn version_id(&self) -> crate::identity::data::VersionId {
        self.version_id
    }

    fn touched_partitions(&self) -> Option<Arc<[crate::identity::data::PartitionId]>> {
        match &self.candidate_inputs {
            Some((inputs, basis)) => inputs.touched_partitions(*basis, self.version_id, || {
                self.state.touched_partition_ids()
            }),
            None => self.state.touched_partition_ids().map(Arc::from),
        }
    }

    fn touched_entity_slots(
        &self,
        partition: crate::identity::data::PartitionId,
    ) -> Option<Arc<[usize]>> {
        match &self.candidate_inputs {
            Some((inputs, basis)) => {
                inputs.touched_entity_slots(*basis, self.version_id, partition, || {
                    self.state.touched_entity_slots(partition)
                })
            }
            None => self.state.touched_entity_slots(partition).map(Arc::from),
        }
    }

    fn touched_relation_slots(
        &self,
        partition: crate::identity::data::PartitionId,
    ) -> Option<Arc<[usize]>> {
        match &self.candidate_inputs {
            Some((inputs, basis)) => {
                inputs.touched_relation_slots(*basis, self.version_id, partition, || {
                    self.state.touched_relation_slots(partition)
                })
            }
            None => self.state.touched_relation_slots(partition).map(Arc::from),
        }
    }

    pub(crate) fn touched_visible_entity_ids_with_budget(
        &self,
        mut charge: impl FnMut(usize) -> bool,
    ) -> Option<Arc<[crate::identity::data::EntityId]>> {
        let mut sources = Vec::new();
        let partition_ids = self.touched_partitions()?;
        for partition_id in partition_ids.iter().copied() {
            let partition = self.state.get_partition(partition_id)?;
            let Some(slots) = self.touched_entity_slots(partition_id) else {
                continue;
            };
            if !charge(slots.len()) {
                return None;
            }
            sources.push((partition_id, partition, slots));
        }
        (!sources.is_empty()).then(|| {
            let gather = || {
                let mut ids = Vec::new();
                for (partition_id, partition, slots) in sources {
                    for slot in slots.iter().copied() {
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
                ids
            };
            match &self.candidate_inputs {
                Some((inputs, basis)) => inputs.touched_entities(*basis, self.version_id, gather),
                None => Arc::from(gather()),
            }
        })
    }

    pub(crate) fn entity_metadata(
        &self,
        entity_id: crate::identity::data::EntityId,
    ) -> Option<VisibleEntityMetadata> {
        if let Some((inputs, basis)) = &self.candidate_inputs {
            return inputs.entity(*basis, self.version_id, entity_id, || {
                self.read_entity_metadata(entity_id)
            });
        }
        self.read_entity_metadata(entity_id)
    }

    fn read_entity_metadata(
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
        if let Some((inputs, basis)) = &self.candidate_inputs {
            return inputs.relation(*basis, self.version_id, relation_id, || {
                self.read_relation_metadata(relation_id)
            });
        }
        self.read_relation_metadata(relation_id)
    }

    fn read_relation_metadata(
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

    pub(crate) fn touched_visible_relation_ids_with_budget(
        &self,
        mut charge: impl FnMut(usize) -> bool,
    ) -> Option<Arc<[crate::identity::data::RelationId]>> {
        let mut sources = Vec::new();
        let partition_ids = self.touched_partitions()?;
        for partition_id in partition_ids.iter().copied() {
            let partition = self.state.get_partition(partition_id)?;
            let Some(slots) = self.touched_relation_slots(partition_id) else {
                continue;
            };
            if !charge(slots.len()) {
                return None;
            }
            sources.push((partition_id, partition, slots));
        }
        (!sources.is_empty()).then(|| {
            let gather = || {
                let mut ids = Vec::new();
                for (partition_id, partition, slots) in sources {
                    for slot in slots.iter().copied() {
                        let Some(metadata) = self.relation_metadata_at(
                            &partition.relation_arena,
                            partition_id,
                            slot,
                        ) else {
                            continue;
                        };
                        ids.push(metadata.relation_id);
                    }
                }
                ids
            };
            match &self.candidate_inputs {
                Some((inputs, basis)) => inputs.touched_relations(*basis, self.version_id, gather),
                None => Arc::from(gather()),
            }
        })
    }

    fn visible_entity_metadata<'history>(
        &self,
        history: &'history crate::storage::substrate::SharedColumn<
            crate::storage::substrate::VersionedEntityMetadata,
        >,
    ) -> Option<&'history crate::storage::substrate::VersionedEntityMetadata> {
        self.visible_metadata_index(history)
            .map(|index| &history[index])
    }

    fn visible_relation_metadata<'history>(
        &self,
        history: &'history crate::storage::substrate::SharedColumn<
            crate::storage::substrate::VersionedRelationMetadata,
        >,
    ) -> Option<&'history crate::storage::substrate::VersionedRelationMetadata> {
        self.visible_metadata_index(history)
            .map(|index| &history[index])
    }

    fn visible_metadata_index<T: HistoricalMetadata + Clone>(
        &self,
        history: &crate::storage::substrate::SharedColumn<T>,
    ) -> Option<usize> {
        let end = history.partition_point(|entry| entry.effective_at() <= self.version_id);
        (0..end).rev().find(|&index| {
            let entry = &history[index];
            entry.effective_at() <= self.version_id
                && entry
                    .retired_at()
                    .is_none_or(|retired| self.version_id < retired)
        })
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
