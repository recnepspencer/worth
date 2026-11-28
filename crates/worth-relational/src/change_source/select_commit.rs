//! Runtime admission and observation accounting for exact commit selection.
use super::commit_selection::*;
use crate::history::{data::CommitId, CommitAncestryPosture};
use crate::mvcc::observation::RelationalBranchObservation;
use crate::runtime::RelationalRuntime;
use worth_relational_branch::internal_port::change_source::commit_selection::assemble_commit_selection;
use worth_relational_branch::internal_port::change_source::commit_selection::assemble_commit_selection_work;
use worth_relational_branch::internal_port::change_source::commit_selection::assemble_selected_commit;

impl RelationalRuntime {
    /// Select `requested` if it is the observation's commit or one of its
    /// ancestors. The cost is one ancestry inspection from the selected
    /// commit.
    pub fn select_reachable_commit(
        &self,
        observation: &RelationalBranchObservation,
        requested: CommitId,
    ) -> RelationalCommitSelection {
        let selected = match self.observed_commit(observation) {
            Ok(selected) => selected,
            Err(denial) => return unselected(denial),
        };
        let ancestry = self.history().inspect_commit_ancestry(selected);
        self.classify_selected_commit(observation, selected, requested, &ancestry)
    }

    pub(super) fn select_commit_for_pinned_read(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        observation: &RelationalBranchObservation,
        requested: CommitId,
    ) -> Result<RelationalSelectedCommit, super::RelationalPinnedCommitDenial> {
        use super::RelationalPinnedCommitDenial as Denial;

        let selected = self
            .observed_commit(observation)
            .map_err(Denial::Selection)?;
        let (selection, _) = execution
            .run_native_with_checkpoint(|context| {
                context.checkpoint(1)?;
                if selected == requested {
                    return Ok(self.select_exact_commit(observation, requested));
                }
                let history = self.history();
                let ancestry =
                    crate::history::access::ancestry_walk::inspect_commit_ancestry_with_checkpoint(
                        selected,
                        |id| history.immutable_commit_receipt(id),
                        |units| context.checkpoint(units),
                    )?;
                context.checkpoint(1)?;
                Ok(self.classify_selected_commit(observation, selected, requested, &ancestry))
            })
            .map_err(|error| Denial::Execution(error.into()))?;
        selection
            .map_err(|error: worth_execution::MapKernelStop| Denial::Execution(error.into()))?
            .into_outcome()
            .map_err(Denial::Selection)
    }

    fn classify_selected_commit(
        &self,
        observation: &RelationalBranchObservation,
        selected: CommitId,
        requested: CommitId,
        ancestry: &crate::history::access::ancestry_walk::CommitAncestryInspection,
    ) -> RelationalCommitSelection {
        let classification = self
            .history()
            .classify_commit_in_ancestry(ancestry, requested);
        let work = self.count_commit_selection(classification.traversal_work());
        let outcome = match classification.posture() {
            CommitAncestryPosture::SelectedCommitUnavailable => {
                Err(RelationalCommitSelectionDenial::SelectedCommitUnavailable { selected })
            }
            CommitAncestryPosture::RequestedCommitUnavailable => {
                Err(RelationalCommitSelectionDenial::RequestedCommitUnavailable { requested })
            }
            CommitAncestryPosture::Unreachable => {
                Err(RelationalCommitSelectionDenial::Unreachable {
                    selected,
                    requested,
                })
            }
            CommitAncestryPosture::Reachable => Ok(self.selected(requested, observation)),
        };
        assemble_commit_selection(outcome, work)
    }

    /// Select `requested` only if it is exactly the observation's commit.
    /// The cost is constant.
    pub fn select_exact_commit(
        &self,
        observation: &RelationalBranchObservation,
        requested: CommitId,
    ) -> RelationalCommitSelection {
        let selected = match self.observed_commit(observation) {
            Ok(selected) => selected,
            Err(denial) => return unselected(denial),
        };
        let work = self.count_commit_selection(0);
        let outcome = if selected == requested {
            Ok(self.selected(requested, observation))
        } else {
            Err(RelationalCommitSelectionDenial::NotSelectedCommit {
                selected,
                requested,
            })
        };
        assemble_commit_selection(outcome, work)
    }

    fn observed_commit(
        &self,
        observation: &RelationalBranchObservation,
    ) -> Result<CommitId, RelationalCommitSelectionDenial> {
        self.admit_observation_read(observation).map_err(
            |super::observation_read::ForeignObservation| {
                RelationalCommitSelectionDenial::ForeignObservation
            },
        )?;
        observation
            .commit_id()
            .ok_or(RelationalCommitSelectionDenial::NoSelectedCommit)
    }

    fn selected(
        &self,
        commit_id: CommitId,
        observation: &RelationalBranchObservation,
    ) -> RelationalSelectedCommit {
        assemble_selected_commit(self.runtime_instance_id(), commit_id, observation.clone())
    }

    fn count_commit_selection(&self, ancestry_visits: usize) -> RelationalCommitSelectionWork {
        self.performance_access()
            .count_observation_commit_selection(ancestry_visits);
        assemble_commit_selection_work(1, ancestry_visits)
    }
}
