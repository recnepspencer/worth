//! Domain evidence retained verbatim by a rejected invariant candidate.

use serde::{Deserialize, Serialize};

/// The rule's declared application codec owns the bytes. Relational neither
/// interprets them nor substitutes a diagnostic message for them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustomInvariantRefusal {
    /// An explicitly cause-free native rule has no application value.
    Empty,
    DeclaredValue {
        value_identity: String,
        bytes: Vec<u8>,
    },
}

impl CustomInvariantRefusal {
    pub fn owned_allocation_capacity_bytes(&self) -> u64 {
        match self {
            Self::Empty => 0,
            Self::DeclaredValue {
                value_identity,
                bytes,
            } => (value_identity.capacity() as u64).saturating_add(bytes.capacity() as u64),
        }
    }
}
