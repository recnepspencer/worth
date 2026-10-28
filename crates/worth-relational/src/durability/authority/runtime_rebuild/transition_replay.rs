//! Replay of one durable branch transition during recovery (D8).
//!
//! A replayed transition is performed by the same fork, archive and delete
//! operations a live caller uses, against the recovering runtime's own
//! registry, with a durable step that requires the operation to reproduce
//! exactly the recorded transition. So a fork applies only when its target is
//! absent and not retired and its source is live with the recorded
//! generation, truth version and head; an archive or delete applies only to
//! the recorded prior generation and truth version. Anything else refuses
//! before any effect.

use std::fmt::Debug;

use crate::branch::{
    PositionedBranchTransition, RelationalBranchDeletionKind, RelationalBranchTransition,
    ReplayedBranchTransitionDivergence, ReplayedBranchTransitionStep,
};
use crate::durability::data::{DurabilityError, RecoveryFailureClass, RelationalRecoveryStop};
use crate::runtime::{BranchTransitionRecordError, RelationalRuntime};

use super::root_inventory::RecoveredRootInventory;

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
        RelationalBranchTransition::Fork { target, source, .. } => {
            restored
                .fork_port()
                .fork_branch_with(execution, &step, target.clone(), source)
                .map_err(|failure| replay_stop(recorded, failure))?;
            recovered_roots.retain_current(restored, target)?;
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
) -> Result<crate::branch::RelationalBranchIdentity, RelationalRecoveryStop> {
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

/// A replay that did not apply is a recovery refusal; a stopped request
/// stays a request stop.
fn replay_stop<D: Debug>(
    recorded: &PositionedBranchTransition,
    failure: BranchTransitionRecordError<D, ReplayedBranchTransitionDivergence>,
) -> RelationalRecoveryStop {
    match failure {
        BranchTransitionRecordError::ReservationWaitStopped(stop) => {
            RelationalRecoveryStop::Request(crate::execution::RelationalExecutionDenialCause::from(
                stop,
            ))
        }
        BranchTransitionRecordError::Denied(denial) => {
            refused(recorded, format!("does not apply: {denial:?}"))
        }
        BranchTransitionRecordError::SequenceCapacityExhausted => {
            refused(recorded, "exhausts the transition sequence".to_owned())
        }
        BranchTransitionRecordError::Step(divergence) => refused(
            recorded,
            format!(
                "would replay as {:?} rather than as recorded",
                divergence.performed
            ),
        ),
    }
}

fn refused(recorded: &PositionedBranchTransition, detail: String) -> RelationalRecoveryStop {
    DurabilityError::new(
        RecoveryFailureClass::ReplayFailure,
        format!(
            "durable branch transition {} ({:?}) {detail}",
            recorded.sequence().0,
            recorded.transition()
        ),
    )
    .into()
}
