//! Store construction before the first checkpoint: recovery observed
//! `checkpoint.current` absent and replayed the whole WAL from the canonical
//! origin, so the unanchored root rejoins against the generation-zero basis.

use sha2::{Digest, Sha256};
use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;

use crate::physical_runtime::record_serving::{
    GenerationZeroNoReleaseCustody, RecoveredNoReleaseCustody,
};
use crate::physical_runtime::{
    AbsentCheckpointWitness, ClosedPhysicalRecoveryCleanup,
    ConfiguredPhysicalDurabilityDeclaration, PhysicalRecoveryCoordination,
};

use super::{
    selected_rejoin, validate_construction_binding, PhysicalRecoveryConstructionPort,
    RecoveredPhysicalRuntimeConstructionDenial, RecoveredPhysicalRuntimeCore,
};

impl PhysicalRecoveryConstructionPort {
    /// Seals a recovered runtime whose basis is generation zero, keyed by the
    /// absence witness `coordination` minted. Every WAL binding must carry the
    /// identity of `declaration` admitted over `media`.
    pub fn construct_generation_zero(
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        declaration: ConfiguredPhysicalDurabilityDeclaration,
        absent: AbsentCheckpointWitness,
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        Self::construct_generation_zero_inner(
            coordination,
            media,
            cleanup,
            declaration,
            absent,
            || {},
        )
    }

    #[cfg(feature = "certification-test-authority")]
    #[doc(hidden)]
    pub fn certification_construct_generation_zero_with_rejoin_pause(
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        declaration: ConfiguredPhysicalDurabilityDeclaration,
        absent: AbsentCheckpointWitness,
        pause_before_final_reread: impl FnOnce(),
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        Self::construct_generation_zero_inner(
            coordination,
            media,
            cleanup,
            declaration,
            absent,
            pause_before_final_reread,
        )
    }

    fn construct_generation_zero_inner(
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        declaration: ConfiguredPhysicalDurabilityDeclaration,
        absent: AbsentCheckpointWitness,
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
                selected_rejoin::no_release::NoReleaseRejoinBasis::GenerationZero(
                    &absent,
                    declaration,
                ),
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
        let root_sha256: [u8; 32] =
            Sha256::digest(reopen.fresh_reopen_occurrence().root().bytes()).into();
        let custody = GenerationZeroNoReleaseCustody::rejoined(reopen.root().clone(), root_sha256);
        Self::construct_inner(
            coordination,
            media,
            reopen,
            Some(RecoveredNoReleaseCustody::GenerationZero(custody)),
            None,
            Some(selected_wal),
            Some(selected_controls),
        )
    }
}
