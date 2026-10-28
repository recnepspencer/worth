use std::collections::BTreeMap;

use crate::facade::history::BranchId;
use crate::facade::replay::{
    RelationalReplayOutcome, RelationalReplayRequest, ReplayExecutionMode, ReplayVerificationMode,
};
use crate::facade::runtime::RelationalRuntime;
use crate::runtime::RecoveryOutcome;
use worth_foundational::FoundationalBranchTarget;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ReplayProbe {
    pub(crate) branch_name: String,
    pub(crate) commit_id: Option<u64>,
    pub(crate) mismatch_count: usize,
    pub(crate) failure: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RecoveryProbe {
    pub(crate) latest_commit_id: Option<u64>,
    pub(crate) branch_heads: BTreeMap<String, u64>,
    pub(crate) recovered_commits: usize,
    pub(crate) checkpoint_covered_position: Option<u64>,
}

pub(crate) fn capture_replay_probe(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut super::super::fixture::FintechWorld,
    branch_id: BranchId,
) -> ReplayProbe {
    let latest = world
        .runtime
        .history()
        .latest_commit()
        .map(|commit| commit.commit_id);
    let replay = latest.map(|commit_id| {
        world
            .runtime
            .replay_authority()
            .replay_commit(
                RelationalReplayRequest {
                    commit_id,
                    branch_id: branch_id.clone(),
                    execution_mode: ReplayExecutionMode::SerialDeterministic,
                    verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
                },
                execution,
            )
            .unwrap()
    });
    replay_probe_from_outcome(branch_id.0.clone(), replay.as_ref())
}

pub(crate) fn replay_probe_from_outcome(
    branch_name: String,
    replay: Option<&RelationalReplayOutcome>,
) -> ReplayProbe {
    ReplayProbe {
        branch_name,
        commit_id: replay
            .and_then(|outcome| outcome.commit.as_ref().map(|commit| commit.commit_id.0)),
        mismatch_count: replay
            .map(|outcome| outcome.mismatches.len())
            .unwrap_or_default(),
        failure: replay.and_then(|outcome| {
            outcome
                .failure
                .as_ref()
                .map(|failure| format!("{failure:?}"))
        }),
    }
}

pub(crate) fn capture_recovery_probe(
    runtime: &RelationalRuntime,
    outcome: &RecoveryOutcome,
) -> RecoveryProbe {
    RecoveryProbe {
        latest_commit_id: runtime
            .history()
            .latest_commit()
            .map(|commit| commit.commit_id.0),
        branch_heads: runtime
            .history()
            .branch_cells_snapshot()
            .into_iter()
            .filter_map(|cell| {
                let FoundationalBranchTarget::Basis(target) = cell.observation.target() else {
                    return None;
                };
                Some((cell.branch_id.0, target.selected_commit_id()))
            })
            .collect(),
        recovered_commits: outcome.recovered_commits,
        checkpoint_covered_position: outcome
            .coverage
            .checkpoint_covered_position
            .map(|position| position.0),
    }
}
