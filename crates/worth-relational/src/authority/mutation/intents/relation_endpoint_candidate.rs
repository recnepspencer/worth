//! Endpoint augmentation retains the original operation Vec and its funding.
use super::field_authoring_candidate::{self as allocation, FieldCandidate};
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::identity::data::EntityId;
use crate::schema::data::{AspectBinding, LoweredAspectContractPlan};
use worth_foundational::facade::{
    AspectValue, ContractValidationInput, PortableAspectContractBasis, PortableAspectPatchOperation,
};

pub(super) fn append_authoritative_endpoints(
    request: &worth_execution::ExecutionRequest<'_, '_>,
    mut candidate: FieldCandidate,
    plan: Option<&LoweredAspectContractPlan>,
    source: EntityId,
    target: EntityId,
) -> Result<FieldCandidate, super::record_aspect_patch::MutationPatchDenial> {
    let Some(plan) = plan else {
        return Ok(candidate);
    };
    let mut count = 0usize;
    let mut keys = 0u64;
    for binding in &plan.executable_bindings {
        if matches!(
            binding.target,
            AspectBinding::RelationSourceEndpoint | AspectBinding::RelationTargetEndpoint
        ) {
            count = count.checked_add(1).ok_or(Cause::ChargedBytesOverflow)?;
            keys = keys
                .checked_add(
                    u64::try_from(binding.aspect_key().as_str().len())
                        .map_err(|_| Cause::ChargedBytesOverflow)?,
                )
                .ok_or(Cause::ChargedBytesOverflow)?;
        }
    }
    let length = candidate
        .operations
        .len()
        .checked_add(count)
        .ok_or(Cause::ChargedBytesOverflow)?;
    let old_capacity = candidate.operations.capacity();
    let grew = length > old_capacity;
    let old_backing = bytes(old_capacity)?;
    let new_backing = if grew { bytes(length)? } else { old_backing };
    let old_bill = candidate
        .funding
        .as_ref()
        .map_or(0, worth_execution::ExecutionMemoryReservation::bytes);
    let final_bill = old_bill
        .checked_add(keys)
        .and_then(|n| n.checked_add(new_backing.checked_sub(old_backing)?))
        .ok_or(Cause::ChargedBytesOverflow)?;
    let peak = final_bill
        .checked_add(if grew { old_backing } else { 0 })
        .ok_or(Cause::ChargedBytesOverflow)?;
    allocation::resize(request, &mut candidate.funding, peak)?;
    candidate.operations.reserve_exact(count);
    for binding in &plan.executable_bindings {
        let endpoint = match binding.target {
            AspectBinding::RelationSourceEndpoint => source,
            AspectBinding::RelationTargetEndpoint => target,
            _ => continue,
        };
        candidate
            .operations
            .push(PortableAspectPatchOperation::SetWhole {
                basis: PortableAspectContractBasis::from_contract(&binding.contract),
                value: ContractValidationInput::Scalar(AspectValue::EntityRef(foundational_id(
                    endpoint,
                ))),
            });
    }
    // reserve_exact's displaced buffer is gone before its overlap bill refunds.
    allocation::resize(request, &mut candidate.funding, final_bill)?;
    Ok(candidate)
}
fn bytes(count: usize) -> Result<u64, Cause> {
    std::alloc::Layout::array::<PortableAspectPatchOperation>(count)
        .ok()
        .and_then(|layout| u64::try_from(layout.size()).ok())
        .ok_or(Cause::ChargedBytesOverflow)
}
fn foundational_id(entity_id: EntityId) -> worth_foundational::facade::EntityId {
    worth_foundational::facade::EntityId {
        partition_id: worth_foundational::facade::PartitionId(entity_id.partition_id.as_u32()),
        local_slot: worth_foundational::facade::LocalSlot(entity_id.local_slot_value()),
        generation: worth_foundational::facade::Generation(entity_id.generation_value()),
    }
}
