use worth_foundational::facade::AuthoritativeRecordAspectState;

use crate::identity::data::{EntityId, RelationId};

use super::InvariantStateView;

impl<'state> InvariantStateView<'state> {
    pub(crate) fn entity_aspect_state(
        &self,
        entity_id: EntityId,
    ) -> Option<&'state AuthoritativeRecordAspectState> {
        if let Some((inputs, basis)) = &self.candidate_inputs {
            return inputs.entity_aspect(*basis, self.version_id, entity_id, || {
                self.read_entity_aspect_state(entity_id)
            });
        }
        self.read_entity_aspect_state(entity_id)
    }

    fn read_entity_aspect_state(
        &self,
        entity_id: EntityId,
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
        relation_id: RelationId,
    ) -> Option<&'state AuthoritativeRecordAspectState> {
        if let Some((inputs, basis)) = &self.candidate_inputs {
            return inputs.relation_aspect(*basis, self.version_id, relation_id, || {
                self.read_relation_aspect_state(relation_id)
            });
        }
        self.read_relation_aspect_state(relation_id)
    }

    fn read_relation_aspect_state(
        &self,
        relation_id: RelationId,
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
}
