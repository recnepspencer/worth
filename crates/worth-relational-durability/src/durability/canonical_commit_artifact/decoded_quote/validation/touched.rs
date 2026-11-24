//! Concrete intrinsic backing for each touched-summary slice.
use super::*;
use contracts::validation::data::custom_rule::touched_scope::touched_ids::entity_ids::InvariantEntityIdsOwnerPort as _;
use contracts::validation::data::custom_rule::touched_scope::touched_ids::partition_ids::InvariantPartitionIdsOwnerPort as _;
use contracts::validation::data::custom_rule::touched_scope::touched_ids::relation_ids::InvariantRelationIdsOwnerPort as _;
use worth_relational_contracts::internal_port as contracts;
pub(super) fn touched_entities(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes =
        shared_vector::<contracts::identity::data::record_ids::EntityId>(value, request, |_| {
            Ok(OwnerAllocation::default())
        })?;
    add(
        &mut bytes,
        contracts::validation::data::custom_rule::touched_scope::touched_ids::entity_ids::InvariantEntityIds::control_bytes()
            .ok_or_else(malformed)?,
    )?;
    Ok(bytes)
}
pub(super) fn touched_relations(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes =
        shared_vector::<contracts::identity::data::record_ids::RelationId>(value, request, |_| {
            Ok(OwnerAllocation::default())
        })?;
    add(
        &mut bytes,
        contracts::validation::data::custom_rule::touched_scope::touched_ids::relation_ids::InvariantRelationIds::control_bytes()
            .ok_or_else(malformed)?,
    )?;
    Ok(bytes)
}
pub(super) fn touched_partitions(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = shared_vector::<contracts::identity::data::primitives::PartitionId>(
        value,
        request,
        |_| Ok(OwnerAllocation::default()),
    )?;
    add(
        &mut bytes,
        contracts::validation::data::custom_rule::touched_scope::touched_ids::partition_ids::InvariantPartitionIds::control_bytes()
            .ok_or_else(malformed)?,
    )?;
    Ok(bytes)
}
