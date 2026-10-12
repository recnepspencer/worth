use std::sync::Arc;

use crate::branch::RelationalBranchRoot;
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};
use crate::storage::data::{EntityReadRecord, RelationReadRecord};
use crate::storage::overlay::PartitionAccess;

use super::read_record_identity_ordering::{
    authoritative_entity_records_are_identity_ordered,
    authoritative_relation_records_are_identity_ordered, relation_identity_order_key,
};
use super::VisibilityProjectionView;

pub(super) struct ExactProjectionReader<'view> {
    view: &'view VisibilityProjectionView,
    root: &'view Arc<RelationalBranchRoot>,
}

impl<'view> ExactProjectionReader<'view> {
    pub(super) const fn new(
        view: &'view VisibilityProjectionView,
        root: &'view Arc<RelationalBranchRoot>,
    ) -> Self {
        Self { view, root }
    }

    pub(super) fn entity_records(&self, kind_id: KindId) -> Vec<EntityReadRecord> {
        self.all_entity_records()
            .into_iter()
            .filter(|record| record.kind.kind_id == kind_id)
            .collect()
    }

    pub(super) fn entity_record(&self, entity_id: EntityId) -> Option<EntityReadRecord> {
        self.view
            .reader()
            .authoritative_entity_record_for_id_from_exact_state(
                self.root.as_ref(),
                self.root.schema_authority().registry(),
                entity_id,
            )
    }

    pub(super) fn all_entity_records(&self) -> Vec<EntityReadRecord> {
        let records = self
            .root
            .partition_ids()
            .into_iter()
            .flat_map(|partition_id| self.all_entity_records_in(partition_id))
            .collect::<Vec<_>>();
        debug_assert!(authoritative_entity_records_are_identity_ordered(&records));
        records
    }

    pub(super) fn entity_records_in(
        &self,
        partition_id: PartitionId,
        kind_id: KindId,
    ) -> Vec<EntityReadRecord> {
        self.all_entity_records_in(partition_id)
            .into_iter()
            .filter(|record| record.kind.kind_id == kind_id)
            .collect()
    }

    pub(super) fn relation_records(&self, kind_id: KindId) -> Vec<RelationReadRecord> {
        let mut records = self
            .all_relation_records()
            .into_iter()
            .filter(|record| record.kind.kind_id == kind_id)
            .collect::<Vec<_>>();
        records.sort_by_key(relation_identity_order_key);
        records
    }

    pub(super) fn relation_record(&self, relation_id: RelationId) -> Option<RelationReadRecord> {
        self.view
            .reader()
            .authoritative_relation_record_for_id_from_exact_state(
                self.root.as_ref(),
                self.root.schema_authority().registry(),
                relation_id,
            )
    }

    pub(super) fn all_relation_records(&self) -> Vec<RelationReadRecord> {
        let mut records = self
            .root
            .partition_ids()
            .into_iter()
            .flat_map(|partition_id| self.all_relation_records_in(partition_id))
            .collect::<Vec<_>>();
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
        let mut records = self
            .all_relation_records_in(partition_id)
            .into_iter()
            .filter(|record| record.kind.kind_id == kind_id)
            .collect::<Vec<_>>();
        records.sort_by_key(relation_identity_order_key);
        records
    }

    fn all_entity_records_in(&self, partition_id: PartitionId) -> Vec<EntityReadRecord> {
        let Some(partition) = self.root.get_partition(partition_id) else {
            return Vec::new();
        };
        partition
            .entity_arena
            .live_bitset
            .iter_set_slots()
            .into_iter()
            .filter_map(|slot| self.entity_record(EntityId::new(partition_id, slot as u64, 0)))
            .collect()
    }

    fn all_relation_records_in(&self, partition_id: PartitionId) -> Vec<RelationReadRecord> {
        let Some(partition) = self.root.get_partition(partition_id) else {
            return Vec::new();
        };
        partition
            .relation_arena
            .live_bitset
            .iter_set_slots()
            .into_iter()
            .filter_map(|slot| self.relation_record(RelationId::new(partition_id, slot as u64, 0)))
            .collect()
    }
}
