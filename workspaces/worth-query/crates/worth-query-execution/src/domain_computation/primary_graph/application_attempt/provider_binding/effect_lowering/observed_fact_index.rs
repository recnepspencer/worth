use crate::domain_computation::primary_graph::application_checkpoint::facts::FactSequence;

use worth_foundational::facade::{
    prepare_aspect_value_identity_basis, AspectFieldLocator, AspectValue,
};
use worth_relational::facade::identity::{EntityId, KindId, RelationId};
use worth_relational::facade::indexes::DerivedIndexId;

use super::super::{
    progression_denial, WorthQueryApplicationAttemptDenial, WorthQueryApplicationObservedFact,
};

/// First matching sealed fact for each effect target. Indexing preserves the
/// previous first-match semantics while avoiding a full fact scan per effect.
pub(in crate::domain_computation::primary_graph::application_attempt::provider_binding) struct ObservedFactIndex<
    'facts,
    Facts: FactSequence + ?Sized,
> {
    facts: &'facts Facts,
    routes:
        std::sync::Arc<crate::domain_computation::primary_graph::application_attempt::FactRoutes>,
}

impl<'facts, Facts: FactSequence + ?Sized> ObservedFactIndex<'facts, Facts> {
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph::application_attempt::provider_binding) fn new(
        facts: &'facts Facts,
    ) -> Self {
        use crate::domain_computation::primary_graph::application_attempt::{
            retained_decision_facts::StorageControl, FactRoutes,
        };
        let routes = FactRoutes::build_iter(
            facts.iter(),
            0,
            None,
            StorageControl::new(
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                None,
            ),
        )
        .expect("unrestricted full effect routing");
        Self { facts, routes }
    }
    pub(in crate::domain_computation::primary_graph::application_attempt::provider_binding) fn completed(
        facts: &'facts Facts,
        routes: std::sync::Arc<
            crate::domain_computation::primary_graph::application_attempt::FactRoutes,
        >,
    ) -> Self {
        Self { facts, routes }
    }

    /// The entity kind the sealed fact for this field records.
    pub(super) fn field_kind(
        &self,
        entity_id: EntityId,
        locator: &AspectFieldLocator,
    ) -> Result<KindId, WorthQueryApplicationAttemptDenial> {
        match self
            .routes
            .field(entity_id, locator)
            .map(|position| self.facts.get(position).expect("indexed observed fact"))
        {
            Some(
                WorthQueryApplicationObservedFact::Field { kind, .. }
                | WorthQueryApplicationObservedFact::AbsentField { kind, .. },
            ) => Ok(*kind),
            _ => Err(progression_denial()),
        }
    }

    /// Every live entity the sealed selection found holding `value`. A sealed
    /// selection did not overflow, so the list is exact whatever its limit.
    pub(super) fn selection_candidates(
        &self,
        index_id: DerivedIndexId,
        kind: KindId,
        locator: &AspectFieldLocator,
        value: &AspectValue,
    ) -> Result<&'facts [EntityId], WorthQueryApplicationAttemptDenial> {
        let key = (
            index_id,
            kind,
            locator.clone(),
            prepare_aspect_value_identity_basis(value),
        );
        match self
            .routes
            .selection(&key)
            .map(|position| self.facts.get(position).expect("indexed observed fact"))
        {
            Some(WorthQueryApplicationObservedFact::IndexedEntitySelection {
                candidates, ..
            }) => Ok(candidates),
            _ => Err(progression_denial()),
        }
    }

    pub(super) fn field_identity(
        &self,
        entity_id: EntityId,
        locator: &AspectFieldLocator,
    ) -> Result<String, WorthQueryApplicationAttemptDenial> {
        self.routes
            .field(entity_id, locator)
            .map(|position| {
                self.facts
                    .get(position)
                    .expect("indexed observed fact")
                    .locator_identity()
            })
            .ok_or_else(progression_denial)
    }

    pub(super) fn entity_identity(
        &self,
        entity_id: EntityId,
    ) -> Result<String, WorthQueryApplicationAttemptDenial> {
        self.routes
            .entity(entity_id)
            .map(|position| {
                self.facts
                    .get(position)
                    .expect("indexed observed fact")
                    .locator_identity()
            })
            .ok_or_else(progression_denial)
    }

    pub(super) fn relation_identity(
        &self,
        relation_id: RelationId,
    ) -> Result<String, WorthQueryApplicationAttemptDenial> {
        self.routes
            .relation(relation_id)
            .map(|position| {
                self.facts
                    .get(position)
                    .expect("indexed observed fact")
                    .locator_identity()
            })
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
    fn completed_routes_refuse_equal_unrelated_backing_and_keep_first_ordinal() {
        use crate::domain_computation::primary_graph::application_attempt::{
            retained_decision_facts::StorageControl, FactContent,
        };
        use crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload;
        use worth_execution::ExecutionAllocationPolicy;
        let locator = planned_single_field_locator(
            AspectKey::new("details").unwrap(),
            FieldKey::new("value").unwrap(),
        );
        let entity = EntityId::new(PartitionId::new(1), 7, 1);
        let fact = WorthQueryApplicationObservedFact::AbsentField {
            entity_id: entity,
            kind: KindId(7),
            locator: locator.clone(),
        };
        let body: FactPayload = std::sync::Arc::<[_]>::from([fact.clone(), fact.clone()]).into();
        let unrelated: FactPayload = std::sync::Arc::<[_]>::from([fact.clone(), fact]).into();
        let control = StorageControl::new(ExecutionAllocationPolicy::SystemAllocation, None);
        let (content, _) = FactContent::build(body.clone(), control).unwrap();
        assert!(content.effect_routes(&unrelated).is_none());
        assert!(!content
            .effect_routes(&body)
            .unwrap()
            .matches_body(&unrelated));
        let (_, routes) = content.effect_routes(&body).unwrap().into_parts();
        assert_eq!(routes.field(entity, &locator), Some(0));
        let index = ObservedFactIndex::completed(&body, routes);
        assert_eq!(index.field_kind(entity, &locator).unwrap(), KindId(7));
    }

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
