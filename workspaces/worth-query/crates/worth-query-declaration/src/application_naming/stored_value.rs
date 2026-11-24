//! Held names and targets as canonical values, so an action input, a
//! structured result or a record column carries them like any other value.
//!
//! The wire form is the one stored text of [`super::stored_name`], as a JSON
//! string. Its declared grammar includes the kind's own subject tag, so text
//! stored as another kind is outside this type's grammar and refuses.

use serde::{Deserializer, Serialize, Serializer};
use worth_foundational::facade::{
    value_decode_error, CanonicalValue, ValueCodecDenial, ValueDecodeAdmission, ValueDecodeBounds,
    ValueDelegate, ValueShapeBody, ValueShapeEdge,
};

use super::stored_name::{decode_name, decode_target, encode_name, encode_target, VERSION};
use super::{NamedTarget, StoredNameDenial, WorthQueryName, WorthQueryNamedKind};

fn codec_denial(denial: StoredNameDenial) -> ValueCodecDenial {
    match denial {
        StoredNameDenial::UnsupportedVersion { observed } => ValueCodecDenial::UnsupportedVersion {
            expected: VERSION,
            observed,
        },
        StoredNameDenial::NonCanonicalSelection => ValueCodecDenial::NonCanonical,
        StoredNameDenial::Malformed
        | StoredNameDenial::SubjectKindMismatch
        | StoredNameDenial::SubjectMismatch
        | StoredNameDenial::UnencodableSubject => ValueCodecDenial::Malformed,
    }
}

macro_rules! stored_text_value {
    ($Value:ident, $encode:path, $decode:path) => {
        impl<K: WorthQueryNamedKind> Serialize for $Value<K> {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let text = $encode(self).map_err(serde::ser::Error::custom)?;
                serializer.serialize_str(&text)
            }
        }

        impl<K: WorthQueryNamedKind> CanonicalValue for $Value<K> {
            const SUPPORTS_ADMISSION: bool = true;
            const WIRE_NULLABLE: bool = false;
            const SHAPE_BODY: ValueShapeBody =
                ValueShapeBody::Transparent(ValueShapeEdge::value::<String>());

            fn validate_bounds(&self, bounds: ValueDecodeBounds) -> Result<(), ValueCodecDenial> {
                let text = $encode(self).map_err(codec_denial)?;
                bounds.check_string_bytes(text.len())
            }

            fn deserialize_bounded<'de, Decoder: Deserializer<'de>>(
                decoder: Decoder,
                bounds: ValueDecodeBounds,
            ) -> Result<Self, Decoder::Error> {
                let text = String::deserialize_bounded(decoder, bounds)?;
                $decode(&text).map_err(|denial| value_decode_error(codec_denial(denial), None))
            }

            fn deserialize_reporting<'de, Decoder: Deserializer<'de>>(
                decoder: Decoder,
                bounds: ValueDecodeBounds,
                reporter: &mut ValueDecodeAdmission<'_>,
            ) -> Result<Self, Decoder::Error> {
                let text = String::deserialize_reporting(decoder, bounds, reporter)?;
                $decode(&text)
                    .map_err(|denial| value_decode_error(codec_denial(denial), Some(reporter)))
            }

            fn deserialize_admitted<'de, Decoder: Deserializer<'de>>(
                decoder: Decoder,
                bounds: ValueDecodeBounds,
                admission: &mut ValueDecodeAdmission<'_>,
            ) -> Result<Self, Decoder::Error> {
                let text = String::deserialize_admitted(decoder, bounds, admission)?;
                $decode(&text)
                    .map_err(|denial| value_decode_error(codec_denial(denial), Some(admission)))
            }
        }
    };
}

stored_text_value!(WorthQueryName, encode_name, decode_name);
stored_text_value!(NamedTarget, encode_target, decode_target);

/// The admitted delegate of a held name field in a `#[derive(WorthValue)]`
/// struct, whose derive asks a generic owner type to name its delegate:
/// `#[worth_value(delegate = WorthQueryNameDelegate<Face>)]`.
pub type WorthQueryNameDelegate<K> = ValueDelegate<WorthQueryName<K>>;

/// The admitted delegate of a held target field in a `#[derive(WorthValue)]`
/// struct: `#[worth_value(delegate = NamedTargetDelegate<Edge>)]`.
pub type NamedTargetDelegate<K> = ValueDelegate<NamedTarget<K>>;
