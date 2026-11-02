//! Portable invariant Source requirements. These values describe meaning, not admission.

/// Fixed platform meaning for logical invariant Source and physical availability.
/// It cannot choose a transaction posture or authorize any live Source read.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ManagedSourceInterpretationCommitment {
    LogicalMaterializationV1,
}

impl ManagedSourceInterpretationCommitment {
    pub const fn revision(self) -> u16 {
        match self {
            Self::LogicalMaterializationV1 => 1,
        }
    }

    pub const fn from_revision(revision: u16) -> Option<Self> {
        match revision {
            1 => Some(Self::LogicalMaterializationV1),
            _ => None,
        }
    }

    pub const fn canonical_token(self) -> &'static str {
        match self {
            Self::LogicalMaterializationV1 => "managed-source-interpretation-v1",
        }
    }
}

/// Whether Native may attempt an admitted cold acquisition for an invariant Source.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub enum ManagedIndexColdSourcePosture {
    #[default]
    Refuse,
    AdmitRebuild,
}

impl ManagedIndexColdSourcePosture {
    pub const fn canonical_token(self) -> &'static str {
        match self {
            Self::Refuse => "refuse",
            Self::AdmitRebuild => "admit-rebuild",
        }
    }
}

/// Portable structure exposed by an index, not a query-completeness or READ witness.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub enum ManagedIndexSourceStructure {
    /// Provider-private topology; dependencies remain conservative.
    #[default]
    Opaque,
    /// One connected binary decomposition containing each Source occurrence once.
    BinarySourceTree,
}
impl ManagedIndexSourceStructure {
    pub const fn canonical_token(self) -> &'static str {
        match self {
            Self::Opaque => "opaque",
            Self::BinarySourceTree => "binary-source-tree-v1",
        }
    }
}
