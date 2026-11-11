use crate::execution::RelationalExecutionDenialCause;
use crate::identity::data::{EntityId, KindId};
use crate::storage::data::RecordLifecycleState;
use crate::visibility::snapshot_states::ResidentVisibilitySnapshotBasis;
use worth_foundational::facade::AspectFieldLocator;

/// The immutable selected root supplies both slot state and schema authority.
pub(in crate::indexes) fn exact_entity_field_matches(
    basis: &ResidentVisibilitySnapshotBasis,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    entity_id: EntityId,
    locator: &AspectFieldLocator,
    expected: &worth_foundational::facade::AspectValue,
) -> Result<Option<(KindId, bool)>, RelationalExecutionDenialCause> {
    basis.demand_regions_head(execution, [entity_id.partition_id])?;
    let Some(partition) = basis.head_region(entity_id.partition_id) else {
        return Ok(None);
    };
    let Some(slot) = partition.entity_arena().get_slot(entity_id.slot_index()) else {
        return Ok(None);
    };
    if slot.lifecycle() != RecordLifecycleState::Live
        || (!entity_id.generation.is_zero() && slot.generation() != entity_id.generation_value())
    {
        return Ok(None);
    }
    let Some(kind) = slot.kind_id() else {
        return Ok(None);
    };
    if basis
        .root()
        .schema_authority()
        .registry()
        .entity_registration(kind)
        .is_err()
    {
        return Ok(None);
    }
    let matches =
        crate::visibility::materialization::read_records::authoritative_state_query_locus_value(
            slot.extra().authoritative_aspect_state(),
            locator,
        )
        .is_some_and(|actual| actual == expected);
    Ok(Some((kind, matches)))
}
