pub use {std::collections::BTreeMap};

pub use {crate::relational_certification::ordinary_context::world::supply_chain::definition_entities};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::definition_relations};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord, crate::relational_certification::ordinary_context::world::supply_chain::schema::RelationEdge, crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError, crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion, crate::relational_certification::ordinary_context::world::supply_chain::schema::SupplyChainSchema};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DefinitionError {
    CountMismatch {
        kind: EntityKind,
        expected: usize,
        observed: usize,
    },
    MissingAnchor(Anchor),
    InvalidRelation(SchemaError),
    ArrivalBeforeDeparture(EntityKey),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupplyChainWorldDefinition {
    pub scale: SupplyChainScale,
    pub schema: SupplyChainSchema,
    pub entities: BTreeMap<EntityKey, EntityRecord>,
    pub relations: BTreeMap<RelationKey, RelationEdge>,
}

impl SupplyChainWorldDefinition {
    pub fn empty(scale: SupplyChainScale) -> Self {
        Self {
            scale,
            schema: crate::relational_certification::ordinary_context::world::supply_chain::schema::SupplyChainSchema::canonical(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1),
            entities: std::collections::BTreeMap::new(),
            relations: std::collections::BTreeMap::new(),
        }
    }

    pub fn operating(scale: SupplyChainScale) -> Result<Self, DefinitionError> {
        let definition = Self {
            scale,
            schema: crate::relational_certification::ordinary_context::world::supply_chain::schema::SupplyChainSchema::canonical(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1),
            entities: crate::relational_certification::ordinary_context::world::supply_chain::definition_entities::build(scale),
            relations: crate::relational_certification::ordinary_context::world::supply_chain::definition_relations::build(scale),
        };
        definition.validate()
    }

    pub fn validate(self) -> Result<Self, DefinitionError> {
        self.validate_entity_counts()?;
        if self.entities.is_empty()
            && self.relations.is_empty()
            && self.scale.count_for(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port, 0)) == 0
        {
            return Ok(self);
        }
        self.validate_anchors()?;
        self.validate_relations()?;
        self.schema
            .validate_complete_world(
                &self.relations.values().copied().collect::<Vec<_>>(),
                &self.entities,
            )
            .map_err(crate::relational_certification::ordinary_context::world::supply_chain::definition::DefinitionError::InvalidRelation)?;
        self.validate_voyage_times()?;
        Ok(self)
    }

    pub fn validate_entity_counts(&self) -> Result<(), DefinitionError> {
        for kind in [
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Terminal,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Berth,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::CargoLot,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Inspection,
        ] {
            let observed = self
                .entities
                .values()
                .filter(|record| record.kind() == kind)
                .count();
            let expected = self.scale.count_for(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(kind, 0));
            if observed != expected {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::definition::DefinitionError::CountMismatch {
                    kind,
                    expected,
                    observed,
                });
            }
        }
        Ok(())
    }

    pub fn validate_anchors(&self) -> Result<(), DefinitionError> {
        for anchor in self.scale.anchors() {
            if !self.entities.contains_key(&anchor.entity()) {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::definition::DefinitionError::MissingAnchor(anchor));
            }
        }
        Ok(())
    }

    pub fn validate_relations(&self) -> Result<(), DefinitionError> {
        for edge in self.relations.values().copied() {
            let source = self.entities.get(&edge.source).map(crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::kind);
            let target = self.entities.get(&edge.target).map(crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::kind);
            let (Some(source), Some(target)) = (source, target) else {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::definition::DefinitionError::InvalidRelation(
                    crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::InvalidEndpoint {
                        relation: edge.key.kind,
                        source: source.unwrap_or(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port),
                        target: target.unwrap_or(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port),
                    },
                ));
            };
            self.schema
                .validate_edge(edge, source, target)
                .map_err(crate::relational_certification::ordinary_context::world::supply_chain::definition::DefinitionError::InvalidRelation)?;
        }
        Ok(())
    }

    pub fn validate_voyage_times(&self) -> Result<(), DefinitionError> {
        for (key, record) in &self.entities {
            if let crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Voyage(voyage) = record
                && voyage.arrival < voyage.departure
            {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::definition::DefinitionError::ArrivalBeforeDeparture(*key));
            }
        }
        Ok(())
    }

    pub fn entity(&self, key: EntityKey) -> Option<&EntityRecord> {
        self.entities.get(&key)
    }
}