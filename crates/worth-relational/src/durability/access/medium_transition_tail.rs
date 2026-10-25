//! Readmission of the branch transition tail a durable medium recovered.
//!
//! Every artifact is readmitted, and the tail must continue the checkpoint
//! exactly: sequences contiguous from the checkpoint's transition coverage,
//! followed positions non-decreasing, the first at or after the checkpoint's
//! commit coverage and the last at or below the recovered commit head. A
//! transition outside that order could not have been performed by the
//! history recovery replays, so it refuses rather than being reordered.

use crate::branch::{PositionedBranchTransition, RelationalBranchTransitionSequence};
use crate::durability::branch_transition_artifact::{
    RelationalBranchTransitionArtifact, RelationalBranchTransitionArtifactRefusal,
};
use crate::durability::data::{DurabilityError, RecoveryFailureClass, RelationalRecoveryStop};
use crate::publication::patch::data::PatchStreamPosition;

/// The bounds a recovered transition tail must lie within.
pub(super) struct TransitionTailBounds {
    /// The last transition sequence the checkpoint covers.
    pub(super) covered_transitions: RelationalBranchTransitionSequence,
    /// The last commit position the checkpoint covers.
    pub(super) covered_position: PatchStreamPosition,
    /// The last commit position the recovered tail holds.
    pub(super) commit_head: PatchStreamPosition,
}

/// Readmit `artifacts`, oldest first, within `bounds`.
///
/// Cost: one readmission per artifact, O(payload bytes) each.
pub(super) fn readmit_transition_tail(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    bounds: &TransitionTailBounds,
    artifacts: &[RelationalBranchTransitionArtifact],
) -> Result<Vec<PositionedBranchTransition>, RelationalRecoveryStop> {
    let mut tail = Vec::with_capacity(artifacts.len());
    let mut expected = bounds.covered_transitions;
    let mut earliest_follows = bounds.covered_position;
    for artifact in artifacts {
        expected = expected
            .next()
            .ok_or_else(|| out_of_order("branch transition tail sequence overflows".to_owned()))?;
        if artifact.sequence() != expected {
            return Err(out_of_order(format!(
                "branch transition tail holds sequence {} where sequence {} is required",
                artifact.sequence().0,
                expected.0
            )));
        }
        let positioned =
            RelationalBranchTransitionArtifact::readmit(artifact.as_bytes(), execution)
                .map_err(|refusal| artifact_refusal_stop(expected, refusal))?;
        let follows = positioned.follows();
        if follows < earliest_follows {
            return Err(out_of_order(format!(
                "branch transition {} follows position {} but must follow position {} or later",
                expected.0, follows.0, earliest_follows.0
            )));
        }
        if follows > bounds.commit_head {
            return Err(out_of_order(format!(
                "branch transition {} follows position {} beyond the recovered commit head {}",
                expected.0, follows.0, bounds.commit_head.0
            )));
        }
        earliest_follows = follows;
        tail.push(positioned);
    }
    Ok(tail)
}

fn out_of_order(detail: String) -> RelationalRecoveryStop {
    DurabilityError::new(
        RecoveryFailureClass::NonContiguousBranchTransitionTail,
        detail,
    )
    .into()
}

pub(super) fn artifact_refusal_stop(
    sequence: RelationalBranchTransitionSequence,
    refusal: RelationalBranchTransitionArtifactRefusal,
) -> RelationalRecoveryStop {
    match refusal {
        RelationalBranchTransitionArtifactRefusal::Request(cause) => {
            RelationalRecoveryStop::Request(cause)
        }
        refusal => DurabilityError::new(
            RecoveryFailureClass::CorruptBranchTransitionArtifact,
            format!(
                "branch transition artifact at sequence {} refused: {refusal:?}",
                sequence.0
            ),
        )
        .into(),
    }
}
