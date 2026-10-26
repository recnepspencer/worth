//! Shared FIFO discovery; each pinned traversal keeps its own read/cost policy.
use std::collections::{BTreeSet, VecDeque};

use crate::identity::data::EntityId;

pub(super) struct BreadthFirstFrontier<Entry> {
    seen: BTreeSet<EntityId>,
    queue: VecDeque<Entry>,
}

impl<Entry> BreadthFirstFrontier<Entry> {
    pub(super) fn new() -> Self {
        Self {
            seen: BTreeSet::new(),
            queue: VecDeque::new(),
        }
    }

    pub(super) fn contains(&self, entity: &EntityId) -> bool {
        self.seen.contains(entity)
    }

    pub(super) fn enqueue<E>(
        &mut self,
        entity: EntityId,
        entry: Entry,
        admit: impl FnOnce() -> Result<(), E>,
    ) -> Result<bool, E> {
        if self.seen.contains(&entity) {
            return Ok(false);
        }
        admit()?;
        self.seen.insert(entity);
        self.queue.push_back(entry);
        Ok(true)
    }

    pub(super) fn pop(&mut self) -> Option<Entry> {
        self.queue.pop_front()
    }
}
