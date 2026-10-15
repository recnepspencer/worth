use std::collections::BTreeMap;

use super::definition_entities;
use super::definition_relations;
use super::scale::SupplyChainScale;
use super::schema::{EntityRecord, RelationEdge, SchemaError, SchemaVersion, SupplyChainSchema};
use super::semantic_key::{Anchor, EntityKey, EntityKind, RelationKey};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum DefinitionError {
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
pub(crate) struct SupplyChainWorldDefinition {
    pub(crate) scale: SupplyChainScale,
    pub(crate) schema: SupplyChainSchema,
    pub(crate) entities: BTreeMap<EntityKey, EntityRecord>,
    pub(crate) relations: BTreeMap<RelationKey, RelationEdge>,
}

impl SupplyChainWorldDefinition {
    pub(crate) fn empty(scale: SupplyChainScale) -> Self {
        Self {
            scale,
            schema: SupplyChainSchema::canonical(SchemaVersion::V1),
            entities: BTreeMap::new(),
            relations: BTreeMap::new(),
        }
    }

    pub(crate) fn operating(scale: SupplyChainScale) -> Result<Self, DefinitionError> {
        let definition = Self {
            scale,
            schema: SupplyChainSchema::canonical(SchemaVersion::V1),
            entities: definition_entities::build(scale),
            relations: definition_relations::build(scale),
        };
        definition.validate()
    }

    pub(crate) fn validate(self) -> Result<Self, DefinitionError> {
        self.validate_entity_counts()?;
        if self.entities.is_empty()
            && self.relations.is_empty()
            && self.scale.count_for(EntityKey::new(EntityKind::Port, 0)) == 0
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
            .map_err(DefinitionError::InvalidRelation)?;
        self.validate_voyage_times()?;
        Ok(self)
    }

    fn validate_entity_counts(&self) -> Result<(), DefinitionError> {
        for kind in [
            EntityKind::Port,
            EntityKind::Terminal,
            EntityKind::Berth,
            EntityKind::Vessel,
            EntityKind::Voyage,
            EntityKind::PortCall,
            EntityKind::CargoLot,
            EntityKind::Inspection,
        ] {
            let observed = self
                .entities
                .values()
                .filter(|record| record.kind() == kind)
                .count();
            let expected = self.scale.count_for(EntityKey::new(kind, 0));
            if observed != expected {
                return Err(DefinitionError::CountMismatch {
                    kind,
                    expected,
                    observed,
                });
            }
        }
        Ok(())
    }

    fn validate_anchors(&self) -> Result<(), DefinitionError> {
        for anchor in self.scale.anchors() {
            if !self.entities.contains_key(&anchor.entity()) {
                return Err(DefinitionError::MissingAnchor(anchor));
            }
        }
        Ok(())
    }

    fn validate_relations(&self) -> Result<(), DefinitionError> {
        for edge in self.relations.values().copied() {
            let source = self.entities.get(&edge.source).map(EntityRecord::kind);
            let target = self.entities.get(&edge.target).map(EntityRecord::kind);
            let (Some(source), Some(target)) = (source, target) else {
                return Err(DefinitionError::InvalidRelation(
                    SchemaError::InvalidEndpoint {
                        relation: edge.key.kind,
                        source: source.unwrap_or(EntityKind::Port),
                        target: target.unwrap_or(EntityKind::Port),
                    },
                ));
            };
            self.schema
                .validate_edge(edge, source, target)
                .map_err(DefinitionError::InvalidRelation)?;
        }
        Ok(())
    }

    fn validate_voyage_times(&self) -> Result<(), DefinitionError> {
        for (key, record) in &self.entities {
            if let EntityRecord::Voyage(voyage) = record
                && voyage.arrival < voyage.departure
            {
                return Err(DefinitionError::ArrivalBeforeDeparture(*key));
            }
        }
        Ok(())
    }

    pub(crate) fn entity(&self, key: EntityKey) -> Option<&EntityRecord> {
        self.entities.get(&key)
    }
}
