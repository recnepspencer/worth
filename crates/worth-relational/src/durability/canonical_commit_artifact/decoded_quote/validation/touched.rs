//! Concrete intrinsic backing for each touched-summary slice.
use super::*;
pub(super) fn touched_entities(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = shared_vector::<crate::identity::data::EntityId>(value, request, |_| {
        Ok(OwnerAllocation::default())
    })?;
    add(
        &mut bytes,
        crate::validation::data::InvariantEntityIds::control_bytes().ok_or_else(malformed)?,
    )?;
    Ok(bytes)
}
pub(super) fn touched_relations(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = shared_vector::<crate::identity::data::RelationId>(value, request, |_| {
        Ok(OwnerAllocation::default())
    })?;
    add(
        &mut bytes,
        crate::validation::data::InvariantRelationIds::control_bytes().ok_or_else(malformed)?,
    )?;
    Ok(bytes)
}
pub(super) fn touched_partitions(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = shared_vector::<crate::identity::data::PartitionId>(value, request, |_| {
        Ok(OwnerAllocation::default())
    })?;
    add(
        &mut bytes,
        crate::validation::data::InvariantPartitionIds::control_bytes().ok_or_else(malformed)?,
    )?;
    Ok(bytes)
}
