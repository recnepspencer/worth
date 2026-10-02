//! Independent C4 reread and C9 WAL admission with continuously owned native storage.
mod admission;
mod resident_memory;
mod serving_freshness;
mod storage;
use super::resident::{
    PhysicalRecoveryRejoinResidentDenial as ResidentDenial, StoreRejoinResidentLedger,
};
use super::SelectedMediaRejoinDenial as Denial;
use crate::physical_runtime::{
    recovery_wal::WalSegmentArtifactIdentity, IntegrityAdmittedRecoveryWalFrame,
    PhysicalRecoveryCoordination, PhysicalRecoveryReadAllocation,
};
use sha2::{Digest, Sha256};
use storage::WalRoster;
use worth_store_physical_backend::{BoundedRecoveryFilesystemDiscovery, ObservedWalArtifact};
use worth_store_physical_format::store_namespace::NamespaceEntryType;
const MAX_WAL_SEGMENTS: u64 = 4_096;
pub(super) const MAX_WAL_BYTES: u64 = 128 << 20;
const MAX_WAL_FRAMES: usize = 65_536;
pub(super) struct AdmittedWalInventory {
    frames: WalRoster<IntegrityAdmittedRecoveryWalFrame>,
    artifacts: WalRoster<WalArtifactFingerprint>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WalArtifactFingerprint {
    identity: WalSegmentArtifactIdentity,
    length: u64,
    sha256: [u8; 32],
}
/// The roster retains its actual native owner through the Serving handoff.
pub(in crate::physical_runtime) struct SelectedWalMediaFingerprint {
    artifacts: WalRoster<WalArtifactFingerprint>,
}
impl AdmittedWalInventory {
    pub(super) fn into_fingerprint(self) -> SelectedWalMediaFingerprint {
        drop(self.frames);
        SelectedWalMediaFingerprint {
            artifacts: self.artifacts,
        }
    }
    pub(super) fn into_fingerprint_with_resident(
        self,
        resident: &mut StoreRejoinResidentLedger,
    ) -> Result<SelectedWalMediaFingerprint, ResidentDenial> {
        let discarded = self.frame_memory().ok_or_else(|| overflow(resident))?;
        drop(self.frames);
        resident.release(discarded);
        Ok(SelectedWalMediaFingerprint {
            artifacts: self.artifacts,
        })
    }
    pub(super) fn frames(&self) -> &[IntegrityAdmittedRecoveryWalFrame] {
        &self.frames
    }
    pub(super) fn matches_reread(&self, other: &Self) -> bool {
        self.artifacts == other.artifacts
            && self.frames.len() == other.frames.len()
            && self
                .frames
                .iter()
                .zip(other.frames.iter())
                .all(|(before, after)| {
                    before.source_name() == after.source_name()
                        && before.scope() == after.scope()
                        && before.lsn_start() == after.lsn_start()
                        && before.lsn_end() == after.lsn_end()
                        && before.identity_digest() == after.identity_digest()
                        && before.payload_digest() == after.payload_digest()
                })
    }
    fn frame_memory(&self) -> Option<u64> {
        self.frames
            .iter()
            .try_fold(self.frames.owned_heap_bytes()?, |bytes, frame| {
                bytes.checked_add(frame.owned_heap_bytes()?)
            })
    }
}
impl SelectedWalMediaFingerprint {
    pub(in crate::physical_runtime) fn owned_heap_bytes(&self) -> Option<u64> {
        self.artifacts.owned_heap_bytes()
    }
    pub(in crate::physical_runtime) fn charged_bytes(&self) -> u64 {
        self.artifacts.charged_bytes()
    }
}
fn overflow(resident: &StoreRejoinResidentLedger) -> ResidentDenial {
    ResidentDenial::SizeOverflow {
        admitted: resident.used().saturating_add(resident.remaining()),
    }
}
pub(super) fn admit_complete_inventory(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    coordination: &PhysicalRecoveryCoordination,
) -> Result<AdmittedWalInventory, Denial> {
    admit_complete(discovery, coordination, None)
}
pub(super) fn admit_complete_inventory_with_resident(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    coordination: &PhysicalRecoveryCoordination,
    resident: &mut StoreRejoinResidentLedger,
) -> Result<AdmittedWalInventory, Denial> {
    admit_complete(discovery, coordination, Some(resident))
}
fn admit_complete(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    coordination: &PhysicalRecoveryCoordination,
    mut resident: Option<&mut StoreRejoinResidentLedger>,
) -> Result<AdmittedWalInventory, Denial> {
    let mut window = PhysicalRecoveryReadAllocation::for_coordination(coordination)
        .map_err(Denial::WalReadOwnership)?;
    let raw = window
        .read_wal_payloads(discovery, MAX_WAL_SEGMENTS, MAX_WAL_BYTES)
        .map_err(|cause| Denial::WalRead {
            boundary: None,
            cause,
        })?;
    let raw_charge = raw.charged_bytes();
    if let Some(resident) = resident.as_deref_mut() {
        resident.retain(raw_charge).map_err(Denial::Resident)?;
    }
    let inventory = admission::admit_artifacts(
        discovery,
        coordination,
        raw.artifacts(),
        resident.as_deref_mut(),
    )?;
    drop(raw);
    drop(window);
    if let Some(resident) = resident {
        resident.release(raw_charge);
    }
    Ok(inventory)
}
fn finish_inventory_identity(fingerprints: &mut [WalArtifactFingerprint]) -> Result<(), Denial> {
    fingerprints.sort_unstable_by_key(|entry| entry.identity);
    if fingerprints
        .windows(2)
        .any(|pair| pair[0].identity == pair[1].identity)
    {
        return Err(Denial::WalFate);
    }
    Ok(())
}
#[cfg(test)]
mod tests;
