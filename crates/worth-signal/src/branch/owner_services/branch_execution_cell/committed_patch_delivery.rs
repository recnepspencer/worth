use std::collections::BTreeSet;
use std::sync::atomic::Ordering;

use worth_proof::TransitionOutcome;

use crate::branch::owner_services::conditional_execution::{
    SignalConditionalBasisCaptureDenial, SignalConditionalServiceAuthority,
    SignalInstalledDefinitionBinding, SignalPreparedConditionalTransition,
};
use crate::branch::owner_services::{SignalBranchCellState, SignalOwnerOperationAdmission};
use crate::branch::{
    AdmittedSignalBranchBasis, SignalCommittedPatchDeliveryCompletion,
    SignalCommittedPatchDeliveryDenial,
};
use crate::data::aspect::{
    apply_installed_scoped_changes, AspectMask, InstalledSignalScopedChange,
    SignalInstalledScopedChangeDenial,
};
use crate::data::conditional_execution::InstalledSignalConditionalContract;
use crate::data::graph::SignalGraph;
use crate::data::retained_storage::SignalConditionalRetentionLedger;

use super::basis::map_basis_cell_denial;
use super::{SignalBranchCellAdmissionDenial, SignalBranchExecutionCell};

impl<D, I, T> SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn deliver_committed_patch(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        contract: &InstalledSignalConditionalContract,
        issuance_basis: &std::sync::Arc<
            crate::branch::owner_services::conditional_execution::SignalRetainedExecutionBasis,
        >,
        prepared_transition: SignalPreparedConditionalTransition,
        service_authority: std::sync::Arc<SignalConditionalServiceAuthority>,
        ledger: &std::sync::Arc<SignalConditionalRetentionLedger>,
    ) -> Result<SignalCommittedPatchDeliveryCompletion, SignalCommittedPatchDeliveryDenial> {
        use SignalCommittedPatchDeliveryDenial as Denial;
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
        contract
            .bind_for_conditional_service(state.state().graph())
            .ok_or(Denial::DefinitionMismatch)?;
        let predecessor = state
            .current_conditional_basis()
            .unwrap_or_else(|| std::sync::Arc::clone(issuance_basis));

        let graph_instance_id = state.state().graph().runtime_instance_id();
        let targets = prepared_transition.targets();
        if targets.is_empty() {
            return Err(Denial::EmptyChangeSet);
        }
        let mut unique = BTreeSet::new();
        for target in targets {
            if target.graph_instance_id() != graph_instance_id {
                return Err(Denial::ForeignGraph);
            }
            if target.node() != contract.node()
                || !contract
                    .dependency_aspects()
                    .contains(AspectMask::from_aspect(target.aspect()))
            {
                return Err(Denial::ForeignContractTarget);
            }
            if !unique.insert((target.node(), target.aspect())) {
                return Err(Denial::DuplicateTarget);
            }
        }

        let (mut staged_graph, fork_work) = state
            .committed_patch_graph_mut()
            .fork_conditional_successor();
        debug_assert_eq!(fork_work.copied_mutable_graph_nodes(), 0);
        let staged_changes = admit_changes(&mut staged_graph, targets)?;
        apply_staged_changes(&mut staged_graph, staged_changes)?;
        let successor = state
            .capture_conditional_basis_from_graph(&mut staged_graph, ledger)
            .map_err(map_capture_denial)?;

        let canonical_changes = admit_changes(state.committed_patch_graph_mut(), targets)
            .expect("staged committed-patch admission proves canonical admission");
        let completion = match apply_installed_scoped_changes(
            state.committed_patch_graph_mut(),
            canonical_changes,
        ) {
            TransitionOutcome::Success(changes) => changes,
            TransitionOutcome::Denied(_) => {
                panic!("staged committed-patch denial diverged from canonical graph")
            }
            TransitionOutcome::Failed(_) => {
                panic!("staged committed-patch mutation diverged from canonical graph")
            }
            TransitionOutcome::Deferred(never)
            | TransitionOutcome::Stale(never)
            | TransitionOutcome::RebindRequired(never) => match never {},
        };
        let transition = prepared_transition.complete(
            service_authority,
            definition.clone(),
            self.incarnation(),
            predecessor,
            std::sync::Arc::clone(&successor),
        );
        state.publish_conditional_basis(successor);
        Ok(SignalCommittedPatchDeliveryCompletion::new(
            completion, transition,
        ))
    }
}

pub(super) fn admit_changes(
    graph: &mut SignalGraph,
    targets: &[crate::branch::SignalCommittedPatchTarget],
) -> Result<Vec<InstalledSignalScopedChange>, crate::branch::SignalCommittedPatchDeliveryDenial> {
    let mut changes = Vec::with_capacity(targets.len());
    for target in targets {
        let TransitionOutcome::Success(capability) =
            graph.admit_installed_aspect(target.node(), target.aspect())
        else {
            return Err(crate::branch::SignalCommittedPatchDeliveryDenial::MissingOrStaleTarget);
        };
        changes.push(InstalledSignalScopedChange::new(
            capability,
            target.changed_regions().as_slice().iter().cloned(),
        ));
    }
    Ok(changes)
}

pub(super) fn apply_staged_changes(
    graph: &mut SignalGraph,
    changes: Vec<InstalledSignalScopedChange>,
) -> Result<(), crate::branch::SignalCommittedPatchDeliveryDenial> {
    use crate::branch::SignalCommittedPatchDeliveryDenial as Denial;
    match apply_installed_scoped_changes(graph, changes) {
        TransitionOutcome::Success(_) => Ok(()),
        TransitionOutcome::Denied(denial) => Err(match denial {
            SignalInstalledScopedChangeDenial::EmptyChangeSet => Denial::EmptyChangeSet,
            SignalInstalledScopedChangeDenial::ForeignCapability => Denial::ForeignGraph,
            SignalInstalledScopedChangeDenial::DuplicateTarget => Denial::DuplicateTarget,
            SignalInstalledScopedChangeDenial::MissingOrStaleTarget => Denial::MissingOrStaleTarget,
        }),
        TransitionOutcome::Failed(error) => Err(Denial::SignalMutation(error)),
        TransitionOutcome::Deferred(never)
        | TransitionOutcome::Stale(never)
        | TransitionOutcome::RebindRequired(never) => match never {},
    }
}

fn map_capture_denial(
    denial: SignalConditionalBasisCaptureDenial,
) -> crate::branch::SignalCommittedPatchDeliveryDenial {
    use crate::branch::SignalCommittedPatchDeliveryDenial as Denial;
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
