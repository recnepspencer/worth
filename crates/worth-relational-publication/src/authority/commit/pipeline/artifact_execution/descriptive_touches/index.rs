use super::*;
use worth_relational_contracts::internal_port as contracts;

pub(super) fn append_index(
    selected: &worth_relational_branch::internal_port::branch::root_selection::SelectedRelationalBranchState,
    working: &worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState,
    delta: &CanonicalRecordAspectDelta,
    binding: &EvaluatedAspectBinding,
    field: &worth_foundational::facade::FieldKey,
    definition: &contracts::indexes::data::definition::DerivedIndexDefinition,
    touches: &mut Vec<Touch>,
    budget: &mut TouchBudget<'_, '_, '_, '_, '_>,
) -> Result<(), Failure> {
    let locator = match (&delta.target, &definition.kind) {
        (RecordRef::Entity(_), DerivedIndexKind::EntityField { field_locator })
        | (RecordRef::Relation(_), DerivedIndexKind::RelationField { field_locator }) => {
            field_locator
        }
        _ => return Ok(()),
    };
    if !binding_targets_field(binding, &delta.target, field) {
        return Ok(());
    }
    let locator_visits = u64::try_from(locator.field_path().fields().len())
        .map_err(|_| MapKernelFailure::ResultCapacityExceeded)?;
    budget.checkpoint(locator_visits.saturating_mul(2).saturating_add(2))?;
    let old = authoritative_state(selected, &delta.target);
    let new = authoritative_state(working, &delta.target);
    let old_value =
        worth_relational_branch::internal_port::visibility::materialization::read_records::projection::query_locus_projection::authoritative_state_query_locus_value(
            old, locator,
        );
    let new_value =
        worth_relational_branch::internal_port::visibility::materialization::read_records::projection::query_locus_projection::authoritative_state_query_locus_value(
            new, locator,
        );
    let key_bytes = old_value
        .into_iter()
        .chain(new_value)
        .try_fold(0_u64, |total, value| {
            total
                .checked_add(
                    contracts::storage::data::authoritative_field_comparison_key::AuthoritativeFieldComparisonKey::required_encoded_capacity_bytes(value)
                        .ok_or(MapKernelFailure::ResultCapacityExceeded)?,
                )
                .ok_or(MapKernelFailure::ResultCapacityExceeded)
        })?;
    budget.checkpoint(2 + (key_bytes / 64))?;
    let metadata_bytes = (binding.aspect_key.owned_allocation_capacity_bytes() as u64)
        .checked_add(field.owned_allocation_capacity_bytes() as u64)
        .and_then(|bytes| {
            bytes.checked_add(size_of::<worth_foundational::facade::FieldKey>() as u64)
        })
        .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
    budget.claim(
        key_bytes
            .checked_add(metadata_bytes)
            .ok_or(MapKernelFailure::ResultCapacityExceeded)?,
    )?;
    let old_key = worth_relational_branch::internal_port::visibility::materialization::read_records::projection::query_locus_projection::authoritative_state_query_locus_comparison_key(old, locator);
    let new_key = worth_relational_branch::internal_port::visibility::materialization::read_records::projection::query_locus_projection::authoritative_state_query_locus_comparison_key(new, locator);
    if old_key != new_key {
        push_index_touch(
            touches,
            Touch::IndexMembership {
                index: definition.index_id,
                kind: delta.kind_id,
                aspect: binding.aspect_key.clone(),
                path: CanonicalFieldPath::single(field.clone()),
                old_key,
                new_key,
            },
            budget,
        )?;
    }
    Ok(())
}

fn push_index_touch(
    touches: &mut Vec<Touch>,
    touch: Touch,
    budget: &mut TouchBudget<'_, '_, '_, '_, '_>,
) -> Result<(), Failure> {
    if touches.len() == touches.capacity() {
        let new_capacity = touches.capacity().saturating_mul(2).max(4);
        let growth = new_capacity
            .checked_sub(touches.capacity())
            .and_then(|slots| slots.checked_mul(size_of::<Touch>()))
            .and_then(|bytes| bytes.checked_mul(2))
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
        budget.claim(growth)?;
        touches
            .try_reserve_exact(new_capacity - touches.len())
            .map_err(|_| MapKernelFailure::ResultCapacityExceeded)?;
    }
    touches.push(touch);
    Ok(())
}

fn binding_targets_field(
    binding: &EvaluatedAspectBinding,
    record: &RecordRef,
    field: &worth_foundational::facade::FieldKey,
) -> bool {
    let bound = match (record, &binding.binding) {
        (RecordRef::Entity(_), AspectBinding::EntityField { field })
        | (RecordRef::Relation(_), AspectBinding::RelationField { field }) => Some(field),
        _ => None,
    };
    match &binding.aspect_shape {
        AspectShape::Scalar(_) => bound == Some(field),
        AspectShape::Struct(shape) => bound.is_some() && shape.field(field).is_some(),
        _ => false,
    }
}

pub(super) fn authoritative_state<'a>(
    access: &'a impl CurrentPartitionAccess,
    target: &RecordRef,
) -> Option<&'a worth_foundational::facade::AuthoritativeRecordAspectState> {
    match target {
        RecordRef::Entity(entity) => {
            let arena = &access
                .get_current_partition(partition_of::<EntityRecordKind>(entity))?
                .entity_arena();
            let index = slot_of::<EntityRecordKind>(entity);
            let slot = arena.get_slot(index)?;
            (slot.generation() == entity.generation.0
                && slot.lifecycle() == RecordLifecycleState::Live)
                .then_some(arena.extra_at(index)?.authoritative_aspect_state())
                .flatten()
        }
        RecordRef::Relation(relation) => {
            let arena = &access
                .get_current_partition(partition_of::<RelationRecordKind>(relation))?
                .relation_arena();
            let index = slot_of::<RelationRecordKind>(relation);
            let slot = arena.get_slot(index)?;
            (slot.generation() == relation.generation.0
                && slot.lifecycle() == RecordLifecycleState::Live)
                .then_some(arena.extra_at(index)?.authoritative_aspect_state())
                .flatten()
        }
    }
}

pub(super) fn targets_field(
    binding: &EvaluatedAspectBinding,
    record: &RecordRef,
    field: &worth_foundational::facade::FieldKey,
    definition: &contracts::indexes::data::definition::DerivedIndexDefinition,
) -> bool {
    matches!(
        (record, &definition.kind),
        (RecordRef::Entity(_), DerivedIndexKind::EntityField { .. })
            | (
                RecordRef::Relation(_),
                DerivedIndexKind::RelationField { .. }
            )
    ) && binding_targets_field(binding, record, field)
}
