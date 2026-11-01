//! Native canonical locator fields embed their own length-prefixed codec.
use super::*;

pub(super) fn locator(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut octets = value.octets(request)?;
    let count = octets.len();
    let nested = crate::aspect_wire::locator_decoded_bytes(count, || octets.byte(), malformed)?;
    octets.finish()?;
    with_encoded_scratch(nested, count)
}

/// The ordinary Serde scalar form used by invariant violation evidence.
pub(super) fn scalar(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    if matches!(value.0.first(), Some(0xa0..=0xbf | 0xd9..=0xdb)) {
        return Ok(OwnerAllocation::default());
    }
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |variant, value| {
        add(
            &mut bytes,
            match variant {
                "Decimal" | "BigInt" => text(value, request)?,
                "Rational" => {
                    let mut bytes = OwnerAllocation::default();
                    value.fields(request, |field, value| {
                        if matches!(field, "numerator" | "denominator") {
                            add(&mut bytes, text(value, request)?)?;
                        }
                        Ok(())
                    })?;
                    bytes
                }
                "String" => {
                    let mut bytes = OwnerAllocation::default();
                    value.fields(request, |variant, value| {
                        if variant == "Raw" {
                            add(&mut bytes, text(value, request)?)?;
                        }
                        Ok(())
                    })?;
                    bytes
                }
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}

pub(super) fn canonical_value(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut octets = value.octets(request)?;
    let count = octets.len();
    let nested = crate::aspect_wire::value_frame_decoded_bytes(count, || octets.byte(), malformed)?;
    octets.finish()?;
    with_encoded_scratch(nested, count)
}

pub(super) fn field_patch(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut octets = value.octets(request)?;
    let count = octets.len();
    let nested = crate::aspect_wire::field_patch_decoded_bytes(count, || octets.byte(), malformed)?;
    octets.finish()?;
    with_encoded_scratch(nested, count)
}

fn with_encoded_scratch(
    nested: crate::aspect_wire::CanonicalDecodeStorage,
    count: usize,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::from(nested);
    // Serde's temporary encoded Vec and the later canonical check's same
    // typed byte buffer are sequential; neither belongs to retained meaning.
    // Vec growth can overlap old/new byte backing during the encoding pass.
    bytes.transient = bytes
        .transient
        .checked_add(
            vector_storage::<u8>(count)?
                .checked_mul(2)
                .ok_or_else(malformed)?,
        )
        .ok_or_else(malformed)?;
    Ok(bytes)
}
