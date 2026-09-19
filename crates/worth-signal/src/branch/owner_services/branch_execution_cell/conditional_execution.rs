use std::sync::atomic::Ordering;

use crate::branch::owner_services::conditional_execution::{
    SignalConditionalEvaluationAdmission, SignalConditionalEvaluationReadmissionCounters,
    SignalConditionalEvaluationReadmissionDenial, SignalConditionalEvaluationState,
    SignalConditionalServiceAuthority, SignalConditionalServiceCompletion,
    SignalConditionalServiceExecutionDenial, SignalConditionalSuccessorTransition,
    SignalInstalledDefinitionBinding, SignalRetainedExecutionBasis,
};
use crate::branch::owner_services::{SignalBranchCellState, SignalOwnerOperationAdmission};
use crate::branch::AdmittedSignalBranchBasis;
use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::conditional_execution::{
    InstalledSignalConditionResolver, InstalledSignalConditionalContract,
    SignalConditionalExecutionRequest, SignalConditionalServiceContractBinding,
};
use crate::data::error::SignalError;
use crate::data::output::NodeEvaluationResult;

use super::basis::map_basis_cell_denial;
use super::{SignalBranchCellAdmissionDenial, SignalBranchExecutionCell};

impl<D, I, T> SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    #[allow(clippy::too_many_arguments)]
    pub(in crate::branch::owner_services) fn readmit_conditional_evaluation(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        service_authority: &std::sync::Arc<SignalConditionalServiceAuthority>,
        predecessor: &SignalConditionalEvaluationAdmission,
        predecessor_state: &mut SignalConditionalEvaluationState,
        transitions: &[&SignalConditionalSuccessorTransition],
        target_basis: &std::sync::Arc<SignalRetainedExecutionBasis>,
        admission_custody: &mut crate::data::retained_storage::SignalConditionalRetentionReservation,
    ) -> Result<
        (
            SignalConditionalServiceContractBinding,
            crate::data::graph::storage::evaluation_partition::SignalEvaluationPartition,
            SignalConditionalEvaluationReadmissionCounters,
        ),
        SignalConditionalEvaluationReadmissionDenial,
    > {
        use SignalConditionalEvaluationReadmissionDenial as Denial;
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
        let contract_binding = predecessor
            .contract
            .bind_for_conditional_service(state.state().graph())
            .ok_or(Denial::DefinitionMismatch)?;
        validate_transition_chain(
            service_authority,
            definition,
            self.incarnation(),
            predecessor,
            transitions,
            state.current_conditional_basis().as_ref(),
            target_basis,
        )?;
        if predecessor_state
            .slot
            .as_ref()
            .is_some_and(|slot| slot.partition.has_pending_conditional_unwind())
        {
            return Err(Denial::UnconsumedUnwind);
        }
        let maximum_visits = state
            .state()
            .graph()
            .installed_runtime_policy()
            .conditional_evaluation_budget()
            .maximum_attempt_visits;
        let mut work =
            crate::data::retained_storage::RetainedStoragePreparation::new(maximum_visits);
        let slot = predecessor_state
            .slot
            .as_mut()
            .ok_or(Denial::PredecessorNotExecuted)?;
        let mut partition = target_basis
            .fork_evaluation_partition_from(&mut slot.partition, &mut work, admission_custody)
            .map_err(Denial::SlotAdmission)?;
        let targets_applied = transitions
            .iter()
            .map(|transition| transition.inner.targets.len())
            .sum();
        let application = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let application = partition
                .execute(state.conditional_execution_graph_mut(), |graph| {
                    for transition in transitions {
                        let changes = super::committed_patch_delivery::admit_changes(
                            graph,
                            &transition.inner.targets,
                        )
                        .map_err(|_| Denial::TransitionChainMismatch)?;
                        super::committed_patch_delivery::apply_staged_changes(graph, changes)
                            .map_err(|denial| {
                                match denial {
                                crate::branch::SignalCommittedPatchDeliveryDenial::SignalMutation(
                                    error,
                                ) => Denial::SlotAdmission(error),
                                _ => Denial::TransitionChainMismatch,
                            }
                            })?;
                        #[cfg(test)]
                        crate::branch::owner_services::conditional_execution::panic_after_transition_apply_if_armed();
                    }
                    Ok::<_, Denial>(())
                })
                .map_err(Denial::SlotAdmission)?;
            application?;
            partition
                .prepare_persistent_fork_readiness(&mut work)
                .map_err(Denial::SlotAdmission)
        }));
        match application {
            Ok(result) => result?,
            Err(payload) => {
                drop(partition);
                drop(state);
                std::panic::resume_unwind(payload)
            }
        }
        debug_assert!(partition.has_retention_ledger());
        Ok((
            contract_binding,
            partition,
            SignalConditionalEvaluationReadmissionCounters::new(
                transitions.len(),
                targets_applied,
                1,
            ),
        ))
    }

    pub(in crate::branch::owner_services) fn admit_conditional_evaluation(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        contract: &InstalledSignalConditionalContract,
        issuance_basis: &std::sync::Arc<
            crate::branch::owner_services::conditional_execution::SignalRetainedExecutionBasis,
        >,
    ) -> Result<
        (
            SignalConditionalServiceContractBinding,
            std::sync::Arc<
                crate::branch::owner_services::conditional_execution::SignalRetainedExecutionBasis,
            >,
        ),
        SignalConditionalServiceExecutionDenial,
    > {
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
        if !current_definition.matches(definition) {
            return Err(Denial::DefinitionMismatch);
        }
        let contract_binding = contract
            .bind_for_conditional_service(state.state().graph())
            .ok_or(Denial::DefinitionMismatch)?;
        let retained_basis = state
            .current_conditional_basis()
            .unwrap_or_else(|| std::sync::Arc::clone(issuance_basis));
        Ok((contract_binding, retained_basis))
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::branch::owner_services) fn execute_conditional(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        evaluation: &SignalConditionalEvaluationAdmission,
        evaluation_state: &mut SignalConditionalEvaluationState,
        request: SignalConditionalExecutionRequest<'_>,
        condition: &mut impl InstalledSignalConditionResolver,
        comparator: &mut impl ComparatorPolicyResolver,
        compute: impl FnOnce() -> Result<NodeEvaluationResult, SignalError>,
    ) -> Result<SignalConditionalServiceCompletion, SignalConditionalServiceExecutionDenial> {
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
        if !current_definition.matches(definition) {
            return Err(Denial::DefinitionMismatch);
        }
        if !evaluation.contract_binding.matches(request.contract())
            || !evaluation
                .contract_binding
                .is_current_for(state.state().graph())
        {
            return Err(Denial::DefinitionMismatch);
        }
        let execution = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            super::super::conditional_execution::execute_conditional_against_graph(
                state.conditional_execution_graph_mut(),
                evaluation,
                evaluation_state,
                request,
                condition,
                comparator,
                compute,
            )
        }));
        match execution {
            Ok(completion) => completion,
            Err(payload) => {
                drop(state);
                std::panic::resume_unwind(payload)
            }
        }
    }
}

fn validate_transition_chain(
    service_authority: &std::sync::Arc<SignalConditionalServiceAuthority>,
    definition: &SignalInstalledDefinitionBinding,
    incarnation: crate::branch::owner_services::SignalBranchCellIncarnation,
    predecessor: &SignalConditionalEvaluationAdmission,
    transitions: &[&SignalConditionalSuccessorTransition],
    current_basis: Option<&std::sync::Arc<SignalRetainedExecutionBasis>>,
    target_basis: &std::sync::Arc<SignalRetainedExecutionBasis>,
) -> Result<(), SignalConditionalEvaluationReadmissionDenial> {
    use SignalConditionalEvaluationReadmissionDenial as Denial;
    let Some(current_basis) = current_basis else {
        return Err(Denial::DefinitionReadmissionRequired);
    };
    let mut expected = &predecessor.retained_basis;
    for transition in transitions {
        if !std::sync::Arc::ptr_eq(service_authority, &transition.inner.service_authority)
            || !transition.inner.definition.matches(definition)
            || transition.inner.incarnation != incarnation
            || !transition.predecessor_is(expected)
        {
            return Err(Denial::TransitionChainMismatch);
        }
        expected = &transition.inner.successor;
    }
    if !std::sync::Arc::ptr_eq(expected, target_basis) {
        return Err(Denial::TransitionChainMismatch);
    }
    if !std::sync::Arc::ptr_eq(target_basis, current_basis) {
        return Err(Denial::TransitionChainIncomplete);
    }
    Ok(())
}
