//! Portable graph aggregate names; these values grant no write authority.
use serde::{Deserialize, Serialize};

/// Meaning of one occurrence-qualified graph aggregate lineage event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AggregateLineageKind {
    SplitFrom,
    MergedInto,
}

/// Stable namespace of one declared aggregate definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AggregateDefinitionIdentity([u8; 32]);

impl AggregateDefinitionIdentity {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Persistent name of a graph aggregate, distinct from its changing anchor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AggregateIdentity([u8; 32]);

impl AggregateIdentity {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}
