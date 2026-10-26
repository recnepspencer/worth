use super::{
    require_local_identity, DeletedRelationalBranch, RelationalBranchDeleteDenial,
    RelationalBranchDeletionOutcome, RelationalBranchDeletionPending,
};
use crate::branch::{
    LiveBranchTransitionStep, RelationalBranchIdentity, RelationalBranchLifecyclePosture,
    RelationalBranchPublicationCell, RelationalBranchTransition,
    RelationalBranchTransitionDurableStep, RelationalBranchTransitionStop,
};
use crate::history::retention::RelationalRetentionAcquisitionDenial;
use crate::runtime::BranchTransitionRecordError;

/// Which deletion transition to perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RelationalBranchDeletionKind {
    /// Decide from the branch's active operations, as a live delete does:
    /// request deletion while any remain, otherwise delete.
    ByActiveOperations,
    /// Mark the branch for deletion and retire its name.
    Requested,
    /// Remove the branch, retire its head and its name.
    Delete,
}

type Failure<S> = BranchTransitionRecordError<RelationalBranchDeleteDenial, S>;

impl crate::runtime::RelationalRuntime {
    /// Delete a branch: remove it and retire its head and name, or, while
    /// operations still hold it, mark it for deletion. Either is durable
    /// before it is live; a refusal or an unresolved fate leaves the
    /// registry, retention and head index exactly as they were.
    pub fn delete_branch(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        identity: &RelationalBranchIdentity,
    ) -> Result<
        RelationalBranchDeletionOutcome,
        RelationalBranchTransitionStop<RelationalBranchDeleteDenial>,
    > {
        let step = LiveBranchTransitionStep::new(self.durability.medium(), execution);
        self.delete_branch_with(
            execution,
            &step,
            identity,
            RelationalBranchDeletionKind::ByActiveOperations,
        )
        .map_err(Into::into)
    }

    /// Perform the deletion transition `kind` names for `identity`, making it
    /// durable through `step`. The branch's critical section is held from
    /// the identity re-check through the cutover.
    pub(crate) fn delete_branch_with<S: RelationalBranchTransitionDurableStep>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        step: &S,
        identity: &RelationalBranchIdentity,
        kind: RelationalBranchDeletionKind,
    ) -> Result<RelationalBranchDeletionOutcome, Failure<S::Stop>> {
        let denied = BranchTransitionRecordError::Denied;
        require_local_identity(self, identity)
            .map_err(|()| denied(RelationalBranchDeleteDenial::ForeignRuntime))?;
        if identity.branch_id() == &self.config.history.main_branch {
            return Err(denied(RelationalBranchDeleteDenial::MainBranch));
        }
        let unknown = || {
            denied(RelationalBranchDeleteDenial::UnknownBranch(
                identity.branch_id().clone(),
            ))
        };
        let publication = self.history.begin_branch_transition();
        let publication_cell = self
            .history
            .branch_cell(identity.branch_id())
            .filter(|cell| cell.identity() == identity)
            .map(|cell| cell.publication_cell())
            .ok_or_else(unknown)?;
        let _coordination = publication_cell.coordination().enter();
        let _cell = self
            .history
            .branch_cell(identity.branch_id())
            .filter(|cell| cell.identity() == identity)
            .ok_or_else(unknown)?;
        let mut state = publication_cell.enter_state();
        let active_operation_count = publication_cell
            .head_retention()
            .binding()
            .map_err(|_| denied(RelationalBranchDeleteDenial::OwnerFailure))?
            .active_operation_count(identity)
            .map_err(|_| denied(RelationalBranchDeleteDenial::OwnerFailure))?;
        let kind = match kind {
            RelationalBranchDeletionKind::ByActiveOperations if active_operation_count > 0 => {
                if state.lifecycle_posture() == RelationalBranchLifecyclePosture::Deleting {
                    // Already requested and durable; nothing changes.
                    return Ok(waiting(identity, active_operation_count));
                }
                RelationalBranchDeletionKind::Requested
            }
            RelationalBranchDeletionKind::ByActiveOperations => {
                RelationalBranchDeletionKind::Delete
            }
            explicit => explicit,
        };
        let prior_generation = state.observation().generation();
        let prior_truth_version = state.truth_version();
        let branch = identity.branch_id().clone();
        if kind == RelationalBranchDeletionKind::Requested {
            let transition = RelationalBranchTransition::DeletionRequested {
                branch: branch.clone(),
                prior_generation,
                prior_truth_version,
            };
            publication.record_with_cutover(
                execution,
                step,
                || self.admit_name_retirement(&branch, transition),
                |_| {
                    self.history.retire_admitted_branch_name(branch.clone());
                    state.mark_deleting();
                },
            )?;
            return Ok(waiting(identity, active_operation_count));
        }
        self.delete_now(
            execution,
            step,
            identity,
            &publication_cell,
            &publication,
            &mut state,
            prior_generation,
            prior_truth_version,
        )
    }

    fn delete_now<S: RelationalBranchTransitionDurableStep>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        step: &S,
        identity: &RelationalBranchIdentity,
        publication_cell: &RelationalBranchPublicationCell,
        publication: &crate::runtime::BranchTransitionPublication<'_>,
        state: &mut crate::branch::reference_publication_cell::RelationalBranchPublicationStateGuard<'_>,
        prior_generation: worth_foundational::FoundationalBranchReferenceGeneration,
        prior_truth_version: crate::branch::RelationalBranchVersion,
    ) -> Result<RelationalBranchDeletionOutcome, Failure<S::Stop>> {
        let denied = BranchTransitionRecordError::Denied;
        let selected_root = state
            .selected_root()
            .ok_or(denied(RelationalBranchDeleteDenial::OwnerFailure))?;
        let previous_head_version = selected_root
            .canonical_envelope()
            .map(|envelope| envelope.commit.version_id);
        let head_retirement = self
            .history
            .reserve_branch_head_retirement(
                identity,
                &selected_root,
                publication_cell.head_retention(),
            )
            .map_err(|denial| denied(head_retirement_denial(denial)))?;
        let branch = identity.branch_id().clone();
        let transition = RelationalBranchTransition::Delete {
            branch: branch.clone(),
            prior_generation,
            prior_truth_version,
            head_commit: selected_root.commit_id(),
        };
        let retired_root_identity = publication.record_with_cutover(
            execution,
            step,
            || self.admit_name_retirement(&branch, transition),
            |_| {
                self.history.retire_admitted_branch_name(branch.clone());
                state.mark_deleting();
                let removed_cell = self
                    .history
                    .remove_branch_cell(&branch)
                    .expect("coordinated branch cell remains registered until deletion cutover");
                assert_eq!(removed_cell.identity(), identity);
                // Coordination and state have been held since selection; the
                // removed cell still selects this exact prepared root.
                let retired_root_identity = head_retirement.retire_head(selected_root);
                self.history
                    .move_branch_head_version(previous_head_version, None);
                self.visibility_pins()
                    .move_branch_head_visibility_residency(
                        &branch,
                        previous_head_version,
                        None,
                        None,
                    );
                retired_root_identity
            },
        )?;
        Ok(RelationalBranchDeletionOutcome::Deleted(
            DeletedRelationalBranch {
                identity: identity.clone(),
                retired_root_identity,
            },
        ))
    }

    /// Inside the reservation, where only transitions retire names: refuse a
    /// deletion whose name the bounded retired set cannot hold.
    fn admit_name_retirement(
        &self,
        branch: &crate::history::data::BranchId,
        transition: RelationalBranchTransition,
    ) -> Result<RelationalBranchTransition, RelationalBranchDeleteDenial> {
        self.history
            .admits_branch_name_retirement(branch)
            .then_some(transition)
            .ok_or(RelationalBranchDeleteDenial::RetiredIdentityCapacityExhausted)
    }
}

fn waiting(
    identity: &RelationalBranchIdentity,
    active_operation_count: u64,
) -> RelationalBranchDeletionOutcome {
    RelationalBranchDeletionOutcome::WaitingForActiveOperations(RelationalBranchDeletionPending {
        identity: identity.clone(),
        active_operation_count,
    })
}

fn head_retirement_denial(
    denial: RelationalRetentionAcquisitionDenial,
) -> RelationalBranchDeleteDenial {
    match denial {
        RelationalRetentionAcquisitionDenial::CapacityExhausted => {
            RelationalBranchDeleteDenial::RetentionBackpressure
        }
        RelationalRetentionAcquisitionDenial::IdentityExhausted => {
            RelationalBranchDeleteDenial::RetentionIdentityExhausted
        }
        _ => RelationalBranchDeleteDenial::OwnerFailure,
    }
}
