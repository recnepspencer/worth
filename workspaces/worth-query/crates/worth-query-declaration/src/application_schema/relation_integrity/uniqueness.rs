/// Which physical relation records must describe a unique semantic edge.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationRelationUniquenessScope {
    /// At most one record for each ordered source/target pair.
    DirectedSemanticEdge,
    /// At most one record across both directions of an unordered pair.
    NormalizedSymmetricEdge,
}

impl ApplicationRelationUniquenessScope {
    pub(super) const fn canonical_value(self) -> u64 {
        match self {
            Self::DirectedSemanticEdge => 1,
            Self::NormalizedSymmetricEdge => 2,
        }
    }
}
