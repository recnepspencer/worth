use crate::branch::PositionedBranchTransition;
use crate::publication::patch::data::PatchStreamPosition;

/// The branch transitions a recovery replays, merged into its commit tail by
/// the position each follows: before replaying the commit at position `q`,
/// every transition that follows a position below `q` is applied.
#[derive(Debug, Clone)]
pub(crate) enum RecoveryBranchTransitions {
    /// Every durable transition after the checkpoint's coverage, contiguous.
    /// Replay must reproduce each exactly, its sequence and followed position
    /// included.
    Contiguous(Vec<PositionedBranchTransition>),
    /// The transitions a partial replay chain needs, selected from a longer
    /// history. Replay must reproduce each transition; its sequence and
    /// followed position are the ones the partial history gives it.
    Selected(Vec<PositionedBranchTransition>),
}

impl RecoveryBranchTransitions {
    pub(crate) fn transitions(&self) -> &[PositionedBranchTransition] {
        match self {
            Self::Contiguous(transitions) | Self::Selected(transitions) => transitions,
        }
    }

    /// Whether replay must reproduce the recorded sequence and followed
    /// position as well as the transition.
    pub(crate) fn replays_positions_exactly(&self) -> bool {
        matches!(self, Self::Contiguous(_))
    }

    /// The transitions to apply before the commit at `position`, oldest first,
    /// removed from the front of `pending`.
    ///
    /// Cost: O(transitions returned).
    pub(crate) fn take_before(
        pending: &mut std::collections::VecDeque<PositionedBranchTransition>,
        position: PatchStreamPosition,
    ) -> Vec<PositionedBranchTransition> {
        let mut due = Vec::new();
        while pending
            .front()
            .is_some_and(|transition| transition.follows() < position)
        {
            due.extend(pending.pop_front());
        }
        due
    }
}
