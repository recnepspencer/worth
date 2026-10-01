//! Exact disposal of the initial WAL inventory after its final reread agrees.

use super::{AdmittedWalInventory, Denial, ResidentDenial, StoreRejoinResidentLedger};

impl AdmittedWalInventory {
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn discard_with_resident(
        self,
        resident: &mut StoreRejoinResidentLedger,
    ) -> Result<(), Denial> {
        let overflow = || {
            Denial::Resident(ResidentDenial::SizeOverflow {
                admitted: resident.used().saturating_add(resident.remaining()),
            })
        };
        let total = resident
            .vector_bytes(&self.frames)
            .map_err(Denial::Resident)?
            .checked_add(
                resident
                    .vector_bytes(&self.artifacts)
                    .map_err(Denial::Resident)?,
            )
            .ok_or_else(overflow)?;
        let total = self.frames.iter().try_fold(total, |bytes, frame| {
            bytes.checked_add(frame.owned_heap_bytes()?)
        });
        let total = total.ok_or_else(overflow)?;
        drop(self);
        resident.release(total);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physical_runtime::{
        recovery_wal::WalSegmentArtifactIdentity, PhysicalRecoveryAllocationAdmission,
    };
    use worth_store_physical_format::store_namespace::{
        ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
    };
    use worth_store_wal::{WalSegmentGeneration, WalSegmentId};

    #[test]
    fn matched_initial_inventory_releases_only_its_backing() {
        let store = StoreNamespaceIdentityRecord::new(
            StoreNamespaceVersion::CURRENT,
            ProposedStoreIdentity::from_nonzero_bytes([1; 16]).unwrap(),
        )
        .published_identity();
        let admission = PhysicalRecoveryAllocationAdmission::new(store, 4096);
        let mut resident = StoreRejoinResidentLedger::for_test(admission, 100, 2048).unwrap();
        let mut inventory = || {
            let mut artifacts = resident.reserve_vec(1).unwrap();
            artifacts.push(super::super::WalArtifactFingerprint {
                identity: WalSegmentArtifactIdentity::new(
                    WalSegmentId::new(1).unwrap(),
                    WalSegmentGeneration::new(1).unwrap(),
                ),
                length: 0,
                sha256: [0; 32],
            });
            AdmittedWalInventory {
                frames: Vec::new(),
                artifacts,
            }
        };
        let first = inventory();
        let final_read = inventory();
        assert!(first.matches_reread(&final_read));
        let backing = resident.vector_bytes(&first.artifacts).unwrap();
        assert_eq!(resident.used(), 100 + 2 * backing);
        first.discard_with_resident(&mut resident).unwrap();
        assert_eq!(resident.used(), 100 + backing);
        let fingerprint = final_read
            .into_fingerprint_with_resident(&mut resident)
            .unwrap();
        assert_eq!(fingerprint.owned_heap_bytes(), Some(backing));
    }
}
