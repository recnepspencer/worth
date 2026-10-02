//! One-shot bridge from an independently verified C.8 selection to Store
//! serving admission. The sealed witness is never constructed from raw bytes.

use sha2::{Digest, Sha256};
use worth_store_physical_backend::QualifiedFilesystemMedia;
use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_physical_format::{
    decode_checkpoint_certificate, CheckpointCertificateKind, DurableFreeSpaceManifestHeader,
    DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration, ReleaseCheckpointCertificateV1,
};
use worth_store_recovery_physics::{
    VerifiedEffectiveReleaseHeadRosterV14, VerifiedOrderedHistoricalReleaseCustody,
    VerifiedPendingWalReleaseCustody, VerifiedSelectedNoReleaseCustody,
    VerifiedSelectedReleaseHeadCustodyV2, VerifiedSelectedTierEpochCustody,
};

#[path = "recovered_custody/checkpoint.rs"]
mod checkpoint;
#[path = "recovered_custody/construction.rs"]
mod construction;
#[path = "recovered_custody/head_v2.rs"]
mod head_v2;
#[cfg(feature = "recovery-runtime-owner")]
#[path = "recovered_custody/media_freshness.rs"]
mod media_freshness;
#[path = "recovered_custody/pending.rs"]
mod pending;
#[cfg(feature = "recovery-runtime-owner")]
pub(in crate::physical_runtime) use media_freshness::FundedMediaVerifiedRecoveredCustodyEvidence;
#[cfg(feature = "recovery-runtime-owner")]
#[path = "recovered_custody/resident_memory.rs"]
mod resident_memory;

pub struct RecoveredPhysicalCheckpointCustody {
    residency: crate::physical_runtime::instance::PhysicalResidencyOwner,
    evidence: RecoveredCheckpointCustodyEvidence,
}

pub(in crate::physical_runtime) struct RecoveredCheckpointCustodyEvidence {
    #[cfg(feature = "recovery-runtime-owner")]
    checkpoint_ownership:
        crate::physical_runtime::recovery_coordination::RecoveryCheckpointOwnership,
    store: StableStoreIdentity,
    recovery_allocation: crate::physical_runtime::PhysicalRecoveryAllocationAdmission,
    root: DurablePhysicalRootManifest,
    head_v2: Option<VerifiedSelectedReleaseHeadCustodyV2>,
    no_release: Option<VerifiedSelectedNoReleaseCustody>,
    pending_wal_release: Option<VerifiedPendingWalReleaseCustody>,
    effective_release_heads: Option<VerifiedEffectiveReleaseHeadRosterV14>,
    historical_release: Option<VerifiedOrderedHistoricalReleaseCustody>,
    tier: Option<VerifiedSelectedTierEpochCustody>,
    #[cfg(feature = "recovery-runtime-owner")]
    selected_wal: crate::physical_runtime::recovery_construction::SelectedWalMediaFingerprint,
    #[cfg(feature = "recovery-runtime-owner")]
    selected_controls:
        Option<crate::physical_runtime::recovery_construction::SelectedControlMediaFingerprint>,
}

/// Verification against the freshly loaded Serving media is a required phase.
pub(in crate::physical_runtime) struct VerifiedRecoveredCheckpointCustody {
    seal: RecoveredCheckpointCustodyEvidence,
}

impl VerifiedRecoveredCheckpointCustody {
    pub(in crate::physical_runtime) fn seal(&self) -> &RecoveredCheckpointCustodyEvidence {
        &self.seal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum RecoveredCheckpointCustodyDenial {
    SelectedRootMismatch,
    SelectedCheckpointMismatch,
    CertifiedTierUnavailable,
    SelectedFreeSpaceMismatch,
}

impl RecoveredPhysicalCheckpointCustody {
    #[cfg(feature = "recovery-runtime-owner")]
    pub fn checkpoint(&self) -> Option<&crate::physical_runtime::SharedRecoveryCheckpoint> {
        self.evidence.checkpoint_ownership.checkpoint()
    }

    pub const fn residency_policy(
        &self,
    ) -> crate::physical_runtime::record_serving::AdmittedPhysicalRecordResidencyPolicy {
        self.residency.admitted_policy()
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_residency_allocations(
        &self,
    ) -> worth_store_buffer_pool::PhysicalResidencyAllocationEventObserver {
        self.residency.ports().allocation_events()
    }

    pub(in crate::physical_runtime) fn into_serving_parts(
        self,
    ) -> (
        crate::physical_runtime::instance::PhysicalResidencyOwner,
        RecoveredCheckpointCustodyEvidence,
    ) {
        (self.residency, self.evidence)
    }

    pub const fn recovery_allocation_admission(
        &self,
    ) -> crate::physical_runtime::PhysicalRecoveryAllocationAdmission {
        self.evidence.recovery_allocation
    }

    #[cfg(feature = "recovery-runtime-owner")]
    pub fn retained_heap_bytes(
        &self,
    ) -> Result<u64, crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial> {
        self.evidence.retained_heap_bytes()
    }
}

impl RecoveredCheckpointCustodyEvidence {
    fn checkpoint_stream(
        &self,
        facts: &worth_store_physical_integrity::VerifiedCheckpointFacts,
    ) -> Result<
        &worth_store_physical_integrity::VerifiedCheckpointStream,
        RecoveredCheckpointCustodyDenial,
    > {
        #[cfg(feature = "recovery-runtime-owner")]
        if let Some(shared) = self.checkpoint_ownership.checkpoint() {
            if shared.facts() == *facts {
                return Ok(shared.stream());
            }
        }
        let _ = facts;
        Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch)
    }

    /// Original Store-issued recovery ceiling; this observation mints no new grant.
    pub(in crate::physical_runtime) const fn recovery_allocation_admission(
        &self,
    ) -> crate::physical_runtime::PhysicalRecoveryAllocationAdmission {
        self.recovery_allocation
    }

    #[cfg(feature = "recovery-runtime-owner")]
    fn verify_controls_for_open(
        &self,
        media: &QualifiedFilesystemMedia,
        store: StableStoreIdentity,
        root: &DurablePhysicalRootManifest,
        free_space: &DurableFreeSpaceManifestHeader,
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<(), RecoveredCheckpointCustodyDenial> {
        let actual_sha256: [u8; 32] = Sha256::digest(root.encode(format)).into();
        if self.store != store || self.root != *root {
            return Err(RecoveredCheckpointCustodyDenial::SelectedRootMismatch);
        }
        if self.historical_release.is_none()
            && !self
                .selected_controls
                .as_ref()
                .is_some_and(|controls| controls.matches_serving_media(media))
        {
            return Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch);
        }
        match self.tier.as_ref() {
            Some(tier) => {
                if tier.selected_root() != root
                    || tier.selected_root_sha256() != actual_sha256
                    || root.tier_epoch_anchor() != Some(tier.tier_epoch_anchor())
                    || free_space != tier.free_header()
                    || free_space.tier_epoch_start() != Some(tier.tier_epoch_start())
                    || <[u8; 32]>::from(Sha256::digest(free_space.encode(format)))
                        != tier.free_header_sha256()
                {
                    return Err(RecoveredCheckpointCustodyDenial::SelectedFreeSpaceMismatch);
                }
            }
            None if root.tier_epoch_anchor().is_some() => {
                return Err(RecoveredCheckpointCustodyDenial::CertifiedTierUnavailable)
            }
            None => {}
        }
        match (
            &self.head_v2,
            &self.no_release,
            &self.pending_wal_release,
            &self.historical_release,
        ) {
            (Some(head_v2), None, None, None) => {
                self.verify_head_v2(head_v2, store, root, actual_sha256, format)?
            }
            (None, Some(no_release), None, None) => {
                self.verify_no_release(no_release, store, root, actual_sha256)?
            }
            (None, None, Some(pending), None) => {
                self.verify_pending_wal_release(pending, store, root, actual_sha256)?
            }
            (None, None, None, Some(historical)) => {
                self.verify_ordered_historical_release(historical, store, root, actual_sha256)?
            }
            _ => return Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch),
        }
        if let Some(tier) = self.tier.as_ref() {
            let selected = self
                .head_v2
                .as_ref()
                .map(|claim| claim.checkpoint())
                .or_else(|| self.no_release.as_ref().map(|claim| claim.checkpoint()))
                .or_else(|| {
                    self.pending_wal_release
                        .as_ref()
                        .map(|claim| claim.checkpoint())
                })
                .or_else(|| {
                    self.historical_release
                        .as_ref()
                        .map(|claim| claim.checkpoint())
                })
                .ok_or(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch)?;
            if selected.source().identity() != tier.checkpoint().source().identity()
                || selected.encoded_bytes() != tier.checkpoint().encoded_bytes()
                || selected.encoded_digest() != tier.checkpoint().encoded_digest()
            {
                return Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch);
            }
        }
        Ok(())
    }

    fn verify_no_release(
        &self,
        verified: &VerifiedSelectedNoReleaseCustody,
        store: StableStoreIdentity,
        root: &DurablePhysicalRootManifest,
        actual_sha256: [u8; 32],
    ) -> Result<(), RecoveredCheckpointCustodyDenial> {
        let marker = verified.marker();
        let stream = self.checkpoint_stream(verified.checkpoint())?;
        let source = stream.source();
        if verified.selected_root() != root
            || verified.selected_root_sha256() != actual_sha256
            || source.identity().store_identity() != store
            || source.root().generation() > root.generation()
            || marker.checkpoint() != source.identity()
            || marker.root_generation() != source.root().generation()
            || marker.root_sha256() != verified.checkpoint_source_root_sha256()
            || <[u8; 32]>::from(Sha256::digest(marker.encode())) != verified.marker_payload_sha256()
        {
            return Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch);
        }
        let mut selected = 0;
        for frame in stream.certificate_records() {
            let (kind, payload) = decode_checkpoint_certificate(frame)
                .map_err(|_| RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch)?;
            if kind == CheckpointCertificateKind::ReleasedDrop {
                let ReleaseCheckpointCertificateV1::NoRelease(value) =
                    ReleaseCheckpointCertificateV1::decode(payload).map_err(|_| {
                        RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch
                    })?
                else {
                    return Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch);
                };
                if value != marker || payload != marker.encode() {
                    return Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch);
                }
                selected += 1;
            }
        }
        if selected != 1 {
            return Err(RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch);
        }
        Ok(())
    }

    pub(in crate::physical_runtime) fn verified_basis(
        &self,
    ) -> (
        Option<&VerifiedSelectedReleaseHeadCustodyV2>,
        Option<&VerifiedSelectedNoReleaseCustody>,
        Option<&VerifiedPendingWalReleaseCustody>,
        Option<&VerifiedEffectiveReleaseHeadRosterV14>,
        Option<&VerifiedOrderedHistoricalReleaseCustody>,
        Option<&VerifiedSelectedTierEpochCustody>,
    ) {
        (
            self.head_v2.as_ref(),
            self.no_release.as_ref(),
            self.pending_wal_release.as_ref(),
            self.effective_release_heads.as_ref(),
            self.historical_release.as_ref(),
            self.tier.as_ref(),
        )
    }

    pub(in crate::physical_runtime) fn pending_wal_release(
        &self,
    ) -> Option<&VerifiedPendingWalReleaseCustody> {
        self.pending_wal_release.as_ref()
    }
}
