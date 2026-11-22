//! Replace a mutation snapshot before its provider execution session is bound.

use super::{WorthQueryGraphWorkBasis, WorthQueryManagedGraphWorkSession};

impl WorthQueryManagedGraphWorkSession {
    /// Restore a temporarily taken mutation lease back into the mutation basis.
    pub fn restore_mutation_lease(&mut self, lease: Box<dyn crate::ports::PrimaryMutationLease>) {
        if let WorthQueryGraphWorkBasis::Mutation(basis) = &mut self.basis {
            basis.lease = Some(lease);
        }
    }

    pub fn readmit_mutation_lease(
        &mut self,
        expected: &crate::basis::WorthQueryProductBranchLease,
        current: Box<dyn crate::ports::PrimaryMutationLease>,
    ) -> Result<(), Box<dyn crate::ports::PrimaryMutationLease>> {
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
            || !basis.admits_replacement_authorization(&self.authorization_branch, current.as_ref())
        {
            return Err(current);
        }
        basis.lease = Some(current);
        Ok(())
    }
}
