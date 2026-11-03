/// Copyable public selector for one owner-issued live product occurrence.
///
/// The token carries no component identity and grants no authority by itself.
/// Selection succeeds only when Runtime World resolves the same live
/// occurrence and Query admits its retained composite observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorthQueryProductBranch {
    occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
}

impl WorthQueryProductBranch {
    pub(crate) const fn from_occurrence(
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    ) -> Self {
        Self { occurrence }
    }

    pub const fn id(self) -> Self {
        self
    }

    pub const fn occurrence_ordinal(self) -> u64 {
        self.occurrence.ordinal()
    }

    pub(crate) const fn occurrence(self) -> worth_runtime_world::facade::ProductBranchIncarnation {
        self.occurrence
    }
}
