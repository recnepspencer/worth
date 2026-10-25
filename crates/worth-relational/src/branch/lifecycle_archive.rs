use super::{require_local_identity, ArchivedRelationalBranch, RelationalBranchArchiveDenial};
use crate::branch::{
    LiveBranchTransitionStep, RelationalBranchIdentity, RelationalBranchLifecyclePosture,
    RelationalBranchTransition, RelationalBranchTransitionDurableStep,
    RelationalBranchTransitionStop,
};
use crate::runtime::BranchTransitionRecordError;

impl crate::runtime::RelationalRuntime {
    /// Archive a live branch: it stops accepting commits and its generation
    /// advances. The archive is durable before it is live; a refusal or an
    /// unresolved fate leaves the branch exactly as it was.
    pub fn archive_branch(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        identity: &RelationalBranchIdentity,
    ) -> Result<
        ArchivedRelationalBranch,
        RelationalBranchTransitionStop<RelationalBranchArchiveDenial>,
    > {
        let step = LiveBranchTransitionStep::new(self.durability.medium(), execution);
        self.archive_branch_with(execution, &step, identity)
            .map_err(Into::into)
    }

    /// Archive `identity`, making the transition durable through `step`.
    ///
    /// The branch's critical section and publication state are held from the
    /// posture check through the cutover, so no commit on the branch can land
    /// between them. Cost: one reservation wait, the durable step and O(1)
    /// state changes.
    pub(crate) fn archive_branch_with<S: RelationalBranchTransitionDurableStep>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        step: &S,
        identity: &RelationalBranchIdentity,
    ) -> Result<
        ArchivedRelationalBranch,
        BranchTransitionRecordError<RelationalBranchArchiveDenial, S::Stop>,
    > {
        let denied = BranchTransitionRecordError::Denied;
        require_local_identity(self, identity)
            .map_err(|()| denied(RelationalBranchArchiveDenial::ForeignRuntime))?;
        let publication_cell = self
            .history
            .branch_cell(identity.branch_id())
            .filter(|cell| cell.identity() == identity)
            .map(|cell| cell.publication_cell())
            .ok_or_else(|| {
                denied(RelationalBranchArchiveDenial::UnknownBranch(
                    identity.branch_id().clone(),
                ))
            })?;
        let _coordination = publication_cell.coordination().enter();
        let mut state = publication_cell.enter_state();
        match state.lifecycle_posture() {
            RelationalBranchLifecyclePosture::Live => {}
            RelationalBranchLifecyclePosture::Archived => {
                return Err(denied(RelationalBranchArchiveDenial::AlreadyArchived(
                    identity.branch_id().clone(),
                )));
            }
            RelationalBranchLifecyclePosture::Deleting => {
                return Err(denied(RelationalBranchArchiveDenial::Deleting(
                    identity.branch_id().clone(),
                )));
            }
        }
        let archived = state
            .archived_observation()
            .map_err(|_| denied(RelationalBranchArchiveDenial::GenerationOverflow))?;
        let transition = RelationalBranchTransition::Archive {
            branch: identity.branch_id().clone(),
            prior_generation: state.observation().generation(),
            prior_truth_version: state.truth_version(),
        };
        self.history.record_branch_transition_with_cutover(
            execution,
            step,
            || Ok(transition),
            |_| state.install_archive(archived.clone()),
        )?;
        Ok(ArchivedRelationalBranch {
            identity: identity.clone(),
            observation: archived,
        })
    }
}
