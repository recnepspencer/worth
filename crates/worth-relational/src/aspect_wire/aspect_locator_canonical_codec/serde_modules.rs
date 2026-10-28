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
        locators
            .iter()
            .map(encode_aspect_field_locator)
            .collect::<Vec<_>>()
            .serialize(serializer)
    }

    pub(crate) fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Arc<[AspectFieldLocator]>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Vec::<Vec<u8>>::deserialize(deserializer)?
            .iter()
            .map(|bytes| decode_aspect_field_locator(bytes))
            .collect::<Result<Vec<_>, _>>()
            .map(Arc::from)
            .map_err(D::Error::custom)
    }
}
