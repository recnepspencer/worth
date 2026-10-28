use serde::{Deserialize, Serialize};

use crate::durability::data::RelationalCheckpointManifest;
use crate::durability::medium::RelationalDurableMemberLocator;

/// Placement is a property of the artifact, not a second checkpoint grammar.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CheckpointWire<'a> {
    pub(super) manifest: RelationalCheckpointManifest,
    #[serde(borrow)]
    pub(super) members: Vec<(RelationalDurableMemberLocator, ByteString<'a>)>,
    #[serde(borrow)]
    pub(super) envelopes: Vec<ByteString<'a>>,
}

pub(super) struct ByteString<'a>(pub(super) &'a [u8]);

impl Serialize for ByteString<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.0)
    }
}

impl<'de: 'a, 'a> Deserialize<'de> for ByteString<'a> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Bytes;
        impl<'de> serde::de::Visitor<'de> for Bytes {
            type Value = ByteString<'de>;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a borrowed native member byte string")
            }
            fn visit_borrowed_bytes<E: serde::de::Error>(
                self,
                bytes: &'de [u8],
            ) -> Result<Self::Value, E> {
                Ok(ByteString(bytes))
            }
        }
        deserializer.deserialize_bytes(Bytes)
    }
}
