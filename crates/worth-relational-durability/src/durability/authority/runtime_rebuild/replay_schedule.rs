//! Fix the commit order and transition position policy before mutating the replay target.
use std::collections::{BTreeSet, VecDeque};

use crate::durability::data::RecoveryPlan;
use crate::durability::migration::ReadmittedCanonicalCommit;
use worth_relational_branch::internal_port::branch::transition::positioned::PositionedBranchTransition;
use worth_relational_branch::internal_port::runtime::state::subsystems::history::HistorySubsystem;
use worth_relational_contracts::internal_port::history::data::coordinates::CommitId;

pub struct RecoveryReplaySchedule {
    pub commits: Vec<ReadmittedCanonicalCommit>,
    pub transitions: VecDeque<PositionedBranchTransition>,
    pub available_commit_ids: BTreeSet<CommitId>,
    pub positioned_exactly: bool,
}

impl RecoveryReplaySchedule {
    pub fn prepare(plan: &RecoveryPlan, history: &HistorySubsystem) -> Self {
        let mut commits = plan.tail_log.clone();
        commits.sort_by_key(|commit| commit.position());
        // Resident commits and the tail; parents inside the cold checkpoint
        // base are answered by the history on demand (`G7`).
        let available_commit_ids = history
            .resident_commit_artifacts()
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
