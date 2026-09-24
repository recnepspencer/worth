use std::collections::BTreeMap;

use worth_foundational::facade::AspectFieldLocator;
use worth_relational::facade::identity::{EntityId, RelationId};

use super::super::{
    progression_denial, WorthQueryApplicationAttemptDenial, WorthQueryApplicationObservedFact,
};

/// First matching sealed fact for each effect target. Indexing preserves the
/// previous first-match semantics while avoiding a full fact scan per effect.
pub(in crate::domain_computation::primary_graph::application_attempt::provider_binding) struct ObservedFactIndex<
    'facts,
> {
    facts: &'facts [WorthQueryApplicationObservedFact],
    fields: BTreeMap<EntityId, BTreeMap<AspectFieldLocator, usize>>,
    entities: BTreeMap<EntityId, usize>,
    relations: BTreeMap<RelationId, usize>,
}

impl<'facts> ObservedFactIndex<'facts> {
    pub(in crate::domain_computation::primary_graph::application_attempt::provider_binding) fn new(
        facts: &'facts [WorthQueryApplicationObservedFact],
    ) -> Self {
        let mut index = Self {
            facts,
            fields: BTreeMap::new(),
            entities: BTreeMap::new(),
            relations: BTreeMap::new(),
        };
        for (position, fact) in facts.iter().enumerate() {
            match fact {
                WorthQueryApplicationObservedFact::Entity { entity_id, .. } => {
                    index.entities.entry(*entity_id).or_insert(position);
                }
                WorthQueryApplicationObservedFact::Field {
                    entity_id, locator, ..
                }
                | WorthQueryApplicationObservedFact::AbsentField {
                    entity_id, locator, ..
                } => {
                    index
                        .fields
                        .entry(*entity_id)
                        .or_default()
                        .entry(locator.clone())
                        .or_insert(position);
                }
                WorthQueryApplicationObservedFact::Relation {
                    matching_relations, ..
                } => {
                    for relation_id in matching_relations {
                        index.relations.entry(*relation_id).or_insert(position);
                    }
                }
                WorthQueryApplicationObservedFact::Adjacency { relations, .. } => {
                    for relation in relations {
                        index
                            .relations
                            .entry(relation.relation_id)
                            .or_insert(position);
                    }
                }
                _ => {}
            }
        }
        index
    }

    pub(super) fn field_identity(
        &self,
        entity_id: EntityId,
        locator: &AspectFieldLocator,
    ) -> Result<String, WorthQueryApplicationAttemptDenial> {
        self.fields
            .get(&entity_id)
            .and_then(|fields| fields.get(locator))
            .map(|position| self.facts[*position].locator_identity())
            .ok_or_else(progression_denial)
    }

    pub(super) fn entity_identity(
        &self,
        entity_id: EntityId,
    ) -> Result<String, WorthQueryApplicationAttemptDenial> {
        self.entities
            .get(&entity_id)
            .map(|position| self.facts[*position].locator_identity())
            .ok_or_else(progression_denial)
    }

    pub(super) fn relation_identity(
        &self,
        relation_id: RelationId,
    ) -> Result<String, WorthQueryApplicationAttemptDenial> {
        self.relations
            .get(&relation_id)
            .map(|position| self.facts[*position].locator_identity())
            .ok_or_else(progression_denial)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_foundational::facade::{AspectKey, FieldKey};
    use worth_relational::facade::identity::{KindId, PartitionId};
    use worth_relational::facade::transactions::planned_single_field_locator;

    #[test]
    fn many_field_targets_resolve_from_one_sealed_index() {
        let locator = planned_single_field_locator(
            AspectKey::new("details").unwrap(),
            FieldKey::new("value").unwrap(),
        );
        let facts = (0..10_000)
            .map(|slot| WorthQueryApplicationObservedFact::AbsentField {
                entity_id: EntityId::new(PartitionId::new(1), slot, 1),
                kind: KindId(7),
                locator: locator.clone(),
            })
            .collect::<Vec<_>>();
        let index = ObservedFactIndex::new(&facts);
        let last = EntityId::new(PartitionId::new(1), 9_999, 1);
        assert_eq!(
            index.field_identity(last, &locator).unwrap(),
            facts.last().unwrap().locator_identity()
        );
        assert!(index
            .field_identity(EntityId::new(PartitionId::new(1), 10_000, 1), &locator)
            .is_err());
    }

    #[test]
    fn duplicate_targets_retain_first_sealed_fact_identity() {
        let entity = EntityId::new(PartitionId::new(1), 1, 1);
        let relation = RelationId::new(PartitionId::new(1), 2, 1);
        let facts = vec![
            WorthQueryApplicationObservedFact::Entity {
                entity_id: entity,
                kind: KindId(7),
            },
            WorthQueryApplicationObservedFact::Relation {
                relation_kind: KindId(8),
                from: entity,
                to: entity,
                matching_relations: vec![relation],
            },
            WorthQueryApplicationObservedFact::Relation {
                relation_kind: KindId(9),
                from: entity,
                to: entity,
                matching_relations: vec![relation],
            },
        ];
        let index = ObservedFactIndex::new(&facts);
        assert_eq!(
            index.entity_identity(entity).unwrap(),
            facts[0].locator_identity()
        );
        assert_eq!(
            index.relation_identity(relation).unwrap(),
            facts[1].locator_identity()
        );
    }
}
