//! Actual native DTO staging plus Foundational's owned ready-sequence data.
use super::*;
use crate::diagnostics::data::RelationalDiagnosticFields;
use worth_foundational::facade::CanonicalBasisEntry;

pub(super) fn quote(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |name, value| {
        match name {
            "version" => {
                // Sequence and its current assumption basis own two real copies.
                let version = text(value, request)?;
                add(&mut bytes, version.checked_mul(2).ok_or_else(malformed)?)?;
            }
            "entries" => {
                let mut nested = OwnerAllocation::default();
                let count = value.elements(request, |entry| {
                    entry.fields(request, |name, value| {
                        add(
                            &mut nested,
                            match name {
                                "locus" => locus(value, request)?,
                                "value" => native_value(value, request)?,
                                _ => OwnerAllocation::default(),
                            },
                        )
                    })
                })?;
                let native = RelationalDiagnosticFields::native_basis_roster_storage_bytes(
                    vector_capacity(count)?,
                )
                .and_then(|n| u64::try_from(n).ok())
                .ok_or_else(malformed)?;
                let final_roster = vector_storage::<CanonicalBasisEntry>(count)?;
                // Native DTO -> ready entries conversion and the ready owner's
                // stable sort overlap only owned contiguous rosters. A duplicate
                // denial can clone one locus; summing actual nested payloads
                // covers that clone without a private-node or wire-length guess.
                nested.transient = nested
                    .transient
                    .checked_add(nested.retained)
                    .and_then(|n| n.checked_add(native.checked_mul(2)?))
                    .and_then(|n| n.checked_add(final_roster.checked_mul(2)?))
                    .ok_or_else(malformed)?;
                add(&mut nested, final_roster)?;
                add(&mut bytes, nested)?;
            }
            _ => {}
        }
        Ok(())
    })?;
    Ok(bytes)
}
fn locus(
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
                "Aspect" => text(value, request)?,
                "AspectField" => {
                    let mut bytes = OwnerAllocation::default();
                    value.fields(request, |name, value| {
                        add(
                            &mut bytes,
                            match name {
                                "aspect" => text(value, request)?,
                                "path" => field_path(value, request)?,
                                _ => OwnerAllocation::default(),
                            },
                        )
                    })?;
                    bytes
                }
                "Named" => interned(value, request)?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
fn native_value(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    if !value.is_map() {
        return Ok(OwnerAllocation::default());
    }
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |variant, value| {
        if matches!(variant, "ExactText" | "DecimalText" | "BigIntText") {
            add(&mut bytes, interned(value, request)?)?;
        }
        if variant == "RationalText" {
            value.fields(request, |name, value| {
                if matches!(name, "numerator" | "denominator") {
                    add(&mut bytes, interned(value, request)?)?;
                }
                Ok(())
            })?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
fn interned(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |variant, value| {
        if variant == "Raw" {
            add(&mut bytes, text(value, request)?)?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
