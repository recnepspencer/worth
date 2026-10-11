//! Intrusive closed-row debt uses the key already funded by record admission.
use super::super::{DemandRecord, DemandRegistryState, WorthQueryOutputDemandKey};
use std::sync::Arc;

#[derive(Default)]
pub(in super::super) struct ClosedCleanupLinks {
    previous: Option<Arc<WorthQueryOutputDemandKey>>,
    next: Option<Arc<WorthQueryOutputDemandKey>>,
    queued: bool,
}

impl ClosedCleanupLinks {
    pub(in super::super) fn queued(&self) -> bool {
        self.queued
    }
}

#[derive(Default)]
pub(in super::super) struct ClosedCleanupQueue {
    pub head: Option<Arc<WorthQueryOutputDemandKey>>,
    tail: Option<Arc<WorthQueryOutputDemandKey>>,
    pub len: usize,
}

impl DemandRegistryState {
    /// Drop borrows an already admitted key; linking creates no new backing.
    pub(in super::super) fn note_closed_row(&mut self, key: &WorthQueryOutputDemandKey) {
        let row = &self.records[key];
        if row.closed_cleanup.queued {
            return;
        }
        let key = row
            .work_membership
            .as_ref()
            .map(|member| member.key_arc())
            .or_else(|| self.required_keys.get(key))
            .expect("an admitted owner retains its key through release");
        let key = Arc::clone(key);
        self.requeue_closed_key(key);
    }

    fn requeue_closed_key(&mut self, key: Arc<WorthQueryOutputDemandKey>) {
        let Some(row) = self.records.get_mut(key.as_ref()) else {
            return;
        };
        if row.closed_cleanup.queued {
            return;
        }
        row.closed_cleanup.previous = self.closed_cleanup.tail.take();
        row.closed_cleanup.queued = true;
        let previous = row.closed_cleanup.previous.clone();
        if let Some(previous) = &previous {
            self.records
                .get_mut(previous.as_ref())
                .unwrap()
                .closed_cleanup
                .next = Some(Arc::clone(&key));
        } else {
            self.closed_cleanup.head = Some(Arc::clone(&key));
        }
        self.closed_cleanup.tail = Some(key);
        self.closed_cleanup.len += 1;
    }

    fn unlink_closed_note(&mut self, key: &WorthQueryOutputDemandKey) {
        let Some(row) = self.records.get_mut(key) else {
            return;
        };
        if !row.closed_cleanup.queued {
            return;
        }
        let previous = row.closed_cleanup.previous.take();
        let next = row.closed_cleanup.next.take();
        row.closed_cleanup.queued = false;
        if let Some(prior) = &previous {
            self.records
                .get_mut(prior.as_ref())
                .unwrap()
                .closed_cleanup
                .next = next.clone();
        } else {
            self.closed_cleanup.head = next.clone();
        }
        if let Some(next) = next {
            self.records
                .get_mut(next.as_ref())
                .unwrap()
                .closed_cleanup
                .previous = previous.clone();
        } else {
            self.closed_cleanup.tail = previous;
        }
        self.closed_cleanup.len -= 1;
    }

    pub(in super::super) fn next_closed_note(
        &self,
        key: &WorthQueryOutputDemandKey,
    ) -> Option<Arc<WorthQueryOutputDemandKey>> {
        self.records[key]
            .closed_cleanup
            .next
            .as_ref()
            .map(Arc::clone)
    }

    /// Removing a row prunes the debt owned by that row, including its links.
    pub(in super::super) fn remove_record(
        &mut self,
        key: &WorthQueryOutputDemandKey,
    ) -> Option<DemandRecord> {
        self.unlink_closed_note(key);
        self.records.remove(key)
    }

    pub(in super::super) fn take_closed_note(
        &mut self,
        key: Arc<WorthQueryOutputDemandKey>,
    ) -> ClosedRowNote<'_> {
        assert!(self.records[&key].closed_cleanup.queued);
        self.unlink_closed_note(&key);
        ClosedRowNote {
            state: self,
            key,
            outstanding: true,
        }
    }
}

/// An interrupted visit restores the note. Only a completed prune consumes it.
#[must_use]
pub(in super::super) struct ClosedRowNote<'a> {
    state: &'a mut DemandRegistryState,
    key: Arc<WorthQueryOutputDemandKey>,
    outstanding: bool,
}

impl ClosedRowNote<'_> {
    pub(in super::super) fn prune<T>(
        mut self,
        prune: impl FnOnce(&mut DemandRegistryState) -> T,
    ) -> T {
        let result = prune(self.state);
        self.outstanding = self
            .state
            .records
            .get(self.key.as_ref())
            .is_some_and(|row| {
                row.terminal()
                    || row.prerequisites.iter().any(|upstream| {
                        self.state
                            .records
                            .get(upstream.as_ref())
                            .is_some_and(super::super::refreshed_rejoin::superseded)
                    })
            });
        result
    }
}

impl Drop for ClosedRowNote<'_> {
    fn drop(&mut self) {
        if self.outstanding {
            self.state.requeue_closed_key(Arc::clone(&self.key));
        }
    }
}
