//! Entry binding metadata for the Query-free canonical value contract.
pub use worth_foundational::facade::{
    decode_canonical_value, decode_canonical_value_with_admission, encode_canonical_value,
    CanonicalValue, ValueCodecDenial, ValueDecodeAdmission, ValueDecodeBounds, ValueShape,
};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationCanonicalValueContract {
    shape: ValueShape,
    bounds: ValueDecodeBounds,
}

impl ApplicationCanonicalValueContract {
    pub fn of<Value: CanonicalValue>(bounds: ValueDecodeBounds) -> Self {
        assert!(
            Value::SHAPE_DEPTH <= 64,
            "canonical value shape must have finite supported nesting"
        );
        Self {
            shape: Value::value_shape(),
            bounds,
        }
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
    decode_canonical_value(text, bounds)
        .map_err(|_| super::ApplicationValueDecodeDenial::CodecRejected { binding_identity })
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
    decode_canonical_value_with_admission(text, bounds, admission)
        .map_err(|_| super::ApplicationValueDecodeDenial::CodecRejected { binding_identity })
}
