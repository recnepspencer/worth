//! Exact field authority for a struct initialized on the validated pre-effect basis.
use super::{ValidatedMutationTouch, ValidatedMutationTouchProjectionWork};
use crate::identity::data::{EntityId, KindId};
use crate::storage::overlay::PartitionAccess;
use crate::transactions::data::planned_aspect_field_locator;
use std::collections::BTreeSet;
use worth_foundational::facade::{
    CanonicalFieldPath, ContractValidationInput, PortableAspectPatchOperation,
};

pub(super) fn project(
    entity: EntityId,
    kind: KindId,
    operation: &PortableAspectPatchOperation,
    state: &crate::storage::overlay::WorkingState,
    touches: &mut BTreeSet<ValidatedMutationTouch>,
    work: &mut ValidatedMutationTouchProjectionWork,
) -> bool {
    let PortableAspectPatchOperation::SetWhole {
        basis,
        value: ContractValidationInput::Struct(value),
    } = operation
    else {
        return false;
    };
    // Empty initialization changes aspect presence without writing any field.
    // It cannot acquire a field-only authority by producing an empty touch set.
    if value.fields().next().is_none() {
        return false;
    }
    work.owner_state_lookups += 1;
    let Some(slot) = state
        .get_partition(entity.partition_id)
        .and_then(|partition| partition.entity_arena.get_slot(entity.slot_index()))
    else {
        return false;
    };
    if slot.generation() != entity.generation_value()
        || slot
            .extra()
            .authoritative_aspect_state
            .as_ref()
            .is_some_and(|aspects| aspects.get(basis.key()).is_some())
    {
        return false;
    }
    // This is the owner's immutable pre-effect basis, not a consumer claim.
    // Validation checked the complete value; commit rechecks the same basis.
    for (field, _) in value.fields() {
        touches.insert(ValidatedMutationTouch::WriteEntityField {
            kind,
            locator: planned_aspect_field_locator(
                basis.key().clone(),
                CanonicalFieldPath::single(field.clone()),
            ),
        });
    }
    true
}
