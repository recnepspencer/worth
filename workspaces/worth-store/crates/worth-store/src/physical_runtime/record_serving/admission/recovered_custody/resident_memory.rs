//! Heap backing retained by the one-shot Serving seal and loaded bootstrap.
//! The enclosing inline objects and a previous C.8 reopen are not charged here.

use worth_store_physical_integrity::VerifiedCheckpointStream;
use worth_store_recovery_physics::VerifiedOrderedRootHistory;

use super::RecoveredPhysicalCheckpointCustody;

impl RecoveredPhysicalCheckpointCustody {
    /// Read-only retained cost for sizing a Serving policy; it mints no grant.
    #[cfg(feature = "recovery-runtime-owner")]
    pub fn retained_heap_bytes(
        &self,
    ) -> Result<u64, crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial> {
        self.owned_heap_bytes().ok_or(
            crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial::SizeOverflow {
                admitted: self.recovery_allocation.byte_limit(),
            },
        )
    }

    /// Claim-local backing plus each distinct shared Arc target exactly once.
    /// This is a measurement, not a new recovery admission or Serving grant.
    pub(in crate::physical_runtime) fn owned_heap_bytes(&self) -> Option<u64> {
        let mut bytes = 0_u64;
        for claim in [
            self.released.as_ref().map(|claim| claim.owned_heap_bytes()),
            self.head_v2.as_ref().map(|claim| claim.owned_heap_bytes()),
            self.pending_wal_release
                .as_ref()
                .map(|claim| claim.owned_heap_bytes()),
            self.effective_release_heads
                .as_ref()
                .map(|claim| claim.owned_heap_bytes()),
            self.historical_release
                .as_ref()
                .map(|claim| claim.owned_heap_bytes()),
        ] {
            if let Some(claim_bytes) = claim {
                bytes = bytes.checked_add(claim_bytes?)?;
            }
        }
        #[cfg(feature = "recovery-runtime-owner")]
        {
            bytes = bytes.checked_add(self.selected_wal.owned_heap_bytes()?)?;
            if let Some(controls) = &self.selected_controls {
                bytes = bytes.checked_add(controls.owned_heap_bytes()?)?;
            }
        }
        bytes = bytes.checked_add(self.shared_checkpoint_bytes()?)?;
        bytes.checked_add(self.shared_history_bytes()?)
    }

    fn shared_checkpoint_bytes(&self) -> Option<u64> {
        let checkpoints: [Option<&VerifiedCheckpointStream>; 10] = [
            self.released.as_ref().map(|claim| claim.checkpoint()),
            self.head_v2.as_ref().map(|claim| claim.checkpoint()),
            self.no_release.as_ref().map(|claim| claim.checkpoint()),
            self.pending_wal_release
                .as_ref()
                .map(|claim| claim.checkpoint()),
            self.historical_release
                .as_ref()
                .map(|claim| claim.checkpoint()),
            self.tier.as_ref().map(|claim| claim.checkpoint()),
            self.pending_wal_release
                .as_ref()
                .and_then(|claim| claim.selected_release())
                .map(|base| base.checkpoint()),
            self.pending_wal_release
                .as_ref()
                .and_then(|claim| claim.addressed_release_base())
                .map(|base| base.checkpoint()),
            self.pending_wal_release
                .as_ref()
                .and_then(|claim| claim.selected_head_v2())
                .map(|base| base.checkpoint()),
            self.historical_release
                .as_ref()
                .and_then(|claim| claim.selected_head_v2())
                .map(|base| base.checkpoint()),
        ];
        let mut bytes = 0_u64;
        for (index, checkpoint) in checkpoints.iter().enumerate() {
            let Some(checkpoint) = checkpoint else {
                continue;
            };
            if checkpoints[..index]
                .iter()
                .flatten()
                .any(|prior| std::ptr::eq(*prior, *checkpoint))
            {
                continue;
            }
            bytes = bytes
                .checked_add(checkpoint.owned_heap_bytes()?)?
                .checked_add(u64::try_from(std::mem::size_of_val(*checkpoint)).ok()?)?
                .checked_add(u64::try_from(2 * std::mem::size_of::<usize>()).ok()?)?;
        }
        Some(bytes)
    }

    fn shared_history_bytes(&self) -> Option<u64> {
        let histories: [Option<&VerifiedOrderedRootHistory>; 2] = [
            self.pending_wal_release
                .as_ref()
                .and_then(|claim| claim.ordered_history()),
            self.historical_release
                .as_ref()
                .map(|claim| claim.history()),
        ];
        let mut bytes = 0_u64;
        for (index, history) in histories.iter().enumerate() {
            let Some(history) = history else { continue };
            if histories[..index]
                .iter()
                .flatten()
                .any(|prior| std::ptr::eq(*prior, *history))
            {
                continue;
            }
            bytes = bytes
                .checked_add(history.owned_heap_bytes()?)?
                .checked_add(u64::try_from(std::mem::size_of_val(*history)).ok()?)?
                .checked_add(u64::try_from(2 * std::mem::size_of::<usize>()).ok()?)?;
        }
        Some(bytes)
    }
}

impl crate::physical_runtime::record_serving::RecordServingState {
    /// The root, free header, policy, counters, and residue are inline values.
    pub(in crate::physical_runtime) fn owned_heap_bytes(&self) -> Option<u64> {
        vector_bytes(&self.displaced_artifacts)?
            .checked_add(vector_bytes(&self.publication_overheads)?)
    }
}

fn vector_bytes<T>(values: &Vec<T>) -> Option<u64> {
    u64::try_from(values.capacity())
        .ok()?
        .checked_mul(u64::try_from(std::mem::size_of::<T>()).ok()?)
}
