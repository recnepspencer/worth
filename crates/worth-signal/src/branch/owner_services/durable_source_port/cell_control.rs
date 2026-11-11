//! These sealed controls operate only on the receiver's actual canonical owner.
use super::{SignalDurableBranchSourcePort, SignalDurableSourceDenial as D};
impl<Dm, I, T> SignalDurableBranchSourcePort<Dm, I, T>
where
    Dm: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    #[doc(hidden)]
    pub fn owner_operation_control_for_test(
        &self,
    ) -> Result<super::super::SignalOwnerOperationControlForTest, D> {
        Ok(self.owner()?.operation_control_for_test())
    }
    #[doc(hidden)]
    pub fn hold_branch_cell_for_test<R>(
        &self,
        expected: &crate::branch::AdmittedSignalBranchBasis,
        hold: impl FnOnce() -> R,
    ) -> Result<R, D> {
        let owner = self.owner()?;
        if !owner.basis_has_owner_affinity(expected) {
            return Err(D::ForeignBasis);
        }
        let admission = owner
            .admit()
            .map_err(super::super::basis_port::denial_mapping::map_basis_admission_denial)
            .map_err(D::Admission)?;
        let id = expected.owner_branch_id();
        let cell = owner
            .lookup_cell(&admission, id)
            .map_err(|cause| super::super::owner::basis::map_basis_registry_denial(cause, id))
            .map_err(D::Observation)?;
        cell.with_held_cell_control(&admission, hold)
            .map_err(|cause| {
                super::super::branch_execution_cell::basis::map_basis_cell_denial(cause, id)
            })
            .map_err(D::Observation)
    }
}
