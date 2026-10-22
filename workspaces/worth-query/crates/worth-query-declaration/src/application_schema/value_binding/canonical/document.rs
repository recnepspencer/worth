//! Entry adapters delegate sealed document construction to the pure envelope owner.
use super::{super::*, *};
use crate::portable_identity::WorthQueryPortableTypeIdentity;
use worth_foundational::facade::{
    AspectValue, CanonicalDocumentCodec, InternedString, ScalarAspectType,
};

pub fn encode_application_document<Codec: CanonicalDocumentCodec>(
    value: &Codec::Document,
    binding_identity: WorthQueryPortableTypeIdentity,
) -> Result<AspectValue, ApplicationValueEncodeDenial> {
    let raw = Codec::encoded(value);
    let copy = || {
        if Codec::BOUNDS
            .maximum_wire_bytes()
            .is_some_and(|maximum| raw.len() > maximum)
        {
            return Err(ValueCodecDenial::WireBytes);
        }
        let mut text = String::new();
        text.try_reserve_exact(raw.len())
            .map_err(|_| ValueCodecDenial::Allocation)?;
        text.push_str(raw);
        Ok(AspectValue::String(text.into()))
    };
    copy().map_err(|cause| ApplicationValueEncodeDenial::Canonical {
        binding_identity,
        cause,
    })
}

pub fn decode_application_document<Codec: CanonicalDocumentCodec>(
    raw: &AspectValue,
    binding_identity: WorthQueryPortableTypeIdentity,
    admission: Option<&mut ValueDecodeAdmission<'_>>,
) -> Result<Codec::Document, ApplicationValueDecodeDenial> {
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
    match admission {
        Some(admission) => Codec::read_admitted(text, admission),
        None => Codec::read(text),
    }
    .map_err(|cause| ApplicationValueDecodeDenial::Canonical {
        binding_identity,
        cause,
    })
}
