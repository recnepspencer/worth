//! Resolve visible aspect state from its original selected slot.
use worth_foundational::facade::AuthoritativeRecordAspectState;

use crate::identity::data::{EntityId, RelationId};

use super::slot_resolution::AspectLocation;
use super::StructuralStateView;

impl<'state> StructuralStateView<'state> {
    pub(crate) fn entity_aspect_state(
        &self,
        entity_id: EntityId,
    ) -> Option<&'state AuthoritativeRecordAspectState> {
        self.entity_aspect_at(entity_id, self.locate_entity_aspect(entity_id)?)
    }

    pub(crate) fn entity_aspect_at(
        &self,
        entity_id: EntityId,
        location: AspectLocation,
    ) -> Option<&'state AuthoritativeRecordAspectState> {
        self.partition_from_source(entity_id.partition_id, location.source)?
            .entity_arena()
            .current_metadata_at(entity_id.slot_index())?
            .authoritative_aspect_state()
    }

    pub(crate) fn locate_entity_aspect(&self, entity_id: EntityId) -> Option<AspectLocation> {
        let slot = entity_id.slot_index();
        let (source, partition) =
            self.entity_partition_source_for_slot(entity_id.partition_id, slot)?;
        let metadata = crate::storage::overlay::record_observation::visible_record_metadata(
            partition.entity_arena(),
            entity_id,
            self.version_id(),
        )?;
        metadata.authoritative_aspect_state()?;
        Some(AspectLocation { source })
    }

    pub(crate) fn relation_aspect_state(
        &self,
        relation_id: RelationId,
    ) -> Option<&'state AuthoritativeRecordAspectState> {
        self.relation_aspect_at(relation_id, self.locate_relation_aspect(relation_id)?)
    }

    pub(crate) fn relation_aspect_at(
        &self,
        relation_id: RelationId,
        location: AspectLocation,
    ) -> Option<&'state AuthoritativeRecordAspectState> {
        self.partition_from_source(relation_id.partition_id, location.source)?
            .relation_arena()
            .current_metadata_at(relation_id.slot_index())?
            .authoritative_aspect_state()
    }

    pub(crate) fn locate_relation_aspect(&self, relation_id: RelationId) -> Option<AspectLocation> {
        let slot = relation_id.slot_index();
        let (source, partition) =
            self.relation_partition_source_for_slot(relation_id.partition_id, slot)?;
        let metadata = crate::storage::overlay::record_observation::visible_record_metadata(
            partition.relation_arena(),
            relation_id,
            self.version_id(),
        )?;
        metadata.authoritative_aspect_state()?;
        Some(AspectLocation { source })
    }
}
