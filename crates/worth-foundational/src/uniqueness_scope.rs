use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UniquenessScope {
    DirectedSemanticEdge,
    NormalizedSymmetricEdge,
}

impl UniquenessScope {
    /// Stable value for canonical relation declarations.
    pub const fn canonical_value(self) -> u64 {
        match self {
            Self::DirectedSemanticEdge => 1,
            Self::NormalizedSymmetricEdge => 2,
        }
    }
}
