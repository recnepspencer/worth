use std::sync::atomic::Ordering;

use crate::branch::owner_services::conditional_execution::{
    SignalConditionalBasisCaptureDenial, SignalInstalledDefinitionBinding,
};
use crate::branch::owner_services::{SignalBranchCellState, SignalOwnerOperationAdmission};
use crate::branch::{
    AdmittedSignalBranchBasis, SignalConditionalInstallationChangeDenial,
    SignalConditionalRetirementCompletion,
};
use crate::data::conditional_execution::InstalledSignalConditionalContract;
use crate::data::retained_storage::SignalConditionalRetentionLedger;

use super::basis::map_basis_cell_denial;
use super::{SignalBranchCellAdmissionDenial, SignalBranchExecutionCell};

impl<D, I, T> SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn retire_conditional_contract(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        contract: &InstalledSignalConditionalContract,
        ledger: &std::sync::Arc<SignalConditionalRetentionLedger>,
    ) -> Result<SignalConditionalRetirementCompletion, SignalConditionalInstallationChangeDenial>
    {
        use SignalConditionalInstallationChangeDenial as Denial;
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
            .map_err(|_| Denial::StaleBasisAdmission)?;
        let current_definition = state
            .state()
            .installed_definition()
            .ok_or(Denial::DefinitionReadmissionRequired)?;
        if !current_definition.matches(definition)
            || contract
                .bind_for_conditional_service(state.state().graph())
                .is_none()
        {
            return Err(Denial::DefinitionMismatch);
        }
        let (mut staged_graph, fork_work) = state
            .committed_patch_graph_mut()
            .fork_conditional_successor();
        debug_assert_eq!(fork_work.copied_mutable_graph_nodes(), 0);
        staged_graph
            .unregister_node(contract.node())
            .map_err(Denial::SignalMutation)?;
        let successor = state
            .capture_conditional_basis_from_graph(&mut staged_graph, ledger)
            .map_err(map_capture_denial)?;
        state
            .committed_patch_graph_mut()
            .unregister_node(contract.node())
            .expect("staged conditional retirement proves canonical retirement");
        state.publish_conditional_basis(successor);
        Ok(SignalConditionalRetirementCompletion::new(contract.node()))
    }
}

fn map_capture_denial(
    denial: SignalConditionalBasisCaptureDenial,
) -> crate::branch::SignalConditionalInstallationChangeDenial {
    use crate::branch::SignalConditionalInstallationChangeDenial as Denial;
    match denial {
        SignalConditionalBasisCaptureDenial::CapacityExhausted => {
            Denial::SuccessorCaptureCapacityExhausted
        }
        SignalConditionalBasisCaptureDenial::WorkExhausted { maximum_visits } => {
            Denial::SuccessorCaptureWorkExhausted { maximum_visits }
        }
        SignalConditionalBasisCaptureDenial::OwnerUnavailable => {
            Denial::OwnerUnavailable(crate::branch::owner_services::SignalOwnerUnavailable)
        }
        SignalConditionalBasisCaptureDenial::Unavailable => Denial::SuccessorCaptureUnavailable,
    }
}
