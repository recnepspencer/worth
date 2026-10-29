//! Entry adapters delegate sealed document construction to the pure envelope owner.
use super::{super::*, *};
use crate::portable_identity::WorthQueryPortableTypeIdentity;
use worth_foundational::facade::{AspectValue, CanonicalDocumentCodec};

pub fn encode_application_document<Codec: CanonicalDocumentCodec>(
    value: &Codec::Document,
    binding_identity: WorthQueryPortableTypeIdentity,
) -> Result<AspectValue, ApplicationValueEncodeDenial> {
    copy_encoded_document(
        Codec::encoded(value),
        Codec::BOUNDS.maximum_wire_bytes(),
        binding_identity,
    )
}

/// Copies one already-encoded document. The shell supplied the codec facts.
#[inline(never)]
fn copy_encoded_document(
    raw: &str,
    maximum_wire_bytes: Option<usize>,
    binding_identity: WorthQueryPortableTypeIdentity,
) -> Result<AspectValue, ApplicationValueEncodeDenial> {
    let copy = || {
        if maximum_wire_bytes.is_some_and(|maximum| raw.len() > maximum) {
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
    mut admission: Option<&mut ValueDecodeAdmission<'_>>,
) -> Result<Codec::Document, ApplicationValueDecodeDenial> {
    let mut decoded = None;
    super::aspect_text::with_raw_aspect_text(
        raw,
        binding_identity,
        &mut |text, binding_identity| {
            let read = match admission.take() {
                Some(admission) => Codec::read_admitted(text, admission),
                None => Codec::read(text),
            };
            decoded = Some(
                read.map_err(|cause| ApplicationValueDecodeDenial::Canonical {
                    binding_identity,
                    cause,
                }),
            );
        },
    )?;
    decoded.expect("raw aspect text runs one document decode")
}
