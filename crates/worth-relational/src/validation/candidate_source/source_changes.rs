//! The provider owner's exact per-scope Source delta for one proposal: the
//! rows whose materialized entry entered, left or changed, on each side.
use crate::identity::data::EntityId;

/// Committed-side rows are those that left or changed; proposed-side rows
/// are those that entered or changed. Each list is in entity order. Empty
/// when the proposal left the scope's Source logically unchanged; an empty
/// side allocates nothing. The sealed root owns it; a loan borrows it from
/// the draft by checked coordinate, so no handle can keep it alive.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ManagedIndexSourceChanges {
    committed: Vec<EntityId>,
    proposed: Vec<EntityId>,
}

impl ManagedIndexSourceChanges {
    pub fn new(committed: Vec<EntityId>, proposed: Vec<EntityId>) -> Self {
        Self {
            committed,
            proposed,
        }
    }
    /// The rows on one side: committed when `before`.
    pub fn side(&self, before: bool) -> &[EntityId] {
        if before {
            &self.committed
        } else {
            &self.proposed
        }
    }
}
