//! One compiled aspect-string gate. The shell performs the single typed decode.
use worth_foundational::facade::{AspectValue, InternedString, ScalarAspectType};

use crate::portable_identity::WorthQueryPortableTypeIdentity;

use super::super::ApplicationValueDecodeDenial;

/// Rejects a non-string aspect value, then runs `decode` on the raw text.
#[inline(never)]
pub(super) fn with_raw_aspect_text(
    raw: &AspectValue,
    binding_identity: WorthQueryPortableTypeIdentity,
    decode: &mut dyn FnMut(&str, WorthQueryPortableTypeIdentity),
) -> Result<(), ApplicationValueDecodeDenial> {
    let AspectValue::String(text) = raw else {
        return Err(ApplicationValueDecodeDenial::ScalarFamilyMismatch {
            binding_identity,
            expected: ScalarAspectType::String,
            observed: raw.value_family(),
        });
    };
    let InternedString::Raw(text) = text else {
        return Err(ApplicationValueDecodeDenial::CodecRejected { binding_identity });
    };
    decode(text, binding_identity);
    Ok(())
}
