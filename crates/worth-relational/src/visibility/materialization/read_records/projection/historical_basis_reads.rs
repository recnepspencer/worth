use std::sync::Arc;

use crate::branch::RelationalBranchRoot;
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};
use crate::storage::data::{EntityReadRecord, RelationReadRecord};
use crate::storage::overlay::{PartitionAccess, PartitionState};
use crate::visibility::snapshot_states::HistoricalVisibilityBasis;

use super::read_record_identity_ordering::{
    authoritative_entity_records_are_identity_ordered,
    authoritative_relation_records_are_identity_ordered, relation_identity_order_key,
};
use super::VisibilityProjectionView;

pub(super) struct HistoricalProjectionReader<'view, 'runtime> {
    view: &'view VisibilityProjectionView<'runtime>,
    basis: &'view HistoricalVisibilityBasis,
}

pub(super) enum HistoricalProjectionStorage<'view> {
    Retained(&'view Arc<RelationalBranchRoot>),
    EmptyGenesis,
}

impl PartitionAccess for HistoricalProjectionStorage<'_> {
    fn get_partition(&self, partition_id: PartitionId) -> Option<&PartitionState> {
        match self {
            Self::Retained(root) => root.get_partition(partition_id),
            Self::EmptyGenesis => None,
        }
    }

    fn partition_ids(&self) -> Vec<PartitionId> {
        match self {
            Self::Retained(root) => root.partition_ids(),
            Self::EmptyGenesis => Vec::new(),
        }
    }
}

impl<'view, 'runtime> HistoricalProjectionReader<'view, 'runtime> {
    pub(super) const fn new(
        view: &'view VisibilityProjectionView<'runtime>,
        basis: &'view HistoricalVisibilityBasis,
    ) -> Self {
        Self { view, basis }
    }

    pub(super) fn entity_records(&self, kind_id: KindId) -> Vec<EntityReadRecord> {
        let storage = self.storage();
        let registry = self.registry();
        let mut records = Vec::new();
        for partition_id in storage.partition_ids() {
            records.extend(
                self.view
                    .reader()
                    .visible_entities_of_kind_in_partition_from_state_with_registry(
                        &storage,
                        registry,
                        partition_id,
                        kind_id,
                        self.view.version_id(),
                    ),
            );
        }
        debug_assert!(authoritative_entity_records_are_identity_ordered(&records));
        records
    }

    pub(super) fn entity_record(&self, entity_id: EntityId) -> Option<EntityReadRecord> {
        let storage = self.storage();
        let registry = self.registry();
        self.view
            .reader()
            .authoritative_entity_record_for_id_at_version_with_registry(
                &storage,
                registry,
                entity_id,
                self.view.version_id(),
            )
    }

    pub(super) fn all_entity_records(&self) -> Vec<EntityReadRecord> {
        let storage = self.storage();
        let mut records = Vec::new();
        for (partition_id, slots) in self
            .view
            .reader()
            .visible_entity_slots_from_state(&storage, self.view.version_id())
        {
            for slot in slots.iter_set_slots() {
                if let Some(record) =
                    self.entity_record(EntityId::new(partition_id, slot as u64, 0))
                {
                    records.push(record);
                }
            }
        }
        debug_assert!(authoritative_entity_records_are_identity_ordered(&records));
        records
    }

    pub(super) fn entity_records_in(
        &self,
        partition_id: PartitionId,
        kind_id: KindId,
    ) -> Vec<EntityReadRecord> {
        let storage = self.storage();
        let registry = self.registry();
        self.view
            .reader()
            .visible_entities_of_kind_in_partition_from_state_with_registry(
                &storage,
                registry,
                partition_id,
                kind_id,
                self.view.version_id(),
            )
    }

    pub(super) fn relation_records(&self, kind_id: KindId) -> Vec<RelationReadRecord> {
        let storage = self.storage();
        let registry = self.registry();
        let mut records = Vec::new();
        for partition_id in storage.partition_ids() {
            records.extend(
                self.view
                    .reader()
                    .visible_relations_of_kind_in_partition_from_state_with_registry(
                        &storage,
                        registry,
                        partition_id,
                        kind_id,
                        self.view.version_id(),
                    ),
            );
        }
        records.sort_by_key(relation_identity_order_key);
        records
    }

    pub(super) fn relation_record(&self, relation_id: RelationId) -> Option<RelationReadRecord> {
        let storage = self.storage();
        let registry = self.registry();
        self.view
            .reader()
            .authoritative_relation_record_for_id_at_version_with_registry(
                &storage,
                registry,
                relation_id,
                self.view.version_id(),
            )
    }

    pub(super) fn all_relation_records(&self) -> Vec<RelationReadRecord> {
        let storage = self.storage();
        let mut records = Vec::new();
        for (partition_id, slots) in self
            .view
            .reader()
            .visible_relation_slots_from_state(&storage, self.view.version_id())
        {
            for slot in slots.iter_set_slots() {
                if let Some(record) =
                    self.relation_record(RelationId::new(partition_id, slot as u64, 0))
                {
                    records.push(record);
                }
            }
        }
        records.sort_by_key(relation_identity_order_key);
        debug_assert!(authoritative_relation_records_are_identity_ordered(
            &records
        ));
        records
    }

    pub(super) fn relation_records_in(
        &self,
        partition_id: PartitionId,
        kind_id: KindId,
    ) -> Vec<RelationReadRecord> {
        let storage = self.storage();
        let registry = self.registry();
        let mut records = self
            .view
            .reader()
            .visible_relations_of_kind_in_partition_from_state_with_registry(
                &storage,
                registry,
                partition_id,
                kind_id,
                self.view.version_id(),
            );
        records.sort_by_key(relation_identity_order_key);
        records
    }

    pub(super) fn storage(&self) -> HistoricalProjectionStorage<'_> {
        match self.basis.root() {
            Some(root) => HistoricalProjectionStorage::Retained(root),
            None => HistoricalProjectionStorage::EmptyGenesis,
        }
    }

    pub(super) fn registry(&self) -> &crate::schema::data::RelationalSchemaRegistry {
        self.basis
            .root()
            .map_or(&self.view.runtime.config.schema.registry, |root| {
                root.schema_authority().registry()
            })
    }
}
