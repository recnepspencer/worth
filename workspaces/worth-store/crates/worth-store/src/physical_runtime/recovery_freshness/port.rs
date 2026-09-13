use crate::physical_runtime::IntegrityAdmittedRecoveryWalFrame;
use worth_store_physical_backend::{
    AdmittedRecoveryFilesystemMedia, QualifiedRecoveryFilesystemMedia,
};
use worth_store_physical_integrity::VerifiedCheckpointStream;

use super::{
    binding, PhysicalRecoveryFreshnessAuthority, StoreRecoveryBindingFreshnessSample,
    StoreRecoveryBindingSampleFailure,
};

/// The sole Store-owned construction port for recovery freshness authority.
pub struct PhysicalRecoveryFreshnessPort {
    _private: (),
}

impl PhysicalRecoveryFreshnessPort {
    pub fn admit(
        media: &QualifiedRecoveryFilesystemMedia,
    ) -> Option<PhysicalRecoveryFreshnessAuthority> {
        PhysicalRecoveryFreshnessAuthority::issue(media.media_generation())
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn admit_for_certification(
        media: &QualifiedRecoveryFilesystemMedia,
    ) -> Option<PhysicalRecoveryFreshnessAuthority> {
        PhysicalRecoveryFreshnessAuthority::issue(media.media_generation())
    }

    pub fn sample_binding<'frame>(
        coordination: &crate::physical_runtime::PhysicalRecoveryCoordination,
        media: &AdmittedRecoveryFilesystemMedia,
        checkpoint: &VerifiedCheckpointStream,
        wal_frames: impl IntoIterator<Item = &'frame IntegrityAdmittedRecoveryWalFrame>,
        maximum_operation_bindings: u64,
        maximum_redo_bytes: u64,
    ) -> Result<StoreRecoveryBindingFreshnessSample, StoreRecoveryBindingSampleFailure> {
        binding::sample_binding(
            coordination.freshness(),
            coordination.checkpoint_binding_basis(),
            media,
            checkpoint,
            wal_frames,
            maximum_operation_bindings,
            maximum_redo_bytes,
        )
    }
}
