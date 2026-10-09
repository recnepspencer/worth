use super::{ArenaAllocationDenial, ExtentArenaAllocationOwner};
use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc, Mutex,
};
use worth_store_physical_format::{ExtentArenaRange, PhysicalTierClass};

pub(in crate::physical_runtime::record_serving) type SharedArenaAllocationOwner =
    Arc<Mutex<ExtentArenaAllocationOwner>>;

/// One allocation claim. Dropping a pre-WAL claim proves no media authority
/// escaped. Once exposed to WAL it remains unavailable until root commitment
/// or recovery, even when every transient carrier is dropped.
pub(in crate::physical_runtime::record_serving) struct ArenaReservation {
    claim: Arc<ArenaReservationClaim>,
}

struct ArenaReservationClaim {
    owner: SharedArenaAllocationOwner,
    token: u64,
    range: ExtentArenaRange,
    state: AtomicU8,
}

/// Retains one allocation obligation without granting publication authority.
pub(in crate::physical_runtime::record_serving) struct ArenaReservationObligation {
    claim: Arc<ArenaReservationClaim>,
}

impl ArenaReservation {
    pub(super) fn from_reserved(
        owner: &SharedArenaAllocationOwner,
        token: u64,
        range: ExtentArenaRange,
    ) -> Self {
        Self {
            claim: Arc::new(ArenaReservationClaim {
                owner: Arc::clone(owner),
                token,
                range,
                state: AtomicU8::new(0),
            }),
        }
    }

    pub(in crate::physical_runtime::record_serving) fn reserve(
        owner: &SharedArenaAllocationOwner,
        bytes: u64,
    ) -> Result<Self, ArenaAllocationDenial> {
        Self::reserve_in_tier(owner, bytes, PhysicalTierClass::Primary)
    }

    pub(in crate::physical_runtime::record_serving) fn reserve_in_tier(
        owner: &SharedArenaAllocationOwner,
        bytes: u64,
        tier: PhysicalTierClass,
    ) -> Result<Self, ArenaAllocationDenial> {
        let (token, range) = owner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .reserve_in_tier(bytes, tier)?;
        Ok(Self::from_reserved(owner, token, range))
    }

    pub(in crate::physical_runtime::record_serving) fn belongs_to(
        &self,
        owner: &SharedArenaAllocationOwner,
    ) -> bool {
        Arc::ptr_eq(&self.claim.owner, owner)
    }

    pub(in crate::physical_runtime::record_serving) fn range(&self) -> ExtentArenaRange {
        self.claim.range
    }

    pub(in crate::physical_runtime::record_serving) fn retain_obligation(
        &self,
    ) -> ArenaReservationObligation {
        ArenaReservationObligation {
            claim: Arc::clone(&self.claim),
        }
    }

    pub(in crate::physical_runtime::record_serving) fn is_live(&self) -> bool {
        self.claim.state.load(Ordering::Acquire) < 2
    }

    pub(in crate::physical_runtime::record_serving) fn expose_to_wal(&self) {
        let _ = self
            .claim
            .state
            .compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire);
    }

    pub(in crate::physical_runtime::record_serving) fn wal_proven_no_effect(&self) {
        let _ = self
            .claim
            .state
            .compare_exchange(1, 0, Ordering::AcqRel, Ordering::Acquire);
    }

    pub(in crate::physical_runtime::record_serving) fn publish(self) {
        let mut owner = self.claim.owner.lock().unwrap_or_else(|e| e.into_inner());
        assert!(self.is_live(), "one reservation may publish only once");
        owner
            .published(self.claim.token)
            .expect("the sealed reservation belongs to this allocation owner");
        self.claim.state.store(2, Ordering::Release);
    }
}

impl ArenaReservationObligation {
    pub(in crate::physical_runtime::record_serving) fn restore(
        owner: &SharedArenaAllocationOwner,
        range: ExtentArenaRange,
    ) -> Result<Self, ArenaAllocationDenial> {
        let token = owner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .restore_claim(range)?;
        Ok(Self {
            claim: Arc::new(ArenaReservationClaim {
                owner: Arc::clone(owner),
                token,
                range,
                state: AtomicU8::new(1),
            }),
        })
    }

    /// The caller has settled the durable abandon resolution and quiesced all
    /// copy effects. This handle cannot be used to publish a destination.
    pub(in crate::physical_runtime::record_serving) fn cancel_after_resolution(
        &self,
    ) -> Result<(), ()> {
        let mut owner = self.claim.owner.lock().unwrap_or_else(|e| e.into_inner());
        if self.claim.state.load(Ordering::Acquire) >= 2 {
            return Err(());
        }
        owner.cancel(self.claim.token).map_err(|_| ())?;
        self.claim.state.store(3, Ordering::Release);
        Ok(())
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime::record_serving) fn certification_has_exact_claim(
        &self,
        range: ExtentArenaRange,
    ) -> bool {
        self.claim.range == range
            && self.claim.state.load(Ordering::Acquire) == 1
            && self
                .claim
                .owner
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .certification_has_exact_claim(self.claim.token, range)
    }
}

impl Drop for ArenaReservationClaim {
    fn drop(&mut self) {
        if self.state.load(Ordering::Acquire) == 0 {
            self.owner
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .cancel(self.token)
                .expect("a pre-WAL reservation is still exclusively owned");
        }
    }
}
