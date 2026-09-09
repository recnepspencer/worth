use std::sync::atomic::Ordering;

use crate::branch::owner_services::conditional_execution::{
    SignalConditionalInstallationExtensionDenial as Denial, SignalConditionalInstallationTarget,
    SignalInstalledDefinitionBinding, SignalPreparedInstallationTarget,
};
use crate::branch::owner_services::{SignalBranchCellState, SignalOwnerOperationAdmission};
use crate::branch::AdmittedSignalBranchBasis;
use crate::data::aspect::SignalAspectLoweringOwner;
use crate::data::conditional_execution::SignalConditionalContractDefinition;
use crate::data::retained_storage::{
    RetainedStorageCharge, RetainedStoragePreparation, RetainedStoragePreparationDenial,
    SignalConditionalRetentionLedger,
};

use super::basis::map_basis_cell_denial;
use super::{SignalBranchCellAdmissionDenial, SignalBranchExecutionCell};

impl<D, I, T> SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    #[cfg(test)]
    pub(in crate::branch::owner_services) fn set_conditional_installation_work_budget_for_test(
        &self,
        maximum_visits: usize,
    ) {
        let mut state = self
            .lock_state_after_contention_observation()
            .expect("test branch cell remains available");
        let graph = state.state_mut().graph_mut();
        let policy = graph.runtime_policy();
        graph.set_runtime_policy(policy.with_conditional_evaluation_budget(
            crate::runtime_policy::SignalConditionalEvaluationBudget {
                maximum_attempt_visits: maximum_visits,
                ..policy.conditional_evaluation_budget
            },
        ));
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::branch::owner_services) fn prepare_conditional_installation(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition_binding: &SignalInstalledDefinitionBinding,
        claimant: &SignalAspectLoweringOwner,
        target: SignalConditionalInstallationTarget,
        definition: SignalConditionalContractDefinition,
        ledger: &std::sync::Arc<SignalConditionalRetentionLedger>,
    ) -> Result<
        (
            SignalPreparedInstallationTarget,
            crate::branch::owner_services::conditional_execution::SignalConditionalInstallationCustody,
        ),
        Denial,
    >{
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
        let live = state.observation().map_err(|error| {
            Denial::OwnerAdmission(
                crate::branch::SignalBranchBasisObservationDenial::InvalidOwnerObservation {
                    error,
                },
            )
        })?;
        live.compare(basis.observation())
            .map_err(|mismatch| Denial::StaleBasis {
                axes: mismatch.axes().to_vec(),
            })?;
        let installed_definition = state
            .state()
            .installed_definition()
            .ok_or(Denial::DefinitionMismatch)?;
        if !installed_definition.matches(definition_binding)
            || !installed_definition.is_claimed_by(claimant)
        {
            return Err(Denial::DefinitionMismatch);
        }

        let maximum_visits = state
            .state()
            .graph()
            .installed_runtime_policy()
            .conditional_evaluation_budget()
            .maximum_attempt_visits;
        let mut work = RetainedStoragePreparation::new(maximum_visits);
        work.visit().map_err(map_work_denial)?;
        let retained_bytes = definition.installation_retained_bytes();
        work.visit().map_err(map_work_denial)?;
        let charge =
            RetainedStorageCharge::capacity::<u8>(retained_bytes).map_err(map_work_denial)?;
        let custody = ledger.reserve(1, charge).map_err(|denial| match denial {
            crate::data::retained_storage::SignalConditionalRetentionDenial::CapacityExhausted => {
                Denial::CapacityExhausted
            }
            _ => Denial::RetentionUnavailable,
        })?;
        let custody = crate::branch::owner_services::conditional_execution::SignalConditionalInstallationCustody::new(custody);
        let prepared = match target {
            SignalConditionalInstallationTarget::Existing {
                node,
                expected_contract_generation,
            } => {
                let entry = state
                    .state()
                    .graph()
                    .get_entry(node)
                    .map_err(|_| Denial::TargetUnavailable)?;
                let observed = entry.conditional_contract_generation();
                if observed != expected_contract_generation {
                    return Err(Denial::TargetGenerationMismatch {
                        expected: expected_contract_generation,
                        observed,
                    });
                }
                let worth_proof::TransitionOutcome::Success(capability) =
                    state.state().graph().admit_installed_node(node)
                else {
                    return Err(Denial::TargetUnavailable);
                };
                let prepared = state
                    .state()
                    .graph()
                    .prepare_conditional_contract(claimant, capability, definition)
                    .map_err(|_| Denial::DefinitionMismatch)?;
                SignalPreparedInstallationTarget::Existing(prepared)
            }
            SignalConditionalInstallationTarget::Allocate => {
                SignalPreparedInstallationTarget::Allocate(definition)
            }
        };
        Ok((prepared, custody))
    }
}

fn map_work_denial(denial: RetainedStoragePreparationDenial) -> Denial {
    match denial {
        RetainedStoragePreparationDenial::WorkExhausted { maximum_visits } => {
            Denial::WorkExhausted { maximum_visits }
        }
        _ => Denial::RetentionUnavailable,
    }
}
