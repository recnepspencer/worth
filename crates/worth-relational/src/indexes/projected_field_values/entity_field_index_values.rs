use std::collections::BTreeMap;

use worth_foundational::facade::AspectFieldLocator;

use crate::capabilities::AspectPlanSource;
use crate::identity::data::EntityId;
use crate::runtime::RelationalRuntime;
use crate::storage::data::AuthoritativeFieldComparisonKey;
use crate::visibility::materialization::read_records::{
    entity_query_locus_comparison_key, entity_query_locus_value,
};

use super::checked_entry_map::{checked_push, IndexKernelStop};
use super::field_projection_scope::{
    entity_index_projection_scope, entity_index_projection_scopes,
    source_entity_index_projection_scope_for_kind,
};
use super::IndexProjectionSource;

pub(in crate::indexes) fn build_entity_aspect_field_index(
    projection: &IndexProjectionSource<'_, '_>,
    field_locator: &AspectFieldLocator,
) -> BTreeMap<AuthoritativeFieldComparisonKey, Vec<EntityId>> {
    let mut entries = BTreeMap::new();
    for scope in entity_index_projection_scopes(projection, field_locator) {
        projection.for_each_entity(scope.kind_id(), |record| {
            if let Some(key) = entity_query_locus_comparison_key(record, field_locator) {
                entries
                    .entry(key)
                    .or_insert_with(Vec::new)
                    .push(record.entity_id);
            }
        });
    }
    entries
}

pub(in crate::indexes) fn build_entity_aspect_field_index_checked(
    projection: &IndexProjectionSource<'_, '_>,
    field_locator: &AspectFieldLocator,
    context: &mut crate::execution::PacketKernelContext<'_, '_, '_>,
) -> Result<BTreeMap<AuthoritativeFieldComparisonKey, Vec<EntityId>>, IndexKernelStop> {
    let mut entries = BTreeMap::new();
    let budget = std::cell::RefCell::new(context);
    if let Some(plans) = projection.aspect_plans() {
        for plan in plans.entity_plans.values() {
            if entity_index_projection_scope(plan, field_locator).is_none() {
                continue;
            }
            projection.try_for_each_entity(
                plan.kind_id,
                |bytes| {
                    budget.borrow_mut().checkpoint(1)?;
                    budget.borrow().check_scratch_peak(bytes)
                },
                |record| {
                    let Some(value) = entity_query_locus_value(record, field_locator) else {
                        return Ok(());
                    };
                    budget.borrow().check_scratch_peak(
                        (value.owned_allocation_capacity_bytes() as u64)
                            .saturating_mul(4)
                            .saturating_add(128),
                    )?;
                    let key = AuthoritativeFieldComparisonKey::from_aspect_value(value);
                    let key_bytes = key.owned_allocation_capacity_bytes();
                    checked_push(
                        &mut entries,
                        key,
                        record.entity_id,
                        key_bytes,
                        0,
                        &mut budget.borrow_mut(),
                    )
                },
            )?;
        }
    }
    Ok(entries)
}

pub(in crate::indexes) fn entity_aspect_field_index_entry(
    runtime: &RelationalRuntime,
    record: &crate::storage::data::EntityReadRecord,
    field_locator: &AspectFieldLocator,
) -> Option<(AuthoritativeFieldComparisonKey, EntityId)> {
    entity_index_projection_scope(
        runtime.entity_aspect_plan(record.kind.kind_id)?,
        field_locator,
    )?;
    entity_query_locus_comparison_key(record, field_locator).map(|value| (value, record.entity_id))
}

pub(in crate::indexes) fn entity_aspect_field_ordering_value(
    projection: &IndexProjectionSource<'_, '_>,
    entity_id: EntityId,
    field_locator: &AspectFieldLocator,
) -> Option<worth_foundational::facade::AspectValue> {
    projection
        .with_entity(entity_id, |record| {
            source_entity_index_projection_scope_for_kind(
                projection,
                record.kind.kind_id,
                field_locator,
            )?;
            entity_query_locus_value(record, field_locator).cloned()
        })
        .flatten()
}
