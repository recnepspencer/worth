//! Replay of one durable branch transition during recovery (D8).
//!
//! A replayed transition is performed by the same fork, archive and delete
//! operations a live caller uses, against the recovering runtime's own
//! registry, with a durable step that requires the operation to reproduce
//! exactly the recorded transition. So a fork applies only when its target is
//! absent and not retired and its source is live with the recorded
//! generation, truth version and head; a retained fork applies only when its
//! recorded historical head resolves to the exact root recovery rebuilt; an
//! archive or delete applies only to the recorded prior generation and truth
//! version. Anything else refuses before any effect.

use crate::branch::fork::recovered_retained::RecordedRetainedForkSource;
use crate::branch::lifecycle::RelationalBranchDeletionKind;
use crate::branch::transition::{
    PositionedBranchTransition, RelationalBranchTransition, ReplayedBranchTransitionStep,
};
use crate::durability::data::RelationalRecoveryStop;
use crate::runtime::RelationalRuntime;

use super::root_inventory::RecoveredRootInventory;
pub(crate) use worth_relational_durability::internal_port::durability::authority::runtime_rebuild::transition_replay::refusal;
use refusal::{refused, replay_stop};

/// Apply `recorded` to `restored`.
///
/// `positioned_exactly` says whether the replay must also reproduce the
/// recorded sequence and followed position: true for a contiguous durable
/// tail, false for a selection a partial replay chain made.
///
/// Cost: the operation's own cost; O(1) beyond it.
pub(super) fn replay_branch_transition(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    restored: &mut RelationalRuntime,
    recorded: &PositionedBranchTransition,
    positioned_exactly: bool,
    recovered_roots: &mut RecoveredRootInventory,
) -> Result<(), RelationalRecoveryStop> {
    let step = if positioned_exactly {
        ReplayedBranchTransitionStep::positioned(recorded)
    } else {
        ReplayedBranchTransitionStep::transition_only(recorded)
    };
    match recorded.transition() {
        RelationalBranchTransition::RetainedFork {
            target,
            source,
            source_generation,
            source_truth_version,
            source_head,
        } => {
            let commit = crate::history::data::CommitId(source_head.selected_commit_id());
            let root = recovered_roots
                .retained_fork_source(
                    execution,
                    &restored.history,
                    &restored.config.schema.registry,
                    &restored.services.symbols,
                    commit,
                )?
                .ok_or_else(|| {
                    refused(
                        recorded,
                        format!(
                            "retained fork source commit {} is not retained by recorded history",
                            commit.0
                        ),
                    )
                })?;
            restored
                .fork_port()
                .fork_recovered_retained_with(
                    execution,
                    &step,
                    target.clone(),
                    RecordedRetainedForkSource {
                        branch: source,
                        generation: *source_generation,
                        truth_version: *source_truth_version,
                        head: source_head,
                        root,
                    },
                )
                .map_err(|failure| replay_stop(recorded, failure))?;
            recovered_roots.retain_current(
                &restored.history,
                &restored.config.schema.registry,
                target,
            )?;
        }
        RelationalBranchTransition::Fork { target, source, .. } => {
            restored
                .fork_port()
                .fork_branch_with(execution, &step, target.clone(), source)
                .map_err(|failure| replay_stop(recorded, failure))?;
            recovered_roots.retain_current(
                &restored.history,
                &restored.config.schema.registry,
                target,
            )?;
        }
        RelationalBranchTransition::Archive { branch, .. } => {
            let identity = recorded_branch_identity(restored, recorded, branch)?;
            restored
                .archive_branch_with(execution, &step, &identity)
                .map_err(|failure| replay_stop(recorded, failure))?;
        }
        RelationalBranchTransition::DeletionRequested { branch, .. } => {
            let identity = recorded_branch_identity(restored, recorded, branch)?;
            restored
                .delete_branch_with(
                    execution,
                    &step,
                    &identity,
                    RelationalBranchDeletionKind::Requested,
                )
                .map_err(|failure| replay_stop(recorded, failure))?;
        }
        RelationalBranchTransition::Delete { branch, .. } => {
            let identity = recorded_branch_identity(restored, recorded, branch)?;
            restored
                .delete_branch_with(
                    execution,
                    &step,
                    &identity,
                    RelationalBranchDeletionKind::Delete,
                )
                .map_err(|failure| replay_stop(recorded, failure))?;
        }
    }
    Ok(())
}

fn recorded_branch_identity(
    restored: &RelationalRuntime,
    recorded: &PositionedBranchTransition,
    branch: &crate::history::data::BranchId,
) -> Result<crate::branch::identity::RelationalBranchIdentity, RelationalRecoveryStop> {
    restored.branch_identity(branch).map_err(|denial| {
        refused(
            recorded,
            format!(
                "names branch `{}`, which recovery does not hold: {denial:?}",
                branch.0
            ),
        )
    })
}
