//! Portable managed predicate execution meaning, never a coverage or READ witness.

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub enum ManagedIndexMembership {
    /// Provider-selected candidates with conservative complete Source dependencies.
    #[default]
    Candidates,
    /// Query visits the whole checked Source tree and applies its installed predicate.
    /// This explicit full traversal grants no pruning or narrowed influence.
    ExhaustiveSourceTree,
}
impl ManagedIndexMembership {
    pub const fn canonical_token(self) -> &'static str {
        match self {
            Self::Candidates => "candidates",
            Self::ExhaustiveSourceTree => "exhaustive-source-tree-v1",
        }
    }
}
