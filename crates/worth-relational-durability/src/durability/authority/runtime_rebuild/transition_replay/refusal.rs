//! Preserve typed transition failures and request stops.
use contracts::durability::data::recovery_errors::{DurabilityError, RecoveryFailureClass};
use contracts::durability::data::recovery_stop::RelationalRecoveryStop;
use std::fmt::Debug;
use worth_relational_branch::internal_port::branch::transition::durable_step::ReplayedBranchTransitionDivergence;
use worth_relational_branch::internal_port::branch::transition::positioned::PositionedBranchTransition;
use worth_relational_branch::internal_port::branch::transition::stop::BranchTransitionRecordError;
use worth_relational_contracts::internal_port as contracts;

/// A replay that did not apply is a recovery refusal; a stopped request
/// stays a request stop.
pub fn replay_stop<D: Debug>(
    recorded: &PositionedBranchTransition,
    failure: BranchTransitionRecordError<D, ReplayedBranchTransitionDivergence>,
) -> RelationalRecoveryStop {
    match failure {
        BranchTransitionRecordError::OriginalInputsStopped(stop) => stop,
        BranchTransitionRecordError::ReservationWaitStopped(stop) => {
            RelationalRecoveryStop::Request(
                contracts::execution::denial_cause::RelationalExecutionDenialCause::from(stop),
            )
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

pub fn refused(recorded: &PositionedBranchTransition, detail: String) -> RelationalRecoveryStop {
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
