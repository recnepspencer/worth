use contracts::publication::close_custody::RelationalPublicationCompanionCustody;
use contracts::publication::close_custody::RelationalPublicationCustodyHolders;
use contracts::publication::close_custody::RelationalPublicationCustodyHoldersOwnerPort as _;
use contracts::publication::close_custody::RelationalPublicationRegistryCustody;
use contracts::publication::close_custody::RelationalPublicationRegistryCustodyOwnerPort as _;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};
use worth_relational_contracts::internal_port as contracts;
pub mod source_registration;

/// Drop-governed publication authority owned directly by one runtime.
#[derive(Debug)]
pub struct RelationalRuntimePublicationOwner {
    binding: RelationalRuntimePublicationBinding,
}

/// Cloneable runtime binding carried by independently borrowable publication artifacts.
#[derive(Debug, Clone)]
pub struct RelationalRuntimePublicationBinding {
    pub(super) lifecycle: Arc<RelationalRuntimePublicationLifecycle>,
}

#[derive(Debug)]
pub(super) struct RelationalRuntimePublicationLifecycle {
    pub(super) companion: Arc<crate::mvcc::publication::CompanionRegistry>,
    next_candidate_id: AtomicU64,
    candidates: Mutex<HashMap<u64, RegisteredCandidate>>,
    pub(super) settlements: Arc<crate::runtime::state::runtime_state::publication_settlement_registry::RelationalPublicationSettlementRegistry>,
}

/// A live candidate's registration. The candidate handle holds the payload's
/// only strong reference, so a dead weak reference means the handle was leaked
/// without its `Drop`; nothing else evicts a registration.
#[derive(Debug)]
struct RegisteredCandidate {
    payload: Weak<Mutex<Option<crate::mvcc::publication::CandidatePayload>>>,
}

pub(crate) enum RelationalCandidateRegistrationDenial {
    CapacityExhausted { maximum_candidates: usize },
    IdentityExhausted,
}

#[derive(Debug)]
pub struct PublicationCloseContended;

impl RelationalRuntimePublicationOwner {
    /// Unlike ordinary close, this never reaps candidates or drains settlements.
    /// A retained candidate owns real reservations until its holder drops it.
    pub fn try_memory_close_custody(
        &self,
    ) -> Result<Option<RelationalPublicationCustodyHolders>, PublicationCloseContended> {
        let candidates = match self.binding.lifecycle.candidates.try_lock() {
            Ok(value) => value,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Err(PublicationCloseContended),
        };
        let candidate_custody = RelationalPublicationRegistryCustody::observed(
            candidates.len(),
            candidates.keys().next().copied(),
        );
        drop(candidates);
        RelationalPublicationCustodyHolders {
            candidates: candidate_custody,
            settlements: self.binding.lifecycle.settlements.try_close_custody(),
            companion: self.companion_close_custody(),
        }
        .memory_result()
        .map_err(|()| PublicationCloseContended)
    }
    pub fn live_publication_custody(&self) -> Option<RelationalPublicationCustodyHolders> {
        self.binding.reap_abandoned_candidates();
        let candidates = self
            .binding
            .lifecycle
            .candidates
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let candidate_custody = RelationalPublicationRegistryCustody::observed(
            candidates.len(),
            candidates.keys().next().copied(),
        );
        drop(candidates);
        RelationalPublicationCustodyHolders {
            candidates: candidate_custody,
            settlements: self.binding.lifecycle.settlements.close_custody(),
            companion: self.companion_close_custody(),
        }
        .retained()
    }

    fn companion_close_custody(&self) -> RelationalPublicationCompanionCustody {
        match self.binding.lifecycle.companion.enter() {
            Ok(epoch) => epoch.close_custody(),
            Err(stop) => RelationalPublicationCompanionCustody::Unavailable(stop),
        }
    }

    pub fn standalone() -> Self {
        Self::with_companion(crate::mvcc::publication::CompanionRegistry::new())
    }

    pub fn fork_from(source: &Self) -> Self {
        Self::with_companion(source.binding.lifecycle.companion.fork())
    }

    fn with_companion(companion: crate::mvcc::publication::CompanionRegistry) -> Self {
        Self {
            binding: RelationalRuntimePublicationBinding {
                lifecycle: Arc::new(RelationalRuntimePublicationLifecycle {
                    companion: Arc::new(companion),
                    next_candidate_id: AtomicU64::new(1),
                    candidates: Mutex::new(HashMap::new()),
                    settlements: Arc::new(crate::runtime::state::runtime_state::publication_settlement_registry::RelationalPublicationSettlementRegistry::default()),
                }),
            },
        }
    }

    pub fn binding(&self) -> RelationalRuntimePublicationBinding {
        self.binding.clone()
    }

    pub fn held_only_by_owner_and_close(
        &self,
        close: &RelationalRuntimePublicationBinding,
    ) -> bool {
        // State and its close authority are the two native owner handles.
        // Prepared candidates and independently bound services retain another.
        Arc::ptr_eq(&self.binding.lifecycle, &close.lifecycle)
            && Arc::strong_count(&self.binding.lifecycle) == 2
            && Arc::weak_count(&self.binding.lifecycle) == 0
            && self
                .binding
                .lifecycle
                .companion
                .enter()
                .is_ok_and(|epoch| matches!(epoch.active(), Ok(None)))
    }
}

impl RelationalRuntimePublicationBinding {
    #[doc(hidden)]
    pub fn prepared_candidate_count_for_test(&self) -> usize {
        self.lifecycle
            .candidates
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }

    pub fn companion_registry(&self) -> &Arc<crate::mvcc::publication::CompanionRegistry> {
        &self.lifecycle.companion
    }

    pub fn companion_epoch(
        &self,
    ) -> Result<
        crate::mvcc::publication::CompanionRegistrationEpoch<'_>,
        contracts::mvcc::publication::companion::registration_stop::PublicationCompanionRegistrationStop,
    >{
        self.lifecycle.companion.enter()
    }
    /// Close settlement admission and resolve remaining retention with typed
    /// owner-loss accounting. Owner authority, so it stays inside the runtime
    /// module tree even though the binding itself is carried by services.
    pub fn close(&self) {
        self.lifecycle.settlements.close();
    }

    pub(crate) fn register_candidate(
        &self,
        maximum_candidates: Option<usize>,
        payload: &Arc<Mutex<Option<crate::mvcc::publication::CandidatePayload>>>,
    ) -> Result<u64, RelationalCandidateRegistrationDenial> {
        let mut candidates = self
            .lifecycle
            .candidates
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(maximum_candidates) =
            maximum_candidates.filter(|maximum| candidates.len() >= *maximum)
        {
            return Err(RelationalCandidateRegistrationDenial::CapacityExhausted {
                maximum_candidates,
            });
        }
        let id = {
            let update_counter = &self.lifecycle.next_candidate_id;
            let mut current = update_counter.load(Ordering::Relaxed);
            loop {
                let Some(updated_value) =
                    (current != 0).then(|| current.checked_add(1).unwrap_or(0))
                else {
                    break Err(current);
                };
                match update_counter.compare_exchange_weak(
                    current,
                    updated_value,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(previous) => break Ok(previous),
                    Err(reloaded_value) => current = reloaded_value,
                }
            }
        }
        .map_err(|_| RelationalCandidateRegistrationDenial::IdentityExhausted)?;
        assert_ne!(
            /* Proof: register_candidate obtains the id from the checked next_candidate_id allocator, which rejects zero and permanently exhausts on wrap. That private allocator is never reset in a live binding, and registration/insertion holds the candidate-map Mutex throughout. */
            id,
            0,
            "candidate identity zero is reserved"
        );
        assert!(
            /* Proof: register_candidate obtains the id from the checked next_candidate_id allocator, which rejects zero and permanently exhausts on wrap. That private allocator is never reset in a live binding, and registration/insertion holds the candidate-map Mutex throughout. */
            !candidates.contains_key(&id),
            "candidate identity allocator collided with a live candidate"
        );
        let replaced = candidates.insert(
            id,
            RegisteredCandidate {
                payload: Arc::downgrade(payload),
            },
        );
        debug_assert!(
            /* Proof: register_candidate obtains the id from the checked next_candidate_id allocator, which rejects zero and permanently exhausts on wrap. That private allocator is never reset in a live binding, and registration/insertion holds the candidate-map Mutex throughout. */
            replaced.is_none()
        );
        Ok(id)
    }

    /// Remove a consumed candidate's registration; the consuming receiver
    /// takes the payload from its own strong reference.
    pub(crate) fn release_candidate_registration(&self, candidate_id: u64) {
        self.lifecycle
            .candidates
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(&candidate_id);
    }

    pub(crate) fn discard_candidate(&self, candidate_id: u64) {
        let registered = self
            .lifecycle
            .candidates
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(&candidate_id);
        if let Some(payload) = registered.and_then(|entry| entry.payload.upgrade()) {
            payload
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .take();
        }
    }

    fn reap_abandoned_candidates(&self) {
        self.lifecycle
            .candidates
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .retain(|_, entry| entry.payload.strong_count() > 0);
    }

    pub fn belongs_to_same_owner(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.lifecycle, &other.lifecycle)
    }

    /// Install the one pending-settlement record for this attempt before the
    /// publication critical section, so a moved branch head is never observable
    /// without its owner recovery record.
    pub fn reserve_pending_settlement(
        &self,
        commit_id: contracts::history::data::coordinates::CommitId,
        runtime_instance_id: u64,
        reserved: crate::runtime::state::runtime_state::publication_settlement_registry::ReservedRelationalSettlement,
    ) -> Result<
        crate::runtime::state::runtime_state::publication_settlement_registry::RelationalPendingSettlementReservation,
        crate::runtime::state::runtime_state::publication_settlement_registry::RelationalSettlementReservationDenial,
    >{
        crate::runtime::state::runtime_state::publication_settlement_registry::RelationalPublicationSettlementRegistry::reserve(
            &self.lifecycle.settlements,
            commit_id,
            runtime_instance_id,
            reserved,
        )
    }

    pub fn pending_settlement(
        &self,
        commit_id: contracts::history::data::coordinates::CommitId,
    ) -> Option<Arc<crate::runtime::state::runtime_state::publication_settlement_registry::PendingRelationalPublicationSettlement>>{
        self.lifecycle.settlements.record(commit_id)
    }

    /// Whether a commit at or before `basis` is published but unsettled.
    pub fn has_unsettled_publication_through(
        &self,
        basis: contracts::history::data::coordinates::CommitId,
    ) -> bool {
        self.lifecycle.settlements.has_unsettled_through(basis)
    }

    pub(crate) fn release_pending_settlement(
        &self,
        record: &Arc<crate::runtime::state::runtime_state::publication_settlement_registry::PendingRelationalPublicationSettlement>,
    ) {
        self.lifecycle.settlements.release(record);
    }

    pub fn settlement_admission_is_closed(&self) -> bool {
        self.lifecycle.settlements.is_closed()
    }

    #[doc(hidden)]
    pub fn pending_settlement_count_for_test(&self) -> usize {
        self.lifecycle.settlements.pending_count()
    }

    #[doc(hidden)]
    pub fn pending_settlement_contact_count_for_test(&self) -> u64 {
        self.lifecycle.settlements.contact_count()
    }

    #[doc(hidden)]
    pub fn pending_settlement_owner_loss_count_for_test(&self) -> u64 {
        self.lifecycle.settlements.owner_loss_release_count()
    }
}

impl RelationalRuntimePublicationBinding {
    #[doc(hidden)]
    pub fn while_empty_candidate_registry_locked_for_test(&self, action: impl FnOnce()) {
        let locked = self.lifecycle.candidates.lock().unwrap();
        action();
        assert!(locked.is_empty());
    }
}
