//! Owned closed-row debt; links live in admitted note backing, never in rows.
use super::super::{DemandRecord, DemandRegistryState, WorthQueryOutputDemandKey};
use std::sync::{Arc, Weak};

pub(in super::super) struct ClosedCleanupLinks(Arc<ClosedNoteBacking>);
pub(in super::super) struct ClosedNoteBacking {
    pub(in super::super) claim_owner: super::super::claim_readthrough::OwnerState,
    pub(in super::super) claimants:
        std::sync::Mutex<super::super::claim_readthrough::IncomingClaims>,
    key: std::sync::Mutex<Option<ClosedNoteKey>>,
    closed_note: std::sync::Mutex<ClosedNoteLinks>,
    _capacity: super::super::record_capacity::RecordCapacity,
}
impl ClosedCleanupLinks {
    pub(in super::super) fn new(
        key: Arc<super::super::required_work::RequiredWorkMembership>,
        capacity: super::super::record_capacity::RecordCapacity,
    ) -> Self {
        Self(Arc::new(ClosedNoteBacking {
            claim_owner: Default::default(),
            claimants: Default::default(),
            key: std::sync::Mutex::new(Some(ClosedNoteKey::Funded(key))),
            closed_note: Default::default(),
            _capacity: capacity,
        }))
    }
    pub(in super::super) fn node(&self) -> Arc<ClosedNoteBacking> {
        Arc::clone(&self.0)
    }
    pub(in super::super) fn publish_owner_state(&self, row: &DemandRecord) {
        *self.0.claim_owner.lock().unwrap() =
            super::super::claim_readthrough::ClaimOwnerState::from_record(row);
    }
    pub(in super::super) fn queued(&self) -> bool {
        self.0.closed_note.lock().unwrap().queued
    }
}
#[cfg(test)]
impl Default for ClosedCleanupLinks {
    fn default() -> Self {
        Self(Arc::new(ClosedNoteBacking {
            claim_owner: Default::default(),
            claimants: Default::default(),
            key: Default::default(),
            closed_note: Default::default(),
            _capacity: super::super::record_capacity::fixture_capacity(),
        }))
    }
}
enum ClosedNoteKey {
    // Keeping the original owner keeps its key-allocation credit alive too.
    Funded(Arc<super::super::required_work::RequiredWorkMembership>),
    #[cfg(test)]
    Fixture(Arc<WorthQueryOutputDemandKey>),
}
impl ClosedNoteBacking {
    fn key_arc(&self) -> Arc<WorthQueryOutputDemandKey> {
        match self
            .key
            .lock()
            .unwrap()
            .as_ref()
            .expect("noted key was admitted")
        {
            ClosedNoteKey::Funded(member) => Arc::clone(member.key_arc()),
            #[cfg(test)]
            ClosedNoteKey::Fixture(key) => Arc::clone(key),
        }
    }
}
#[derive(Default)]
pub(in super::super) struct ClosedNoteLinks {
    previous: Weak<ClosedNoteBacking>,
    next: Option<Arc<ClosedNoteBacking>>,
    queued: bool,
}
#[derive(Default)]
pub(in super::super) struct ClosedCleanupQueue {
    first: Option<Arc<ClosedNoteBacking>>,
    last: Weak<ClosedNoteBacking>,
    pub len: usize,
}
impl ClosedCleanupQueue {
    pub(in super::super) fn head(&self) -> Option<Arc<WorthQueryOutputDemandKey>> {
        self.first.as_ref().map(|node| node.key_arc())
    }
    fn enqueue(&mut self, node: Arc<ClosedNoteBacking>) {
        let mut links = node.closed_note.lock().unwrap();
        if links.queued {
            return;
        }
        links.queued = true;
        links.previous = self.last.clone();
        if let Some(last) = self.last.upgrade() {
            last.closed_note.lock().unwrap().next = Some(Arc::clone(&node));
        } else {
            self.first = Some(Arc::clone(&node));
        }
        self.last = Arc::downgrade(&node);
        self.len += 1;
    }
    fn unlink(&mut self, node: &Arc<ClosedNoteBacking>) {
        let mut links = node.closed_note.lock().unwrap();
        assert!(links.queued);
        let previous = std::mem::take(&mut links.previous);
        let next = links.next.take();
        links.queued = false;
        if let Some(prior) = previous.upgrade() {
            prior.closed_note.lock().unwrap().next = next.clone();
        } else {
            self.first = next.clone();
        }
        if let Some(next) = next {
            next.closed_note.lock().unwrap().previous = previous;
        } else {
            self.last = previous;
        }
        self.len -= 1;
    }
    #[cfg(test)]
    pub(in super::super) fn valid_order(&self) -> bool {
        let mut node = self.first.clone();
        let mut previous = None;
        let mut count = 0;
        while let Some(current) = node {
            if count >= self.len {
                return false;
            }
            let links = current.closed_note.lock().unwrap();
            if !links.queued || links.previous.upgrade().as_ref().map(Arc::as_ptr) != previous {
                return false;
            }
            previous = Some(Arc::as_ptr(&current));
            node = links.next.clone();
            count += 1;
        }
        count == self.len && self.last.upgrade().as_ref().map(Arc::as_ptr) == previous
    }
}
impl Drop for ClosedCleanupQueue {
    fn drop(&mut self) {
        while let Some(node) = self.first.clone() {
            self.unlink(&node);
        }
    }
}
impl DemandRegistryState {
    /// The note's allocation and key were paid before the owner was issued.
    pub(in super::super) fn note_closed_row(&mut self, key: &WorthQueryOutputDemandKey) {
        let row = self.records.get_mut(key).expect("noting a live row");
        let node = Arc::clone(&row.closed_cleanup.0);
        let mut funded_key = node.key.lock().unwrap();
        if funded_key.is_none() {
            #[cfg(test)]
            {
                *funded_key = Some(if let Some(member) = &row.work_membership {
                    ClosedNoteKey::Funded(Arc::clone(member))
                } else {
                    ClosedNoteKey::Fixture(Arc::clone(
                        self.required_keys
                            .get(key)
                            .expect("a fixture owner retains its key through release"),
                    ))
                });
            }
        }
        drop(funded_key);
        self.closed_cleanup.enqueue(node);
    }
    pub(in super::super) fn remove_record(
        &mut self,
        key: &WorthQueryOutputDemandKey,
    ) -> Option<DemandRecord> {
        self.records.remove(key)
    }
    pub(in super::super) fn take_closed_note(
        &mut self,
        key: Arc<WorthQueryOutputDemandKey>,
    ) -> ClosedRowNote<'_> {
        let node = self
            .records
            .get(&key)
            .map(|row| Arc::clone(&row.closed_cleanup.0))
            .or_else(|| {
                self.closed_cleanup
                    .first
                    .as_ref()
                    .filter(|node| node.key_arc().as_ref() == key.as_ref())
                    .cloned()
            })
            .expect("the owned order retains the note even after its row leaves");
        self.closed_cleanup.unlink(&node);
        ClosedRowNote {
            state: self,
            node,
            outstanding: true,
        }
    }
}
/// An interrupted visit restores the owned note, independent of the record map.
#[must_use]
pub(in super::super) struct ClosedRowNote<'a> {
    state: &'a mut DemandRegistryState,
    node: Arc<ClosedNoteBacking>,
    outstanding: bool,
}
impl ClosedRowNote<'_> {
    pub(in super::super) fn prune<T>(
        mut self,
        prune: impl FnOnce(&mut DemandRegistryState) -> T,
    ) -> T {
        let result = prune(self.state);
        // A missing row consumes closed debt: its claims left with it. Custody
        // remains an independent resource, so a missing row cannot consume its note.
        self.outstanding = self
            .state
            .records
            .get(self.node.key_arc().as_ref())
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
            self.state.closed_cleanup.enqueue(Arc::clone(&self.node));
        }
    }
}

impl Drop for ClosedCleanupLinks {
    fn drop(&mut self) {
        self.0.claim_owner.lock().unwrap().live = false;
    }
}
