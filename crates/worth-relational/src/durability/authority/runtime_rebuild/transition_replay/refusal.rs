//! Preserve typed transition failures and request stops.
use crate::branch::transition::{
    BranchTransitionRecordError, PositionedBranchTransition, ReplayedBranchTransitionDivergence,
};
use crate::durability::data::{DurabilityError, RecoveryFailureClass, RelationalRecoveryStop};
use std::fmt::Debug;

/// A replay that did not apply is a recovery refusal; a stopped request
/// stays a request stop.
pub(super) fn replay_stop<D: Debug>(
    recorded: &PositionedBranchTransition,
    failure: BranchTransitionRecordError<D, ReplayedBranchTransitionDivergence>,
) -> RelationalRecoveryStop {
    match failure {
        BranchTransitionRecordError::OriginalInputsStopped(stop) => stop,
        BranchTransitionRecordError::ReservationWaitStopped(stop) => {
            RelationalRecoveryStop::Request(crate::execution::RelationalExecutionDenialCause::from(
                stop,
            ))
        }
        BranchTransitionRecordError::Denied(denial) => {
            refused(recorded, format!("does not apply: {denial:?}"))
        }
        BranchTransitionRecordError::OriginalPrefixMismatch => refused(
            recorded,
            "branch transition original prefix mismatch".to_owned(),
        ),
        BranchTransitionRecordError::StorageCapacityExhausted => refused(
            recorded,
            "branch transition storage capacity exhausted".to_owned(),
        ),
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

pub(super) fn refused(
    recorded: &PositionedBranchTransition,
    detail: String,
) -> RelationalRecoveryStop {
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
