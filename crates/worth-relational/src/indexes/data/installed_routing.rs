//! Coverage compiled at installation; commits consult only changed record kinds.
use super::{DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind};
use crate::identity::data::KindId;
use crate::schema::data::AspectContractPlanCatalog;
use std::collections::{BTreeMap, BTreeSet};
use worth_foundational::facade::{AspectFieldLocator, AspectKey, FieldKey};

#[derive(Clone, Debug, Default)]
pub(crate) struct InstalledIndexRouting {
    maintenance: BTreeSet<crate::storage::substrate::maintenance::MaintenanceRoute>,
    entities: BTreeMap<KindId, Vec<DerivedIndexId>>,
    relations: BTreeMap<KindId, Vec<DerivedIndexId>>,
    entity_fields: BTreeMap<KindId, Vec<AspectFieldLocator>>,
    entity_aspect_fields: BTreeMap<KindId, BTreeMap<AspectKey, Vec<AspectFieldLocator>>>,
    entity_topology: BTreeMap<KindId, Vec<DerivedIndexId>>,
    relation_topology: BTreeMap<KindId, Vec<DerivedIndexId>>,
    fields: BTreeMap<(bool, KindId, AspectKey, FieldKey), Vec<DerivedIndexId>>,
}
impl InstalledIndexRouting {
    /// cost-lane: cold-start/schema transition. Cost: installed definitions times schema plans.
    pub(crate) fn compile<'a>(
        definitions: impl Iterator<Item = &'a DerivedIndexDefinition>,
        plans: &AspectContractPlanCatalog,
    ) -> Self {
        let mut routing = Self::default();
        let mut fields: BTreeMap<KindId, BTreeSet<AspectFieldLocator>> = BTreeMap::new();
        use crate::storage::substrate::maintenance::MaintenanceRoute as Route;
        for definition in definitions {
            routing
                .maintenance
                .insert(Route::Index(definition.index_id));
            routing.maintenance.insert(Route::Schema);
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
                            routing.route_field(
                                true,
                                plan.kind_id,
                                field_locator,
                                definition.index_id,
                            );
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
                            routing.route_field(
                                false,
                                plan.kind_id,
                                field_locator,
                                definition.index_id,
                            );
                        }
                    }
                }
                DerivedIndexKind::RelatedEntityOrdering {
                    relation_kind,
                    child_kind,
                    ordering,
                    ..
                } => {
                    routing.maintenance.insert(Route::EntityKind(*child_kind));
                    routing
                        .maintenance
                        .insert(Route::RelationKind(*relation_kind));
                    routing
                        .entity_topology
                        .entry(*child_kind)
                        .or_default()
                        .push(definition.index_id);
                    routing
                        .relation_topology
                        .entry(*relation_kind)
                        .or_default()
                        .push(definition.index_id);
                    for field in ordering {
                        routing.route_field(
                            true,
                            *child_kind,
                            field.locator(),
                            definition.index_id,
                        );
                    }
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
                        routing.maintenance.insert(Route::EntityKind(kind));
                        routing
                            .entity_topology
                            .entry(kind)
                            .or_default()
                            .push(definition.index_id);
                        routing
                            .entities
                            .entry(kind)
                            .or_default()
                            .push(definition.index_id);
                    }
                    for leg in [join.left(), join.right()] {
                        routing
                            .maintenance
                            .insert(Route::RelationKind(leg.relation_kind()));
                        routing
                            .relation_topology
                            .entry(leg.relation_kind())
                            .or_default()
                            .push(definition.index_id);
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
        for (kind, fields) in &routing.entity_fields {
            let aspects = routing.entity_aspect_fields.entry(*kind).or_default();
            for field in fields {
                aspects
                    .entry(field.aspect().aspect_key().clone())
                    .or_default()
                    .push(field.clone());
            }
        }
        routing
    }

    pub(crate) fn maintenance_routes(
        &self,
    ) -> &BTreeSet<crate::storage::substrate::maintenance::MaintenanceRoute> {
        &self.maintenance
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
    pub(crate) fn entity_fields_in_aspect(
        &self,
        kind: KindId,
        aspect: &AspectKey,
    ) -> &[AspectFieldLocator] {
        self.entity_aspect_fields
            .get(&kind)
            .and_then(|fields| fields.get(aspect))
            .map_or(&[], Vec::as_slice)
    }
    pub(crate) fn topology(&self, kind: KindId, entity: bool) -> &[DerivedIndexId] {
        let routes = if entity {
            &self.entity_topology
        } else {
            &self.relation_topology
        };
        routes.get(&kind).map_or(&[], Vec::as_slice)
    }
    fn route_field(
        &mut self,
        entity: bool,
        kind: KindId,
        locator: &AspectFieldLocator,
        index: DerivedIndexId,
    ) {
        use crate::storage::substrate::maintenance::MaintenanceRoute as Route;
        self.maintenance.insert(if entity {
            Route::EntityKind(kind)
        } else {
            Route::RelationKind(kind)
        });
        self.maintenance.insert(Route::Field {
            kind,
            aspect: locator.aspect().aspect_key().clone(),
            path: locator.field_path().clone(),
        });
        if let [field] = locator.field_path().fields() {
            self.fields
                .entry((
                    entity,
                    kind,
                    locator.aspect().aspect_key().clone(),
                    field.clone(),
                ))
                .or_default()
                .push(index);
        }
    }
    pub(crate) fn field(
        &self,
        entity: bool,
        kind: KindId,
        aspect: &AspectKey,
        field: &FieldKey,
    ) -> &[DerivedIndexId] {
        self.fields
            .get(&(entity, kind, aspect.clone(), field.clone()))
            .map_or(&[], Vec::as_slice)
    }
    pub(crate) fn kind_count(&self) -> usize {
        self.entities.len().max(self.relations.len())
    }
}
