//! Concrete borrowed-roster, portable and admitted patch allocations.
use super::*;
use std::alloc::Layout;
use worth_foundational::facade::*;
pub(super) fn backing<T>(count: usize) -> Result<u64, Cause> {
    Layout::array::<T>(count)
        .ok()
        .and_then(|layout| u64::try_from(layout.size()).ok())
        .ok_or(Cause::ChargedBytesOverflow)
}
fn add(total: &mut u64, bytes: u64) -> Result<(), Cause> {
    *total = total
        .checked_add(bytes)
        .ok_or(Cause::ChargedBytesOverflow)?;
    Ok(())
}
fn nested(total: &mut u64, bytes: usize) -> Result<(), Cause> {
    add(
        total,
        u64::try_from(bytes).map_err(|_| Cause::ChargedBytesOverflow)?,
    )
}
pub(super) fn input(
    rows: &[selection::Selected<'_>],
    purpose: PortablePatchReadmissionPurpose,
) -> Result<(u64, u64), Cause> {
    let mut retained = backing::<PortableAspectPatchOperation>(selection::groups(rows).count())?;
    let mut transient = 0;
    for group in selection::groups(rows) {
        nested(&mut retained, group[0].binding.aspect_key().as_str().len())?;
        if !group[0].scalar {
            if purpose == PortablePatchReadmissionPurpose::RecordCreation {
                add(
                    &mut retained,
                    backing::<(FieldKey, AspectValue)>(group.len())?,
                )?;
                add(
                    &mut transient,
                    backing::<(FieldKey, AspectValue)>(group.len())?,
                )?;
            } else {
                add(&mut retained, backing::<FieldKey>(group.len())?)?;
                add(
                    &mut retained,
                    backing::<PortableAspectFieldSet>(group.len())?,
                )?;
            }
        }
        for row in group {
            nested(
                &mut retained,
                row.value
                    .clone_allocation_bytes()
                    .ok_or(Cause::ChargedBytesOverflow)?,
            )?;
            if !row.scalar {
                nested(&mut retained, row.field.as_str().len())?;
                if purpose != PortablePatchReadmissionPurpose::RecordCreation {
                    nested(&mut retained, row.field.as_str().len())?;
                }
            }
        }
    }
    Ok((retained, transient))
}
/// Grow the SAME admitted bill under the SAME original request controls.
pub(in crate::authority::mutation::intents) fn resize(
    request: &ExecutionRequest<'_, '_>,
    memory: &mut Option<ExecutionMemoryReservation>,
    bytes: u64,
) -> Result<(), Cause> {
    if let Some(memory) = memory {
        let (result, _) = request
            .run_native_with_checkpoint(|context| {
                context.poll().map_err(Cause::from)?;
                memory.resize(bytes).map_err(|denial| {
                    crate::execution::denial::lease_denial(
                        worth_execution::LeaseDenial::MemoryExhausted(denial),
                    )
                })
            })
            .map_err(Cause::from)?;
        result?;
    } else {
        *memory = request.reserve_retained_memory(bytes)?;
    }
    Ok(())
}
pub(in crate::authority::mutation::intents) fn admit_readmission(
    request: &ExecutionRequest<'_, '_>,
    memory: &mut Option<ExecutionMemoryReservation>,
    candidate: &PortableRecordAspectPatch,
    plan: Option<&LoweredAspectContractPlan>,
) -> Result<(), Cause> {
    let bytes = readmission_bytes(
        memory.as_ref().map_or(0, ExecutionMemoryReservation::bytes),
        candidate,
        plan,
    )?;
    resize(request, memory, bytes)
}
pub(in crate::authority::mutation::intents) fn readmission_bytes(
    prior_bytes: u64,
    candidate: &PortableRecordAspectPatch,
    plan: Option<&LoweredAspectContractPlan>,
) -> Result<u64, Cause> {
    let bounds = candidate
        .readmission_storage_bounds()
        .ok_or(Cause::ChargedBytesOverflow)?;
    let mut bytes = prior_bytes;
    nested(&mut bytes, bounds.retained_bytes)?;
    nested(&mut bytes, bounds.transient_bytes)?;
    let mut lookup_peak = 0;
    for operation in candidate.operations() {
        // Quote the existing lookup without cloning or introducing a refusal.
        // Missing/mismatched declarations remain the real readmitter's decision.
        let contract = plan
            .and_then(|plan| {
                plan.executable_bindings
                    .iter()
                    .find(|binding| binding.contract.key() == operation.basis().key())
            })
            .map(|binding| &binding.contract);
        if let Some(contract) = contract {
            let owned = contract
                .clone_allocation_bytes()
                .ok_or(Cause::ChargedBytesOverflow)?;
            lookup_peak = lookup_peak.max(owned);
            nested(&mut bytes, owned)?;
            if matches!(operation, PortableAspectPatchOperation::SetWhole { .. }) {
                nested(&mut bytes, contract.key().as_str().len())?;
            }
        }
        if let PortableAspectPatchOperation::PatchFields {
            selected_fields, ..
        } = operation
        {
            for key in selected_fields {
                nested(&mut bytes, key.as_str().len())?;
            }
        }
    }
    nested(&mut bytes, lookup_peak)?;
    Ok(bytes)
}
/// Actual retained output only: all dead candidate/admission scratch is refunded.
pub(in crate::authority::mutation::intents) fn retained_patch_bytes(
    patch: &AuthoritativeRecordAspectPatch,
) -> Result<u64, Cause> {
    let mut bytes = backing::<ContractValidatedAspectValue>(patch.whole_aspect_sets().count())?;
    add(
        &mut bytes,
        backing::<AspectContract>(patch.whole_aspect_clears().count())?,
    )?;
    add(
        &mut bytes,
        backing::<FieldLevelAspectPatch>(patch.field_patches().count())?,
    )?;
    for (_, value) in patch.whole_aspect_sets() {
        nested(&mut bytes, value.owned_allocation_capacity_bytes())?;
    }
    for (_, contract) in patch.whole_aspect_clear_contracts() {
        nested(&mut bytes, contract.owned_allocation_capacity_bytes())?;
    }
    for (_, field) in patch.field_patches() {
        nested(
            &mut bytes,
            field.contract().owned_allocation_capacity_bytes(),
        )?;
        nested(&mut bytes, field.mask().owned_allocation_capacity_bytes())?;
        add(
            &mut bytes,
            backing::<(FieldKey, AspectValue)>(field.field_sets().count())?,
        )?;
        add(
            &mut bytes,
            backing::<FieldKey>(field.field_clears().count())?,
        )?;
        for (key, value) in field.field_sets() {
            nested(&mut bytes, key.owned_allocation_capacity_bytes())?;
            nested(&mut bytes, value.owned_allocation_capacity_bytes())?;
        }
        for key in field.field_clears() {
            nested(&mut bytes, key.owned_allocation_capacity_bytes())?;
        }
    }
    Ok(bytes)
}
