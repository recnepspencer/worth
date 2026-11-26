//! The canonical (logical) meaning of a chunk, independent of how its
//! history is layered, flattened or spilled into continuation pages.
use serde::{Deserialize, Serialize};

use crate::identity::data::{KindId, RelationId};

/// One canonical version of a slot's record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlotVersion {
    pub kind: KindId,
    /// Canonical record bytes. Large field values are carried out of line
    /// behind value locators and streamed by `ValueReader`.
    #[serde(with = "byte_vec")]
    pub payload: Vec<u8>,
}

/// Encodes `Vec<u8>` using MessagePack binary (`bin`) framing rather than an
/// integer array so every payload byte takes 1 byte on the wire.
mod byte_vec {
    use std::fmt;

    use serde::{de, Deserializer, Serializer};

    pub(super) fn serialize<S: Serializer>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(bytes)
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<u8>, D::Error> {
        struct BytesVisitor;

        impl<'de> de::Visitor<'de> for BytesVisitor {
            type Value = Vec<u8>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a byte buffer")
            }

            fn visit_bytes<E: de::Error>(self, value: &[u8]) -> Result<Self::Value, E> {
                Ok(value.to_vec())
            }

            fn visit_byte_buf<E: de::Error>(self, value: Vec<u8>) -> Result<Self::Value, E> {
                Ok(value)
            }
        }

        deserializer.deserialize_byte_buf(BytesVisitor)
    }
}

/// One incident relation of a vertex slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AdjacencyEdge {
    pub relation_kind: KindId,
    pub relation: RelationId,
}

/// One slot as its producer streams it into the sealer. History is ordered
/// oldest to newest; both sequences are consumed exactly once, so a slot whose
/// history or adjacency exceeds memory never has to be materialized.
pub struct SlotProjection<'source> {
    pub slot: u64,
    pub head: Option<SlotVersion>,
    pub history: Box<dyn Iterator<Item = SlotVersion> + 'source>,
    pub adjacency: Box<dyn Iterator<Item = AdjacencyEdge> + 'source>,
}

/// Receives a chunk's canonical meaning in canonical order: slots ascending;
/// within a slot its head, then history oldest to newest, then adjacency.
///
/// The sealer and every cold reader drive the same visitor, so a commitment
/// derived on either side is computed by one function over one order.
pub trait CanonicalChunkVisitor {
    type Error;

    fn slot_begin(&mut self, slot: u64, head: Option<&SlotVersion>) -> Result<(), Self::Error>;
    fn history_version(&mut self, version: &SlotVersion) -> Result<(), Self::Error>;
    fn adjacency_edge(&mut self, edge: &AdjacencyEdge) -> Result<(), Self::Error>;
    fn slot_end(&mut self) -> Result<(), Self::Error>;
}
