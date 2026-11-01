use std::sync::atomic::Ordering;

use crate::branch::{
    AdmittedSignalInitialEmptyRoot, RecordedSignalInitialEmptyRoot, SignalBranchObservation,
    SignalInitialEmptyRootRefusal,
};
use crate::state::SignalBranchId;

use super::super::{
    SignalBranchCellState, SignalBranchExecutionCell, SignalOwnerOperationAdmission,
};
use super::SignalOwner;

pub(super) struct InitialEmptyRootWitness {
    pub(super) branch: SignalBranchId,
    pub(super) incarnation: u64,
    pub(super) observation: SignalBranchObservation,
    pub(super) recorded: RecordedSignalInitialEmptyRoot,
}

impl<D, I, T> SignalOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(super) fn install_initial_empty_witness(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        cell: &SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>,
        schema_commitment: &str,
    ) {
        let witness = InitialEmptyRootWitness {
            branch: cell.branch_id(),
            incarnation: cell.incarnation().get(),
            observation: cell
                .observe_exact(admission)
                .expect("the newly transferred initial cell is observable"),
            recorded: RecordedSignalInitialEmptyRoot::new(schema_commitment.to_owned()),
        };
        assert!(self.initial_empty_root.set(witness).is_ok());
    }

    pub(in crate::branch::owner_services) fn readmit_initial_empty_root(
        self: &std::sync::Arc<Self>,
        recorded: Option<&RecordedSignalInitialEmptyRoot>,
    ) -> Result<AdmittedSignalInitialEmptyRoot, SignalInitialEmptyRootRefusal> {
        use SignalInitialEmptyRootRefusal as Refusal;
        if let Some(recorded) = recorded {
            recorded.validate_version()?;
        }
        let witness = self
            .initial_empty_root
            .get()
            .ok_or(Refusal::InitialRootRequired)?;
        if recorded.is_some_and(|recorded| recorded != &witness.recorded) {
            return Err(Refusal::RecordedMeaningMismatch);
        }
        let admission = self.admit().map_err(|_| Refusal::OwnerAdmissionRequired)?;
        let cell = self
            .lookup_cell(&admission, witness.branch)
            .map_err(|_| Refusal::CurrentBasisRequired)?;
        let observation = cell
            .observe_exact(&admission)
            .map_err(|_| Refusal::CurrentBasisRequired)?;
        if cell.incarnation().get() != witness.incarnation
            || observation != witness.observation
            || self.next_branch_id.load(Ordering::Acquire) != 1
            || !self
                .metadata
                .has_initial_empty_history(&admission)
                .map_err(|_| Refusal::OwnerAdmissionRequired)?
        {
            return Err(Refusal::CurrentBasisRequired);
        }
        let basis = self
            .admit_canonical_basis_with_retention(
                &admission,
                observation,
                witness.branch,
                witness.incarnation,
                || self.acquire_admitted_retention(&admission, witness.branch),
            )
            .map_err(|_| Refusal::RetentionRequired)?;
        Ok(AdmittedSignalInitialEmptyRoot::new(
            recorded.unwrap_or(&witness.recorded).clone(),
            basis,
        ))
    }
}
