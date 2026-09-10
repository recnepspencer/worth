use std::sync::atomic::Ordering;

use crate::branch::owner_services::conditional_execution::{
    SignalConditionalServiceExecutionDenial, SignalInstalledDefinitionBinding,
};
use crate::branch::owner_services::{SignalBranchCellState, SignalOwnerOperationAdmission};
use crate::branch::AdmittedSignalBranchBasis;

use super::basis::map_basis_cell_denial;
use super::{SignalBranchCellAdmissionDenial, SignalBranchExecutionCell};

impl<D, I, T> SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn admit_current_conditional_service(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
    ) -> Result<(), SignalConditionalServiceExecutionDenial> {
        self.inspect_conditional_definition(admission, basis, definition, |_| Ok(()))
    }

    pub(in crate::branch::owner_services) fn inspect_conditional_active_node_count(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
    ) -> Result<usize, SignalConditionalServiceExecutionDenial> {
        self.inspect_conditional_definition(admission, basis, definition, |graph| {
            Ok(graph.active_node_count())
        })
    }

    pub(in crate::branch::owner_services) fn readmit_conditional_contract(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        contract: &crate::data::conditional_execution::InstalledSignalConditionalContract,
    ) -> Result<
        crate::data::conditional_execution::InstalledSignalConditionalContract,
        SignalConditionalServiceExecutionDenial,
    > {
        self.inspect_conditional_definition(admission, basis, definition, |graph| {
            contract
                .bind_for_conditional_service(graph)
                .ok_or(SignalConditionalServiceExecutionDenial::DefinitionMismatch)?;
            Ok(contract.clone())
        })
    }

    fn inspect_conditional_definition<Output>(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        inspect: impl FnOnce(
            &crate::data::graph::SignalGraph,
        ) -> Result<Output, SignalConditionalServiceExecutionDenial>,
    ) -> Result<Output, SignalConditionalServiceExecutionDenial> {
        use SignalConditionalServiceExecutionDenial as Denial;
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
        let state = self
            .lock_state_after_contention_observation()
            .map_err(map_cell)?;
        self.require_live_posture().map_err(map_cell)?;
        state
            .observation()
            .map_err(|error| {
                Denial::OwnerAdmission(
                    crate::branch::SignalBranchBasisObservationDenial::InvalidOwnerObservation {
                        error,
                    },
                )
            })?
            .compare(basis.observation())
            .map_err(|_| Denial::StaleBasisAdmission)?;
        let current_definition = state
            .state()
            .installed_definition()
            .ok_or(Denial::DefinitionReadmissionRequired)?;
        if !current_definition.matches(definition) {
            return Err(Denial::DefinitionMismatch);
        }
        inspect(state.state().graph())
    }
}
