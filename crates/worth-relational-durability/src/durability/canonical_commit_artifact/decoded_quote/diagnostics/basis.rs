//! Quote the owned basis roster and the memory needed to certify its sequence.
use super::*;
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
                let final_roster = vector_storage::<CanonicalBasisEntry>(count)?;
                // The direct entry decode moves its roster into readiness checking.
                // Allow a full roster during collection and another for stable sort
                // scratch. A duplicate denial can clone one locus.
                nested.transient = nested
                    .transient
                    .checked_add(nested.retained)
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
