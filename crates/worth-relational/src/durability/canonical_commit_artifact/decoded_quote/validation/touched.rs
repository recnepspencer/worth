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
        crate::validation::data::custom_rule::touched_scope::InvariantEntityIds::control_bytes()
            .ok_or_else(malformed)?,
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
        crate::validation::data::custom_rule::touched_scope::InvariantRelationIds::control_bytes()
            .ok_or_else(malformed)?,
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
        crate::validation::data::custom_rule::touched_scope::InvariantPartitionIds::control_bytes()
            .ok_or_else(malformed)?,
    )?;
    Ok(bytes)
}
