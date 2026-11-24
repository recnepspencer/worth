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
        let classification = self
            .history()
            .classify_commit_in_ancestry(&ancestry, requested);
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
