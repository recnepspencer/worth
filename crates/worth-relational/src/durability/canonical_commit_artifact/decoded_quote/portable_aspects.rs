//! Portable aspect candidates quote Foundation's actual storage contract.
use super::*;
use worth_foundational::facade::{
    FieldKey, PortableAspectFieldSet, PortableAspectPatchOperation, StructAspectValue,
};

pub(super) fn patch(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        if field == "operations" {
            add(
                &mut bytes,
                vector::<PortableAspectPatchOperation>(value, request, |value| {
                    operation(value, request)
                })?,
            )?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
fn operation(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |variant, value| {
        if !matches!(variant, "SetWhole" | "ClearWhole" | "PatchFields") {
            return Err(malformed());
        }
        value.fields(request, |field, value| {
            let nested = match field {
                "basis" => basis(value, request)?,
                "value" => validation_input(value, request)?,
                "selected_fields" | "field_clears" => {
                    vector::<FieldKey>(value, request, |value| text(value, request))?
                }
                "field_sets" => vector::<PortableAspectFieldSet>(value, request, |value| {
                    let mut bytes = OwnerAllocation::default();
                    value.fields(request, |field, value| {
                        add(
                            &mut bytes,
                            match field {
                                "field" => text(value, request)?,
                                "value" => super::aspects::scalar(value, request)?,
                                _ => OwnerAllocation::default(),
                            },
                        )
                    })?;
                    Ok(bytes)
                })?,
                _ => OwnerAllocation::default(),
            };
            add(&mut bytes, nested)
        })
    })?;
    Ok(bytes)
}
fn basis(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        if field == "key" {
            add(&mut bytes, text(value, request)?)?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
pub(super) fn validation_input(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |variant, value| {
        add(
            &mut bytes,
            match variant {
                "Scalar" => super::aspects::scalar(value, request)?,
                "Struct" => structure(value, request)?,
                _ => return Err(malformed()),
            },
        )
    })?;
    Ok(bytes)
}
pub(super) fn structure(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        if field != "fields" {
            return Ok(());
        }
        let mut count = 0_usize;
        let mut all = OwnerAllocation::default();
        let mut final_values =
            worth_execution::ExecutionOrderedMap::<&str, OwnerAllocation>::default();
        value.fields(request, |key, value| {
            count = count.checked_add(1).ok_or_else(malformed)?;
            let mut nested = super::aspects::scalar(value, request)?;
            add(
                &mut nested,
                u64::try_from(key.len()).map_err(|_| malformed())?,
            )?;
            add(&mut all, nested)?;
            // The actual owner keeps the first equal key and last value.
            // Quote every raw heap before decode; only final heaps stay charged.
            final_values
                .insert_cloned_native_for_request(&key, &nested, *request)
                .map_err(|cause| match cause {
                    worth_execution::ExecutionOrderedMapDenial::Request(cause) => {
                        Refusal::Request(cause.into())
                    }
                    worth_execution::ExecutionOrderedMapDenial::Allocation(cause) => {
                        Refusal::Request(match cause.classify() {
                            worth_execution::ExecutionAllocationDenialClass::Request(cause) => {
                                cause.into()
                            }
                            worth_execution::ExecutionAllocationDenialClass::Physical(_) => {
                                crate::execution::RelationalExecutionDenialCause::AllocationFailed
                            }
                        })
                    }
                    worth_execution::ExecutionOrderedMapDenial::CapacityOverflow => malformed(),
                })
        })?;
        let backing = StructAspectValue::decode_storage_bounds(count).ok_or_else(malformed)?;
        let final_backing =
            StructAspectValue::decode_storage_bounds(final_values.len()).ok_or_else(malformed)?;
        let mut final_heaps = 0u64;
        for nested in final_values.values() {
            final_heaps = final_heaps
                .checked_add(nested.retained)
                .ok_or_else(malformed)?;
        }
        let final_fields =
            u64::try_from(final_backing.retained_fields_bytes).map_err(|_| malformed())?;
        let peak_fields = backing
            .retained_fields_bytes
            .checked_add(backing.transient_decode_bytes)
            .and_then(|n| u64::try_from(n).ok())
            .ok_or_else(malformed)?;
        add(
            &mut bytes,
            OwnerAllocation {
                retained: final_heaps
                    .checked_add(final_fields)
                    .ok_or_else(malformed)?,
                transient: all
                    .transient
                    .checked_add(
                        all.retained
                            .checked_sub(final_heaps)
                            .ok_or_else(malformed)?,
                    )
                    .and_then(|n| n.checked_add(peak_fields.checked_sub(final_fields)?))
                    .ok_or_else(malformed)?,
            },
        )
    })?;
    Ok(bytes)
}
