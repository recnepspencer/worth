use sha2::{Digest, Sha256};
use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_recovery_physics::{
    VerifiedSelectedCheckpointCustody, VerifiedSelectedNoReleaseCustody,
    VerifiedSelectedTierEpochCustody,
};

use crate::physical_runtime::{
    ClosedPhysicalRecoveryCleanup, CompletedPhysicalRecoveryFreshReopen,
    PhysicalRecoveryCoordination,
};

use super::{
    selected_rejoin, RecoveredPhysicalRuntimeConstructionDenial, RecoveredPhysicalRuntimeCore,
};
mod binding;
#[path = "port/head_v2.rs"]
mod head_v2;
#[path = "port/historical.rs"]
mod historical;
#[path = "port/pending_wal.rs"]
mod pending_wal;
#[path = "port/rejoin_denial.rs"]
mod rejoin_denial;
use binding::{fresh_runtime_identity, validate_construction_binding};

/// The sole Store-owned construction port for a recovered physical runtime.
pub struct PhysicalRecoveryConstructionPort {
    _private: (),
}

impl PhysicalRecoveryConstructionPort {
    pub fn construct(
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        if cleanup.live_media_handle_delta() != 0 {
            let _ = coordination.shutdown_is_quiescent();
            return Err(RecoveredPhysicalRuntimeConstructionDenial::CleanupMediaNotQuiescent);
        }
        Self::construct_inner(
            coordination,
            media,
            cleanup.into_reopen(),
            None,
            None,
            None,
            None,
            None,
        )
    }

    pub fn construct_with_verified_custody(
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        custody: VerifiedSelectedCheckpointCustody,
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        Self::construct_verified(coordination, media, cleanup, custody, None, || {})
    }

    /// Exercises the production Store media rejoin and owner-install path.
    #[cfg(feature = "certification-test-authority")]
    #[doc(hidden)]
    pub fn certification_construct_with_verified_custody(
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        custody: VerifiedSelectedCheckpointCustody,
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        Self::construct_verified(coordination, media, cleanup, custody, None, || {})
    }

    /// Pauses only between the initial actual-media join and its final
    /// bounded reread. The callback grants no alternate seal path.
    #[cfg(feature = "certification-test-authority")]
    #[doc(hidden)]
    pub fn certification_construct_with_verified_custody_and_rejoin_pause(
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        custody: VerifiedSelectedCheckpointCustody,
        pause_before_final_reread: impl FnOnce(),
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        Self::construct_verified(
            coordination,
            media,
            cleanup,
            custody,
            None,
            pause_before_final_reread,
        )
    }

    pub fn construct_with_verified_tier_and_release(
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        tier: VerifiedSelectedTierEpochCustody,
        released: VerifiedSelectedCheckpointCustody,
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        Self::construct_verified(coordination, media, cleanup, released, Some(tier), || {})
    }

    #[cfg(feature = "certification-test-authority")]
    #[doc(hidden)]
    pub fn certification_construct_with_verified_tier_and_release(
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        tier: VerifiedSelectedTierEpochCustody,
        released: VerifiedSelectedCheckpointCustody,
        pause_before_final_reread: impl FnOnce(),
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        Self::construct_verified(
            coordination,
            media,
            cleanup,
            released,
            Some(tier),
            pause_before_final_reread,
        )
    }

    fn construct_verified(
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        custody: VerifiedSelectedCheckpointCustody,
        tier: Option<VerifiedSelectedTierEpochCustody>,
        pause_before_final_reread: impl FnOnce(),
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        if cleanup.live_media_handle_delta() != 0 {
            let _ = coordination.shutdown_is_quiescent();
            return Err(RecoveredPhysicalRuntimeConstructionDenial::CleanupMediaNotQuiescent);
        }
        let reopen = cleanup.into_reopen();
        if let Err(denial) = validate_construction_binding(&coordination, &media, &reopen) {
            let _ = coordination.shutdown_is_quiescent();
            return Err(denial);
        }
        // A physics transcript is a claim, not authority. Re-read the same
        // admitted media now; WAL/compaction fate still must be joined before
        // this constructor can issue a serving seal.
        let (media, selected_wal, selected_controls) = match selected_rejoin::observe_claim(
            &coordination,
            media,
            &reopen,
            &custody,
            tier.as_ref(),
            pause_before_final_reread,
        ) {
            Ok(media) => media,
            Err(_) => {
                let _ = coordination.shutdown_is_quiescent();
                return Err(RecoveredPhysicalRuntimeConstructionDenial::SelectedCustodyMismatch);
            }
        };
        Self::construct_inner(
            coordination,
            media,
            reopen,
            Some(custody),
            None,
            tier,
            Some(selected_wal),
            Some(selected_controls),
        )
    }

    pub fn construct_with_verified_tier_no_release(
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        tier: VerifiedSelectedTierEpochCustody,
        no_release: VerifiedSelectedNoReleaseCustody,
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        Self::construct_tier_no_release(coordination, media, cleanup, tier, no_release, || {})
    }

    #[cfg(feature = "certification-test-authority")]
    #[doc(hidden)]
    pub fn certification_construct_with_verified_tier_no_release(
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        tier: VerifiedSelectedTierEpochCustody,
        no_release: VerifiedSelectedNoReleaseCustody,
        pause_before_final_reread: impl FnOnce(),
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        Self::construct_tier_no_release(
            coordination,
            media,
            cleanup,
            tier,
            no_release,
            pause_before_final_reread,
        )
    }

    fn construct_tier_no_release(
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        tier: VerifiedSelectedTierEpochCustody,
        no_release: VerifiedSelectedNoReleaseCustody,
        pause_before_final_reread: impl FnOnce(),
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        if cleanup.live_media_handle_delta() != 0 {
            let _ = coordination.shutdown_is_quiescent();
            return Err(RecoveredPhysicalRuntimeConstructionDenial::CleanupMediaNotQuiescent);
        }
        let reopen = cleanup.into_reopen();
        if let Err(denial) = validate_construction_binding(&coordination, &media, &reopen) {
            let _ = coordination.shutdown_is_quiescent();
            return Err(denial);
        }
        let (media, selected_wal, selected_controls) = match selected_rejoin::tier::observe_claim(
            &coordination,
            media,
            &reopen,
            &tier,
            Some(&no_release),
            pause_before_final_reread,
        ) {
            Ok(media) => media,
            Err(_) => {
                let _ = coordination.shutdown_is_quiescent();
                return Err(RecoveredPhysicalRuntimeConstructionDenial::SelectedCustodyMismatch);
            }
        };
        Self::construct_inner(
            coordination,
            media,
            reopen,
            None,
            Some(no_release),
            Some(tier),
            Some(selected_wal),
            Some(selected_controls),
        )
    }

    pub fn construct_with_verified_no_release(
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        no_release: VerifiedSelectedNoReleaseCustody,
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        Self::construct_no_release(coordination, media, cleanup, no_release, || {})
    }

    #[cfg(feature = "certification-test-authority")]
    #[doc(hidden)]
    pub fn certification_construct_with_verified_no_release(
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        no_release: VerifiedSelectedNoReleaseCustody,
        pause_before_final_reread: impl FnOnce(),
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        Self::construct_no_release(
            coordination,
            media,
            cleanup,
            no_release,
            pause_before_final_reread,
        )
    }

    fn construct_no_release(
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        no_release: VerifiedSelectedNoReleaseCustody,
        pause_before_final_reread: impl FnOnce(),
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        if cleanup.live_media_handle_delta() != 0 {
            let _ = coordination.shutdown_is_quiescent();
            return Err(RecoveredPhysicalRuntimeConstructionDenial::CleanupMediaNotQuiescent);
        }
        let reopen = cleanup.into_reopen();
        if let Err(denial) = validate_construction_binding(&coordination, &media, &reopen) {
            let _ = coordination.shutdown_is_quiescent();
            return Err(denial);
        }
        let (media, selected_wal, selected_controls) =
            match selected_rejoin::no_release::observe_claim(
                &coordination,
                media,
                &reopen,
                &no_release,
                pause_before_final_reread,
            ) {
                Ok(media) => media,
                Err(_) => {
                    let _ = coordination.shutdown_is_quiescent();
                    return Err(
                        RecoveredPhysicalRuntimeConstructionDenial::SelectedCustodyMismatch,
                    );
                }
            };
        Self::construct_inner(
            coordination,
            media,
            reopen,
            None,
            Some(no_release),
            None,
            Some(selected_wal),
            Some(selected_controls),
        )
    }

    fn construct_inner(
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        reopen: CompletedPhysicalRecoveryFreshReopen,
        checkpoint_custody: Option<VerifiedSelectedCheckpointCustody>,
        no_release_custody: Option<VerifiedSelectedNoReleaseCustody>,
        tier_custody: Option<VerifiedSelectedTierEpochCustody>,
        selected_wal: Option<selected_rejoin::SelectedWalMediaFingerprint>,
        selected_controls: Option<selected_rejoin::SelectedControlMediaFingerprint>,
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        let observed_root_sha256: [u8; 32] =
            Sha256::digest(reopen.fresh_reopen_occurrence().root().bytes()).into();
        let observed_accumulator_sha256 = checkpoint_custody.as_ref().map(|verified| {
            let digest: [u8; 32] = Sha256::digest(verified.accumulator().encode()).into();
            digest
        });
        if checkpoint_custody.as_ref().is_some_and(|verified| {
            verified.selected_root() != reopen.root()
                || verified.selected_root_sha256() != observed_root_sha256
                || verified.accumulator().checkpoint() != verified.checkpoint().source().identity()
                || Some(verified.accumulator_payload_sha256()) != observed_accumulator_sha256
        }) {
            let _ = coordination.shutdown_is_quiescent();
            return Err(RecoveredPhysicalRuntimeConstructionDenial::SelectedCustodyMismatch);
        }
        if no_release_custody.as_ref().is_some_and(|verified| {
            let marker_digest: [u8; 32] = Sha256::digest(verified.marker().encode()).into();
            verified.selected_root() != reopen.root()
                || verified.selected_root_sha256() != observed_root_sha256
                || verified.marker().checkpoint() != verified.checkpoint().source().identity()
                || verified.marker().root_sha256() != verified.checkpoint_source_root_sha256()
                || verified.marker_payload_sha256() != marker_digest
        }) || tier_custody.as_ref().is_some_and(|verified| {
            verified.selected_root() != reopen.root()
                || verified.selected_root_sha256() != observed_root_sha256
                || reopen.root().tier_epoch_anchor() != Some(verified.tier_epoch_anchor())
        }) || (checkpoint_custody.is_some() && no_release_custody.is_some())
        {
            let _ = coordination.shutdown_is_quiescent();
            return Err(RecoveredPhysicalRuntimeConstructionDenial::SelectedCustodyMismatch);
        }
        let recovery_allocation =
            match validate_construction_binding(&coordination, &media, &reopen) {
                Ok(allocation) => allocation,
                Err(denial) => {
                    let _ = coordination.shutdown_is_quiescent();
                    return Err(denial);
                }
            };
        let _ = coordination.reconcile_signal_settlements();
        if !coordination.is_ready() {
            let _ = coordination.shutdown_is_quiescent();
            return Err(RecoveredPhysicalRuntimeConstructionDenial::CoordinationNotQuiescent);
        }
        let recovery_runtime = coordination.runtime_identity();
        let Some(runtime) = fresh_runtime_identity(recovery_runtime) else {
            let _ = coordination.shutdown_is_quiescent();
            return Err(RecoveredPhysicalRuntimeConstructionDenial::RuntimeIdentityUnavailable);
        };
        if !coordination.shutdown_is_quiescent() {
            return Err(RecoveredPhysicalRuntimeConstructionDenial::CoordinationNotQuiescent);
        }
        Ok(RecoveredPhysicalRuntimeCore {
            store: media.store_identity(),
            recovery_allocation,
            runtime,
            recovery_runtime,
            root: reopen.root().clone(),
            media,
            reopen,
            checkpoint_custody,
            head_v2_custody: None,
            no_release_custody,
            pending_wal_release_custody: None,
            effective_release_heads: None,
            historical_release_custody: None,
            tier_custody,
            selected_wal,
            selected_controls,
        })
    }
}
