use serde::{Deserialize, Serialize};

use crate::branch::{
    PositionedBranchTransition, RelationalBranchLifecyclePosture, RelationalBranchTransition,
    RelationalBranchTransitionSequence,
};
use crate::history::data::BranchId;
use crate::publication::patch::data::PatchStreamPosition;

/// The lifecycle posture a checkpoint records for one branch it holds. A
/// checkpoint records one for every branch cell, so recovery restores an
/// archive or a pending deletion instead of reviving the branch as live.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DurableBranchLifecycleImage {
    pub(crate) branch_id: BranchId,
    pub(crate) posture: RelationalBranchLifecyclePosture,
}

/// One branch transition a checkpoint covers, retained as it retains every
/// covered commit envelope, so a recovered runtime holds its whole transition
/// stream. Derived: the durable transition journal is its authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DurableBranchTransitionImage {
    pub(crate) sequence: RelationalBranchTransitionSequence,
    pub(crate) follows: PatchStreamPosition,
    pub(crate) transition: RelationalBranchTransition,
}

impl DurableBranchTransitionImage {
    pub(crate) fn capture(positioned: &PositionedBranchTransition) -> Self {
        Self {
            sequence: positioned.sequence(),
            follows: positioned.follows(),
            transition: positioned.transition().clone(),
        }
    }
}

/// Readmit a checkpoint's retained transitions: sequences contiguous from
/// one through the checkpoint's coverage, followed positions non-decreasing
/// and within its commit coverage.
///
/// Cost: O(retained transitions).
pub(crate) fn readmit_branch_transition_images(
    images: &[DurableBranchTransitionImage],
    covered_transitions: RelationalBranchTransitionSequence,
    covered_position: PatchStreamPosition,
) -> Result<Vec<PositionedBranchTransition>, String> {
    if images.len() as u64 != covered_transitions.0 {
        return Err(format!(
            "checkpoint covers {} branch transitions but retains {}",
            covered_transitions.0,
            images.len()
        ));
    }
    let mut previous_follows = PatchStreamPosition(0);
    images
        .iter()
        .enumerate()
        .map(|(index, image)| {
            let expected = RelationalBranchTransitionSequence(index as u64 + 1);
            if image.sequence != expected
                || image.follows < previous_follows
                || image.follows > covered_position
            {
                return Err(format!(
                    "checkpoint branch transition {} at sequence {} follows position {} out of order",
                    index, image.sequence.0, image.follows.0
                ));
            }
            previous_follows = image.follows;
            Ok(PositionedBranchTransition::admit(
                image.sequence,
                image.follows,
                image.transition.clone(),
            ))
        })
        .collect()
}
