//! Typed generational handle for signal graph nodes.
//!
//! DOMAIN: Stable identity for reactive signal nodes.
//!
//! INVARIANTS:
//! - Each node has a unique `(index, generation)` pair
//! - Stale handles are detected via generation mismatch
//! - Handles are `Copy` for cheap passing
//!
//! DEPENDENCIES: None

use serde::{Deserialize, Serialize};

mod retained_charge;

/// A typed, generational handle for a signal graph node.
///
/// - `index`: slot position in the graph's node arena
/// - `generation`: incremented when a slot is reused after deletion
///
/// Stale handles (from deleted nodes) are detected by generation
/// mismatch, preventing use-after-free without `unsafe`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId {
    index: u32,
    generation: u32,
}

impl NodeId {
    /// Create a new handle. Typically only called by the graph allocator.
    pub fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    /// The slot index in the arena.
    pub fn index(self) -> u32 {
        self.index
    }

    /// The generation counter (for stale-handle detection).
    pub fn generation(self) -> u32 {
        self.generation
    }
}

impl Serialize for NodeId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = format!("{}:{}", self.index, self.generation);
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for NodeId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct NodeIdVisitor;

        impl<'de> serde::de::Visitor<'de> for NodeIdVisitor {
            type Value = NodeId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string formatted as 'index:generation'")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let parts: Vec<&str> = value.split(':').collect();
                if parts.len() != 2 {
                    return Err(E::custom("invalid format, expected 'index:generation'"));
                }
                let index = parts[0].parse().map_err(E::custom)?;
                let generation = parts[1].parse().map_err(E::custom)?;
                Ok(NodeId { index, generation })
            }
        }

        deserializer.deserialize_str(NodeIdVisitor)
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NodeId({}:gen{})", self.index, self.generation)
    }
}
