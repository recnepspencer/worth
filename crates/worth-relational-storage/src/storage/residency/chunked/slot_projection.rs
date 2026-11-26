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
    pub payload: Vec<u8>,
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
