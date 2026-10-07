use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, TryLockError, Weak};

/// Drop-governed publication authority owned directly by one runtime.
#[derive(Debug)]
pub(in crate::runtime) struct RelationalRuntimePublicationOwner {
    binding: RelationalRuntimePublicationBinding,
}

/// Cloneable runtime binding carried by independently borrowable publication artifacts.
#[derive(Debug, Clone)]
pub(crate) struct RelationalRuntimePublicationBinding {
    pub(super) lifecycle: Arc<RelationalRuntimePublicationLifecycle>,
}

#[derive(Debug)]
pub(super) struct RelationalRuntimePublicationLifecycle {
    pub(super) companion: Arc<crate::mvcc::publication::CompanionRegistry>,
    next_candidate_id: AtomicU64,
    candidates: Mutex<HashMap<u64, RegisteredCandidate>>,
    #[cfg(test)]
    test_candidate_cleanup_pause:
        Mutex<Option<(std::sync::mpsc::Sender<()>, std::sync::mpsc::Receiver<()>)>>,
    pub(super) settlements: Arc<super::RelationalPublicationSettlementRegistry>,
}

#[derive(Debug)]
struct RegisteredCandidate {
    expires_at: std::time::Instant,
    payload: Weak<Mutex<Option<crate::mvcc::publication::CandidatePayload>>>,
}

pub(crate) enum RelationalCandidateRegistrationDenial {
    CapacityExhausted,
    IdentityExhausted,
}

impl RelationalRuntimePublicationOwner {
    pub(in crate::runtime) fn new(maximum_companion_cell_bytes: u64) -> Self {
        Self::with_companion(crate::mvcc::publication::CompanionRegistry::new(
            maximum_companion_cell_bytes,
        ))
    }

    pub(in crate::runtime) fn fork_from(source: &Self) -> Self {
        Self::with_companion(source.binding.lifecycle.companion.fork())
    }

    fn with_companion(companion: crate::mvcc::publication::CompanionRegistry) -> Self {
        Self {
            binding: RelationalRuntimePublicationBinding {
                lifecycle: Arc::new(RelationalRuntimePublicationLifecycle {
                    companion: Arc::new(companion),
                    next_candidate_id: AtomicU64::new(1),
                    candidates: Mutex::new(HashMap::new()),
                    #[cfg(test)]
                    test_candidate_cleanup_pause: Mutex::new(None),
                    settlements: Arc::new(super::RelationalPublicationSettlementRegistry::default()),
                }),
            },
        }
    }

    pub(super) fn binding(&self) -> RelationalRuntimePublicationBinding {
        self.binding.clone()
    }
}

impl RelationalRuntimePublicationBinding {
    pub(crate) fn companion_registry(&self) -> &Arc<crate::mvcc::publication::CompanionRegistry> {
        &self.lifecycle.companion
    }

    pub(crate) fn companion_epoch(
        &self,
    ) -> Result<
        crate::mvcc::publication::CompanionRegistrationEpoch<'_>,
        crate::mvcc::PublicationCompanionRegistrationStop,
    > {
        self.lifecycle.companion.enter()
    }
    /// Close settlement admission and resolve remaining retention with typed
    /// owner-loss accounting. Owner authority, so it stays inside the runtime
    /// module tree even though the binding itself is carried by services.
    pub(in crate::runtime) fn close(&self) {
        self.lifecycle.settlements.close();
    }

    pub(crate) fn register_candidate(
        &self,
        expires_at: std::time::Instant,
        maximum_candidates: usize,
        payload: &Arc<Mutex<Option<crate::mvcc::publication::CandidatePayload>>>,
    ) -> Result<u64, RelationalCandidateRegistrationDenial> {
        let mut candidates = self
            .lifecycle
            .candidates
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if candidates.len() >= maximum_candidates {
            return Err(RelationalCandidateRegistrationDenial::CapacityExhausted);
        }
        let id = self
            .lifecycle
            .next_candidate_id
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                (current != 0).then(|| current.checked_add(1).unwrap_or(0))
            })
            .map_err(|_| RelationalCandidateRegistrationDenial::IdentityExhausted)?;
        assert_ne!(id, 0, "candidate identity zero is reserved");
        assert!(
            !candidates.contains_key(&id),
            "candidate identity allocator collided with a live candidate"
        );
        let replaced = candidates.insert(
            id,
            RegisteredCandidate {
                expires_at,
                payload: Arc::downgrade(payload),
            },
        );
        debug_assert!(replaced.is_none());
        Ok(id)
    }

    pub(crate) fn take_candidate(
        &self,
        candidate_id: u64,
    ) -> Option<crate::mvcc::publication::CandidatePayload> {
        let registered = self
            .lifecycle
            .candidates
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(&candidate_id)?;
        registered.payload.upgrade()?.lock().ok()?.take()
    }

    pub(crate) fn discard_candidate(&self, candidate_id: u64) {
        // Keep the registry locked until checkpointed record reservations are
        // released. An empty-registry observation must not miss detached cleanup.
        let mut candidates = self
            .lifecycle
            .candidates
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let registered = candidates.remove(&candidate_id);
        #[cfg(test)]
        if let Some((ack, resume)) = self
            .lifecycle
            .test_candidate_cleanup_pause
            .lock()
            .unwrap()
            .take()
        {
            ack.send(()).unwrap();
            resume.recv().unwrap();
        }
        if let Some(payload) = registered.and_then(|entry| entry.payload.upgrade()) {
            payload
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .take();
        }
    }

    /// A busy registry can be detached cleanup; refuse rather than wait for it.
    pub(in crate::runtime) fn candidates_are_quiescent(&self) -> bool {
        match self.lifecycle.candidates.try_lock() {
            Ok(candidates) => candidates.is_empty(),
            Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner().is_empty(),
            Err(TryLockError::WouldBlock) => false,
        }
    }

    #[cfg(test)]
    pub(crate) fn install_test_candidate_cleanup_pause(
        &self,
        ack: std::sync::mpsc::Sender<()>,
        resume: std::sync::mpsc::Receiver<()>,
    ) {
        *self.lifecycle.test_candidate_cleanup_pause.lock().unwrap() = Some((ack, resume));
    }

    pub(crate) fn reap_expired_candidates(&self) -> usize {
        let mut candidates = self
            .lifecycle
            .candidates
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        reap_expired_candidates(&mut candidates, std::time::Instant::now())
    }

    pub(crate) fn belongs_to_same_owner(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.lifecycle, &other.lifecycle)
    }

    /// Install the one pending-settlement record for this attempt before the
    /// publication critical section, so a moved branch head is never observable
    /// without its owner recovery record.
    pub(crate) fn reserve_pending_settlement(
        &self,
        commit_id: crate::history::data::CommitId,
        runtime_instance_id: u64,
        reserved: super::ReservedRelationalSettlement,
    ) -> Result<
        super::RelationalPendingSettlementReservation,
        super::RelationalSettlementReservationDenial,
    > {
        super::RelationalPublicationSettlementRegistry::reserve(
            &self.lifecycle.settlements,
            commit_id,
            runtime_instance_id,
            reserved,
        )
    }

    pub(crate) fn pending_settlement(
        &self,
        commit_id: crate::history::data::CommitId,
    ) -> Option<Arc<super::PendingRelationalPublicationSettlement>> {
        self.lifecycle.settlements.record(commit_id)
    }

    pub(crate) fn release_pending_settlement(
        &self,
        record: &Arc<super::PendingRelationalPublicationSettlement>,
    ) {
        self.lifecycle.settlements.release(record);
    }

    pub(crate) fn settlement_admission_is_closed(&self) -> bool {
        self.lifecycle.settlements.is_closed()
    }

    #[cfg(test)]
    pub(crate) fn pending_settlement_count(&self) -> usize {
        self.lifecycle.settlements.pending_count()
    }

    #[cfg(test)]
    pub(crate) fn pending_settlement_contact_count(&self) -> u64 {
        self.lifecycle.settlements.contact_count()
    }

    #[cfg(test)]
    pub(crate) fn pending_settlement_owner_loss_count(&self) -> u64 {
        self.lifecycle.settlements.owner_loss_release_count()
    }
}

fn reap_expired_candidates(
    candidates: &mut HashMap<u64, RegisteredCandidate>,
    now: std::time::Instant,
) -> usize {
    let expired = candidates
        .iter()
        .filter_map(|(id, entry)| {
            (now >= entry.expires_at || entry.payload.strong_count() == 0).then_some(*id)
        })
        .collect::<Vec<_>>();
    for id in &expired {
        if let Some(payload) = candidates
            .remove(id)
            .and_then(|entry| entry.payload.upgrade())
        {
            payload
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .take();
        }
    }
    expired.len()
}
