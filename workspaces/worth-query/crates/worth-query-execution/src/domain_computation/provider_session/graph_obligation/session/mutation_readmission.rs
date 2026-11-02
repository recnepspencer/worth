//! Replace a mutation snapshot before its provider execution session is bound.

use super::{
    WorthQueryApplicationSnapshotLease, WorthQueryGraphWorkBasis, WorthQueryManagedGraphWorkSession,
};

impl WorthQueryManagedGraphWorkSession {
    pub(in crate::domain_computation) fn readmit_mutation_lease(
        &mut self,
        expected: &crate::basis::WorthQueryProductBranchLease,
        current: WorthQueryApplicationSnapshotLease,
    ) -> Result<(), WorthQueryApplicationSnapshotLease> {
        let WorthQueryGraphWorkBasis::Mutation(basis) = &mut self.basis else {
            return Err(current);
        };
        let Some(retained) = &basis.lease else {
            return Err(current);
        };
        if !retained.product().has_same_selected_occurrence(expected)
            || current.product().observation().lifecycle_incarnation()
                != expected.observation().lifecycle_incarnation()
            || !self.branch.admits_snapshot(current.snapshot())
            || !basis.admits_replacement_authorization(&self.authorization_branch, &current)
        {
            return Err(current);
        }
        basis.lease = Some(current);
        Ok(())
    }
}
