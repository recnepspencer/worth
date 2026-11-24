//! Compare the actual Stored aspect-version rows under the original request.
#[cfg(test)]
use crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;
#[cfg(test)]
use crate::runtime::state::subsystems::storage::StorageSubsystem;
#[cfg(test)]
use worth_foundational::facade::AspectKey;

/// Enumerates aspect versions under the caller request.
#[cfg(test)]
pub(crate) fn entity_aspect_versions(
    partitions: &StorageSubsystem,
    symbols: &RuntimeSymbolTable,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    entity_id: crate::identity::data::EntityId,
) -> Result<Option<Vec<(AspectKey, u64)>>, super::query_execution::QueryReadExecutionStop> {
    super::query_preparation::consult_query_preparation_request(execution)?;
    let Some(partition) = partitions.partition(entity_id.partition_id) else {
        return Ok(None);
    };
    let slot = entity_id.slot_index();
    let Some(slot_view) = partition.entity_arena.get_slot(slot) else {
        return Ok(None);
    };
    if slot_view.generation() != entity_id.generation_value()
        || slot_view.partition_id() != entity_id.partition_id
    {
        return Ok(None);
    }
    let Some(symbolic) = partition.entity_arena.aspect_versions_at(slot) else {
        return Ok(None);
    };
    let versions = super::aspect_version_read::read_versions(symbols, execution, symbolic, false)?;
    debug_assert!(aspect_versions_are_canonical(&versions));
    Ok(Some(versions))
}
/// Enumerates and orders aspect versions under the caller request.
#[cfg(test)]
pub(crate) fn relation_aspect_versions(
    partitions: &StorageSubsystem,
    symbols: &RuntimeSymbolTable,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    relation_id: crate::identity::data::RelationId,
) -> Result<Option<Vec<(AspectKey, u64)>>, super::query_execution::QueryReadExecutionStop> {
    super::query_preparation::consult_query_preparation_request(execution)?;
    let Some(partition) = partitions.partition(relation_id.partition_id) else {
        return Ok(None);
    };
    let slot = relation_id.slot_index();
    let Some(slot_view) = partition.relation_arena.get_slot(slot) else {
        return Ok(None);
    };
    if slot_view.generation() != relation_id.generation_value()
        || slot_view.partition_id() != relation_id.partition_id
    {
        return Ok(None);
    }
    let Some(symbolic) = partition.relation_arena.aspect_versions_at(slot) else {
        return Ok(None);
    };
    let versions = super::aspect_version_read::read_versions(symbols, execution, symbolic, true)?;
    debug_assert!(aspect_versions_are_canonical(&versions));
    Ok(Some(versions))
}
#[cfg(test)]
fn aspect_versions_are_canonical(versions: &[(AspectKey, u64)]) -> bool {
    versions.windows(2).all(|window| window[0] <= window[1])
}
