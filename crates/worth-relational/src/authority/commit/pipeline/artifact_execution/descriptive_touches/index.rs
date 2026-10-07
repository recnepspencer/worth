use super::*;

pub(super) fn append_index(
    _runtime: &crate::runtime::RelationalPreparationRuntime,
    selected: &crate::branch::SelectedRelationalBranchState,
    working: &crate::runtime::WorkingState,
    delta: &CanonicalRecordAspectDelta,
    binding: &EvaluatedAspectBinding,
    field: &worth_foundational::facade::FieldKey,
    definition: &crate::indexes::data::DerivedIndexDefinition,
    touches: &mut Vec<Touch>,
    budget: &mut TouchBudget<'_, '_, '_, '_>,
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
    append_index_for_field(
        IndexTouchInput {
            selected,
            working,
            old_record: &delta.target,
            new_record: &delta.target,
            old_kind: Some(delta.kind_id),
            new_kind: Some(delta.kind_id),
            aspect: &binding.aspect_key,
            field,
            definition,
            locator,
        },
        touches,
        budget,
    )
}

pub(super) struct IndexTouchInput<'a> {
    pub(super) selected: &'a crate::branch::SelectedRelationalBranchState,
    pub(super) working: &'a crate::runtime::WorkingState,
    pub(super) old_record: &'a RecordRef,
    pub(super) new_record: &'a RecordRef,
    pub(super) old_kind: Option<crate::identity::data::KindId>,
    pub(super) new_kind: Option<crate::identity::data::KindId>,
    pub(super) aspect: &'a worth_foundational::facade::AspectKey,
    pub(super) field: &'a worth_foundational::facade::FieldKey,
    pub(super) definition: &'a crate::indexes::data::DerivedIndexDefinition,
    pub(super) locator: &'a worth_foundational::facade::AspectFieldLocator,
}

pub(super) fn append_index_for_field(
    input: IndexTouchInput<'_>,
    touches: &mut Vec<Touch>,
    budget: &mut TouchBudget<'_, '_, '_, '_>,
) -> Result<(), Failure> {
    let IndexTouchInput {
        selected,
        working,
        old_record,
        new_record,
        old_kind,
        new_kind,
        aspect,
        field,
        definition,
        locator,
    } = input;
    let locator_visits = u64::try_from(locator.field_path().fields().len())
        .map_err(|_| MapKernelFailure::ResultCapacityExceeded)?;
    budget.checkpoint(locator_visits.saturating_mul(2).saturating_add(2))?;
    let old = authoritative_state(selected, old_record);
    let new = authoritative_state(working, new_record);
    let old_value =
        crate::visibility::materialization::read_records::authoritative_state_query_locus_value(
            old, locator,
        );
    let new_value =
        crate::visibility::materialization::read_records::authoritative_state_query_locus_value(
            new, locator,
        );
    let key_bytes = old_value
        .into_iter()
        .chain(new_value)
        .try_fold(0_u64, |total, value| {
            total
                .checked_add(
                    crate::storage::data::AuthoritativeFieldComparisonKey::required_encoded_capacity_bytes(value)
                        .ok_or(MapKernelFailure::ResultCapacityExceeded)?,
                )
                .ok_or(MapKernelFailure::ResultCapacityExceeded)
        })?;
    budget.checkpoint(2 + (key_bytes / 64))?;
    let metadata_bytes = (aspect.owned_allocation_capacity_bytes() as u64)
        .checked_add(field.owned_allocation_capacity_bytes() as u64)
        .and_then(|bytes| {
            bytes.checked_add(size_of::<worth_foundational::facade::FieldKey>() as u64)
        })
        .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
    let metadata_copies = if old_kind != new_kind { 2_u64 } else { 1_u64 };
    budget.claim(
        key_bytes
            .checked_add(
                metadata_bytes
                    .checked_mul(metadata_copies)
                    .ok_or(MapKernelFailure::ResultCapacityExceeded)?,
            )
            .ok_or(MapKernelFailure::ResultCapacityExceeded)?,
    )?;
    let old_key = crate::visibility::materialization::read_records::authoritative_state_query_locus_comparison_key(old, locator);
    let new_key = crate::visibility::materialization::read_records::authoritative_state_query_locus_comparison_key(new, locator);
    if old_key != new_key || old_kind != new_kind || old_record != new_record {
        if old_kind == new_kind {
            if let Some(kind) = new_kind {
                super::merge::push(
                    touches,
                    Touch::IndexMembership {
                        index: definition.index_id,
                        kind,
                        aspect: aspect.clone(),
                        path: CanonicalFieldPath::single(field.clone()),
                        old_key,
                        new_key,
                    },
                    budget,
                )?;
            }
        } else {
            if let (Some(kind), Some(old_key)) = (old_kind, old_key) {
                super::merge::push(
                    touches,
                    Touch::IndexMembership {
                        index: definition.index_id,
                        kind,
                        aspect: aspect.clone(),
                        path: CanonicalFieldPath::single(field.clone()),
                        old_key: Some(old_key),
                        new_key: None,
                    },
                    budget,
                )?;
            }
            if let (Some(kind), Some(new_key)) = (new_kind, new_key) {
                super::merge::push(
                    touches,
                    Touch::IndexMembership {
                        index: definition.index_id,
                        kind,
                        aspect: aspect.clone(),
                        path: CanonicalFieldPath::single(field.clone()),
                        old_key: None,
                        new_key: Some(new_key),
                    },
                    budget,
                )?;
            }
        }
    }
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

fn authoritative_state<'a>(
    access: &'a impl PartitionAccess,
    target: &RecordRef,
) -> Option<&'a worth_foundational::facade::AuthoritativeRecordAspectState> {
    match target {
        RecordRef::Entity(entity) => {
            let arena = &access
                .get_partition(partition_of::<EntityRecordKind>(entity))?
                .entity_arena;
            let index = slot_of::<EntityRecordKind>(entity);
            let slot = arena.get_slot(index)?;
            (slot.generation() == entity.generation.0
                && slot.lifecycle() == RecordLifecycleState::Live)
                .then_some(arena.extra_at(index)?.authoritative_aspect_state.as_ref())
                .flatten()
        }
        RecordRef::Relation(relation) => {
            let arena = &access
                .get_partition(partition_of::<RelationRecordKind>(relation))?
                .relation_arena;
            let index = slot_of::<RelationRecordKind>(relation);
            let slot = arena.get_slot(index)?;
            (slot.generation() == relation.generation.0
                && slot.lifecycle() == RecordLifecycleState::Live)
                .then_some(arena.extra_at(index)?.authoritative_aspect_state.as_ref())
                .flatten()
        }
    }
}
