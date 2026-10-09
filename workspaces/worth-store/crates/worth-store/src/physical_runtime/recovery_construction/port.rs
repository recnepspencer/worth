use sha2::{Digest, Sha256};
use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_recovery_physics::{
    VerifiedSelectedNoReleaseCustody, VerifiedSelectedTierEpochCustody,
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
        no_release_custody: Option<VerifiedSelectedNoReleaseCustody>,
        tier_custody: Option<VerifiedSelectedTierEpochCustody>,
        selected_wal: Option<selected_rejoin::SelectedWalMediaFingerprint>,
        selected_controls: Option<selected_rejoin::SelectedControlMediaFingerprint>,
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        if coordination.require_observed_checkpoint().is_err() {
            let _ = coordination.shutdown_is_quiescent();
            return Err(RecoveredPhysicalRuntimeConstructionDenial::SelectedCustodyMismatch);
        }
        let observed_root_sha256: [u8; 32] =
            Sha256::digest(reopen.fresh_reopen_occurrence().root().bytes()).into();
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
        }) {
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
        let (residency, checkpoint_ownership) = coordination
            .into_quiescent_recovery_parts()
            .ok_or(RecoveredPhysicalRuntimeConstructionDenial::CoordinationNotQuiescent)?;
        Ok(RecoveredPhysicalRuntimeCore {
            residency,
            checkpoint_ownership,
            store: media.store_identity(),
            recovery_allocation,
            runtime,
            recovery_runtime,
            root: reopen.root().clone(),
            media,
            reopen,
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
