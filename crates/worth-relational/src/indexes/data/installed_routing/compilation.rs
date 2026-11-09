//! One route compiler for installation and request-funded private preparation.
use super::super::DerivedIndexMaintenanceDenialKind as Denial;
use super::allocation::{self as allocate, Payment};
use super::*;
use crate::storage::substrate::maintenance::MaintenanceRoute as Route;
pub(super) fn compile<'a>(
    definitions: impl Iterator<Item = &'a DerivedIndexDefinition>,
    plans: &AspectContractPlanCatalog,
    pay: &mut Payment<'_>,
) -> Result<InstalledIndexRouting, Denial> {
    let mut routing = InstalledIndexRouting::default();
    let mut fields: BTreeMap<KindId, BTreeSet<AspectFieldLocator>> = BTreeMap::new();
    for definition in definitions {
        pay(1, 0)?;
        allocate::set(
            &mut routing.maintenance,
            Route::Index(definition.index_id),
            1,
            pay,
        )?;
        allocate::set(&mut routing.maintenance, Route::Schema, 1, pay)?;
        match &definition.kind {
            DerivedIndexKind::EntityField { field_locator } => {
                for plan in plans.entity_plans.values() {
                    allocate::scope(plan, field_locator, pay)?;
                    if crate::indexes::projected_field_values::entity_index_projection_scope(
                        plan,
                        field_locator,
                    )
                    .is_some()
                    {
                        allocate::push(
                            &mut routing.entities,
                            plan.kind_id,
                            definition.index_id,
                            1,
                            pay,
                        )?;
                        routing.route_field(
                            true,
                            plan.kind_id,
                            field_locator,
                            definition.index_id,
                            pay,
                        )?;
                        let values = allocate::map(&mut fields, plan.kind_id, 1, pay)?;
                        let width = allocate::locator(field_locator, pay)?;
                        allocate::set(values, field_locator.clone(), width, pay)?;
                    }
                }
            }
            DerivedIndexKind::RelationField { field_locator } => {
                for plan in plans.relation_plans.values() {
                    allocate::scope(plan, field_locator, pay)?;
                    if crate::indexes::projected_field_values::relation_index_projection_scope(
                        plan,
                        field_locator,
                    )
                    .is_some()
                    {
                        allocate::push(
                            &mut routing.relations,
                            plan.kind_id,
                            definition.index_id,
                            1,
                            pay,
                        )?;
                        routing.route_field(
                            false,
                            plan.kind_id,
                            field_locator,
                            definition.index_id,
                            pay,
                        )?;
                    }
                }
            }
            DerivedIndexKind::RelatedEntityOrdering {
                relation_kind,
                child_kind,
                ordering,
                ..
            } => {
                allocate::set(
                    &mut routing.maintenance,
                    Route::EntityKind(*child_kind),
                    1,
                    pay,
                )?;
                allocate::set(
                    &mut routing.maintenance,
                    Route::RelationKind(*relation_kind),
                    1,
                    pay,
                )?;
                allocate::push(
                    &mut routing.entity_topology,
                    *child_kind,
                    definition.index_id,
                    1,
                    pay,
                )?;
                allocate::push(
                    &mut routing.relation_topology,
                    *relation_kind,
                    definition.index_id,
                    1,
                    pay,
                )?;
                for field in ordering {
                    routing.route_field(
                        true,
                        *child_kind,
                        field.locator(),
                        definition.index_id,
                        pay,
                    )?;
                }
                allocate::push(
                    &mut routing.entities,
                    *child_kind,
                    definition.index_id,
                    1,
                    pay,
                )?;
                allocate::push(
                    &mut routing.relations,
                    *relation_kind,
                    definition.index_id,
                    1,
                    pay,
                )?;
            }
            DerivedIndexKind::RelationJoin(join) => {
                for kind in [
                    join.shared_entity_kind(),
                    join.left().external_entity_kind(),
                    join.right().external_entity_kind(),
                ] {
                    allocate::set(&mut routing.maintenance, Route::EntityKind(kind), 1, pay)?;
                    allocate::push(
                        &mut routing.entity_topology,
                        kind,
                        definition.index_id,
                        1,
                        pay,
                    )?;
                    allocate::push(&mut routing.entities, kind, definition.index_id, 1, pay)?;
                }
                for leg in [join.left(), join.right()] {
                    allocate::set(
                        &mut routing.maintenance,
                        Route::RelationKind(leg.relation_kind()),
                        1,
                        pay,
                    )?;
                    allocate::push(
                        &mut routing.relation_topology,
                        leg.relation_kind(),
                        definition.index_id,
                        1,
                        pay,
                    )?;
                    allocate::push(
                        &mut routing.relations,
                        leg.relation_kind(),
                        definition.index_id,
                        1,
                        pay,
                    )?;
                }
            }
        }
    }
    for indexes in routing
        .entities
        .values_mut()
        .chain(routing.relations.values_mut())
    {
        let levels = u64::from(usize::BITS - indexes.len().leading_zeros()) + 1;
        pay(
            (indexes.len() as u64)
                .checked_mul(levels + 1)
                .ok_or(Denial::WorkBudgetExceeded)?,
            0,
        )?;
        indexes.sort_unstable();
        indexes.dedup();
    }
    for (kind, fields) in fields {
        for field in fields {
            allocate::push(&mut routing.entity_fields, kind, field, 1, pay)?;
        }
    }
    for (kind, fields) in &routing.entity_fields {
        let aspects = allocate::map(&mut routing.entity_aspect_fields, *kind, 1, pay)?;
        for field in fields {
            let width = allocate::locator(field, pay)?;
            let aspect = field.aspect().aspect_key();
            pay(aspect.as_str().len() as u64, aspect.as_str().len() as u64)?;
            allocate::push(aspects, aspect.clone(), field.clone(), width, pay)?;
        }
    }
    Ok(routing)
}
impl InstalledIndexRouting {
    fn route_field(
        &mut self,
        entity: bool,
        kind: KindId,
        locator: &AspectFieldLocator,
        index: DerivedIndexId,
        pay: &mut Payment<'_>,
    ) -> Result<(), Denial> {
        allocate::set(
            &mut self.maintenance,
            if entity {
                Route::EntityKind(kind)
            } else {
                Route::RelationKind(kind)
            },
            1,
            pay,
        )?;
        let width = allocate::locator(locator, pay)?;
        allocate::set(
            &mut self.maintenance,
            Route::Field {
                kind,
                aspect: locator.aspect().aspect_key().clone(),
                path: locator.field_path().clone(),
            },
            width,
            pay,
        )?;
        if let [field] = locator.field_path().fields() {
            let aspect = locator.aspect().aspect_key();
            let width = aspect
                .as_str()
                .len()
                .checked_add(field.as_str().len())
                .and_then(|n| n.checked_add(1))
                .ok_or(Denial::WorkBudgetExceeded)?;
            pay(width as u64, (width - 1) as u64)?;
            allocate::push(
                &mut self.fields,
                (entity, kind, aspect.clone(), field.clone()),
                index,
                width,
                pay,
            )?;
        }
        Ok(())
    }
}
