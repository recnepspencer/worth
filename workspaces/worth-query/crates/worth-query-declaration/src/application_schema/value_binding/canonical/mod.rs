//! Entry binding metadata for the Query-free canonical value contract.
mod binary;
mod document;
mod reference;
pub use binary::{decode_application_binary_value, encode_application_binary_value};
pub use document::{decode_application_document, encode_application_document};
pub use reference::{ApplicationCanonicalValueReference, ApplicationCanonicalValueReferenceParts};
pub use worth_foundational::facade::{
    decode_canonical_value, decode_canonical_value_with_admission,
    decode_delegated_canonical_value, decode_delegated_canonical_value_with_admission,
    encode_canonical_value, encode_delegated_canonical_value, BinaryValueDelegate,
    BinaryValueWriter, CanonicalDocumentCodec, CanonicalValue, ValueCodecDenial,
    ValueDecodeAdmission, ValueDecodeBounds, ValueShape, ValueShapeDefinition,
};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationCanonicalValueContract {
    protocol: WireProtocol,
    shape: ValueShape,
    bounds: ValueDecodeBounds,
    supports_admission: bool,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum WireProtocol {
    StructuredCanonical,
    DomainCanonical,
    DomainBinary,
}

impl ApplicationCanonicalValueContract {
    /// A foreign domain value's complete grammar, owned by its entry binding.
    pub fn delegated<Delegate: worth_foundational::facade::ValueSeedDelegate>(
        bounds: ValueDecodeBounds,
    ) -> Self {
        let definition = ValueShapeDefinition::from_delegate::<Delegate>();
        Self {
            protocol: WireProtocol::StructuredCanonical,
            shape: definition.materialize(),
            bounds,
            supports_admission: definition.supports_admission(),
        }
    }
    pub fn of<Value: CanonicalValue>(bounds: ValueDecodeBounds) -> Self {
        let definition = ValueShapeDefinition::of::<Value>();
        Self {
            protocol: WireProtocol::StructuredCanonical,
            shape: definition.materialize(),
            bounds,
            supports_admission: definition.supports_admission(),
        }
    }
    pub const fn supports_admission(&self) -> bool {
        self.supports_admission
    }
    pub const fn wire_protocol(&self) -> &'static str {
        match self.protocol {
            WireProtocol::StructuredCanonical => "worth.canonical-json.v1",
            WireProtocol::DomainCanonical => "worth.domain-canonical-json.v1",
            WireProtocol::DomainBinary => "worth.domain-binary.v1",
        }
    }
    /// Explicit leaf-value posture. Its binding must pass the source guard.
    pub fn domain_of<Value: CanonicalValue>(bounds: ValueDecodeBounds) -> Self {
        let mut contract = Self::of::<Value>(bounds);
        contract.protocol = WireProtocol::DomainCanonical;
        contract
    }
    pub fn domain_delegated<Delegate: worth_foundational::facade::ValueSeedDelegate>(
        bounds: ValueDecodeBounds,
    ) -> Self {
        let mut contract = Self::delegated::<Delegate>(bounds);
        contract.protocol = WireProtocol::DomainCanonical;
        contract
    }
    pub fn binary<Delegate: BinaryValueDelegate>() -> Self {
        const {
            assert!(
                Delegate::MAXIMUM_WIRE_BYTES > 0
                    && Delegate::MAXIMUM_WIRE_BYTES
                        <= worth_foundational::facade::InlineBytes::MAXIMUM_LENGTH
            );
        }
        Self {
            protocol: WireProtocol::DomainBinary,
            shape: ValueShape::Sequence(Box::new(ValueShape::Primitive(
                worth_foundational::facade::ValuePrimitive::U8,
            ))),
            bounds: ValueDecodeBounds::wire_bytes(Delegate::MAXIMUM_WIRE_BYTES),
            supports_admission: true,
        }
    }
    pub const fn is_domain_value(&self) -> bool {
        !matches!(self.protocol, WireProtocol::StructuredCanonical)
    }
    pub const fn scalar_family(&self) -> worth_foundational::facade::ScalarAspectType {
        match self.protocol {
            WireProtocol::DomainBinary => worth_foundational::facade::ScalarAspectType::InlineBytes,
            _ => worth_foundational::facade::ScalarAspectType::String,
        }
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
