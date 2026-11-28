use crate::contracts::indexes::projected_field_values::field_projection_scope::entity_index_projection_scope;
use std::collections::BTreeMap;

use worth_foundational::facade::AspectFieldLocator;

use crate::branch::visibility::materialization::read_records::projection::query_locus_projection::entity_query_locus_value;
use crate::contracts::identity::data::record_ids::EntityId;
use crate::contracts::storage::data::authoritative_field_comparison_key::AuthoritativeFieldComparisonKey;

use super::checked_entry_map::{checked_push, IndexKernelStop};
use super::selected_projection_scopes::{
    entity_index_projection_scopes, source_entity_index_projection_scope_for_kind,
};
use super::IndexProjectionSource;

pub(in crate::internal_port::indexes) fn build_entity_aspect_field_index(
    _lane: &crate::contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<
        '_,
    >,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    projection: &IndexProjectionSource<'_>,
    field_locator: &AspectFieldLocator,
    kind_scope: Option<crate::contracts::identity::data::primitives::KindId>,
) -> Result<
    BTreeMap<AuthoritativeFieldComparisonKey, Vec<EntityId>>,
    crate::contracts::execution::denial_cause::RelationalExecutionDenialCause,
> {
    let mut entries = BTreeMap::new();
    for scope in entity_index_projection_scopes(projection, field_locator) {
        if kind_scope.is_some_and(|kind| kind != scope.kind_id()) {
            continue;
        }
        projection.try_for_each_entity_field(execution, scope.kind_id(), field_locator,
            |_, _| Ok::<_, crate::contracts::execution::denial_cause::RelationalExecutionDenialCause>(()),
            |entity_id, value| {
                if let Some(value) = value {
                    let key = AuthoritativeFieldComparisonKey::from_aspect_value(value);
                    entries.entry(key).or_insert_with(Vec::new).push(entity_id);
                }
                Ok(())
            }).map_err(|stop| match stop {
                worth_relational_contracts::internal_port::execution::request_or::RequestOr::Request(cause)
                | worth_relational_contracts::internal_port::execution::request_or::RequestOr::Inner(cause) => cause,
            })?;
    }
    Ok(entries)
}

pub(in crate::internal_port::indexes) fn build_entity_aspect_field_index_checked(
    _lane: &crate::contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<
        '_,
    >,
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    projection: &IndexProjectionSource<'_>,
    field_locator: &AspectFieldLocator,
    kind_scope: Option<crate::contracts::identity::data::primitives::KindId>,
    context: &mut crate::contracts::execution::packet_memory::PacketKernelContext<'_, '_, '_>,
) -> Result<BTreeMap<AuthoritativeFieldComparisonKey, Vec<EntityId>>, IndexKernelStop> {
    let mut entries = BTreeMap::new();
    let budget = std::cell::RefCell::new(context);
    let projected_bytes = std::cell::Cell::new(0);
    if let Some(plans) = projection.aspect_plans() {
        for plan in plans.entity_plans.values() {
            if kind_scope.is_some_and(|kind| kind != plan.kind_id)
                || entity_index_projection_scope(plan, field_locator).is_none()
            {
                continue;
            }
            projection
                .try_for_each_entity_field(
                    request_execution,
                    plan.kind_id,
                    field_locator,
                    |units, bytes| {
                        projected_bytes.set(bytes);
                        budget.borrow_mut().checkpoint(units)?;
                        budget.borrow().settle_temporary_scratch(bytes)
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

pub(in crate::internal_port::indexes) fn entity_aspect_field_ordering_value(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    projection: &IndexProjectionSource<'_>,
    entity_id: EntityId,
    field_locator: &AspectFieldLocator,
) -> Result<
    Option<worth_foundational::facade::AspectValue>,
    crate::contracts::execution::denial_cause::RelationalExecutionDenialCause,
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
