pub use {std::collections::BTreeMap, std::collections::BTreeSet};

pub use crate::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition;
pub use {
    crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord,
    crate::relational_certification::ordinary_context::world::supply_chain::schema::RelationEdge,
    crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError,
    crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion,
    crate::relational_certification::ordinary_context::world::supply_chain::schema::SupplyChainSchema,
};
pub use {
    crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::AbsenceKind,
    crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey,
    crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleState {
    pub schema: SupplyChainSchema,
    pub entities: BTreeMap<EntityKey, EntityRecord>,
    pub relations: BTreeMap<RelationKey, RelationEdge>,
    pub absent_entities: BTreeSet<EntityKey>,
    pub absent_relations: BTreeSet<RelationKey>,
}

impl OracleState {
    pub fn empty(schema: SupplyChainSchema) -> Self {
        Self {
            schema,
            entities: std::collections::BTreeMap::new(),
            relations: std::collections::BTreeMap::new(),
            absent_entities: std::collections::BTreeSet::new(),
            absent_relations: std::collections::BTreeSet::new(),
        }
    }

    pub fn from_definition(definition: &SupplyChainWorldDefinition) -> Self {
        Self {
            schema: definition.schema.clone(),
            entities: definition.entities.clone(),
            relations: definition.relations.clone(),
            absent_entities: std::collections::BTreeSet::new(),
            absent_relations: std::collections::BTreeSet::new(),
        }
    }

    pub fn schema_version(&self) -> SchemaVersion {
        self.schema.version
    }

    pub fn absence_marker(&self, key: EntityKey) -> Option<(AbsenceKind, EntityKey)> {
        self.absent_entities
            .contains(&key)
            .then_some((crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::AbsenceKind::Entity, key))
    }

    pub fn relation_absence_marker(&self, key: RelationKey) -> Option<(AbsenceKind, RelationKey)> {
        self.absent_relations
            .contains(&key)
            .then_some((crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::AbsenceKind::Relation, key))
    }

    pub fn entity(&self, key: EntityKey) -> Option<&EntityRecord> {
        self.entities.get(&key)
    }

    pub fn relation(&self, key: RelationKey) -> Option<&RelationEdge> {
        self.relations.get(&key)
    }

    pub fn replace_entity(&self, key: EntityKey, value: EntityRecord) -> Self {
        let mut next = self.clone();
        next.entities.insert(key, value);
        next.absent_entities.remove(&key);
        next
    }

    pub fn remove_entity(&self, key: EntityKey) -> Self {
        let mut next = self.clone();
        next.entities.remove(&key);
        next.absent_entities.insert(key);
        next
    }

    pub fn replace_relation(&self, edge: RelationEdge) -> Self {
        let mut next = self.clone();
        next.relations.insert(edge.key, edge);
        next.absent_relations.remove(&edge.key);
        next
    }

    pub fn remove_relation(&self, key: RelationKey) -> Self {
        let mut next = self.clone();
        next.relations.remove(&key);
        next.absent_relations.insert(key);
        next
    }

    pub fn upgrade_hazard_schema(&self) -> Self {
        let mut next = self.clone();
        next.schema = crate::relational_certification::ordinary_context::world::supply_chain::schema::SupplyChainSchema::canonical(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V2);
        next
    }

    pub fn validate_complete(&self) -> Result<(), SchemaError> {
        self.schema.validate_complete_world(
            &self.relations.values().copied().collect::<Vec<_>>(),
            &self.entities,
        )
    }
}
