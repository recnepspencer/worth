//! Concrete borrowed-roster, portable and admitted patch allocations.
use super::*;
use crate::schema::portable_patch::quote::{add, nested, readmission_bytes};
use std::alloc::Layout;
use worth_foundational::facade::*;
pub(super) fn backing<T>(count: usize) -> Result<u64, Cause> {
    Layout::array::<T>(count)
        .ok()
        .and_then(|layout| u64::try_from(layout.size()).ok())
        .ok_or(Cause::ChargedBytesOverflow)
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
                    u64::try_from(
                        StructAspectValue::sorted_construction_storage_bytes(group.len())
                            .ok_or(Cause::ChargedBytesOverflow)?,
                    )
                    .map_err(|_| Cause::ChargedBytesOverflow)?,
                )?;
                add(
                    &mut transient,
                    backing::<(FieldKey, AspectValue)>(group.len())?
                        .checked_mul(2)
                        .ok_or(Cause::ChargedBytesOverflow)?,
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
                    .clone_retained_storage_bytes()
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
