//! Native diagnostic wire retains typed values, not terminal JSON projections.
use super::*;
use crate::diagnostics::data::{
    DiagnosticObjectFields, RelationalDiagnosticValue, RelationalDiagnosticsEntry,
};
use worth_foundational::facade::{CanonicalFieldPath, FieldKey};
mod basis;

pub(super) fn quote(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |name, value| {
        if name == "entries" {
            add(
                &mut bytes,
                vector::<RelationalDiagnosticsEntry>(value, request, |entry| {
                    let mut bytes = OwnerAllocation::default();
                    entry.fields(request, |name, value| {
                        add(
                            &mut bytes,
                            match name {
                                "message" => text(value, request)?,
                                "fields" => diagnostic_value(value, request)?,
                                _ => OwnerAllocation::default(),
                            },
                        )
                    })?;
                    Ok(bytes)
                })?,
            )?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
fn diagnostic_value(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    if !value.is_map() {
        return Ok(OwnerAllocation::default());
    }
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |variant, value| {
        add(
            &mut bytes,
            match variant {
                "String" | "AspectKey" | "FieldKey" | "BranchId" | "SchemaId" | "ContractId" => {
                    text(value, request)?
                }
                "CanonicalBytes" => {
                    vector::<u8>(value, request, |_| Ok(OwnerAllocation::default()))?
                }
                "Array" => vector::<RelationalDiagnosticValue>(value, request, |value| {
                    diagnostic_value(value, request)
                })?,
                "Object" => object(value, request)?,
                "AspectValue" => super::aspects::scalar(value, request)?,
                "StructAspectValue" => super::portable_aspects::structure(value, request)?,
                "AspectFieldLocator" | "AspectValueLocator" => {
                    super::aspects::locator(value, request)?
                }
                "FieldPath" => field_path(value, request)?,
                "DiagnosticMask" => mask(value, request)?,
                "DiagnosticMaskLocator" => mask_locator(value, request)?,
                "CanonicalBasis" => basis::quote(value, request)?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
fn object(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    // This temporary quote table holds only borrowed keys and Copy counts.
    // Actual SDK cells fund themselves before growth; it is gone before the
    // real decoder begins. Replacing a key models the last decoded value.
    let mut final_values = worth_execution::ExecutionOrderedMap::<&str, OwnerAllocation>::default();
    let mut all = OwnerAllocation::default();
    let mut count = 0usize;
    value.fields(request, |key, value| {
        count = count.checked_add(1).ok_or_else(malformed)?;
        let mut nested = diagnostic_value(value, request)?;
        add(
            &mut nested,
            u64::try_from(key.len()).map_err(|_| malformed())?,
        )?;
        add(&mut all, nested)?;
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
    let storage = DiagnosticObjectFields::decode_storage_bounds(count).ok_or_else(malformed)?;
    let final_storage =
        DiagnosticObjectFields::decode_storage_bounds(final_values.len()).ok_or_else(malformed)?;
    let mut final_heaps = 0u64;
    for nested in final_values.values() {
        final_heaps = final_heaps
            .checked_add(nested.retained)
            .ok_or_else(malformed)?;
    }
    let final_fields =
        u64::try_from(final_storage.retained_fields_bytes).map_err(|_| malformed())?;
    let peak_fields = storage
        .retained_fields_bytes
        .checked_add(storage.transient_decode_bytes)
        .and_then(|n| u64::try_from(n).ok())
        .ok_or_else(malformed)?;
    Ok(OwnerAllocation {
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
    })
}
fn field_path(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    vector::<FieldKey>(value, request, |key| text(key, request))
}
fn mask(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = vector::<CanonicalFieldPath>(value, request, |path| field_path(path, request))?;
    // The native Vec<Vec<FieldKey>> is converted to Vec<CanonicalFieldPath>,
    // then AspectMask's ordinary stable sort may hold one additional roster.
    let count = value.elements(request, |_| Ok(()))?;
    bytes.transient = bytes
        .transient
        .checked_add(vector_storage::<Vec<FieldKey>>(count)?)
        .and_then(|n| n.checked_add(vector_storage::<CanonicalFieldPath>(count).ok()?))
        .ok_or_else(malformed)?;
    Ok(bytes)
}
fn mask_locator(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "aspect_key" => text(value, request)?,
                "paths" => {
                    let mut mask = mask(value, request)?;
                    // AspectMaskLocator::diagnostic clones the admitted mask paths.
                    mask.transient = mask
                        .transient
                        .checked_add(mask.retained)
                        .ok_or_else(malformed)?;
                    mask
                }
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
