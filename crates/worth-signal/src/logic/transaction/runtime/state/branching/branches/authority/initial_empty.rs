use super::BranchState;

impl<D, I, T> BranchState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn has_initial_empty_origin(&self) -> bool {
        self.graph().has_initial_empty_origin()
            && self.installed_definition.is_none()
            && self.ancestry.parent_branch_id.is_none()
            && self.ancestry.forked_from_snapshot_id.is_none()
            && self.ancestry.latest_merge_reference.is_none()
            && self.mutation_ledger.pending.is_empty()
            && self.mutation_ledger.baseline_snapshot_id.is_none()
    }
}
