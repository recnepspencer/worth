//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPendingApplicationPublicationRegistry {
    #[doc(hidden)]
    pub fn corrections_for_test(&self) -> &BTreeMap<[u8; 32], ProductBranchIncarnation> {
        &self.corrections
    }
}

impl WorthQueryApplicationPublicationRecoveryReservation {
    #[doc(hidden)]
    pub fn slot_for_test(&self) -> &Arc<WorthQueryApplicationPublicationRecoverySlot> {
        &self.slot
    }
}

impl WorthQueryPendingApplicationPublicationRegistry {
    #[doc(hidden)]
    pub fn correction_slot_for_test(
        &self,
        key: &[u8; 32],
    ) -> Option<Arc<WorthQueryApplicationPublicationRecoverySlot>> {
        Self::correction_slot(self, key)
    }
}

impl WorthQueryPendingApplicationPublicationRegistry {
    #[doc(hidden)]
    pub fn new_for_test(maximum_slots: usize) -> Self {
        Self::new(maximum_slots)
    }
}

impl WorthQueryPendingApplicationPublicationRegistry {
    #[doc(hidden)]
    pub fn reserve_for_test(
        registry: &Arc<Mutex<Self>>,
        occurrence: ProductBranchIncarnation,
        binding: Option<WorthQueryApplicationIdempotencyBinding>,
    ) -> Result<WorthQueryApplicationPublicationRecoveryReservation, &'static str> {
        Self::reserve(registry, occurrence, binding)
    }
}

impl WorthQueryApplicationPublicationRecoverySlot {
    #[doc(hidden)]
    pub fn inspect_idempotency_for_test(&self) -> Option<WorthQueryPendingApplicationIdempotency> {
        Self::inspect_idempotency(self)
    }
}
