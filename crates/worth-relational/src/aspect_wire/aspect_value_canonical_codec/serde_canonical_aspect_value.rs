use serde::{Deserializer, Serializer};
use worth_foundational::facade::AspectValue;

use super::reading::LengthPrefixedAspectValueReader;
use super::writing::encode_length_prefixed_aspect_value;

pub(crate) fn serialize<S>(value: &AspectValue, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut bytes = Vec::new();
    encode_length_prefixed_aspect_value(&mut bytes, value);
    serializer.serialize_bytes(&bytes)
}

pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<AspectValue, D::Error>
where
    D: Deserializer<'de>,
{
    let bytes = crate::aspect_wire::read_canonical_bytes(deserializer)?;
    let mut reader = LengthPrefixedAspectValueReader::new(&bytes);
    let value = reader
        .read_length_prefixed_aspect_value()
        .map_err(serde::de::Error::custom)?;
    reader.finish().map_err(serde::de::Error::custom)?;
    Ok(value)
}
