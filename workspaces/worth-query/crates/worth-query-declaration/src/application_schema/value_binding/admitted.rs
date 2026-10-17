//! Resource loan boundary around the installed binding's erased output.
use std::any::Any;

use worth_foundational::facade::AspectValue;

use super::{
    ApplicationReadableScalarValueBinding, ApplicationValueDecodeDenial, ValueDecodeAdmission,
};

pub(super) fn decode_erased<Binding: ApplicationReadableScalarValueBinding>(
    value: &AspectValue,
    admission: &mut ValueDecodeAdmission<'_>,
) -> Result<Box<dyn Any>, ApplicationValueDecodeDenial> {
    let rejected = || ApplicationValueDecodeDenial::CodecRejected {
        binding_identity: Binding::IDENTITY,
    };
    let mut refusal = None;
    let mut loan = |work, bytes| {
        if let Some(denial) = refusal {
            return Err(denial);
        }
        admission(work, bytes).inspect_err(|denial| refusal = Some(*denial))
    };
    // Type erasure owns this allocation; seeds cannot account for it themselves.
    loan(1, std::mem::size_of::<Binding::Value>()).map_err(|_| rejected())?;
    let decoded = Binding::decode_admitted(value, &mut loan);
    if refusal.is_some() {
        return Err(rejected());
    }
    let decoded = decoded?;
    Binding::validate(&decoded)?;
    Ok(Box::new(decoded))
}
