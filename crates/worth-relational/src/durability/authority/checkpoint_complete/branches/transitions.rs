//! Fold performed branch transitions at their original canonical position.
use super::*;

impl<'a> BranchImages<'a> {
    pub(super) fn transition(
        &mut self,
        main: &BranchId,
        entry: &'a RetainedBranchTransition,
        roots: &FoldedRoots,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let record = entry.positioned.transition();
        if matches!(record, RelationalBranchTransition::RetainedFork { .. }) {
            return Err(Denial::Durability(crate::durability::data::DurabilityError::new(
                crate::durability::data::RecoveryFailureClass::UnsupportedRetainedForkReplay,
                "retained-source forks require original live root custody; historical tail reconstruction is unsupported",
            )));
        }

        if let RelationalBranchTransition::Fork {
            target,
            source,
            source_generation,
            source_truth_version,
            source_head,
        } = record
        {
            let prior = self
                .current
                .get(&source)
                .ok_or(Denial::OriginalAuthorityUnavailable)?;
            if self.current.get(&target).is_some()
                || prior.removed
                || prior.lifecycle != Lifecycle::Live
                || (prior.generation != *source_generation
                    || prior.truth != *source_truth_version
                    || !prior.matches_basis(source_head, roots))
                || !Arc::ptr_eq(&prior.schema, &entry.schema)
            {
                return Err(Denial::OriginalAuthorityUnavailable);
            }
            let mut next = prior.clone();
            next.generation = Generation::initial();
            next.truth = RelationalBranchVersion::initial();
            self.record(target, next, execution)?;
            return Ok(());
        }
        let branch = record.branch();
        let mut prior = self
            .current
            .get(&branch)
            .cloned()
            .ok_or(Denial::OriginalAuthorityUnavailable)?;
        let (generation, truth) = match record {
            RelationalBranchTransition::Archive {
                prior_generation,
                prior_truth_version,
                ..
            }
            | RelationalBranchTransition::DeletionRequested {
                prior_generation,
                prior_truth_version,
                ..
            }
            | RelationalBranchTransition::Delete {
                prior_generation,
                prior_truth_version,
                ..
            } => (*prior_generation, *prior_truth_version),
            RelationalBranchTransition::Fork { .. }
            | RelationalBranchTransition::RetainedFork { .. } => {
                unreachable!("fork is handled above")
            }
        };
        if prior.removed
            || prior.generation != generation
            || prior.truth != truth
            || !Arc::ptr_eq(&prior.schema, &entry.schema)
        {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
        match record {
            RelationalBranchTransition::Archive { .. } => {
                if prior.lifecycle != Lifecycle::Live {
                    return Err(Denial::OriginalAuthorityUnavailable);
                }
                prior.generation = prior
                    .generation
                    .checked_advance()
                    .map_err(|_| Denial::OriginalAuthorityUnavailable)?;
                prior.lifecycle = Lifecycle::Archived;
            }
            RelationalBranchTransition::DeletionRequested { .. } => {
                if branch == main {
                    return Err(Denial::OriginalAuthorityUnavailable);
                }
                prior.lifecycle = Lifecycle::Deleting;
            }
            RelationalBranchTransition::Delete { head_commit, .. } => {
                if branch == main || prior.commit != *head_commit {
                    return Err(Denial::OriginalAuthorityUnavailable);
                }
                prior.lifecycle = Lifecycle::Deleting;
                prior.removed = true;
            }
            RelationalBranchTransition::Fork { .. }
            | RelationalBranchTransition::RetainedFork { .. } => {
                unreachable!("fork is handled above")
            }
        }
        self.record(branch, prior, execution)
    }
}
