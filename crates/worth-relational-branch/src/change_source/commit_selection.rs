use crate::history::data::CommitId;
use crate::mvcc::observation::RelationalBranchObservation;

/// One commit that a retained observation can see, selected by the runtime
/// that issued the observation.
///
/// Only [`crate::runtime::RelationalRuntime::select_reachable_commit`] and
/// [`crate::runtime::RelationalRuntime::select_exact_commit`] mint it, so a raw [`CommitId`]
/// or a copied observation cannot stand in for a selection.
#[derive(Debug)]
pub struct RelationalSelectedCommit {
    pub(super) runtime_instance_id: u64,
    pub(super) commit_id: CommitId,
    pub(super) observation: RelationalBranchObservation,
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
    pub(super) selections: usize,
    pub(super) ancestry_visits: usize,
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
    pub(super) outcome: Result<RelationalSelectedCommit, RelationalCommitSelectionDenial>,
    pub(super) work: RelationalCommitSelectionWork,
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

pub fn unselected(denial: RelationalCommitSelectionDenial) -> RelationalCommitSelection {
    RelationalCommitSelection {
        outcome: Err(denial),
        work: RelationalCommitSelectionWork::default(),
    }
}

pub fn assemble_commit_selection(
    outcome: Result<RelationalSelectedCommit, RelationalCommitSelectionDenial>,
    work: RelationalCommitSelectionWork,
) -> RelationalCommitSelection {
    RelationalCommitSelection { outcome, work }
}

pub fn assemble_selected_commit(
    runtime_instance_id: u64,
    commit_id: CommitId,
    observation: RelationalBranchObservation,
) -> RelationalSelectedCommit {
    RelationalSelectedCommit {
        runtime_instance_id,
        commit_id,
        observation,
    }
}

pub fn assemble_commit_selection_work(
    selections: usize,
    ancestry_visits: usize,
) -> RelationalCommitSelectionWork {
    RelationalCommitSelectionWork {
        selections,
        ancestry_visits,
    }
}
