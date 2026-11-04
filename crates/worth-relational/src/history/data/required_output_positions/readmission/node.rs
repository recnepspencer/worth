//! Consume each node once; family validation, ordering and copies are bounded.
use super::*;
use crate::identity::data::{EntityId, PartitionId};

pub(super) fn read(
    cursor: &mut Cursor<'_, '_, '_>,
    context: &mut Readmission<'_, '_, '_>,
    depth: usize,
) -> Result<Option<Arc<PositionNode>>, Refusal> {
    context.charge(1)?;
    if cursor.take_nil().map_err(projected_stop)? {
        return Ok(None);
    }
    if depth == MAXIMUM_HEIGHT {
        return Err(Refusal::InvalidProjectedMeaning);
    }
    require_count(cursor.map_length(), 6)?;
    key(cursor, "family")?;
    let family = cursor.string_bytes().map_err(projected_stop)?;
    key(cursor, "source")?;
    let source = source(cursor)?;
    key(cursor, "partition")?;
    if cursor.marker().map_err(projected_stop)? != 0xc4
        || cursor.length(1).map_err(projected_stop)? != 32
    {
        return Err(Refusal::InvalidProjectedMeaning);
    }
    let partition = cursor
        .take(32)
        .map_err(projected_stop)?
        .try_into()
        .map_err(|_| Refusal::InvalidProjectedMeaning)?;
    key(cursor, "publication")?;
    require_count(cursor.array_length(), 2)?;
    let tag = cursor.unsigned().map_err(projected_stop)?;
    let commit = CommitId(cursor.unsigned().map_err(projected_stop)?);
    if !(context.contains)(commit) {
        return Err(Refusal::InvalidProjectedMeaning);
    }
    let publication = match tag {
        0 => RequiredOutputRoutePosition::Published(commit),
        1 => RequiredOutputRoutePosition::CodecRetired(commit),
        _ => return Err(Refusal::InvalidProjectedMeaning),
    };
    let route_memory = context.reserve::<PublicationRoute>(family.len())?;
    let family = family::copy(family, context)?;
    let route = Arc::new(PublicationRoute {
        family,
        source,
        partition,
        _funding: route_memory,
    });
    key(cursor, "left")?;
    let left = read(cursor, context, depth + 1)?;
    if let Some(previous) = &context.previous {
        if family::compare(previous, &route, context)? != std::cmp::Ordering::Less {
            return Err(Refusal::InvalidProjectedMeaning);
        }
    }
    context.previous = Some(route.clone());
    key(cursor, "right")?;
    let right = read(cursor, context, depth + 1)?;
    let left_height = super::super::height(&left);
    let right_height = super::super::height(&right);
    if left_height.abs_diff(right_height) > 1 {
        return Err(Refusal::InvalidProjectedMeaning);
    }
    let height = left_height
        .max(right_height)
        .checked_add(1)
        .ok_or(Refusal::CapacityOverflow)?;
    let memory = context.reserve::<PositionNode>(0)?;
    Ok(Some(Arc::new(PositionNode {
        route,
        publication,
        left,
        right,
        height,
        _funding: memory,
    })))
}
fn source(cursor: &mut Cursor<'_, '_, '_>) -> Result<EntityId, Refusal> {
    require_count(cursor.map_length(), 3)?;
    key(cursor, "partition_id")?;
    let partition = u32::try_from(cursor.unsigned().map_err(projected_stop)?)
        .map_err(|_| Refusal::InvalidProjectedMeaning)?;
    key(cursor, "local_slot")?;
    let slot = cursor.unsigned().map_err(projected_stop)?;
    key(cursor, "generation")?;
    let generation = u32::try_from(cursor.unsigned().map_err(projected_stop)?)
        .map_err(|_| Refusal::InvalidProjectedMeaning)?;
    Ok(EntityId::new(PartitionId::new(partition), slot, generation))
}
