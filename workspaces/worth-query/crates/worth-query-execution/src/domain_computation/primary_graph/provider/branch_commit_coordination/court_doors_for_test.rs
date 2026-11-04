//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationBranchCommitCoordinator {
    #[doc(hidden)]
    pub fn entry_count_for_test(&self) -> usize {
        Self::entry_count(self)
    }
}

impl WorthQueryApplicationBranchCommitCoordinator {
    #[doc(hidden)]
    pub fn lane_for_for_test(
        &self,
        observation: &ProductBranchObservation,
    ) -> Arc<WorthQueryApplicationBranchCommitLane> {
        Self::lane_for(self, observation)
    }
}

impl WorthQueryApplicationBranchCommitCoordinator {
    #[doc(hidden)]
    pub fn new_for_test() -> Self {
        Self::new()
    }
}

impl WorthQueryApplicationBranchCommitCoordinator {
    #[doc(hidden)]
    pub fn retire_for_test(&self, occurrence: ProductBranchIncarnation) {
        Self::retire(self, occurrence)
    }
}
