use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::history::data::CommitId;

mod record;

pub(crate) use record::{
    PendingRelationalPublicationSettlement, PerformedRelationalSettlement,
    RelationalSettlementClaim, ReservedRelationalSettlement,
};

/// Why one publication attempt could not install its pending-settlement record
/// before movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RelationalSettlementReservationDenial {
    DuplicateCommitIdentity,
    OwnerUnavailable,
}

/// The one runtime-owned registry of pending publication settlements.
///
/// Admission, lookup, transition, and removal are O(1) by owner-issued commit
/// identity.
///
/// Only a prepared candidate can reach admission. Every prepared candidate
/// already owns a published-snapshot reservation, which remains accounted
/// until its settled snapshot is released. When a finite policy is installed,
/// preparation enforces it; absence never discards this custody or installs a
/// second population allowance here.
///
/// The index lock below is taken only for those constant-bounded operations; it
/// never spans branch waiting, durability I/O, derived work, or a test pause.
#[derive(Debug, Default)]
pub(crate) struct RelationalPublicationSettlementRegistry {
    records: Mutex<HashMap<CommitId, Arc<PendingRelationalPublicationSettlement>>>,
    contacts: AtomicU64,
    owner_loss_releases: AtomicU64,
    closed: AtomicBool,
}

/// Armed pre-effect reservation.
///
/// Dropping it releases the record and its capacity, so a no-movement path
/// cannot leak a reservation by forgetting an explicit cleanup call. Only
/// successful movement disarms it, through [`Self::authorize`].
#[must_use = "an unauthorized settlement reservation must release its capacity"]
pub(crate) struct RelationalPendingSettlementReservation {
    registry: Arc<RelationalPublicationSettlementRegistry>,
    record: Arc<PendingRelationalPublicationSettlement>,
    armed: bool,
}

impl RelationalPublicationSettlementRegistry {
    /// Reserve capacity and install one pending record before the publication
    /// critical section.
    pub(crate) fn reserve(
        registry: &Arc<Self>,
        commit_id: CommitId,
        runtime_instance_id: u64,
        reserved: ReservedRelationalSettlement,
    ) -> Result<RelationalPendingSettlementReservation, RelationalSettlementReservationDenial> {
        registry.contacts.fetch_add(1, Ordering::Relaxed);
        if registry.closed.load(Ordering::Acquire) {
            return Err(RelationalSettlementReservationDenial::OwnerUnavailable);
        }
        let record = Arc::new(PendingRelationalPublicationSettlement::reserved(
            commit_id,
            runtime_instance_id,
            reserved,
        ));
        let mut records = registry.records();
        if records.contains_key(&commit_id) {
            return Err(RelationalSettlementReservationDenial::DuplicateCommitIdentity);
        }
        records.insert(commit_id, Arc::clone(&record));
        drop(records);
        Ok(RelationalPendingSettlementReservation {
            registry: Arc::clone(registry),
            record,
            armed: true,
        })
    }

    fn records(
        &self,
    ) -> std::sync::MutexGuard<'_, HashMap<CommitId, Arc<PendingRelationalPublicationSettlement>>>
    {
        self.records
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(crate) fn record(
        &self,
        commit_id: CommitId,
    ) -> Option<Arc<PendingRelationalPublicationSettlement>> {
        self.contacts.fetch_add(1, Ordering::Relaxed);
        self.records().get(&commit_id).map(Arc::clone)
    }

    /// Remove one record, but only if it is still the exact record supplied.
    pub(crate) fn release(&self, record: &Arc<PendingRelationalPublicationSettlement>) {
        self.contacts.fetch_add(1, Ordering::Relaxed);
        let mut records = self.records();
        if records
            .get(&record.commit_id())
            .is_some_and(|installed| Arc::ptr_eq(installed, record))
        {
            records.remove(&record.commit_id());
        }
    }

    pub(crate) fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Acquire)
    }

    /// Owner shutdown. Admission closes first, then remaining retention is
    /// resolved with typed owner-loss accounting.
    pub(crate) fn close(&self) {
        self.closed.store(true, Ordering::Release);
        let drained = std::mem::take(&mut *self.records());
        self.owner_loss_releases
            .fetch_add(drained.len() as u64, Ordering::Relaxed);
    }

    #[cfg(test)]
    pub(crate) fn pending_count(&self) -> usize {
        self.records().len()
    }

    #[cfg(test)]
    pub(crate) fn contact_count(&self) -> u64 {
        self.contacts.load(Ordering::Relaxed)
    }

    #[cfg(test)]
    pub(crate) fn owner_loss_release_count(&self) -> u64 {
        self.owner_loss_releases.load(Ordering::Relaxed)
    }
}

impl RelationalPendingSettlementReservation {
    /// Successful movement authorizes the already-installed record against its
    /// exact positioned canonical commit and disarms pre-effect cleanup. After
    /// this returns, the commit is recovery-addressable by identity alone.
    pub(crate) fn authorize(
        mut self,
        positioned: Arc<crate::history::data::PositionedCanonicalCommit>,
        settlement_retention: crate::history::retention::RelationalPerformedSettlementObligation,
        late_interruption: Option<crate::runtime::RelationalInterruptionEvent>,
    ) -> Arc<PendingRelationalPublicationSettlement> {
        assert!(
            self.record
                .authorize_performed(positioned, settlement_retention, late_interruption),
            "one linearized candidate authorizes its reservation exactly once"
        );
        self.armed = false;
        Arc::clone(&self.record)
    }
}

impl Drop for RelationalPendingSettlementReservation {
    fn drop(&mut self) {
        if self.armed {
            self.registry.release(&self.record);
        }
    }
}
