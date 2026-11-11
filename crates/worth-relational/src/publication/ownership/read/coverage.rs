//! Retained selector coverage includes both record claims and requested scopes.
use super::{EntityId, RecordRef, RelationId};
use std::collections::BTreeSet;

pub(super) fn covers(
    entities: &BTreeSet<EntityId>,
    relations: &BTreeSet<RelationId>,
    entity_scopes: &BTreeSet<EntityId>,
    relation_scopes: &BTreeSet<RelationId>,
    records: &[RecordRef],
    scopes: &[RecordRef],
) -> bool {
    records.iter().all(|r| match r {
        RecordRef::Entity(id) => entities.contains(id),
        RecordRef::Relation(id) => relations.contains(id),
    }) && scopes.iter().all(|r| match r {
        RecordRef::Entity(id) => entity_scopes.contains(id),
        RecordRef::Relation(id) => relation_scopes.contains(id),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn claim_coverage_does_not_cover_entity_or_relation_scope_selection() {
        let partition = crate::identity::data::PartitionId(1);
        let entity = EntityId::new(partition, 0, 1);
        let relation = RelationId::new(partition, 0, 1);
        let entities = BTreeSet::from([entity]);
        let relations = BTreeSet::from([relation]);
        let records = [RecordRef::Entity(entity), RecordRef::Relation(relation)];
        assert!(covers(
            &entities,
            &relations,
            &BTreeSet::new(),
            &BTreeSet::new(),
            &records,
            &[]
        ));
        assert!(!covers(
            &entities,
            &relations,
            &BTreeSet::new(),
            &relations,
            &records,
            &records[..1]
        ));
        assert!(!covers(
            &entities,
            &relations,
            &entities,
            &BTreeSet::new(),
            &records,
            &records[1..]
        ));
        assert!(covers(
            &entities, &relations, &entities, &relations, &records, &records
        ));
    }
}
