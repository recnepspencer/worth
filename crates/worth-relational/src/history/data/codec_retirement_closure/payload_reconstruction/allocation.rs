//! Allocation admission for the actual owned portable/readmitted phases.
use super::portable_input::add;
use super::*;

pub(super) fn reserve(
    lease: Option<&ExecutionResourceLease<'_>>,
    bytes: usize,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<worth_execution::ExecutionMemoryReservation, Stop> {
    super::super::charge(context, bytes)?;
    super::super::reserve(lease, bytes)
}

pub(super) fn admission(
    candidate: &PortableRecordAspectPatch,
    plan: &LoweredAspectContractPlan,
) -> Result<AspectPatchStorageBounds, Stop> {
    let mut bounds = candidate
        .readmission_storage_bounds()
        .ok_or_else(super::super::memory_overflow)?;
    let mut lookup_peak = 0usize;
    for operation in candidate.operations() {
        let contract = &plan
            .executable_bindings
            .iter()
            .find(|binding| binding.contract.key() == operation.basis().key())
            .ok_or("output patch has no installed physical contract")?
            .contract;
        // The closed lookup clones once; validation keeps one distinct cloned
        // contract in the admitted patch. Portable values themselves MOVE.
        let contract_bytes = contract.owned_allocation_capacity_bytes();
        lookup_peak = lookup_peak.max(contract_bytes);
        add(&mut bounds.retained_bytes, contract_bytes)?;
        match operation {
            PortableAspectPatchOperation::SetWhole { .. } => add(
                &mut bounds.retained_bytes,
                contract.key().owned_allocation_capacity_bytes(),
            )?,
            PortableAspectPatchOperation::ClearWhole { .. } => {}
            PortableAspectPatchOperation::PatchFields {
                selected_fields, ..
            } => {
                // The first mask consumes these owned keys; the admitted
                // mask clones them. Its typed backing is already quoted above.
                for key in selected_fields {
                    add(
                        &mut bounds.retained_bytes,
                        key.owned_allocation_capacity_bytes(),
                    )?;
                }
            }
        }
    }
    add(&mut bounds.transient_bytes, lookup_peak)?;
    Ok(bounds)
}
