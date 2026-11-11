//! Original performed branch transitions share the exact chronology fold.
use super::*;
impl<'a> Fold<'a> {
    pub(super) fn transition(
        &mut self,
        main: &BranchId,
        entry: &'a TransitionOriginalRecord,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Refusal> {
        if matches!(&entry.transition, Transition::RetainedFork { .. }) {
            return Err(Refusal::Native(crate::durability::data::DurabilityError::new(
                crate::durability::data::RecoveryFailureClass::UnsupportedRetainedForkReplay,
                "retained-source forks require original live root custody; historical tail reconstruction is unsupported",
            ).into()));
        }
        if let Transition::Fork {
            target,
            source,
            source_generation,
            source_truth_version,
            source_head,
        } = &entry.transition
        {
            let mut state = self
                .current
                .get(&source)
                .cloned()
                .ok_or(Refusal::Malformed)?;
            if self.current.get(&target).is_some()
                || state.removed
                || state.lifecycle != Lifecycle::Live
                || state.generation != *source_generation
                || state.truth != *source_truth_version
                || !state.matches_basis(source_head)
                || state.schema != entry.schema
            {
                return Err(Refusal::Malformed);
            }
            state.generation = worth_foundational::FoundationalBranchReferenceGeneration::initial();
            state.truth = crate::branch::RelationalBranchVersion::initial();
            return self.record(target, state, request);
        }
        let branch = entry.transition.branch();
        let mut state = self
            .current
            .get(&branch)
            .cloned()
            .ok_or(Refusal::Malformed)?;
        let (generation, truth) = match &entry.transition {
            Transition::Archive {
                prior_generation,
                prior_truth_version,
                ..
            }
            | Transition::DeletionRequested {
                prior_generation,
                prior_truth_version,
                ..
            }
            | Transition::Delete {
                prior_generation,
                prior_truth_version,
                ..
            } => (*prior_generation, *prior_truth_version),
            Transition::Fork { .. } | Transition::RetainedFork { .. } => {
                unreachable!("fork is handled above")
            }
        };
        if state.removed
            || state.generation != generation
            || state.truth != truth
            || state.schema != entry.schema
        {
            return Err(Refusal::Malformed);
        }
        match &entry.transition {
            Transition::Archive { .. } => {
                if state.lifecycle != Lifecycle::Live {
                    return Err(Refusal::Malformed);
                }
                state.generation = state
                    .generation
                    .checked_advance()
                    .map_err(|_| Refusal::Malformed)?;
                state.lifecycle = Lifecycle::Archived;
            }
            Transition::DeletionRequested { .. } => {
                if branch == main {
                    return Err(Refusal::Malformed);
                }
                state.lifecycle = Lifecycle::Deleting;
            }
            Transition::Delete { head_commit, .. } => {
                if branch == main
                    || state.commit.map(|index| index.commit.commit_id) != *head_commit
                {
                    return Err(Refusal::Malformed);
                }
                state.lifecycle = Lifecycle::Deleting;
                state.removed = true;
            }
            Transition::Fork { .. } | Transition::RetainedFork { .. } => {
                unreachable!("fork is handled above")
            }
        }
        self.record(branch, state, request)
    }
}
