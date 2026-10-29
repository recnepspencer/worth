pub(crate) use std::collections::{BTreeMap, BTreeSet};

pub(crate) use super::super::definition::SupplyChainWorldDefinition;
pub(crate) use super::super::schema::{
    EntityRecord, RelationEdge, SchemaError, SchemaVersion, SupplyChainSchema,
};
pub(crate) use super::super::semantic_key::{AbsenceKind, EntityKey, RelationKey};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OracleState {
    pub(crate) schema: SupplyChainSchema,
    pub(crate) entities: BTreeMap<EntityKey, EntityRecord>,
    pub(crate) relations: BTreeMap<RelationKey, RelationEdge>,
    pub(crate) absent_entities: BTreeSet<EntityKey>,
    pub(crate) absent_relations: BTreeSet<RelationKey>,
}

impl OracleState {
    pub(crate) fn empty(schema: SupplyChainSchema) -> Self {
        Self {
            schema,
            entities: BTreeMap::new(),
            relations: BTreeMap::new(),
            absent_entities: BTreeSet::new(),
            absent_relations: BTreeSet::new(),
        }
    }

    pub(crate) fn from_definition(definition: &SupplyChainWorldDefinition) -> Self {
        Self {
            schema: definition.schema.clone(),
            entities: definition.entities.clone(),
            relations: definition.relations.clone(),
            absent_entities: BTreeSet::new(),
            absent_relations: BTreeSet::new(),
        }
    }

    pub(crate) fn schema_version(&self) -> SchemaVersion {
        self.schema.version
    }

    pub(crate) fn absence_marker(&self, key: EntityKey) -> Option<(AbsenceKind, EntityKey)> {
        self.absent_entities
            .contains(&key)
            .then_some((AbsenceKind::Entity, key))
    }

    pub(crate) fn relation_absence_marker(
        &self,
        key: RelationKey,
    ) -> Option<(AbsenceKind, RelationKey)> {
        self.absent_relations
            .contains(&key)
            .then_some((AbsenceKind::Relation, key))
    }

    pub(crate) fn entity(&self, key: EntityKey) -> Option<&EntityRecord> {
        self.entities.get(&key)
    }

    pub(crate) fn relation(&self, key: RelationKey) -> Option<&RelationEdge> {
        self.relations.get(&key)
    }

    pub(crate) fn replace_entity(&self, key: EntityKey, value: EntityRecord) -> Self {
        let mut next = self.clone();
        next.entities.insert(key, value);
        next.absent_entities.remove(&key);
        next
    }

    pub(crate) fn remove_entity(&self, key: EntityKey) -> Self {
        let mut next = self.clone();
        next.entities.remove(&key);
        next.absent_entities.insert(key);
        next
    }

    pub(crate) fn replace_relation(&self, edge: RelationEdge) -> Self {
        let mut next = self.clone();
        next.relations.insert(edge.key, edge);
        next.absent_relations.remove(&edge.key);
        next
    }

    pub(crate) fn remove_relation(&self, key: RelationKey) -> Self {
        let mut next = self.clone();
        next.relations.remove(&key);
        next.absent_relations.insert(key);
        next
    }

    pub(crate) fn upgrade_hazard_schema(&self) -> Self {
        let mut next = self.clone();
        next.schema = SupplyChainSchema::canonical(SchemaVersion::V2);
        next
    }

    pub(crate) fn validate_complete(&self) -> Result<(), SchemaError> {
        self.schema.validate_complete_world(
            &self.relations.values().copied().collect::<Vec<_>>(),
            &self.entities,
        )
    }
}
