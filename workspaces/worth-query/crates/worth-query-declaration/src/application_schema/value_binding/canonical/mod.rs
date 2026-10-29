//! Entry binding metadata for the Query-free canonical value contract.
mod aspect_text;
mod document;
mod reference;
pub use document::{decode_application_document, encode_application_document};
pub use reference::{ApplicationCanonicalValueReference, ApplicationCanonicalValueReferenceParts};
pub use worth_foundational::facade::{
    decode_canonical_value, decode_canonical_value_with_admission,
    decode_delegated_canonical_value, decode_delegated_canonical_value_with_admission,
    encode_canonical_value, encode_delegated_canonical_value, CanonicalDocumentCodec,
    CanonicalValue, ValueCodecDenial, ValueDecodeAdmission, ValueDecodeBounds, ValueShape,
    ValueShapeDefinition,
};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationCanonicalValueContract {
    shape: ValueShape,
    bounds: ValueDecodeBounds,
    supports_admission: bool,
}

impl ApplicationCanonicalValueContract {
    /// A foreign domain value's complete grammar, owned by its entry binding.
    pub fn delegated<Delegate: worth_foundational::facade::ValueSeedDelegate>(
        bounds: ValueDecodeBounds,
    ) -> Self {
        let definition = ValueShapeDefinition::from_delegate::<Delegate>();
        Self {
            shape: definition.materialize(),
            bounds,
            supports_admission: definition.supports_admission(),
        }
    }
    pub fn of<Value: CanonicalValue>(bounds: ValueDecodeBounds) -> Self {
        let definition = ValueShapeDefinition::of::<Value>();
        Self {
            shape: definition.materialize(),
            bounds,
            supports_admission: definition.supports_admission(),
        }
    }
    pub const fn supports_admission(&self) -> bool {
        self.supports_admission
    }
    pub const fn wire_protocol(&self) -> &'static str {
        "worth.canonical-json.v1"
    }
    pub const fn shape(&self) -> &ValueShape {
        &self.shape
    }
    pub const fn bounds(&self) -> ValueDecodeBounds {
        self.bounds
    }
}

pub fn decode_application_canonical_value<Value: CanonicalValue>(
    raw: &worth_foundational::facade::AspectValue,
    bounds: ValueDecodeBounds,
    binding_identity: crate::portable_identity::WorthQueryPortableTypeIdentity,
) -> Result<Value, super::ApplicationValueDecodeDenial> {
    let mut decoded = None;
    aspect_text::with_raw_aspect_text(raw, binding_identity, &mut |text, binding_identity| {
        decoded = Some(decode_canonical_value(text, bounds).map_err(|cause| {
            super::ApplicationValueDecodeDenial::Canonical {
                binding_identity,
                cause,
            }
        }));
    })?;
    decoded.expect("raw aspect text runs one canonical decode")
}

pub fn decode_application_canonical_value_with_admission<Value: CanonicalValue>(
    raw: &worth_foundational::facade::AspectValue,
    bounds: ValueDecodeBounds,
    binding_identity: crate::portable_identity::WorthQueryPortableTypeIdentity,
    admission: &mut ValueDecodeAdmission<'_>,
) -> Result<Value, super::ApplicationValueDecodeDenial> {
    let mut decoded = None;
    aspect_text::with_raw_aspect_text(raw, binding_identity, &mut |text, binding_identity| {
        decoded = Some(
            decode_canonical_value_with_admission(text, bounds, admission).map_err(|cause| {
                super::ApplicationValueDecodeDenial::Canonical {
                    binding_identity,
                    cause,
                }
            }),
        );
    })?;
    decoded.expect("raw aspect text runs one admitted canonical decode")
}
