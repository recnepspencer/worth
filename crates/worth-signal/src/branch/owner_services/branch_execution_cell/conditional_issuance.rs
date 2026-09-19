use std::sync::atomic::Ordering;

use crate::branch::owner_services::conditional_execution::{
    SignalConditionalServiceIssuanceDenial, SignalInstalledDefinitionBinding,
};
use crate::branch::owner_services::{SignalBranchCellState, SignalOwnerOperationAdmission};
use crate::branch::{AdmittedSignalBranchBasis, SignalBranchObservation};
use crate::data::aspect::SignalAspectLoweringOwner;

use super::basis::map_basis_cell_denial;
use super::{SignalBranchCellAdmissionDenial, SignalBranchExecutionCell};

impl<D, I, T> SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    /// Check definition custody while holding exactly the selected live cell.
    pub(in crate::branch::owner_services) fn admit_conditional_definition(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        claimant: &SignalAspectLoweringOwner,
        ledger: &std::sync::Arc<crate::data::retained_storage::SignalConditionalRetentionLedger>,
    ) -> Result<
        (
            SignalInstalledDefinitionBinding,
            SignalBranchObservation,
            std::sync::Arc<
                crate::branch::owner_services::conditional_execution::SignalRetainedExecutionBasis,
            >,
        ),
        SignalConditionalServiceIssuanceDenial,
    > {
        use SignalConditionalServiceIssuanceDenial as Denial;
        let map_cell =
            |denial| Denial::OwnerAdmission(map_basis_cell_denial(denial, self.branch_id));
        self.validate_admission(admission).map_err(map_cell)?;
        let _hold = admission
            .hold_branch_cell()
            .map_err(SignalBranchCellAdmissionDenial::from)
            .map_err(map_cell)?;
        self.counters.record_target_cell_contact();
        self.contacts.fetch_add(1, Ordering::SeqCst);
        self.require_live_posture().map_err(map_cell)?;
        let mut state = self
            .lock_state_after_contention_observation()
            .map_err(map_cell)?;
        self.require_live_posture().map_err(map_cell)?;
        let observation = state.observation().map_err(|error| {
            Denial::OwnerAdmission(
                crate::branch::SignalBranchBasisObservationDenial::InvalidOwnerObservation {
                    error,
                },
            )
        })?;
        observation
            .compare(basis.observation())
            .map_err(|mismatch| Denial::BasisMismatch {
                axes: mismatch.axes().to_vec(),
            })?;
        let definition = state
            .state()
            .installed_definition()
            .ok_or(Denial::DefinitionReadmissionRequired)?;
        if !definition.is_claimed_by(claimant) {
            return Err(Denial::ClaimantMismatch);
        }
        let definition = definition.clone();
        let retained_basis = state.capture_conditional_basis(ledger)?;
        Ok((definition, observation, retained_basis))
    }
}
