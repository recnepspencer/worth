//! Native scalar bindings for held persistent names and held targets.
//!
//! A column stores the canonical stored text of
//! `application_naming::stored_name` in the string scalar family. Decoding
//! accepts only text stored as the column's own kind and refuses any other
//! version, subject class, subject or malformed value with the stored-name
//! refusal's stable reason; it never reinterprets a value.
use std::marker::PhantomData;

use worth_foundational::facade::{AspectValue, InternedString, ScalarAspectType};

use super::primitive::{decode_primitive_admitted, family_mismatch};
use super::{
    ApplicationNativeInputDecodeProfile, ApplicationReadableScalarValueBinding,
    ApplicationScalarValueBinding, ApplicationValueDecodeAvailable, ApplicationValueDecodeDenial,
    ApplicationValueEncodeDenial, ApplicationValueIsNotIdentity,
    ApplicationValueSignedAggregateUnavailable, ApplicationValueValidationDenial,
};
use crate::application_naming::stored_name::{
    decode_name, decode_target, encode_name, encode_target,
};
use crate::application_naming::{
    NamedTarget, StoredNameDenial, WorthQueryName, WorthQueryNamedKind,
};

macro_rules! stored_name_binding {
    ($(#[$doc:meta])* $Binding:ident, $Value:ident, $identity:literal, $encode:path, $decode:path) => {
        $(#[$doc])*
        pub struct $Binding<K>(PhantomData<fn() -> K>);

        impl<K: WorthQueryNamedKind> ApplicationScalarValueBinding for $Binding<K> {
            type Value = $Value<K>;
            type Unit = ();
            type Decode = ApplicationValueDecodeAvailable;
            type Identity = ApplicationValueIsNotIdentity;
            type SignedAggregate = ApplicationValueSignedAggregateUnavailable;

            const IDENTITY_NAME: &'static str = $identity;
            const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::String;
            const SUPPORTS_ADMITTED_DECODE: bool = true;
            /// Decoding reads the stored text once and copies its digests.
            const NATIVE_INPUT_ADMISSION: Option<ApplicationNativeInputDecodeProfile> =
                Some(ApplicationNativeInputDecodeProfile::new(
                    1,
                    1,
                    std::mem::size_of::<Self::Value>() as u64,
                    1,
                ));

            fn validate(value: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
                $encode(value).map(|_| ()).map_err(rejected::<Self>)
            }

            fn encode(value: &Self::Value) -> Result<AspectValue, ApplicationValueEncodeDenial> {
                let text = $encode(value).map_err(rejected::<Self>)?;
                Ok(AspectValue::String(InternedString::Raw(text)))
            }
        }

        impl<K: WorthQueryNamedKind> ApplicationReadableScalarValueBinding for $Binding<K> {
            fn decode_admitted(
                value: &AspectValue,
                admission: &mut super::ValueDecodeAdmission<'_>,
            ) -> Result<Self::Value, ApplicationValueDecodeDenial> {
                decode_primitive_admitted::<Self>(value, admission)
            }

            fn decode(value: &AspectValue) -> Result<Self::Value, ApplicationValueDecodeDenial> {
                let decoded = match value {
                    AspectValue::String(InternedString::Stored(text)) => $decode(text.as_str()),
                    AspectValue::String(InternedString::Raw(text)) => $decode(text),
                    AspectValue::String(_) => {
                        return Err(ApplicationValueDecodeDenial::CodecRejected {
                            binding_identity: Self::IDENTITY,
                        });
                    }
                    value => return Err(family_mismatch::<Self>(value)),
                };
                decoded.map_err(|denial| rejected::<Self>(denial).into())
            }
        }
    };
}

fn rejected<Binding: ApplicationScalarValueBinding>(
    denial: StoredNameDenial,
) -> ApplicationValueValidationDenial {
    ApplicationValueValidationDenial::rejected(Binding::IDENTITY, denial.reason())
}

stored_name_binding!(
    /// Stores a held [`WorthQueryName<K>`] as one string column.
    WorthQueryNameApplicationValueBinding,
    WorthQueryName,
    "worth.query.persistent-name.v1",
    encode_name,
    decode_name
);
stored_name_binding!(
    /// Stores a held [`NamedTarget<K>`] (name and selection) as one string
    /// column.
    NamedTargetApplicationValueBinding,
    NamedTarget,
    "worth.query.named-target.v1",
    encode_target,
    decode_target
);
