use super::{visible_metadata, visible_relation_metadata};
use crate::storage::data::{EntityReadRecord, RelationReadRecord};
use crate::storage::overlay::PartitionState;
pub(super) fn candidate_entity_bytes(
    source: &(impl super::ProjectionRecordSource + ?Sized),
    partition: &PartitionState,
    slot: usize,
) -> u64 {
    let selected = if source.is_exact_basis() {
        partition.entity_arena.get_slot(slot).and_then(|record| {
            Some((
                record.kind_id()?,
                record
                    .extra()
                    .authoritative_aspect_state
                    .as_ref()
                    .map_or(0, |state| state.owned_allocation_capacity_bytes() as u64),
            ))
        })
    } else {
        partition
            .entity_arena
            .metadata_history_at(slot)
            .and_then(|history| visible_metadata(history, source.version_id()))
            .map(|metadata| {
                (
                    metadata.kind_id,
                    metadata
                        .authoritative_aspect_state
                        .as_ref()
                        .map_or(0, |state| state.owned_allocation_capacity_bytes() as u64),
                )
            })
    };
    let Some((kind_id, state_bytes)) = selected else {
        return 0;
    };
    let names = source
        .selected_schema_authority()
        .and_then(|authority| authority.registry().entity_registration(kind_id).ok())
        .map_or(0, |registration| {
            registration
                .kind_name
                .capacity()
                .saturating_add(registration.schema_id.0.capacity()) as u64
        });
    (std::mem::size_of::<EntityReadRecord>() as u64)
        .saturating_add(names)
        .saturating_add(state_bytes)
}

pub(super) fn candidate_relation_bytes(
    source: &(impl super::ProjectionRecordSource + ?Sized),
    partition: &PartitionState,
    slot: usize,
) -> u64 {
    let selected = if source.is_exact_basis() {
        partition.relation_arena.get_slot(slot).and_then(|record| {
            Some((
                record.kind_id()?,
                record
                    .extra()
                    .authoritative_aspect_state
                    .as_ref()
                    .map_or(0, |state| state.owned_allocation_capacity_bytes() as u64),
            ))
        })
    } else {
        visible_relation_metadata(partition, slot, source.version_id()).map(|metadata| {
            (
                metadata.kind_id,
                metadata
                    .authoritative_aspect_state
                    .as_ref()
                    .map_or(0, |state| state.owned_allocation_capacity_bytes() as u64),
            )
        })
    };
    let Some((kind_id, state_bytes)) = selected else {
        return 0;
    };
    let names = source
        .selected_schema_authority()
        .and_then(|authority| authority.registry().relation_registration(kind_id).ok())
        .map_or(0, |registration| {
            registration
                .kind_name
                .capacity()
                .saturating_add(registration.schema_id.0.capacity()) as u64
        });
    (std::mem::size_of::<RelationReadRecord>() as u64)
        .saturating_add(names)
        .saturating_add(state_bytes)
}
