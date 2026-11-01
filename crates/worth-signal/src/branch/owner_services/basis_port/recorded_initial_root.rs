use crate::branch::{
    AdmittedSignalBranchBasis, AdmittedSignalInitialEmptyRoot, RecordedSignalInitialEmptyRoot,
    SignalInitialEmptyRootRefusal,
};

use super::SignalBranchBasisPort;

impl<D, I, T> SignalBranchBasisPort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn export_recorded_initial_empty_root_in(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &AdmittedSignalBranchBasis,
    ) -> Result<RecordedSignalInitialEmptyRoot, SignalInitialEmptyRootRefusal> {
        self.export_recorded_initial_empty_root(basis)?
            .retained_copy_in(execution)
    }

    pub fn export_recorded_initial_empty_root(
        &self,
        basis: &AdmittedSignalBranchBasis,
    ) -> Result<RecordedSignalInitialEmptyRoot, SignalInitialEmptyRootRefusal> {
        self.compare_current_exact(basis)
            .map_err(|_| SignalInitialEmptyRootRefusal::CurrentBasisRequired)?;
        let admitted = self
            .upgrade_owner()
            .map_err(|_| SignalInitialEmptyRootRefusal::OwnerUnavailable)?
            .readmit_initial_empty_root(None)?;
        if admitted.basis().descriptor() != basis.descriptor() {
            return Err(SignalInitialEmptyRootRefusal::CurrentBasisRequired);
        }
        Ok(admitted.recorded().clone())
    }

    pub fn readmit_recorded_initial_empty_root(
        &self,
        recorded: &RecordedSignalInitialEmptyRoot,
    ) -> Result<AdmittedSignalInitialEmptyRoot, SignalInitialEmptyRootRefusal> {
        self.upgrade_owner()
            .map_err(|_| SignalInitialEmptyRootRefusal::OwnerUnavailable)?
            .readmit_initial_empty_root(Some(recorded))
    }
}
