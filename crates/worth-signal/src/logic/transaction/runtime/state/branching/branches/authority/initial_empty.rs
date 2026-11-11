use super::BranchState;

impl<D, I, T> BranchState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn has_initial_empty_origin(&self) -> bool {
        self.installed_definition.is_none() && self.has_initial_empty_sealed_origin()
    }

    /// Owner sealing installs immutable definition custody; it does not change
    /// the original empty graph, ancestry or mutation origin.
    pub(crate) fn has_initial_empty_sealed_origin(&self) -> bool {
        self.graph().has_initial_empty_origin()
            && self.ancestry.parent_branch_id.is_none()
            && self.ancestry.forked_from_snapshot_id.is_none()
            && self.ancestry.latest_merge_reference.is_none()
            && self.mutation_ledger.pending.is_empty()
            && self.mutation_ledger.baseline_snapshot_id.is_none()
    }
}
