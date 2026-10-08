use crate::identity::data::{EntityId, KindId};
use crate::storage::data::RecordLifecycleState;
use crate::storage::overlay::PartitionAccess;
use worth_foundational::facade::AspectFieldLocator;

/// The immutable selected root supplies both slot state and schema authority.
pub(in crate::indexes) fn exact_entity_field_matches(
    root: &crate::branch::RelationalBranchRoot,
    entity_id: EntityId,
    locator: &AspectFieldLocator,
    expected: &worth_foundational::facade::AspectValue,
) -> Option<(KindId, bool)> {
    let slot = root
        .get_partition(entity_id.partition_id)?
        .entity_arena
        .get_slot(entity_id.slot_index())?;
    if slot.lifecycle() != RecordLifecycleState::Live
        || (!entity_id.generation.is_zero() && slot.generation() != entity_id.generation_value())
    {
        return None;
    }
    let kind = slot.kind_id()?;
    root.schema_authority()
        .registry()
        .entity_registration(kind)
        .ok()?;
    let matches =
        crate::visibility::materialization::read_records::authoritative_state_query_locus_value(
            slot.extra().authoritative_aspect_state.as_ref(),
            locator,
        )
        .is_some_and(|actual| actual == expected);
    Some((kind, matches))
}
