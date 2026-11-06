//! Coverage compiled at installation; commits consult only changed record kinds.
use super::{DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind};
use crate::identity::data::KindId;
use crate::schema::data::AspectContractPlanCatalog;
use std::collections::{BTreeMap, BTreeSet};
use worth_foundational::facade::AspectFieldLocator;

#[derive(Clone, Debug, Default)]
pub(crate) struct InstalledIndexRouting {
    entities: BTreeMap<KindId, Vec<DerivedIndexId>>,
    relations: BTreeMap<KindId, Vec<DerivedIndexId>>,
    entity_fields: BTreeMap<KindId, Vec<AspectFieldLocator>>,
}
impl InstalledIndexRouting {
    /// cost-lane: cold-start/schema transition. Cost: installed definitions times schema plans.
    pub(crate) fn compile<'a>(
        definitions: impl Iterator<Item = &'a DerivedIndexDefinition>,
        plans: &AspectContractPlanCatalog,
    ) -> Self {
        let mut routing = Self::default();
        let mut fields: BTreeMap<KindId, BTreeSet<AspectFieldLocator>> = BTreeMap::new();
        for definition in definitions {
            match &definition.kind {
                DerivedIndexKind::EntityField { field_locator } => {
                    for plan in plans.entity_plans.values() {
                        if crate::indexes::projected_field_values::entity_index_projection_scope(
                            plan,
                            field_locator,
                        )
                        .is_some()
                        {
                            routing
                                .entities
                                .entry(plan.kind_id)
                                .or_default()
                                .push(definition.index_id);
                            fields
                                .entry(plan.kind_id)
                                .or_default()
                                .insert(field_locator.clone());
                        }
                    }
                }
                DerivedIndexKind::RelationField { field_locator } => {
                    for plan in plans.relation_plans.values() {
                        if crate::indexes::projected_field_values::relation_index_projection_scope(
                            plan,
                            field_locator,
                        )
                        .is_some()
                        {
                            routing
                                .relations
                                .entry(plan.kind_id)
                                .or_default()
                                .push(definition.index_id);
                        }
                    }
                }
                DerivedIndexKind::RelatedEntityOrdering {
                    relation_kind,
                    child_kind,
                    ..
                } => {
                    routing
                        .entities
                        .entry(*child_kind)
                        .or_default()
                        .push(definition.index_id);
                    routing
                        .relations
                        .entry(*relation_kind)
                        .or_default()
                        .push(definition.index_id);
                }
                DerivedIndexKind::RelationJoin(join) => {
                    for kind in [
                        join.shared_entity_kind(),
                        join.left().external_entity_kind(),
                        join.right().external_entity_kind(),
                    ] {
                        routing
                            .entities
                            .entry(kind)
                            .or_default()
                            .push(definition.index_id);
                    }
                    for leg in [join.left(), join.right()] {
                        routing
                            .relations
                            .entry(leg.relation_kind())
                            .or_default()
                            .push(definition.index_id);
                    }
                }
            }
        }
        for indexes in routing
            .entities
            .values_mut()
            .chain(routing.relations.values_mut())
        {
            indexes.sort_unstable();
            indexes.dedup();
        }
        routing.entity_fields = fields
            .into_iter()
            .map(|(kind, fields)| (kind, fields.into_iter().collect()))
            .collect();
        routing
    }

    pub(crate) fn entities(&self, kind: KindId) -> &[DerivedIndexId] {
        self.entities.get(&kind).map_or(&[], Vec::as_slice)
    }
    pub(crate) fn relations(&self, kind: KindId) -> &[DerivedIndexId] {
        self.relations.get(&kind).map_or(&[], Vec::as_slice)
    }
    pub(crate) fn entity_fields(&self, kind: KindId) -> &[AspectFieldLocator] {
        self.entity_fields.get(&kind).map_or(&[], Vec::as_slice)
    }
    pub(crate) fn kind_count(&self) -> usize {
        self.entities.len().max(self.relations.len())
    }
}
