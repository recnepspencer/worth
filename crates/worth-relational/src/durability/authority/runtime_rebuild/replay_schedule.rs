//! Fix the commit order and transition position policy before mutating the replay target.
use std::collections::{BTreeSet, VecDeque};

use crate::branch::transition::PositionedBranchTransition;
use crate::durability::data::RecoveryPlan;
use crate::durability::migration::ReadmittedCanonicalCommit;
use crate::history::data::CommitId;
use crate::runtime::state::subsystems::history::HistorySubsystem;

pub(super) struct RecoveryReplaySchedule {
    pub(super) commits: Vec<ReadmittedCanonicalCommit>,
    pub(super) transitions: VecDeque<PositionedBranchTransition>,
    pub(super) available_commit_ids: BTreeSet<CommitId>,
    pub(super) positioned_exactly: bool,
}

impl RecoveryReplaySchedule {
    pub(super) fn prepare(plan: &RecoveryPlan, history: &HistorySubsystem) -> Self {
        let mut commits = plan.tail_log.clone();
        commits.sort_by_key(|commit| commit.position());
        let available_commit_ids = history
            .commit_artifacts()
            .into_iter()
            .map(|artifact| artifact.commit_id())
            .chain(
                commits
                    .iter()
                    .map(|commit| commit.envelope().commit.commit_id),
            )
            .collect();
        // A transition performed after commit p and before p + 1 is applied
        // before replay of commit p + 1.
        Self {
            commits,
            transitions: plan
                .branch_transitions
                .transitions()
                .iter()
                .cloned()
                .collect(),
            available_commit_ids,
            positioned_exactly: plan.branch_transitions.replays_positions_exactly(),
        }
    }
}
