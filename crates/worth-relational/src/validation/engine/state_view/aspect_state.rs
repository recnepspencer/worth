use worth_foundational::facade::AuthoritativeRecordAspectState;

use crate::identity::data::{EntityId, RelationId};

use super::slot_resolution::AspectLocation;
use super::InvariantStateView;

impl<'state> InvariantStateView<'state> {
    pub(crate) fn entity_aspect_state(
        &self,
        entity_id: EntityId,
    ) -> Option<&'state AuthoritativeRecordAspectState> {
        let locate = || self.locate_entity_aspect(entity_id);
        let location = match &self.candidate_inputs {
            Some((inputs, basis)) => {
                inputs.entity_aspect(*basis, self.version_id, entity_id, locate)
            }
            None => locate(),
        }?;
        self.partition_from_source(entity_id.partition_id, location.source)?
            .entity_arena
            .metadata_history_at(entity_id.slot_index())?
            .get(location.history_index)?
            .authoritative_aspect_state
            .as_ref()
    }

    fn locate_entity_aspect(&self, entity_id: EntityId) -> Option<AspectLocation> {
        let slot = entity_id.slot_index();
        let (source, partition) =
            self.entity_partition_source_for_slot(entity_id.partition_id, slot)?;
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
        Some(AspectLocation {
            source,
            history_index: index,
        })
    }

    pub(crate) fn relation_aspect_state(
        &self,
        relation_id: RelationId,
    ) -> Option<&'state AuthoritativeRecordAspectState> {
        let locate = || self.locate_relation_aspect(relation_id);
        let location = match &self.candidate_inputs {
            Some((inputs, basis)) => {
                inputs.relation_aspect(*basis, self.version_id, relation_id, locate)
            }
            None => locate(),
        }?;
        self.partition_from_source(relation_id.partition_id, location.source)?
            .relation_arena
            .metadata_history_at(relation_id.slot_index())?
            .get(location.history_index)?
            .authoritative_aspect_state
            .as_ref()
    }

    fn locate_relation_aspect(&self, relation_id: RelationId) -> Option<AspectLocation> {
        let slot = relation_id.slot_index();
        let (source, partition) =
            self.relation_partition_source_for_slot(relation_id.partition_id, slot)?;
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
        Some(AspectLocation {
            source,
            history_index: index,
        })
    }
}
