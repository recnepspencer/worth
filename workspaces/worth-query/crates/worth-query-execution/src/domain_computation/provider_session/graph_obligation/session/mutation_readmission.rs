//! Replace a mutation snapshot before its provider execution session is bound.

use super::{
    WorthQueryApplicationSnapshotLease, WorthQueryGraphWorkBasis, WorthQueryManagedGraphWorkSession,
};

impl WorthQueryManagedGraphWorkSession {
    /// Lend another share of the actual retained lease without removing the
    /// operation's original authorization and mutation-product custody.
    pub(in crate::domain_computation) fn share_mutation_lease(
        &mut self,
        control: crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl<'_, '_>,
    ) -> Result<Option<WorthQueryApplicationSnapshotLease>, crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial>{
        let WorthQueryGraphWorkBasis::Mutation(basis) = &mut self.basis else {
            return Ok(None);
        };
        let Some(lease) = basis.lease.take() else {
            return Ok(None);
        };
        let (original, retained) = lease.share_original(control)?;
        basis.lease = Some(original);
        Ok(Some(retained))
    }

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
