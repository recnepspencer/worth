//! Borrowed output slot around the actual installed owner's admitted decoder.
use super::{
    ApplicationReadableScalarValueBinding, ApplicationValueDecodeDenial, ValueDecodeAdmission,
};
use std::any::Any;
use worth_foundational::facade::AspectValue;

pub(super) fn decode_erased<Binding: ApplicationReadableScalarValueBinding>(
    value: &AspectValue,
    admission: &mut ValueDecodeAdmission<'_>,
    output: Option<&mut dyn Any>,
) -> Result<(), ApplicationValueDecodeDenial> {
    let rejected = |cause| ApplicationValueDecodeDenial::Canonical {
        binding_identity: Binding::IDENTITY,
        cause,
    };
    let slot = output
        .map(|output| {
            output.downcast_mut::<Option<Binding::Value>>().ok_or(
                ApplicationValueDecodeDenial::CodecRejected {
                    binding_identity: Binding::IDENTITY,
                },
            )
        })
        .transpose()?;
    let mut refusal = None;
    let mut loan = |work, bytes| {
        if let Some(denial) = refusal {
            return Err(denial);
        }
        admission
            .admit(work, bytes)
            .inspect_err(|denial| refusal = Some(*denial))
    };
    let decoded = Binding::decode_admitted(value, &mut loan);
    if let Some(cause) = refusal {
        return Err(rejected(cause));
    }
    let decoded = decoded?;
    Binding::validate(&decoded)?;
    if let Some(slot) = slot {
        *slot = Some(decoded);
    }
    Ok(())
}
