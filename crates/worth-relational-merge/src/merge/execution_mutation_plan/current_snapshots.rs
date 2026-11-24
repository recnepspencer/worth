use crate::contracts::storage::data::read_records::EntityReadRecord;

pub fn current_entity_snapshot(
    current_state: &worth_relational_storage::internal_port::runtime::state::subsystems::storage::partition_edition::PartitionEdition,
    symbols: &worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    schema: &crate::contracts::schema::data::RelationalSchemaRegistry,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    entity_id: crate::contracts::identity::data::record_ids::EntityId,
) -> Result<
    Option<EntityReadRecord>,
    crate::contracts::execution::denial_cause::RelationalExecutionDenialCause,
> {
    worth_relational_branch::internal_port::visibility::materialization::read_records::read_work::with_full_residency(
        execution,
        current_state,
        symbols,
        |state| {
            let Some(partition) = state.get_partition(entity_id.partition_id) else {
                return Ok(None);
            };
            let Some(slot) = partition.entity_arena.get(&entity_id) else {
                return Ok(None);
            };
            let Some(kind_id) = slot.kind_id() else {
                return Ok(None);
            };
            let Ok(kind) = schema.resolve_entity(kind_id) else {
                return Ok(None);
            };
            Ok(Some(EntityReadRecord {
        entity_id,
        lineage_id: None,
        kind,
        lifecycle: slot.lifecycle(),
        created_at_version: partition
            .entity_arena
            .created_at_for_slot(entity_id.local_slot.0 as usize)
            .ok_or(crate::contracts::execution::denial_cause::RelationalExecutionDenialCause::StorageUnavailable(
                crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailable {
                    subject: crate::contracts::storage::data::storage_unavailable::RelationalStorageSubject::Region(entity_id.partition_id),
                    cause: crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause::Damaged,
                }
            ))?,
        retired_at_version: slot.retired_at(),
        authoritative_aspect_state: slot.extra().authoritative_aspect_state().cloned(),
    }))
        },
    )
}
