use serde::{Deserialize, Serialize};

/// The position of one branch transition in the runtime's own transition
/// stream. Sequences start at one and are contiguous; zero precedes every
/// transition and is the coverage of a history with none. They are distinct
/// from patch-stream positions: a transition never consumes a commit
/// position.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct RelationalBranchTransitionSequence(pub u64);

impl RelationalBranchTransitionSequence {
    /// The sequence after this one, or `None` when the stream is exhausted.
    pub(crate) fn next(self) -> Option<Self> {
        self.0.checked_add(1).map(Self)
    }
}
