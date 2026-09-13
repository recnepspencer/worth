use crate::identity::CompositeCommitIdentity;

/// The sole parent form admitted by milestone 9.17.2. Multi-parent sets are a
/// later, explicitly versioned successor and are not represented here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrdinaryParent {
    commit: CompositeCommitIdentity,
}

impl OrdinaryParent {
    pub(crate) fn new(commit: CompositeCommitIdentity) -> Self {
        Self { commit }
    }

    pub fn commit(&self) -> &CompositeCommitIdentity {
        &self.commit
    }
}
