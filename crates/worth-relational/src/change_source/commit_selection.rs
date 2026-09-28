use crate::history::data::CommitId;
use crate::history::CommitAncestryPosture;
use crate::mvcc::RelationalBranchObservation;
use crate::runtime::RelationalRuntime;

/// One commit that a retained observation can see, selected by the runtime
/// that issued the observation.
///
/// Only [`RelationalRuntime::select_reachable_commit`] and
/// [`RelationalRuntime::select_exact_commit`] mint it, so a raw [`CommitId`]
/// or a copied observation cannot stand in for a selection.
#[derive(Debug)]
pub struct RelationalSelectedCommit {
    runtime_instance_id: u64,
    commit_id: CommitId,
    observation: RelationalBranchObservation,
}

impl RelationalSelectedCommit {
    /// The selected commit.
    pub fn commit_id(&self) -> CommitId {
        self.commit_id
    }

    /// The retained observation the commit was selected at.
    pub fn observation(&self) -> &RelationalBranchObservation {
        &self.observation
    }

    /// The runtime that made the selection.
    pub fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }
}

/// Why an observation cannot select a requested commit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationalCommitSelectionDenial {
    /// Another runtime issued the observation.
    ForeignObservation,
    /// The observation has no committed root to select from.
    NoSelectedCommit,
    /// The observation's own commit is no longer retained.
    SelectedCommitUnavailable { selected: CommitId },
    /// The requested commit does not exist or is no longer retained.
    RequestedCommitUnavailable { requested: CommitId },
    /// The requested commit exists but is not an ancestor of the selected one.
    Unreachable {
        selected: CommitId,
        requested: CommitId,
    },
    /// An exact selection asked for a commit other than the selected one.
    NotSelectedCommit {
        selected: CommitId,
        requested: CommitId,
    },
}

/// The work one commit selection did.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RelationalCommitSelectionWork {
    selections: usize,
    ancestry_visits: usize,
}

impl RelationalCommitSelectionWork {
    /// Selections that reached the runtime: `1`, or `0` when the observation
    /// was foreign or had no committed root.
    pub const fn selections(&self) -> usize {
        self.selections
    }

    /// Ancestry nodes, catalog probes, and parent edges visited. An exact
    /// selection visits none.
    pub const fn ancestry_visits(&self) -> usize {
        self.ancestry_visits
    }
}

/// A commit selection together with the work it did.
#[derive(Debug)]
#[must_use]
pub struct RelationalCommitSelection {
    outcome: Result<RelationalSelectedCommit, RelationalCommitSelectionDenial>,
    work: RelationalCommitSelectionWork,
}

impl RelationalCommitSelection {
    /// The work the selection did, whatever its outcome.
    pub const fn work(&self) -> RelationalCommitSelectionWork {
        self.work
    }

    /// The selected commit, or why it could not be selected.
    pub fn into_outcome(self) -> Result<RelationalSelectedCommit, RelationalCommitSelectionDenial> {
        self.outcome
    }
}

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
            CommitAncestryPosture::Unreachable => Err(RelationalCommitSelectionDenial::Unreachable {
                selected,
                requested,
            }),
            CommitAncestryPosture::Reachable => Ok(self.selected(requested, observation)),
        };
        RelationalCommitSelection { outcome, work }
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
        RelationalCommitSelection { outcome, work }
    }

    fn observed_commit(
        &self,
        observation: &RelationalBranchObservation,
    ) -> Result<CommitId, RelationalCommitSelectionDenial> {
        if observation.identity().runtime_instance_id() != self.runtime_instance_id() {
            return Err(RelationalCommitSelectionDenial::ForeignObservation);
        }
        observation
            .commit_id()
            .ok_or(RelationalCommitSelectionDenial::NoSelectedCommit)
    }

    fn selected(
        &self,
        commit_id: CommitId,
        observation: &RelationalBranchObservation,
    ) -> RelationalSelectedCommit {
        RelationalSelectedCommit {
            runtime_instance_id: self.runtime_instance_id(),
            commit_id,
            observation: observation.clone(),
        }
    }

    fn count_commit_selection(&self, ancestry_visits: usize) -> RelationalCommitSelectionWork {
        self.performance_access()
            .count_observation_commit_selection(ancestry_visits);
        RelationalCommitSelectionWork {
            selections: 1,
            ancestry_visits,
        }
    }
}

fn unselected(denial: RelationalCommitSelectionDenial) -> RelationalCommitSelection {
    RelationalCommitSelection {
        outcome: Err(denial),
        work: RelationalCommitSelectionWork::default(),
    }
}
