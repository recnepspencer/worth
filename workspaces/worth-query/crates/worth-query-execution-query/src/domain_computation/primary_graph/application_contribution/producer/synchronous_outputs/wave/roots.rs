//! Root selections (#238, PR C C4): a seeded root owns its selector's
//! support under a selection owner above every task row. One commit per
//! stage evaluates a root once, at its own stage, so that support holds only
//! within the root's stage: qualified use ends its window at the next
//! authored stage's append and never seeds the root again.
use super::*;

/// Selection owners at or above this are roots, never task rows.
const ROOT_SELECTION_OWNER: usize = usize::MAX / 2;

impl<S: ApplicationSchema> SynchronousOutputWave<S> {
    /// The first root selection owner: every owner at or above it is a root.
    pub fn first_root_owner() -> usize {
        ROOT_SELECTION_OWNER
    }
    /// The selection owner of the root seeded next.
    pub fn next_root(&mut self) -> usize {
        self.roots += 1;
        ROOT_SELECTION_OWNER + self.roots - 1
    }
}
