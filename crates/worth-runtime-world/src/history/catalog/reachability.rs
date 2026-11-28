use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use worth_foundational::DagDirtyList;

use crate::identity::CompositeCommitIdentity;

use super::counters::{lock_counters, HistoryCatalogCountersHandle};
use super::denial::CompositeHistoryCatalogDenial;

mod close_retirement;
mod frontier;
mod installed_children;
mod record;
pub(in crate::history) use record::HistoryReachabilityRecord;

/// Catalog-owned exact reachability index. Admission allocates an empty slot;
/// installation fills it in place. Reclamation never reconstructs ancestry.
#[derive(Debug)]
pub(in crate::history) struct HistoryReachabilityIndex {
    records: HashMap<CompositeCommitIdentity, HistoryReachabilitySlot>,
    counters: HistoryCatalogCountersHandle,
    dirty: DagDirtyList<CompositeCommitIdentity>,
}

pub(in crate::history) type HistoryReachabilitySlot = Arc<Mutex<Option<HistoryReachabilityRecord>>>;

fn lock_slot(slot: &HistoryReachabilitySlot) -> MutexGuard<'_, Option<HistoryReachabilityRecord>> {
    slot.lock().unwrap_or_else(|error| error.into_inner())
}

pub(in crate::history) type HistoryReachabilityHandle = Arc<Mutex<HistoryReachabilityIndex>>;

pub(in crate::history) fn lock_index(
    index: &HistoryReachabilityHandle,
) -> MutexGuard<'_, HistoryReachabilityIndex> {
    index
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl HistoryReachabilityIndex {
    pub(super) fn new(counters: HistoryCatalogCountersHandle) -> Self {
        Self {
            records: HashMap::new(),
            counters,
            dirty: DagDirtyList::default(),
        }
    }

    pub(super) fn reserve(&mut self, identity: CompositeCommitIdentity) -> HistoryReachabilitySlot {
        let slot = Arc::new(Mutex::new(None));
        assert!(self.records.insert(identity, Arc::clone(&slot)).is_none());
        slot
    }

    pub(super) fn release_reservation(&mut self, identity: &CompositeCommitIdentity) {
        let slot = self
            .records
            .remove(identity)
            .expect("reserved reachability slot");
        assert!(lock_slot(&slot).is_none());
    }

    pub(super) fn install_reserved_slot(&mut self, slot: &HistoryReachabilitySlot) {
        let mut record = lock_slot(slot);
        assert!(record.is_none());
        *record = Some(HistoryReachabilityRecord::default());
        lock_counters(&self.counters).record_reachability_row_installed();
    }

    pub(super) fn lookup(
        &mut self,
        identity: &CompositeCommitIdentity,
    ) -> Option<HistoryReachabilityRecord> {
        lock_counters(&self.counters).record_reachability_lookup();
        self.records
            .get(identity)
            .and_then(|slot| lock_slot(slot).clone())
    }

    pub(super) fn increment_descendant_dependency(
        &mut self,
        parent: &CompositeCommitIdentity,
    ) -> Result<(), CompositeHistoryCatalogDenial> {
        let slot = self
            .records
            .get(parent)
            .expect("validated installed parent has a reachability row");
        let mut held = lock_slot(slot);
        let record = held.as_mut().expect("installed reachability record");
        record
            .dependencies
            .reserve_child()
            .map_err(|_| CompositeHistoryCatalogDenial::DependencyCountOverflow(parent.clone()))?;
        lock_counters(&self.counters).record_dependency_increment();
        Ok(())
    }

    /// The reserved child of `parent` installed; its dependency stays.
    pub(super) fn install_forming_child(
        &mut self,
        parent: &CompositeCommitIdentity,
        child: &CompositeCommitIdentity,
    ) {
        self.settle_forming_child(parent);
        self.link_installed_child(parent, child);
    }

    /// The reserved child of `parent` was abandoned with its dependency.
    pub(super) fn abandon_forming_child(&mut self, parent: &CompositeCommitIdentity) {
        self.settle_forming_child(parent);
        self.decrement_descendant_dependency(parent);
    }

    fn settle_forming_child(&mut self, parent: &CompositeCommitIdentity) {
        let slot = self
            .records
            .get(parent)
            .expect("a reserved child retains an installed parent row");
        let mut held = lock_slot(slot);
        let record = held.as_mut().expect("installed reachability record");
        record.dependencies.settle_forming_child();
    }

    pub(super) fn decrement_descendant_dependency(&mut self, parent: &CompositeCommitIdentity) {
        let slot = self
            .records
            .get(parent)
            .expect("a reclaimed child retains an installed parent row");
        let mut held = lock_slot(slot);
        let record = held.as_mut().expect("installed reachability record");
        record.dependencies.release_dependency();
        lock_counters(&self.counters).record_dependency_decrement();
        drop(held);
        self.mark_changed(parent);
    }

    /// The catalog still owns its installation lock and has not exposed this
    /// fresh row. Its first protection is exactly one, with no capacity check.
    pub(super) fn protect_reserved_slot(&mut self, slot: &HistoryReachabilitySlot) {
        let mut held = lock_slot(slot);
        let record = held
            .as_mut()
            .expect("new installation has a reachability row");
        assert_eq!(
            (
                record.direct_protections,
                record.descendant_dependencies(),
                record.has_forming_child(),
                record.product_head_protections
            ),
            (0, 0, false, 0)
        );
        record.direct_protections = 1;
        lock_counters(&self.counters).record_direct_protection_acquisition();
    }

    pub(super) fn increment_reserved_protection(
        &self,
        slot: &HistoryReachabilitySlot,
        identity: &CompositeCommitIdentity,
    ) -> Result<(), CompositeHistoryCatalogDenial> {
        let mut held = lock_slot(slot);
        let record = held.as_mut().expect("installed reachability record");
        record.direct_protections = record.direct_protections.checked_add(1).ok_or_else(|| {
            CompositeHistoryCatalogDenial::ProtectionCountOverflow(identity.clone())
        })?;
        lock_counters(&self.counters).record_direct_protection_acquisition();
        Ok(())
    }

    pub(super) fn increment_direct_protection(
        &mut self,
        identity: &CompositeCommitIdentity,
    ) -> Result<(), CompositeHistoryCatalogDenial> {
        let slot = self
            .records
            .get(identity)
            .expect("installed reachability record");
        self.increment_reserved_protection(slot, identity)
    }

    pub(in crate::history) fn decrement_direct_protection(
        &mut self,
        identity: &CompositeCommitIdentity,
    ) {
        let slot = self
            .records
            .get(identity)
            .expect("a live protection obligation retains its installed row");
        let mut held = lock_slot(slot);
        let record = held.as_mut().expect("installed reachability record");
        record.direct_protections = record
            .direct_protections
            .checked_sub(1)
            .expect("each protection obligation releases one direct protection");
        lock_counters(&self.counters).record_direct_protection_release();
        let released = record.direct_protections == 0;
        drop(held);
        if released {
            self.mark_changed(identity);
        }
    }

    /// A live product reference now carries the head protection on
    /// `identity`; it is a share of that commit's direct protections.
    pub(in crate::history) fn seat_product_head(&mut self, identity: &CompositeCommitIdentity) {
        self.adjust_seated_heads(identity, |seated| seated.checked_add(1));
    }

    /// The product reference that carried a head on `identity` let it go.
    pub(in crate::history) fn unseat_product_head(&mut self, identity: &CompositeCommitIdentity) {
        self.adjust_seated_heads(identity, |seated| seated.checked_sub(1));
    }

    fn adjust_seated_heads(
        &mut self,
        identity: &CompositeCommitIdentity,
        adjust: impl FnOnce(usize) -> Option<usize>,
    ) {
        let slot = self
            .records
            .get(identity)
            .expect("a seated head protection retains its installed row");
        let mut held = lock_slot(slot);
        let record = held.as_mut().expect("installed reachability record");
        let seated = adjust(record.product_head_protections)
            .expect("each head is seated at most once and unseated once");
        assert!(seated <= record.direct_protections);
        record.product_head_protections = seated;
    }

    pub(super) fn remove_installed(
        &mut self,
        identity: &CompositeCommitIdentity,
    ) -> HistoryReachabilityRecord {
        self.unlink_installed_child(identity);
        self.forget_changed(identity);
        let slot = self
            .records
            .remove(identity)
            .expect("reclaiming installed slot");
        let record = lock_slot(&slot)
            .take()
            .expect("installed reachability record");
        assert_eq!(record, HistoryReachabilityRecord::default());
        lock_counters(&self.counters).record_reachability_row_removed();
        record
    }
}
