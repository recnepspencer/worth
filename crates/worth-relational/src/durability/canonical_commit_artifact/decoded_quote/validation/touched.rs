//! Concrete intrinsic backing for each touched-summary slice.
use worth_relational_contracts::internal_port::validation::data::custom_rule::touched_scope::touched_ids::entity_ids::InvariantEntityIdsOwnerPort as _;
use worth_relational_contracts::internal_port::validation::data::custom_rule::touched_scope::touched_ids::partition_ids::InvariantPartitionIdsOwnerPort as _;
use worth_relational_contracts::internal_port::validation::data::custom_rule::touched_scope::touched_ids::relation_ids::InvariantRelationIdsOwnerPort as _;
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
