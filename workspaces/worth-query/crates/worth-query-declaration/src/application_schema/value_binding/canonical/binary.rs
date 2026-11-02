use super::super::{ApplicationValueDecodeDenial, ApplicationValueEncodeDenial};
use super::{BinaryValueDelegate, ValueDecodeAdmission};
use crate::portable_identity::WorthQueryPortableTypeIdentity;
use worth_foundational::facade::{AspectValue, ScalarAspectType};

pub fn encode_application_binary_value<Delegate: BinaryValueDelegate>(
    value: &Delegate::Value,
    binding_identity: WorthQueryPortableTypeIdentity,
) -> Result<AspectValue, ApplicationValueEncodeDenial> {
    worth_foundational::facade::encode_binary_value::<Delegate>(value)
        .map(AspectValue::InlineBytes)
        .map_err(|cause| ApplicationValueEncodeDenial::Canonical {
            binding_identity,
            cause,
        })
}

pub fn decode_application_binary_value<Delegate: BinaryValueDelegate>(
    raw: &AspectValue,
    binding_identity: WorthQueryPortableTypeIdentity,
    admission: Option<&mut ValueDecodeAdmission<'_>>,
) -> Result<Delegate::Value, ApplicationValueDecodeDenial> {
    let AspectValue::InlineBytes(bytes) = raw else {
        return Err(ApplicationValueDecodeDenial::ScalarFamilyMismatch {
            binding_identity,
            expected: ScalarAspectType::InlineBytes,
            observed: raw.value_family(),
        });
    };
    let result =
        match admission {
            Some(admission) => worth_foundational::facade::decode_binary_value_with_admission::<
                Delegate,
            >(bytes.as_bytes(), admission),
            None => worth_foundational::facade::decode_binary_value::<Delegate>(bytes.as_bytes()),
        };
    result.map_err(|cause| ApplicationValueDecodeDenial::Canonical {
        binding_identity,
        cause,
    })
}
