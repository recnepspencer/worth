use crate::storage::data::EntityReadRecord;

pub(super) fn current_entity_snapshot(
    runtime: &crate::runtime::RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    entity_id: crate::identity::data::EntityId,
) -> Result<Option<EntityReadRecord>, crate::execution::RelationalExecutionDenialCause> {
    let current_state = runtime.storage_access().current_edition();
    crate::visibility::materialization::read_records::read_work::with_full_residency(
        execution,
        &current_state,
        &runtime.services.symbols,
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
            let Ok(kind) = runtime.config.schema.registry.resolve_entity(kind_id) else {
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
            .ok_or(crate::execution::RelationalExecutionDenialCause::StorageUnavailable(
                crate::storage::data::RelationalStorageUnavailable {
                    subject: crate::storage::data::RelationalStorageSubject::Region(entity_id.partition_id),
                    cause: crate::storage::data::RelationalStorageUnavailableCause::Damaged,
                }
            ))?,
        retired_at_version: slot.retired_at(),
        authoritative_aspect_state: slot.extra().authoritative_aspect_state.clone(),
    }))
        },
    )
}
