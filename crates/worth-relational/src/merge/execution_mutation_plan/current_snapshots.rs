use crate::storage::data::EntityReadRecord;
use crate::storage::overlay::PartitionAccess;

pub(super) fn current_entity_snapshot(
    runtime: &crate::runtime::RelationalRuntime,
    entity_id: crate::identity::data::EntityId,
) -> Option<EntityReadRecord> {
    let current_state = runtime.storage_access().current_edition();
    let partition = current_state.get_partition(entity_id.partition_id)?;
    let slot = partition.entity_arena.get(&entity_id)?;
    let kind_id = slot.kind_id()?;
    let kind = runtime
        .config
        .schema
        .registry
        .resolve_entity(kind_id)
        .ok()?;
    Some(EntityReadRecord {
        entity_id,
        lineage_id: None,
        kind,
        lifecycle: slot.lifecycle(),
        created_at_version: partition
            .entity_arena
            .created_at_for_slot(entity_id.local_slot.0 as usize)
            .expect(/* Proof: The owner selects this slot from the same retained arena after existence and visibility admission; arena materialization installs its kind, extra and creation columns together. */ "current entity snapshot has creation metadata"),
        retired_at_version: slot.retired_at(),
        authoritative_aspect_state: slot.extra().authoritative_aspect_state.clone(),
    })
}
