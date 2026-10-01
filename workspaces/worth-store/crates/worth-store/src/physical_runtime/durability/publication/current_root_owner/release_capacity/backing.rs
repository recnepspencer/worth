//! Mandatory publication backing acquired by the pre-effect lease. Re-census
//! retained capacities on every admission, including canceled spare backing.

use worth_store_physical_format::{DurablePhysicalRootManifest, ReleaseCustodyHeadKeyV1};

use super::{ReleaseCertificateCapacityDenial as Denial, SelectedReleaseCustodyLedger};
use crate::physical_runtime::{
    recovery_residency::StoreRejoinResidentLedger, PhysicalRecoveryAllocationAdmission,
};

impl SelectedReleaseCustodyLedger {
    pub(super) fn publication_backing_bytes(&self) -> Option<u64> {
        self.checkpoint_heads
            .owned_heap_bytes()?
            .checked_add(self.effective_heads.owned_heap_bytes()?)?
            .checked_add(vector_heap_bytes(&self.pending_events)?)
    }

    pub(super) fn prepare_publication_backing(
        &mut self,
        allocation: PhysicalRecoveryAllocationAdmission,
        closure_bytes: u64,
        fence_bytes: u64,
        key: ReleaseCustodyHeadKeyV1,
    ) -> Result<Vec<u8>, Denial> {
        let already_live = self
            .publication_backing_bytes()
            .and_then(|bytes| bytes.checked_add(closure_bytes))
            .and_then(|bytes| bytes.checked_add(fence_bytes))
            .ok_or_else(|| {
                Denial::Resident(
                    crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial::SizeOverflow {
                        admitted: allocation.byte_limit(),
                    },
                )
            })?;
        let mut resident = StoreRejoinResidentLedger::from_retained_with_limit(
            allocation,
            already_live,
            allocation.byte_limit(),
        )
        .map_err(Denial::Resident)?;
        resident
            .grow_vec(&mut self.pending_events, 1)
            .map_err(Denial::Resident)?;
        self.effective_heads
            .reserve_for_key(key, &mut resident)
            .map_err(|denial| match denial {
                super::reopen::RecoveredReleaseLedgerDenial::Resident(cause) => {
                    Denial::Resident(cause)
                }
                super::reopen::RecoveredReleaseLedgerDenial::SelectedFactMismatch => {
                    Denial::SelectedFactMismatch
                }
            })?;
        // This bound also covers the first transition into the HEAD-bound
        // wire shape. The frame belongs to the pending fence until commit.
        resident
            .reserve_vec(DurablePhysicalRootManifest::maximum_encoding_scratch_bytes())
            .map_err(Denial::Resident)
    }
}

pub(in super::super) fn vector_heap_bytes<T>(values: &Vec<T>) -> Option<u64> {
    values
        .capacity()
        .checked_mul(std::mem::size_of::<T>())?
        .try_into()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_store_physical_format::store_namespace::{
        ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
    };

    fn admission(bytes: u64) -> PhysicalRecoveryAllocationAdmission {
        let identity = StoreNamespaceIdentityRecord::new(
            StoreNamespaceVersion::CURRENT,
            ProposedStoreIdentity::from_nonzero_bytes([1; 16]).unwrap(),
        )
        .published_identity();
        PhysicalRecoveryAllocationAdmission::new(identity, bytes)
    }

    #[test]
    fn canceled_scratch_does_not_forget_retained_publication_capacity() {
        let mut ledger = SelectedReleaseCustodyLedger::trusted_genesis();
        let key = ReleaseCustodyHeadKeyV1::new([1; 16], 1).unwrap();
        let scratch = ledger
            .prepare_publication_backing(admission(1 << 20), 128, 64, key)
            .unwrap();
        assert!(
            scratch.capacity() >= DurablePhysicalRootManifest::maximum_encoding_scratch_bytes()
        );
        let retained = ledger.publication_backing_bytes().unwrap();
        assert!(retained > 0);
        drop(scratch); // A proven pre-effect cancellation drops only lease scratch.
        assert_eq!(ledger.publication_backing_bytes(), Some(retained));
        assert!(matches!(
            ledger.prepare_publication_backing(admission(retained + 192), 128, 64, key),
            Err(Denial::Resident(
                crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { .. }
            ))
        ));
        assert_eq!(ledger.publication_backing_bytes(), Some(retained));
        assert!(ledger.pending_events.is_empty());
        assert_eq!(ledger.cumulative_dropped, 0);
    }

    #[test]
    fn backing_denial_preserves_selected_facts_before_growth() {
        let mut ledger = SelectedReleaseCustodyLedger::trusted_genesis();
        let key = ReleaseCustodyHeadKeyV1::new([1; 16], 1).unwrap();
        let bytes = std::mem::size_of::<super::super::PendingReleaseEvent>() as u64;
        assert!(matches!(
            ledger.prepare_publication_backing(admission(128 + bytes - 1), 128, 0, key),
            Err(Denial::Resident(
                crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { .. }
            ))
        ));
        assert_eq!(ledger.publication_backing_bytes(), Some(0));
        assert_eq!(ledger.effective_heads.root(), None);
        assert_eq!(ledger.cumulative_dropped, 0);
        assert_eq!(ledger.cumulative_digest, [0; 32]);
    }
}
