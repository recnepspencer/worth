//! Complete, bounded C.9 admission of the WAL artifacts seen by the same
//! Store-qualified media discovery used for selected control-frame rejoin.

#[path = "wal_inventory/resident_memory.rs"]
mod resident_memory;

use sha2::{Digest, Sha256};
use worth_store_physical_backend::{
    ArtifactTreeDirectory, BoundedRecoveryFilesystemDiscovery, ObservedWalArtifact,
    QualifiedFilesystemMedia,
};
use worth_store_physical_format::store_namespace::NamespaceEntryType;
use worth_store_physical_integrity::{
    validate_wal_frame_prefix, UntrustedPhysicalArtifact, WalFrameIntegrityValidation,
};

use crate::physical_runtime::{
    recovery_wal::{wal_frame_integrity_scope_identity, WalSegmentArtifactIdentity},
    IntegrityAdmittedRecoveryWalFrame, PhysicalRecoveryCoordination,
    PhysicalRecoveryWalResidentStage,
};

use super::resident::{
    discovery_allocation_denial, PhysicalRecoveryRejoinResidentDenial as ResidentDenial,
    StoreRejoinResidentLedger,
};
use super::SelectedMediaRejoinDenial as Denial;

const MAX_WAL_SEGMENTS: u64 = 4_096;
pub(super) const MAX_WAL_BYTES: u64 = 128 << 20;
const MAX_WAL_FRAMES: usize = 65_536;

pub(super) struct AdmittedWalInventory {
    frames: Vec<IntegrityAdmittedRecoveryWalFrame>,
    artifacts: Vec<WalArtifactFingerprint>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WalArtifactFingerprint {
    identity: WalSegmentArtifactIdentity,
    length: u64,
    sha256: [u8; 32],
}

/// Exact selected-WAL artifact inventory retained across the recovery-owner
/// to Serving-owner handoff. Only a C.9-admitted complete discovery mints it.
pub(in crate::physical_runtime) struct SelectedWalMediaFingerprint {
    artifacts: Vec<WalArtifactFingerprint>,
}

impl SelectedWalMediaFingerprint {
    pub(in crate::physical_runtime) fn matches_serving_media(
        &self,
        media: &QualifiedFilesystemMedia,
    ) -> bool {
        let Ok(directory) = ArtifactTreeDirectory::families().child("wal") else {
            return false;
        };
        let tree = media.artifact_tree();
        let Ok(exists) = tree.directory_exists(&directory) else {
            return false;
        };
        if !exists {
            return self.artifacts.is_empty();
        }
        let Ok(mut entries) = tree.list_bounded(&directory, MAX_WAL_SEGMENTS as usize) else {
            return false;
        };
        if entries.len() != self.artifacts.len() {
            return false;
        }
        // The directory's order is not authority. Sorting its already-owned
        // entry vector exposes duplicate names without another allocation.
        entries.sort_unstable_by(|left, right| left.name().cmp(right.name()));
        if entries
            .windows(2)
            .any(|pair| pair[0].name() == pair[1].name())
        {
            return false;
        }
        let mut remaining = MAX_WAL_BYTES;
        for entry in entries {
            if entry.entry_type() != NamespaceEntryType::RegularFile {
                return false;
            }
            let Some(name) = entry.name().to_str() else {
                return false;
            };
            let Some(identity) = WalSegmentArtifactIdentity::parse(name) else {
                return false;
            };
            let Ok(index) = self
                .artifacts
                .binary_search_by_key(&identity, |item| item.identity)
            else {
                return false;
            };
            let fingerprint = self.artifacts[index];
            let Some(next_remaining) = remaining.checked_sub(fingerprint.length) else {
                return false;
            };
            let Ok(file) = directory.file(name) else {
                return false;
            };
            let Ok(bytes) = tree.read_bounded(&file, fingerprint.length) else {
                return false;
            };
            if bytes.len() as u64 != fingerprint.length
                || <[u8; 32]>::from(Sha256::digest(&bytes)) != fingerprint.sha256
            {
                return false;
            }
            remaining = next_remaining;
        }
        true
    }
}

impl AdmittedWalInventory {
    /// Legacy callers retain their inventory after constructing a fingerprint.
    /// This clone is not the V2 carried-resident handoff.
    pub(super) fn fingerprint(&self) -> SelectedWalMediaFingerprint {
        SelectedWalMediaFingerprint {
            artifacts: self.artifacts.clone(),
        }
    }

    pub(super) fn into_fingerprint_with_resident(
        self,
        resident: &mut StoreRejoinResidentLedger,
    ) -> Result<SelectedWalMediaFingerprint, ResidentDenial> {
        let frame_backing = resident.vector_bytes(&self.frames)?;
        let frame_heap = self.frames.iter().try_fold(0_u64, |bytes, frame| {
            bytes.checked_add(frame.owned_heap_bytes()?)
        });
        let discarded = frame_backing
            .checked_add(frame_heap.ok_or(ResidentDenial::SizeOverflow {
                admitted: resident.used().saturating_add(resident.remaining()),
            })?)
            .ok_or(ResidentDenial::SizeOverflow {
                admitted: resident.used().saturating_add(resident.remaining()),
            })?;
        drop(self.frames);
        resident.release(discarded);
        Ok(SelectedWalMediaFingerprint {
            artifacts: self.artifacts,
        })
    }
    pub(super) fn frames(&self) -> &[IntegrityAdmittedRecoveryWalFrame] {
        &self.frames
    }

    /// Discovery incarnations differ; compare canonical artifact identity and
    /// cryptographic bytes, not the per-read observation token.
    pub(super) fn matches_reread(&self, other: &Self) -> bool {
        self.artifacts == other.artifacts
            && self.frames.len() == other.frames.len()
            && self
                .frames
                .iter()
                .zip(&other.frames)
                .all(|(before, after)| {
                    before.source_name() == after.source_name()
                        && before.scope() == after.scope()
                        && before.lsn_start() == after.lsn_start()
                        && before.lsn_end() == after.lsn_end()
                        && before.identity_digest() == after.identity_digest()
                        && before.payload_digest() == after.payload_digest()
                })
    }
}

impl SelectedWalMediaFingerprint {
    pub(in crate::physical_runtime) fn owned_heap_bytes(&self) -> Option<u64> {
        u64::try_from(self.artifacts.capacity())
            .ok()?
            .checked_mul(std::mem::size_of::<WalArtifactFingerprint>() as u64)
    }
}

pub(super) fn admit_complete_inventory(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    coordination: &PhysicalRecoveryCoordination,
) -> Result<AdmittedWalInventory, Denial> {
    let artifacts = discovery
        .read_wal_artifacts(MAX_WAL_SEGMENTS, MAX_WAL_BYTES)
        .map_err(Denial::Discovery)?;
    admit_artifacts(discovery, coordination, artifacts, None)
}

pub(super) fn admit_complete_inventory_with_resident(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    coordination: &PhysicalRecoveryCoordination,
    resident: &mut StoreRejoinResidentLedger,
) -> Result<AdmittedWalInventory, Denial> {
    let before = resident.used();
    let artifacts = discovery
        .read_wal_artifacts_with_payload_allocator(MAX_WAL_SEGMENTS, MAX_WAL_BYTES, |count| {
            resident.reserve_bytes(count)
        })
        .map_err(discovery_allocation_denial)?;
    let raw_payload_charge = resident.used() - before;
    let inventory = admit_artifacts(discovery, coordination, artifacts, Some(resident))?;
    // The raw C4 payload vectors have been dropped by admit_artifacts; the
    // independently admitted C9 frame copies remain separately charged.
    resident.release(raw_payload_charge);
    Ok(inventory)
}

fn admit_artifacts(
    discovery: &BoundedRecoveryFilesystemDiscovery,
    coordination: &PhysicalRecoveryCoordination,
    artifacts: Vec<ObservedWalArtifact>,
    mut resident: Option<&mut StoreRejoinResidentLedger>,
) -> Result<AdmittedWalInventory, Denial> {
    let mut frames = Vec::new();
    let fingerprints = admit_artifact_fingerprints(&artifacts, resident.as_deref_mut())?;
    let artifact_count = artifacts.len();
    for (artifact_ordinal, artifact) in artifacts.iter().enumerate() {
        let name = artifact.name().to_str().ok_or(Denial::WalFate)?;
        let identity = WalSegmentArtifactIdentity::parse(name).ok_or(Denial::WalFate)?;
        let bytes = artifact.bytes().ok_or(Denial::WalFate)?;
        let mut offset = 0_usize;
        while offset < bytes.len() {
            if frames.len() >= MAX_WAL_FRAMES {
                return Err(Denial::BoundExceeded);
            }
            let (validation, _) = validate_wal_frame_prefix(
                UntrustedPhysicalArtifact::from_bounded_bytes(&bytes[offset..]),
                discovery.store_identity(),
                wal_frame_integrity_scope_identity(identity),
                offset as u64,
            );
            let WalFrameIntegrityValidation::Intact(validated) = validation else {
                return Err(Denial::WalFate);
            };
            let scope = validated.scope();
            let next = usize::try_from(scope.byte_range().end_exclusive())
                .map_err(|_| Denial::BoundExceeded)?;
            if next <= offset || next > bytes.len() {
                return Err(Denial::WalFate);
            }
            if let Some(resident) = resident.as_deref_mut() {
                resident
                    .grow_vec_geometrically(&mut frames, 1)
                    .map_err(|cause| Denial::WalResident {
                        boundary: None,
                        stage: PhysicalRecoveryWalResidentStage::FrameRosterGrowth,
                        artifact_count,
                        artifact_ordinal: Some(artifact_ordinal),
                        frame_offset: Some(scope.byte_range().offset()),
                        cause,
                    })?;
                // The canonical observed name and exact validated frame range
                // bound the two owned copies made by C9 binding. The payload
                // projection borrows this encoded frame; it is not a third copy.
                let name_bytes = (artifact.name().as_encoded_bytes().len() as u64)
                    .checked_mul(if cfg!(windows) { 2 } else { 1 })
                    .ok_or(Denial::BoundExceeded)?;
                resident
                    .transient(
                        scope
                            .byte_range()
                            .length()
                            .checked_add(name_bytes)
                            .ok_or(Denial::BoundExceeded)?,
                    )
                    .map_err(|cause| Denial::WalResident {
                        boundary: None,
                        stage: PhysicalRecoveryWalResidentStage::EncodedFrameCopy,
                        artifact_count,
                        artifact_ordinal: Some(artifact_ordinal),
                        frame_offset: Some(scope.byte_range().offset()),
                        cause,
                    })?;
            }
            let admitted = coordination
                .admit_recovery_wal_frame(artifact, scope, scope.byte_range(), validated)
                .map_err(|_| Denial::WalFate)?;
            if let Some(resident) = resident.as_deref_mut() {
                resident
                    .retain(admitted.owned_heap_bytes().ok_or(Denial::BoundExceeded)?)
                    .map_err(|cause| Denial::WalResident {
                        boundary: None,
                        stage: PhysicalRecoveryWalResidentStage::AdmittedFrameRetain,
                        artifact_count,
                        artifact_ordinal: Some(artifact_ordinal),
                        frame_offset: Some(scope.byte_range().offset()),
                        cause,
                    })?;
            }
            frames.push(admitted);
            offset = next;
        }
    }
    // Directory enumeration order is not a selected-WAL ordering witness.
    // Artifact names are unique and each file's offsets strictly advance, so
    // this comparator is a total unique key. Stability carries no semantics;
    // use the allocation-free sort rather than allocating a second roster.
    frames.sort_unstable_by(|left, right| {
        left.lsn_start()
            .cmp(&right.lsn_start())
            .then_with(|| left.source_name().cmp(right.source_name()))
            .then_with(|| {
                left.scope()
                    .byte_range()
                    .offset()
                    .cmp(&right.scope().byte_range().offset())
            })
    });
    Ok(AdmittedWalInventory {
        frames,
        artifacts: fingerprints,
    })
}

fn admit_artifact_fingerprints(
    artifacts: &[ObservedWalArtifact],
    resident: Option<&mut StoreRejoinResidentLedger>,
) -> Result<Vec<WalArtifactFingerprint>, Denial> {
    if artifacts.len() > MAX_WAL_SEGMENTS as usize {
        return Err(Denial::BoundExceeded);
    }
    let mut fingerprints = if let Some(resident) = resident {
        resident
            .reserve_vec::<WalArtifactFingerprint>(artifacts.len())
            .map_err(|cause| Denial::WalResident {
                boundary: None,
                stage: PhysicalRecoveryWalResidentStage::FingerprintRoster,
                artifact_count: artifacts.len(),
                artifact_ordinal: None,
                frame_offset: None,
                cause,
            })?
    } else {
        let mut values = Vec::new();
        let requested = u64::try_from(artifacts.len())
            .ok()
            .and_then(|count| {
                count.checked_mul(std::mem::size_of::<WalArtifactFingerprint>() as u64)
            })
            .ok_or(Denial::BoundExceeded)?;
        values
            .try_reserve_exact(artifacts.len())
            .map_err(|cause| Denial::Resident(ResidentDenial::Allocation { requested, cause }))?;
        values
    };
    for artifact in artifacts {
        if artifact.entry_type() != NamespaceEntryType::RegularFile {
            return Err(Denial::WalFate);
        }
        let name = artifact.name().to_str().ok_or(Denial::WalFate)?;
        let identity = WalSegmentArtifactIdentity::parse(name).ok_or(Denial::WalFate)?;
        let bytes = artifact.bytes().ok_or(Denial::WalFate)?;
        let digest: [u8; 32] = Sha256::digest(bytes).into();
        fingerprints.push(WalArtifactFingerprint {
            identity,
            length: bytes.len() as u64,
            sha256: digest,
        });
    }
    finish_inventory_identity(&mut fingerprints)?;
    Ok(fingerprints)
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
#[path = "wal_inventory/tests.rs"]
mod tests;
