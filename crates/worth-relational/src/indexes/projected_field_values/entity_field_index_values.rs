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
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    projection: &IndexProjectionSource<'_>,
    field_locator: &AspectFieldLocator,
) -> Result<
    BTreeMap<AuthoritativeFieldComparisonKey, Vec<EntityId>>,
    crate::execution::RelationalExecutionDenialCause,
> {
    let mut entries = BTreeMap::new();
    for scope in entity_index_projection_scopes(projection, field_locator) {
        projection.for_each_entity(execution, scope.kind_id(), |record| {
            if let Some(key) = entity_query_locus_comparison_key(record, field_locator) {
                entries
                    .entry(key)
                    .or_insert_with(Vec::new)
                    .push(record.entity_id);
            }
            Ok(())
        })?;
    }
    Ok(entries)
}

pub(in crate::indexes) fn build_entity_aspect_field_index_checked(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    projection: &IndexProjectionSource<'_>,
    field_locator: &AspectFieldLocator,
    context: &mut crate::execution::PacketKernelContext<'_, '_, '_>,
) -> Result<BTreeMap<AuthoritativeFieldComparisonKey, Vec<EntityId>>, IndexKernelStop> {
    let mut entries = BTreeMap::new();
    let budget = std::cell::RefCell::new(context);
    let projected_bytes = std::cell::Cell::new(0);
    if let Some(plans) = projection.aspect_plans() {
        for plan in plans.entity_plans.values() {
            if entity_index_projection_scope(plan, field_locator).is_none() {
                continue;
            }
            projection
                .try_for_each_entity_field(
                    request_execution,
                    plan.kind_id,
                    field_locator,
                    |bytes| {
                        projected_bytes.set(bytes);
                        budget.borrow_mut().checkpoint(1)?;
                        budget.borrow().check_scratch_peak(bytes)
                    },
                    |record_id, value| {
                        let Some(value) = value else {
                            return Ok(());
                        };
                        let encoded =
                            AuthoritativeFieldComparisonKey::required_encoded_capacity_bytes(value)
                                .ok_or(worth_execution::MapKernelFailure::ResultCapacityExceeded)?;
                        // The projected record remains live while its one native
                        // key Vec is encoded. The encoder declares that Vec exactly.
                        budget
                            .borrow()
                            .check_scratch_peak(projected_bytes.get().saturating_add(encoded))?;
                        let key = AuthoritativeFieldComparisonKey::from_aspect_value(value);
                        let key_bytes = key.owned_allocation_capacity_bytes();
                        checked_push(
                            &mut entries,
                            key,
                            record_id,
                            key_bytes,
                            0,
                            &mut budget.borrow_mut(),
                        )
                    },
                )
                .map_err(super::checked_entry_map::read_stop)?;
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
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    projection: &IndexProjectionSource<'_>,
    entity_id: EntityId,
    field_locator: &AspectFieldLocator,
) -> Result<
    Option<worth_foundational::facade::AspectValue>,
    crate::execution::RelationalExecutionDenialCause,
> {
    Ok({
        projection
            .with_entity(execution, entity_id, |record| {
                source_entity_index_projection_scope_for_kind(
                    projection,
                    record.kind.kind_id,
                    field_locator,
                )?;
                entity_query_locus_value(record, field_locator).cloned()
            })?
            .flatten()
    })
}
