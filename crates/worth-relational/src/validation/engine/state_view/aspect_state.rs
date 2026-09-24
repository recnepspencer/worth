use worth_foundational::facade::AuthoritativeRecordAspectState;

use crate::identity::data::{EntityId, RelationId};

use super::InvariantStateView;

impl<'state> InvariantStateView<'state> {
    pub(crate) fn entity_aspect_state(
        &self,
        entity_id: EntityId,
    ) -> Option<&'state AuthoritativeRecordAspectState> {
        let locate = || self.locate_entity_aspect(entity_id);
        let index = match &self.candidate_inputs {
            Some((inputs, basis)) => {
                inputs.entity_aspect(*basis, self.version_id, entity_id, locate)
            }
            None => locate(),
        }?;
        self.entity_partition_for_slot(entity_id.partition_id, entity_id.slot_index())?
            .entity_arena
            .metadata_history_at(entity_id.slot_index())?
            .get(index)?
            .authoritative_aspect_state
            .as_ref()
    }

    fn locate_entity_aspect(&self, entity_id: EntityId) -> Option<usize> {
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
        let history = partition.entity_arena.metadata_history_at(slot)?;
        let index = self.visible_metadata_index(history)?;
        history[index].authoritative_aspect_state.as_ref()?;
        Some(index)
    }

    pub(crate) fn relation_aspect_state(
        &self,
        relation_id: RelationId,
    ) -> Option<&'state AuthoritativeRecordAspectState> {
        let locate = || self.locate_relation_aspect(relation_id);
        let index = match &self.candidate_inputs {
            Some((inputs, basis)) => {
                inputs.relation_aspect(*basis, self.version_id, relation_id, locate)
            }
            None => locate(),
        }?;
        self.relation_partition_for_slot(relation_id.partition_id, relation_id.slot_index())?
            .relation_arena
            .metadata_history_at(relation_id.slot_index())?
            .get(index)?
            .authoritative_aspect_state
            .as_ref()
    }

    fn locate_relation_aspect(&self, relation_id: RelationId) -> Option<usize> {
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
        let history = partition.relation_arena.metadata_history_at(slot)?;
        let index = self.visible_metadata_index(history)?;
        history[index].authoritative_aspect_state.as_ref()?;
        Some(index)
    }
}
