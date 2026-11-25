//! Materialized records owned by one issued snapshot handle.
use crate::identity::data::{EntityId, RelationId};
use crate::query::data::{
    deterministic_query_fragment_key, QueryFragmentCounters, QueryWorkerFragment,
};
use crate::snapshots::data::SnapshotHandle;
use crate::storage::data::read_records::{EntityReadRecord, RelationReadRecord};
use crate::transactions::data::primitives::RecordRef;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalReadView {
    pub(crate) snapshot: SnapshotHandle,
    pub(crate) entities: Vec<EntityReadRecord>,
    pub(crate) relations: Vec<RelationReadRecord>,
}

impl RelationalReadView {
    pub fn snapshot(&self) -> &SnapshotHandle {
        &self.snapshot
    }

    pub fn entities(&self) -> &[EntityReadRecord] {
        &self.entities
    }

    pub fn relations(&self) -> &[RelationReadRecord] {
        &self.relations
    }

    pub fn execute_planned_packet_fragment(
        &self,
        plan_key: crate::query::data::DeterministicQueryPlanKey,
        ordering: crate::query::data::QueryOrderingContract,
        targets: &[RecordRef],
        fragment_ordinal: u64,
    ) -> Option<QueryWorkerFragment> {
        let mut entities = Vec::new();
        let mut relations = Vec::new();
        let mut touched_partitions = std::collections::BTreeSet::new();

        for target in targets {
            match target {
                RecordRef::Entity(entity_id) => {
                    touched_partitions.insert(entity_id.partition_id);
                    if let Some(record) = self.get_entity(*entity_id) {
                        entities.push(record.clone());
                    }
                }
                RecordRef::Relation(relation_id) => {
                    touched_partitions.insert(relation_id.partition_id);
                    if let Some(record) = self.get_relation(*relation_id) {
                        relations.push(record.clone());
                    }
                }
            }
        }
        let authoritative_entity_records_emitted = entities.len();
        let authoritative_relation_records_emitted = relations.len();

        Some(QueryWorkerFragment {
            plan_key,
            fragment_key: deterministic_query_fragment_key(plan_key, fragment_ordinal),
            ordering,
            entities,
            relations,
            counters: QueryFragmentCounters {
                target_count: targets.len(),
                authoritative_entity_records_emitted,
                authoritative_relation_records_emitted,
                touched_partitions: touched_partitions.len(),
            },
            traversal_basis: None,
        })
    }

    pub fn get_entity(&self, entity_id: EntityId) -> Option<&EntityReadRecord> {
        find_entity(&self.entities, entity_id)
    }

    pub fn get_relation(&self, relation_id: RelationId) -> Option<&RelationReadRecord> {
        find_relation(&self.relations, relation_id)
    }
}

pub fn find_entity(records: &[EntityReadRecord], entity_id: EntityId) -> Option<&EntityReadRecord> {
    records
        .binary_search_by_key(&entity_id, |record| record.entity_id)
        .ok()
        .map(|index| &records[index])
}

pub fn find_relation(
    records: &[RelationReadRecord],
    relation_id: RelationId,
) -> Option<&RelationReadRecord> {
    records
        .binary_search_by_key(&relation_id, |record| record.relation_id)
        .ok()
        .map(|index| &records[index])
}

pub trait RelationalReadViewFieldsPort {
    fn owner_entities(&self) -> &Vec<EntityReadRecord>;
    fn owner_relations(&self) -> &Vec<RelationReadRecord>;
    fn owner_snapshot(&self) -> &SnapshotHandle;
}

impl RelationalReadViewFieldsPort for RelationalReadView {
    fn owner_entities(&self) -> &Vec<EntityReadRecord> {
        &self.entities
    }
    fn owner_relations(&self) -> &Vec<RelationReadRecord> {
        &self.relations
    }
    fn owner_snapshot(&self) -> &SnapshotHandle {
        &self.snapshot
    }
}
