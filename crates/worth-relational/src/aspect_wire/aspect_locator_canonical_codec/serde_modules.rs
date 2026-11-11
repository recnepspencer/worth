use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::Arc;
use worth_foundational::facade::{AspectFieldLocator, AspectValueLocator};

use super::reading::{decode_aspect_field_locator, decode_aspect_value_locator};
use super::writing::{encode_aspect_field_locator, encode_aspect_value_locator};

pub(crate) mod serde_canonical_aspect_value_locator {
    use super::*;
    use serde::de::Error;

    pub(crate) fn serialize<S>(
        locator: &AspectValueLocator,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        encode_aspect_value_locator(locator).serialize(serializer)
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<AspectValueLocator, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = Vec::<u8>::deserialize(deserializer)?;
        decode_aspect_value_locator(&bytes).map_err(D::Error::custom)
    }
}

pub(crate) mod serde_canonical_aspect_field_locator {
    use super::*;
    use serde::de::Error;

    pub(crate) fn serialize<S>(
        locator: &AspectFieldLocator,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        encode_aspect_field_locator(locator).serialize(serializer)
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<AspectFieldLocator, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = Vec::<u8>::deserialize(deserializer)?;
        decode_aspect_field_locator(&bytes).map_err(D::Error::custom)
    }
}

pub(crate) mod serde_canonical_aspect_field_locator_arc_slice {
    use super::*;
    use serde::de::Error;

    pub(crate) fn serialize<S>(
        locators: &Arc<[AspectFieldLocator]>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut sequence = serializer.serialize_seq(Some(locators.len()))?;
        for locator in locators.iter() {
            // The canonical wire is unchanged; retain only one actual encoded
            // locator while serializing, rather than Vec<Vec<u8>> staging.
            sequence.serialize_element(&encode_aspect_field_locator(locator))?;
        }
        sequence.end()
    }

    pub(crate) fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Arc<[AspectFieldLocator]>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Locators;
        impl<'de> serde::de::Visitor<'de> for Locators {
            type Value = Arc<[AspectFieldLocator]>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("canonical aspect field locator sequence")
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut sequence: A,
            ) -> Result<Self::Value, A::Error> {
                // Count/typed backing and each binary subcodec are quoted by
                // the enclosing Native artifact before this decoder runs.
                // Ignore the untrusted hint; never retain an encoded roster.
                let mut locators = Vec::new();
                while let Some(encoded) = sequence.next_element::<Vec<u8>>()? {
                    locators.push(decode_aspect_field_locator(&encoded).map_err(A::Error::custom)?);
                }
                Ok(Arc::from(locators))
            }
        }
        deserializer.deserialize_seq(Locators)
    }
}
