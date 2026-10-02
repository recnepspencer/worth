//! C9 derivation while the independently funded raw C4 inventory remains live.
use super::*;
use crate::physical_runtime::{
    recovery_wal::wal_frame_integrity_scope_identity, PhysicalRecoveryWalResidentStage as Stage,
};
use worth_store_physical_integrity::{
    validate_wal_frame_prefix, UntrustedPhysicalArtifact, WalFrameIntegrityValidation,
};
pub(super) fn admit_artifacts(
    discovery: &BoundedRecoveryFilesystemDiscovery,
    coordination: &PhysicalRecoveryCoordination,
    artifacts: &[ObservedWalArtifact],
    mut resident: Option<&mut StoreRejoinResidentLedger>,
) -> Result<AdmittedWalInventory, Denial> {
    let mut frames = WalRoster::empty();
    let fingerprints = fingerprints(coordination, artifacts, resident.as_deref_mut())?;
    for (ordinal, artifact) in artifacts.iter().enumerate() {
        let identity = artifact
            .name()
            .to_str()
            .and_then(WalSegmentArtifactIdentity::parse)
            .ok_or(Denial::WalFate)?;
        let bytes = artifact.bytes().ok_or(Denial::WalFate)?;
        let mut offset = 0usize;
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
            if frames.len() == frames.capacity() {
                let old = frames.owned_heap_bytes().ok_or(Denial::BoundExceeded)?;
                let prospective = storage::bytes::<IntegrityAdmittedRecoveryWalFrame>(
                    frames.next_capacity().map_err(native)?,
                )
                .map_err(native)?;
                if let Some(resident) = resident.as_deref_mut() {
                    resident.retain(prospective).map_err(|cause| {
                        numeric(
                            cause,
                            Stage::FrameRosterGrowth,
                            artifacts.len(),
                            Some(ordinal),
                            Some(scope.byte_range().offset()),
                        )
                    })?;
                }
                frames.reserve_one(coordination).map_err(native)?;
                if let Some(resident) = resident.as_deref_mut() {
                    resident.release(old);
                }
            }
            let admitted = coordination
                .admit_recovery_wal_frame(artifact, scope, scope.byte_range(), validated)
                .map_err(|cause| match cause {
                    crate::physical_runtime::RecoveryWalIntegrityAdmissionDenial::Allocation(
                        cause,
                    ) => native(cause),
                    _ => Denial::WalFate,
                })?;
            if let Some(resident) = resident.as_deref_mut() {
                resident
                    .retain(admitted.owned_heap_bytes().ok_or(Denial::BoundExceeded)?)
                    .map_err(|cause| {
                        numeric(
                            cause,
                            Stage::AdmittedFrameRetain,
                            artifacts.len(),
                            Some(ordinal),
                            Some(scope.byte_range().offset()),
                        )
                    })?;
            }
            frames.push_reserved(admitted);
            offset = next;
        }
    }
    frames.as_mut_slice().sort_unstable_by(|left, right| {
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
fn fingerprints(
    coordination: &PhysicalRecoveryCoordination,
    artifacts: &[ObservedWalArtifact],
    resident: Option<&mut StoreRejoinResidentLedger>,
) -> Result<WalRoster<WalArtifactFingerprint>, Denial> {
    if artifacts.len() > MAX_WAL_SEGMENTS as usize {
        return Err(Denial::BoundExceeded);
    }
    if let Some(resident) = resident {
        let requested =
            storage::bytes::<WalArtifactFingerprint>(artifacts.len()).map_err(native)?;
        resident.retain(requested).map_err(|cause| {
            numeric(cause, Stage::FingerprintRoster, artifacts.len(), None, None)
        })?;
    }
    let mut fingerprints =
        WalRoster::with_capacity(coordination, artifacts.len()).map_err(native)?;
    for artifact in artifacts {
        if artifact.entry_type() != NamespaceEntryType::RegularFile {
            return Err(Denial::WalFate);
        }
        let identity = artifact
            .name()
            .to_str()
            .and_then(WalSegmentArtifactIdentity::parse)
            .ok_or(Denial::WalFate)?;
        let bytes = artifact.bytes().ok_or(Denial::WalFate)?;
        fingerprints.push_reserved(WalArtifactFingerprint {
            identity,
            length: bytes.len() as u64,
            sha256: Sha256::digest(bytes).into(),
        });
    }
    finish_inventory_identity(fingerprints.as_mut_slice())?;
    Ok(fingerprints)
}
fn native(cause: crate::physical_runtime::RecoveryWalAllocationDenial) -> Denial {
    Denial::WalAdmission {
        boundary: None,
        cause,
    }
}
fn numeric(
    cause: ResidentDenial,
    stage: Stage,
    artifact_count: usize,
    artifact_ordinal: Option<usize>,
    frame_offset: Option<u64>,
) -> Denial {
    Denial::WalResident {
        boundary: None,
        stage,
        artifact_count,
        artifact_ordinal,
        frame_offset,
        cause,
    }
}
