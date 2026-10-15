//! Owned closed-row debt; links live in admitted note backing, never in rows.
use super::super::{DemandRecord, DemandRegistryState, WorthQueryOutputDemandKey};
use std::sync::{Arc, Mutex, Weak};
mod transition;
mod visit;
pub(in super::super) use visit::{ClosedRowNote, QueuedClosedRow};

pub(in super::super) struct ClosedCleanupLinks(Arc<ClosedNoteBacking>);
pub(in super::super) struct ClosedNoteBacking {
    pub(in super::super) claim_owner: super::super::claim_readthrough::OwnerState,
    pub(in super::super) claimants:
        std::sync::Mutex<super::super::claim_readthrough::IncomingClaims>,
    key: std::sync::Mutex<Option<ClosedNoteKey>>,
    closed_note: std::sync::Mutex<ClosedNoteLinks>,
    order: Mutex<Weak<Mutex<ClosedOrder>>>,
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
            order: Default::default(),
            _capacity: capacity,
        }))
    }
    pub(in super::super) fn node(&self) -> Arc<ClosedNoteBacking> {
        Arc::clone(&self.0)
    }
    pub(in super::super) fn publish_owner_state(&self, row: &DemandRecord) {
        *self.0.claim_owner.lock().unwrap() =
            super::super::claim_readthrough::ClaimOwnerState::from_record(row);
        transition::publish(row, &self.0);
    }
    pub(in super::super) fn queued_note(&self) -> Option<QueuedClosedRow> {
        self.queued()
            .then(|| QueuedClosedRow::new(Arc::clone(&self.0)))
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
            order: Default::default(),
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
    visiting: bool,
}
/// The fixed actor order owns admitted growing nodes; rows retain only a handle.
#[derive(Default, Clone)]
pub(in super::super) struct ClosedCleanupQueue(Arc<Mutex<ClosedOrder>>);
#[derive(Default)]
struct ClosedOrder {
    first: Option<Arc<ClosedNoteBacking>>,
    last: Weak<ClosedNoteBacking>,
    len: usize,
}
impl ClosedCleanupQueue {
    #[cfg(test)]
    pub(in super::super) fn len(&self) -> usize {
        self.0.lock().unwrap().len
    }
    #[cfg(test)]
    pub(in super::super) fn head(&self) -> Option<Arc<WorthQueryOutputDemandKey>> {
        self.front().map(|note| note.key())
    }
    pub(in super::super) fn front(&self) -> Option<QueuedClosedRow> {
        self.0
            .lock()
            .unwrap()
            .first
            .clone()
            .map(QueuedClosedRow::new)
    }
    pub(in super::super) fn bind(&self, key: &WorthQueryOutputDemandKey, row: &DemandRecord) {
        let node = &row.closed_cleanup.0;
        *node.order.lock().unwrap() = Arc::downgrade(&self.0);
        #[cfg(test)]
        if node.key.lock().unwrap().is_none() {
            *node.key.lock().unwrap() = Some(ClosedNoteKey::Fixture(Arc::new(key.clone())));
        }
        #[cfg(not(test))]
        let _ = key; // Production note backing already owns the admitted membership key.
        row.closed_cleanup.publish_owner_state(row);
    }
    pub(in super::super) fn rotate_front(&self) {
        let mut order = self.0.lock().unwrap();
        if let Some(node) = order.first.clone() {
            order.unlink(&node);
            order.enqueue(node);
        }
    }
    fn enqueue(&self, node: Arc<ClosedNoteBacking>) {
        self.0.lock().unwrap().enqueue(node);
    }
    fn unlink(&self, node: &Arc<ClosedNoteBacking>) {
        self.0.lock().unwrap().unlink(node);
    }
    #[cfg(test)]
    pub(in super::super) fn valid_order(&self) -> bool {
        self.0.lock().unwrap().valid_order()
    }
}
impl ClosedOrder {
    fn enqueue(&mut self, node: Arc<ClosedNoteBacking>) {
        let mut links = node.closed_note.lock().unwrap();
        if links.queued {
            return;
        }
        links.queued = true;
        links.previous = self.last.clone();
        match self.last.upgrade() { Some(last) => {
            last.closed_note.lock().unwrap().next = Some(Arc::clone(&node));
        } _ => {
            self.first = Some(Arc::clone(&node));
        }}
        self.last = Arc::downgrade(&node);
        self.len += 1;
    }
    fn unlink(&mut self, node: &Arc<ClosedNoteBacking>) {
        let mut links = node.closed_note.lock().unwrap();
        assert!(links.queued);
        let previous = std::mem::take(&mut links.previous);
        let next = links.next.take();
        links.queued = false;
        match previous.upgrade() { Some(prior) => {
            prior.closed_note.lock().unwrap().next = next.clone();
        } _ => {
            self.first = next.clone();
        }}
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
impl Drop for ClosedOrder {
    fn drop(&mut self) {
        while let Some(node) = self.first.clone() {
            self.unlink(&node);
        }
    }
}
impl DemandRegistryState {
    #[cfg(test)]
    pub(in super::super) fn note_closed_row(&mut self, key: &WorthQueryOutputDemandKey) {
        // Fixture issuer only: production notes come exclusively from the transition.
        let node = self.records[key].closed_cleanup.node();
        self.closed_cleanup.enqueue(node);
    }
    pub(in super::super) fn remove_record(
        &mut self,
        key: &WorthQueryOutputDemandKey,
    ) -> Option<DemandRecord> {
        let row = self.records.remove(key)?;
        if matches!(
            row.idle_disposition(false),
            super::super::IdleRowDisposition::Retired(
                super::super::RowRetirement::AbandonedPublication
            )
        ) {
            if let Some(scope) = row.source_scope {
                for commit in &row.source_commits {
                    if let Some(custody) = self.source_custody.get_mut(commit) {
                        custody.return_undelivered_root(&key.source, scope);
                    }
                }
            }
        }
        if let Some(member) = &row.work_membership {
            member.set_required(false);
        }
        // Membership is derived from a live row. Once that row leaves, a
        // stale owner's claim cannot keep a membership with no answer behind it.
        let member = self.remove_required_member_with_awaited(key, false);
        self.required_reserved_bytes = self
            .required_reserved_bytes
            .saturating_sub(member.refund_required_bytes);
        drop(member);
        Some(row)
    }
    pub(in super::super) fn take_closed_note(
        &mut self,
        note: QueuedClosedRow,
    ) -> ClosedRowNote<'_> {
        let node = note.into_node();
        self.closed_cleanup.unlink(&node);
        node.closed_note.lock().unwrap().visiting = true;
        ClosedRowNote::new(self, node)
    }
}
impl Drop for ClosedCleanupLinks {
    fn drop(&mut self) {
        self.0.claim_owner.lock().unwrap().live = false;
    }
}
