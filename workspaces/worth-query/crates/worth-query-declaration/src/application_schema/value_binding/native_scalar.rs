//! Stock bindings for foundational native scalars whose carrier itself is the
//! domain value: calendar and clock values, UUIDs, canonical floats, and
//! inline bytes. Each validates the carrier's canonical form on encode and
//! decode, so a non-canonical value refuses rather than being normalized.
use worth_foundational::facade::{
    AspectValue, CanonicalDate, CanonicalF32, CanonicalF64, CanonicalTime, CanonicalTimestamp,
    CanonicalTimestampTz, InlineBytes, ScalarAspectType,
};

use super::primitive::{decode_primitive_admitted, family_mismatch};
use super::{
    ApplicationNativeInputDecodeProfile, ApplicationReadableScalarValueBinding,
    ApplicationScalarValueBinding, ApplicationValueDecodeAvailable, ApplicationValueDecodeDenial,
    ApplicationValueEncodeDenial, ApplicationValueIsNotIdentity,
    ApplicationValueSignedAggregateUnavailable, ApplicationValueValidationDenial,
};

macro_rules! native_scalar_binding {
    ($binding:ident, $value:ty, $identity:literal, $variant:ident) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $binding;

        impl ApplicationScalarValueBinding for $binding {
            type Value = $value;
            type Unit = ();
            type Decode = ApplicationValueDecodeAvailable;
            type Identity = ApplicationValueIsNotIdentity;
            type SignedAggregate = ApplicationValueSignedAggregateUnavailable;

            const IDENTITY_NAME: &'static str = $identity;
            const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::$variant;
            const SUPPORTS_ADMITTED_DECODE: bool = true;
            /// Decoding copies the carrier and any owned payload once.
            const NATIVE_INPUT_ADMISSION: Option<ApplicationNativeInputDecodeProfile> =
                Some(ApplicationNativeInputDecodeProfile::new(
                    1,
                    1,
                    std::mem::size_of::<Self::Value>() as u64,
                    1,
                ));

            fn validate(value: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
                if NativeCanonicalForm::is_canonical(value) {
                    Ok(())
                } else {
                    Err(ApplicationValueValidationDenial::rejected(
                        Self::IDENTITY,
                        "native scalar is not in canonical form",
                    ))
                }
            }

            fn encode(value: &Self::Value) -> Result<AspectValue, ApplicationValueEncodeDenial> {
                Self::validate(value)?;
                Ok(AspectValue::$variant(value.clone()))
            }
        }

        impl ApplicationReadableScalarValueBinding for $binding {
            fn decode_admitted(
                value: &AspectValue,
                admission: &mut super::ValueDecodeAdmission<'_>,
            ) -> Result<Self::Value, ApplicationValueDecodeDenial> {
                decode_primitive_admitted::<Self>(value, admission)
            }

            fn decode(value: &AspectValue) -> Result<Self::Value, ApplicationValueDecodeDenial> {
                match value {
                    AspectValue::$variant(inner) if value.has_canonical_representation() => {
                        Ok(inner.clone())
                    }
                    AspectValue::$variant(_) => Err(ApplicationValueDecodeDenial::CodecRejected {
                        binding_identity: Self::IDENTITY,
                    }),
                    value => Err(family_mismatch::<Self>(value)),
                }
            }
        }
    };
}

native_scalar_binding!(
    CanonicalDateApplicationValueBinding,
    CanonicalDate,
    "worth.foundational.canonical_date.v1",
    Date
);
native_scalar_binding!(
    CanonicalTimeApplicationValueBinding,
    CanonicalTime,
    "worth.foundational.canonical_time.v1",
    Time
);
native_scalar_binding!(
    CanonicalTimestampApplicationValueBinding,
    CanonicalTimestamp,
    "worth.foundational.canonical_timestamp.v1",
    Timestamp
);
native_scalar_binding!(
    CanonicalTimestampTzApplicationValueBinding,
    CanonicalTimestampTz,
    "worth.foundational.canonical_timestamp_tz.v1",
    TimestampTz
);
native_scalar_binding!(
    UuidApplicationValueBinding,
    [u8; 16],
    "worth.native.uuid.v1",
    Uuid
);
native_scalar_binding!(
    CanonicalF32ApplicationValueBinding,
    CanonicalF32,
    "worth.foundational.canonical_f32.v1",
    Float32
);
native_scalar_binding!(
    CanonicalF64ApplicationValueBinding,
    CanonicalF64,
    "worth.foundational.canonical_f64.v1",
    Float64
);
native_scalar_binding!(
    InlineBytesApplicationValueBinding,
    InlineBytes,
    "worth.foundational.inline_bytes.v1",
    InlineBytes
);

/// The carriers' own canonical-form rule, without copying owned payloads.
trait NativeCanonicalForm {
    fn is_canonical(&self) -> bool;
}

macro_rules! copy_canonical_form {
    ($($value:ty => $variant:ident),* $(,)?) => {$(
        impl NativeCanonicalForm for $value {
            fn is_canonical(&self) -> bool {
                AspectValue::$variant(*self).has_canonical_representation()
            }
        }
    )*};
}

copy_canonical_form!(
    CanonicalDate => Date,
    CanonicalTime => Time,
    CanonicalTimestamp => Timestamp,
    CanonicalTimestampTz => TimestampTz,
    [u8; 16] => Uuid,
    CanonicalF32 => Float32,
    CanonicalF64 => Float64,
);

impl NativeCanonicalForm for InlineBytes {
    /// Construction already proved the payload fits native framing.
    fn is_canonical(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests;
