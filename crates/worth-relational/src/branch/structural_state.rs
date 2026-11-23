//! Exact structural reads over the selected native partitions and version.
use crate::storage::overlay::CurrentPartitionAccess;
use crate::storage::residency::RelationalRecordHeadArenaView;
use crate::storage::substrate::{EntityRecordKind, HistoricalMetadata, RelationRecordKind};

mod aspect_state;
mod logical_slots;
pub(crate) mod slot_resolution;
mod structural_adjacency;

#[derive(Clone)]
pub(crate) struct StructuralStateView<'state> {
    state: &'state dyn CurrentPartitionAccess,
    version_id: crate::identity::data::VersionId,
}

impl<'state> StructuralStateView<'state> {
    pub(crate) fn new(
        state: &'state dyn CurrentPartitionAccess,
        version_id: crate::identity::data::VersionId,
    ) -> Self {
        Self { state, version_id }
    }

    pub(crate) fn state(&self) -> &'state dyn CurrentPartitionAccess {
        self.state
    }

    pub(crate) fn version_id(&self) -> crate::identity::data::VersionId {
        self.version_id
    }

    pub(crate) fn entity_metadata(
        &self,
        entity_id: crate::identity::data::EntityId,
    ) -> Option<VisibleEntityMetadata> {
        let slot = entity_id.slot_index();
        let partition = self.entity_partition_for_slot(entity_id.partition_id, slot)?;
        if partition
            .entity_arena()
            .get(&entity_id)
            .map(|slot_view| slot_view.generation())
            != Some(entity_id.generation_value())
        {
            return None;
        }
        self.entity_metadata_at(&partition.entity_arena(), entity_id.partition_id, slot)
    }

    pub(crate) fn entity_metadata_at(
        &self,
        arena: &RelationalRecordHeadArenaView<'state, EntityRecordKind>,
        partition_id: crate::identity::data::PartitionId,
        slot: usize,
    ) -> Option<VisibleEntityMetadata> {
        let metadata = arena
            .current_metadata_at(slot)
            .filter(|metadata| self.metadata_is_visible(*metadata))?;
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
        self.entity_metadata_at(&partition.entity_arena(), partition_id, slot)
    }

    pub(crate) fn relation_metadata(
        &self,
        relation_id: crate::identity::data::RelationId,
    ) -> Option<VisibleRelationMetadata> {
        let slot = relation_id.slot_index();
        let partition = self.relation_partition_for_slot(relation_id.partition_id, slot)?;
        if partition
            .relation_arena()
            .get(&relation_id)
            .map(|slot_view| slot_view.generation())
            != Some(relation_id.generation_value())
        {
            return None;
        }
        self.relation_metadata_at(&partition.relation_arena(), relation_id.partition_id, slot)
    }

    pub(crate) fn relation_metadata_at(
        &self,
        arena: &RelationalRecordHeadArenaView<'state, RelationRecordKind>,
        partition_id: crate::identity::data::PartitionId,
        slot: usize,
    ) -> Option<VisibleRelationMetadata> {
        let metadata = arena
            .current_metadata_at(slot)
            .filter(|metadata| self.metadata_is_visible(*metadata))?;
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

    pub(crate) fn relation_metadata_for_slot(
        &self,
        partition_id: crate::identity::data::PartitionId,
        slot: usize,
    ) -> Option<VisibleRelationMetadata> {
        let partition = self.relation_partition_for_slot(partition_id, slot)?;
        self.relation_metadata_at(&partition.relation_arena(), partition_id, slot)
    }

    pub(crate) fn metadata_is_visible<T: HistoricalMetadata>(&self, metadata: &T) -> bool {
        crate::storage::overlay::record_observation::metadata_is_visible(metadata, self.version_id)
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
