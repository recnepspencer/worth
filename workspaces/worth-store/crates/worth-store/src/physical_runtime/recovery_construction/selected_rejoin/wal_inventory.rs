//! Independent C4 reread and C9 WAL admission with continuously owned native storage.
mod admission;
mod resident_memory;
mod serving_freshness;
mod storage;
mod wal_bytes;
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
use wal_bytes::SelectedWalInventoryBudget;
pub use wal_bytes::{ExceededSelectedWalInventoryBound, SelectedWalInventoryBound};
use worth_store_physical_backend::{
    BoundedRecoveryFilesystemDiscovery, GrantedReadStop, ObservedWalArtifact,
};
use worth_store_physical_format::store_namespace::NamespaceEntryType;
const MAX_WAL_SEGMENTS: std::num::NonZeroU64 = std::num::NonZeroU64::new(4_096).unwrap();
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
    fn frame_memory(&self) -> Option<u64> {
        self.frames
            .iter()
            .try_fold(self.frames.owned_heap_bytes()?, |bytes, frame| {
                bytes.checked_add(frame.owned_heap_bytes()?)
            })
    }
}
impl SelectedWalMediaFingerprint {
    /// Frames are a deterministic validation of the artifact bytes, so equal
    /// identity, length and SHA-256 per artifact prove the reread admits the
    /// same frames without keeping the first frames alive beside it.
    pub(super) fn matches_reread(&self, reread: &AdmittedWalInventory) -> bool {
        self.artifacts == reread.artifacts
    }
    pub(in crate::physical_runtime) fn owned_heap_bytes(&self) -> Option<u64> {
        self.artifacts.owned_heap_bytes()
    }
    #[cfg(test)]
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
    admit_complete(
        discovery,
        coordination,
        SelectedWalInventoryBudget::complete(),
        None,
    )
}
pub(super) fn admit_complete_inventory_with_resident(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    coordination: &PhysicalRecoveryCoordination,
    resident: &mut StoreRejoinResidentLedger,
) -> Result<AdmittedWalInventory, Denial> {
    admit_complete(
        discovery,
        coordination,
        SelectedWalInventoryBudget::complete(),
        Some(resident),
    )
}
/// The inventory's budget grants its whole to the one read, so the backend
/// refuses the file that crosses it before reading it: no frame is retained
/// past what the retained-memory envelope budgets.
fn admit_complete(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    coordination: &PhysicalRecoveryCoordination,
    budget: SelectedWalInventoryBudget,
    mut resident: Option<&mut StoreRejoinResidentLedger>,
) -> Result<AdmittedWalInventory, Denial> {
    let mut window = PhysicalRecoveryReadAllocation::for_coordination(coordination)
        .map_err(Denial::WalReadOwnership)?;
    let raw = window
        .read_wal_payloads(discovery, MAX_WAL_SEGMENTS, budget.grant())
        .map_err(|stop| match stop {
            GrantedReadStop::PastGrant(overrun) => Denial::WalBytes {
                boundary: None,
                cause: SelectedWalInventoryBudget::refuse(overrun),
            },
            GrantedReadStop::Unread(cause) => Denial::WalRead {
                boundary: None,
                cause,
            },
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
