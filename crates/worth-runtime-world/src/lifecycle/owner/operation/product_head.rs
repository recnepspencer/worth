use super::RuntimeWorldOwnerRoot;

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(in crate::lifecycle::owner) fn current_product_head_is(
        &self,
        expected: &crate::branch::ProductBranchObservation,
    ) -> bool {
        #[cfg(feature = "test-execution-observer")]
        crate::lifecycle::read_observation::record_read();
        self.state
            .branches
            .branch_cell(expected.branch_identity())
            .map(|cell| cell.atomic_snapshot())
            .is_some_and(|current| expected.mismatch_against_snapshot(&current).is_none())
    }

    pub(in crate::lifecycle::owner) fn current_product_head_snapshot(
        &self,
        expected: &crate::branch::ProductBranchObservation,
    ) -> Option<crate::branch::ProductBranchReferenceSnapshot> {
        #[cfg(feature = "test-execution-observer")]
        crate::lifecycle::read_observation::record_read();
        self.state
            .branches
            .branch_cell(expected.branch_identity())
            .map(|cell| cell.atomic_snapshot())
    }
}
