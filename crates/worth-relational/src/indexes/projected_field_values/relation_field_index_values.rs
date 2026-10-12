use std::collections::BTreeMap;

use worth_foundational::facade::AspectFieldLocator;

use crate::identity::data::RelationId;
use crate::storage::data::AuthoritativeFieldComparisonKey;
use crate::visibility::materialization::read_records::relation_query_locus_comparison_key;

use super::checked_entry_map::{checked_push, IndexKernelStop};
use super::field_projection_scope::{
    relation_index_projection_scope, relation_index_projection_scopes,
};
use super::IndexProjectionSource;

pub(in crate::indexes) fn build_relation_aspect_field_index(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    projection: &IndexProjectionSource<'_, '_>,
    field_locator: &AspectFieldLocator,
) -> BTreeMap<AuthoritativeFieldComparisonKey, Vec<RelationId>> {
    let mut entries = BTreeMap::new();
    for scope in relation_index_projection_scopes(projection, field_locator) {
        projection.for_each_relation(execution, scope.kind_id(), |record| {
            if let Some(key) = relation_query_locus_comparison_key(record, field_locator) {
                entries
                    .entry(key)
                    .or_insert_with(Vec::new)
                    .push(record.relation_id);
            }
        });
    }
    entries
}

pub(in crate::indexes) fn build_relation_aspect_field_index_checked(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    projection: &IndexProjectionSource<'_, '_>,
    field_locator: &AspectFieldLocator,
    context: &mut crate::execution::PacketKernelContext<'_, '_, '_>,
) -> Result<BTreeMap<AuthoritativeFieldComparisonKey, Vec<RelationId>>, IndexKernelStop> {
    let mut entries = BTreeMap::new();
    let budget = std::cell::RefCell::new(context);
    let projected_bytes = std::cell::Cell::new(0);
    if let Some(plans) = projection.aspect_plans() {
        for plan in plans.relation_plans.values() {
            if relation_index_projection_scope(plan, field_locator).is_none() {
                continue;
            }
            projection.try_for_each_relation_field(
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
            )?;
        }
    }
    Ok(entries)
}
