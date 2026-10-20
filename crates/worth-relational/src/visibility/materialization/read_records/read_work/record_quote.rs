//! Borrowed metadata bounds the established owner-accounted clone payload.
//! Schema strings clone their length; aspect state uses the authoritative
//! `owned_allocation_capacity_bytes` measure. Collectors settle that same measure
//! immediately after cloning. It excludes allocator and BTree node structural
//! backing, so this is not a bound on the clone's complete physical allocation.
use super::super::reader::KindScanVisibility;
use super::super::visibility::{visible_metadata, visible_relation_metadata};
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::schema::data::RelationalSchemaRegistry;
use crate::storage::overlay::PartitionState;

// Compute the payload count while each arena borrowed-record wrapper is alive.
pub(crate) fn entity(
    registry: &RelationalSchemaRegistry,
    partition: &PartitionState,
    slot: usize,
    visibility: KindScanVisibility,
) -> Result<u64, Cause> {
    let selected = match visibility {
        KindScanVisibility::Live => partition.entity_arena.get_slot(slot).and_then(|record| {
            Some((
                record.kind_id()?,
                record
                    .extra()
                    .authoritative_aspect_state
                    .as_ref()
                    .map_or(0, |state| state.owned_allocation_capacity_bytes()),
            ))
        }),
        KindScanVisibility::AtVersion(version) => partition
            .entity_arena
            .metadata_history_at(slot)
            .and_then(|history| visible_metadata(history, version))
            .map(|metadata| {
                (
                    metadata.kind_id,
                    metadata
                        .authoritative_aspect_state
                        .as_ref()
                        .map_or(0, |state| state.owned_allocation_capacity_bytes()),
                )
            }),
    };
    let Some((kind, state)) = selected else {
        return Ok(0);
    };
    let Ok(registration) = registry.entity_registration(kind) else {
        return Ok(0);
    };
    clone_bytes(
        registration.kind_name.len(),
        registration.schema_id.0.len(),
        state,
    )
}

pub(crate) fn relation(
    registry: &RelationalSchemaRegistry,
    partition: &PartitionState,
    slot: usize,
    visibility: KindScanVisibility,
) -> Result<u64, Cause> {
    let selected = match visibility {
        KindScanVisibility::Live => partition.relation_arena.get_slot(slot).and_then(|record| {
            Some((
                record.kind_id()?,
                record
                    .extra()
                    .authoritative_aspect_state
                    .as_ref()
                    .map_or(0, |state| state.owned_allocation_capacity_bytes()),
            ))
        }),
        KindScanVisibility::AtVersion(version) => {
            visible_relation_metadata(partition, slot, version).map(|metadata| {
                (
                    metadata.kind_id,
                    metadata
                        .authoritative_aspect_state
                        .as_ref()
                        .map_or(0, |state| state.owned_allocation_capacity_bytes()),
                )
            })
        }
    };
    let Some((kind, state)) = selected else {
        return Ok(0);
    };
    let Ok(registration) = registry.relation_registration(kind) else {
        return Ok(0);
    };
    clone_bytes(
        registration.kind_name.len(),
        registration.schema_id.0.len(),
        state,
    )
}

fn clone_bytes(kind: usize, schema: usize, state_payload_bytes: usize) -> Result<u64, Cause> {
    kind.checked_add(schema)
        .and_then(|bytes| bytes.checked_add(state_payload_bytes))
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(Cause::ChargedBytesOverflow)
}

pub(crate) fn entity_for_id(
    state: &(impl crate::storage::overlay::PartitionAccess + ?Sized),
    registry: &RelationalSchemaRegistry,
    id: crate::identity::data::EntityId,
    visibility: KindScanVisibility,
) -> Result<u64, Cause> {
    state
        .get_partition(id.partition_id)
        .map_or(Ok(0), |partition| {
            entity(registry, partition, id.slot_index(), visibility)
        })
}
pub(crate) fn relation_for_id(
    state: &(impl crate::storage::overlay::PartitionAccess + ?Sized),
    registry: &RelationalSchemaRegistry,
    id: crate::identity::data::RelationId,
    visibility: KindScanVisibility,
) -> Result<u64, Cause> {
    state
        .get_partition(id.partition_id)
        .map_or(Ok(0), |partition| {
            relation(registry, partition, id.slot_index(), visibility)
        })
}
