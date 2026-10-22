//! Entry binding metadata for the Query-free canonical value contract.
mod document;
pub use document::{decode_application_document, encode_application_document};
pub use worth_foundational::facade::{
    decode_canonical_value, decode_canonical_value_with_admission, encode_canonical_value,
    CanonicalDocumentCodec, CanonicalValue, ValueCodecDenial, ValueDecodeAdmission,
    ValueDecodeBounds, ValueShape, ValueShapeDefinition,
};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationCanonicalValueContract {
    shape: ValueShape,
    bounds: ValueDecodeBounds,
    supports_admission: bool,
}

impl ApplicationCanonicalValueContract {
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
    use worth_foundational::facade::{AspectValue, InternedString, ScalarAspectType};
    let AspectValue::String(text) = raw else {
        return Err(super::ApplicationValueDecodeDenial::ScalarFamilyMismatch {
            binding_identity,
            expected: ScalarAspectType::String,
            observed: raw.value_family(),
        });
    };
    let InternedString::Raw(text) = text else {
        return Err(super::ApplicationValueDecodeDenial::CodecRejected { binding_identity });
    };
    decode_canonical_value(text, bounds).map_err(|cause| {
        super::ApplicationValueDecodeDenial::Canonical {
            binding_identity,
            cause,
        }
    })
}

pub fn decode_application_canonical_value_with_admission<Value: CanonicalValue>(
    raw: &worth_foundational::facade::AspectValue,
    bounds: ValueDecodeBounds,
    binding_identity: crate::portable_identity::WorthQueryPortableTypeIdentity,
    admission: &mut ValueDecodeAdmission<'_>,
) -> Result<Value, super::ApplicationValueDecodeDenial> {
    use worth_foundational::facade::{AspectValue, InternedString, ScalarAspectType};
    let AspectValue::String(text) = raw else {
        return Err(super::ApplicationValueDecodeDenial::ScalarFamilyMismatch {
            binding_identity,
            expected: ScalarAspectType::String,
            observed: raw.value_family(),
        });
    };
    let InternedString::Raw(text) = text else {
        return Err(super::ApplicationValueDecodeDenial::CodecRejected { binding_identity });
    };
    decode_canonical_value_with_admission(text, bounds, admission).map_err(|cause| {
        super::ApplicationValueDecodeDenial::Canonical {
            binding_identity,
            cause,
        }
    })
}
