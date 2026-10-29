//! A whole-aspect dependency includes every genuinely installed field writer.
use super::*;

pub(super) fn measure(
    execution: ExecutionRequest<'_, '_>,
    original: &VisibilityProjectionView,
    target: &VisibilityProjectionView,
    entity: worth_relational::facade::identity::EntityId,
    aspect: &worth_foundational::facade::AspectKey,
) -> Result<(usize, u64), Stop> {
    let mut count = 2usize;
    let aspect_bytes = aspect.owned_allocation_capacity_bytes() as u64;
    let mut bytes = aspect_bytes;
    for field in original
        .entity_aspect_support_fields(&execution, target, entity, aspect)
        .map_err(Stop::FieldSupport)?
    {
        execution.consult().map_err(Stop::Request)?;
        count = count.checked_add(1).ok_or(Stop::SizeOverflow)?;
        bytes = bytes
            .checked_add(aspect_bytes)
            .and_then(|n| n.checked_add(std::mem::size_of_val(field) as u64))
            .and_then(|n| n.checked_add(field.owned_allocation_capacity_bytes() as u64))
            .ok_or(Stop::SizeOverflow)?;
    }
    Ok((count, bytes))
}

pub(super) fn append(
    execution: ExecutionRequest<'_, '_>,
    original: &VisibilityProjectionView,
    target: &VisibilityProjectionView,
    entity: worth_relational::facade::identity::EntityId,
    aspect: &worth_foundational::facade::AspectKey,
    keys: &mut ExecutionRequestArrayBuilder<'_, '_, Key>,
) -> Result<(), Stop> {
    keys.push(Key::Record {
        record: RecordRef::Entity(entity),
        scope: Scope::WholeAspect(aspect.clone()),
    })
    .map_err(array_stop)?;
    for field in original
        .entity_aspect_support_fields(&execution, target, entity, aspect)
        .map_err(Stop::FieldSupport)?
    {
        execution.consult().map_err(Stop::Request)?;
        keys.push(Key::Record {
            record: RecordRef::Entity(entity),
            scope: Scope::Field(aspect.clone(), CanonicalFieldPath::single(field.clone())),
        })
        .map_err(array_stop)?;
    }
    Ok(())
}
