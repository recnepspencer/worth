use crate::branch::structural_state::StructuralStateView;
pub(crate) use crate::branch::structural_state::{VisibleEntityMetadata, VisibleRelationMetadata};
use crate::storage::overlay::CurrentPartitionAccess;
use std::sync::Arc;

mod aspect_state;
pub(crate) use crate::branch::structural_state::slot_resolution;
mod structural_adjacency;

#[derive(Clone)]
pub(crate) struct InvariantStateView<'state> {
    selected: StructuralStateView<'state>,
    candidate_inputs: Option<(
        std::sync::Arc<super::input_preparation::SharedCandidateInputs>,
        super::input_preparation::CandidateInputBasis,
    )>,
}

impl<'state> InvariantStateView<'state> {
    pub(crate) fn new(
        state: &'state dyn CurrentPartitionAccess,
        version_id: crate::identity::data::VersionId,
    ) -> Self {
        Self {
            selected: StructuralStateView::new(state, version_id),
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

    fn touched_partitions(&self) -> Option<Arc<[crate::identity::data::PartitionId]>> {
        match &self.candidate_inputs {
            Some((inputs, basis)) => inputs.touched_partitions(*basis, self.version_id(), || {
                self.state().touched_partition_ids()
            }),
            None => self.state().touched_partition_ids().map(Arc::from),
        }
    }

    fn touched_entity_slots(
        &self,
        partition: crate::identity::data::PartitionId,
    ) -> Option<Arc<[usize]>> {
        match &self.candidate_inputs {
            Some((inputs, basis)) => {
                inputs.touched_entity_slots(*basis, self.version_id(), partition, || {
                    self.state().touched_entity_slots(partition)
                })
            }
            None => self.state().touched_entity_slots(partition).map(Arc::from),
        }
    }

    fn touched_relation_slots(
        &self,
        partition: crate::identity::data::PartitionId,
    ) -> Option<Arc<[usize]>> {
        match &self.candidate_inputs {
            Some((inputs, basis)) => {
                inputs.touched_relation_slots(*basis, self.version_id(), partition, || {
                    self.state().touched_relation_slots(partition)
                })
            }
            None => self
                .state()
                .touched_relation_slots(partition)
                .map(Arc::from),
        }
    }

    pub(crate) fn touched_visible_entity_ids_with_budget(
        &self,
        mut charge: impl FnMut(usize) -> bool,
    ) -> Option<Arc<[crate::identity::data::EntityId]>> {
        let mut sources = Vec::new();
        let partition_ids = self.touched_partitions()?;
        for partition_id in partition_ids.iter().copied() {
            let partition = self.state().get_current_partition(partition_id)?;
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
                            .entity_arena()
                            .current_metadata_at(slot)
                            .filter(|metadata| self.metadata_is_visible(*metadata))
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
                Some((inputs, basis)) => inputs.touched_entities(*basis, self.version_id(), gather),
                None => Arc::from(gather()),
            }
        })
    }

    pub(crate) fn entity_metadata(
        &self,
        entity_id: crate::identity::data::EntityId,
    ) -> Option<VisibleEntityMetadata> {
        if let Some((inputs, basis)) = &self.candidate_inputs {
            return inputs.entity(*basis, self.version_id(), entity_id, || {
                self.selected.entity_metadata(entity_id)
            });
        }
        self.selected.entity_metadata(entity_id)
    }

    pub(crate) fn relation_metadata(
        &self,
        relation_id: crate::identity::data::RelationId,
    ) -> Option<VisibleRelationMetadata> {
        if let Some((inputs, basis)) = &self.candidate_inputs {
            return inputs.relation(*basis, self.version_id(), relation_id, || {
                self.selected.relation_metadata(relation_id)
            });
        }
        self.selected.relation_metadata(relation_id)
    }

    pub(crate) fn touched_visible_relation_ids_with_budget(
        &self,
        mut charge: impl FnMut(usize) -> bool,
    ) -> Option<Arc<[crate::identity::data::RelationId]>> {
        let mut sources = Vec::new();
        let partition_ids = self.touched_partitions()?;
        for partition_id in partition_ids.iter().copied() {
            let partition = self.state().get_current_partition(partition_id)?;
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
                            &partition.relation_arena(),
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
                Some((inputs, basis)) => {
                    inputs.touched_relations(*basis, self.version_id(), gather)
                }
                None => Arc::from(gather()),
            }
        })
    }
}

impl<'state> std::ops::Deref for InvariantStateView<'state> {
    type Target = StructuralStateView<'state>;

    fn deref(&self) -> &Self::Target {
        &self.selected
    }
}

#[cfg(test)]
mod tests;
