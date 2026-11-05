use worth_foundational::facade::{AspectValue, InternedString, ScalarAspectType};

use super::{
    ApplicationIdentityScalarValueBinding, ApplicationNativeInputDecodeProfile,
    ApplicationReadableScalarValueBinding, ApplicationScalarValueBinding,
    ApplicationSignedAggregateValueBinding, ApplicationValueDecodeAvailable,
    ApplicationValueDecodeDenial, ApplicationValueEncodeDenial, ApplicationValueIsIdentity,
    ApplicationValueIsNotIdentity, ApplicationValueSignedAggregateAvailable,
    ApplicationValueSignedAggregateUnavailable, ApplicationValueValidationDenial,
};

#[cfg(test)]
mod admitted_tests;

macro_rules! primitive_binding {
    ($binding:ident, $value:ty, $identity:literal, $family:ident, $variant:ident,
     $identity_posture:ty, $aggregate_posture:ty) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $binding;

        impl ApplicationScalarValueBinding for $binding {
            type Value = $value;
            type Unit = ();
            type Decode = ApplicationValueDecodeAvailable;
            type Identity = $identity_posture;
            type SignedAggregate = $aggregate_posture;

            const IDENTITY_NAME: &'static str = $identity;
            const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::$family;
            const SUPPORTS_ADMITTED_DECODE: bool = true;
            const NATIVE_INPUT_ADMISSION: Option<ApplicationNativeInputDecodeProfile> =
                Some(ApplicationNativeInputDecodeProfile::new(
                    1,
                    1,
                    std::mem::size_of::<Self::Value>() as u64,
                    1,
                ));

            fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
                Ok(())
            }

            fn encode(value: &Self::Value) -> Result<AspectValue, ApplicationValueEncodeDenial> {
                Ok(AspectValue::$variant(value.clone().into()))
            }
        }
    };
}

primitive_binding!(
    ContentRefApplicationValueBinding,
    worth_foundational::facade::ContentRefId,
    "worth.native.content-ref.v1",
    ContentRef,
    ContentRef,
    ApplicationValueIsNotIdentity,
    ApplicationValueSignedAggregateUnavailable
);
primitive_binding!(
    BoolApplicationValueBinding,
    bool,
    "worth.rust.bool",
    Bool,
    Bool,
    ApplicationValueIsNotIdentity,
    ApplicationValueSignedAggregateUnavailable
);
primitive_binding!(
    I64ApplicationValueBinding,
    i64,
    "worth.rust.i64",
    Int64,
    Int64,
    ApplicationValueIsNotIdentity,
    ApplicationValueSignedAggregateAvailable
);
primitive_binding!(
    U64ApplicationValueBinding,
    u64,
    "worth.rust.u64",
    UInt64,
    UInt64,
    ApplicationValueIsIdentity,
    ApplicationValueSignedAggregateUnavailable
);
primitive_binding!(
    StringApplicationValueBinding,
    String,
    "worth.rust.string",
    String,
    String,
    ApplicationValueIsIdentity,
    ApplicationValueSignedAggregateUnavailable
);
primitive_binding!(
    InternedStringApplicationValueBinding,
    InternedString,
    "worth.foundational.interned_string.v1",
    String,
    String,
    ApplicationValueIsIdentity,
    ApplicationValueSignedAggregateUnavailable
);

macro_rules! readable_copy_binding {
    ($binding:ident, $variant:ident) => {
        impl ApplicationReadableScalarValueBinding for $binding {
            fn decode_admitted(
                value: &AspectValue,
                admission: &mut super::ValueDecodeAdmission<'_>,
            ) -> Result<Self::Value, ApplicationValueDecodeDenial> {
                decode_primitive_admitted::<Self>(value, admission)
            }
            fn decode(value: &AspectValue) -> Result<Self::Value, ApplicationValueDecodeDenial> {
                match value {
                    AspectValue::$variant(value) => Ok(*value),
                    value => Err(family_mismatch::<Self>(value)),
                }
            }
        }
    };
}

readable_copy_binding!(ContentRefApplicationValueBinding, ContentRef);
readable_copy_binding!(BoolApplicationValueBinding, Bool);
readable_copy_binding!(I64ApplicationValueBinding, Int64);
readable_copy_binding!(U64ApplicationValueBinding, UInt64);

impl ApplicationReadableScalarValueBinding for StringApplicationValueBinding {
    fn decode_admitted(
        value: &AspectValue,
        admission: &mut super::ValueDecodeAdmission<'_>,
    ) -> Result<Self::Value, ApplicationValueDecodeDenial> {
        decode_primitive_admitted::<Self>(value, admission)
    }
    fn decode(value: &AspectValue) -> Result<Self::Value, ApplicationValueDecodeDenial> {
        match value {
            AspectValue::String(InternedString::Raw(value)) => Ok(value.clone()),
            AspectValue::String(_) => Err(ApplicationValueDecodeDenial::CodecRejected {
                binding_identity: Self::IDENTITY,
            }),
            value => Err(family_mismatch::<Self>(value)),
        }
    }
}

impl ApplicationReadableScalarValueBinding for InternedStringApplicationValueBinding {
    fn decode_admitted(
        value: &AspectValue,
        admission: &mut super::ValueDecodeAdmission<'_>,
    ) -> Result<Self::Value, ApplicationValueDecodeDenial> {
        decode_primitive_admitted::<Self>(value, admission)
    }
    fn decode(value: &AspectValue) -> Result<Self::Value, ApplicationValueDecodeDenial> {
        match value {
            AspectValue::String(value) => Ok(value.clone()),
            value => Err(family_mismatch::<Self>(value)),
        }
    }
}

impl ApplicationIdentityScalarValueBinding for U64ApplicationValueBinding {}
impl ApplicationIdentityScalarValueBinding for StringApplicationValueBinding {}
impl ApplicationIdentityScalarValueBinding for InternedStringApplicationValueBinding {}

impl ApplicationSignedAggregateValueBinding for I64ApplicationValueBinding {
    fn decode_aggregate(value: i64) -> Result<Self::Value, ApplicationValueDecodeDenial> {
        Ok(value)
    }
}

fn decode_primitive_admitted<Binding: ApplicationReadableScalarValueBinding>(
    value: &AspectValue,
    admission: &mut super::ValueDecodeAdmission<'_>,
) -> Result<Binding::Value, ApplicationValueDecodeDenial> {
    if value.value_family() != Binding::SCALAR_FAMILY {
        return Err(family_mismatch::<Binding>(value));
    }
    admission
        .admit(1, value.owned_allocation_capacity_bytes())
        .map_err(|cause| ApplicationValueDecodeDenial::Canonical {
            binding_identity: Binding::IDENTITY,
            cause,
        })?;
    Binding::decode(value)
}

fn family_mismatch<Binding: ApplicationScalarValueBinding>(
    value: &AspectValue,
) -> ApplicationValueDecodeDenial {
    ApplicationValueDecodeDenial::ScalarFamilyMismatch {
        binding_identity: Binding::IDENTITY,
        expected: Binding::SCALAR_FAMILY,
        observed: value.value_family(),
    }
}
