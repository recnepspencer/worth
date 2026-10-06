use worth_store_physical_backend::{
    AdmittedRecoveryFilesystemMedia, QualifiedRecoveryFilesystemMedia,
};

use super::{
    binding, IntegrityAdmittedRecoveryWalFrameView, PhysicalRecoveryFreshnessAuthority,
    StoreRecoveryBindingFreshnessSample, StoreRecoveryBindingSampleFailure,
    StoreRecoverySamplingBasis,
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

    /// Interprets borrowed C9 frames under Coordination's original native
    /// Recovery admission, including already-live grants in that pool.
    /// The result retains its exact backing until disposal; owning clones are
    /// unavailable. Allocation denial preserves its cause and admits no effects.
    pub fn sample_binding<'frame>(
        coordination: &crate::physical_runtime::PhysicalRecoveryCoordination,
        media: &AdmittedRecoveryFilesystemMedia,
        basis: StoreRecoverySamplingBasis<'_>,
        wal_frames: IntegrityAdmittedRecoveryWalFrameView<'frame>,
        maximum_operation_bindings: u64,
        maximum_redo_bytes: u64,
        maximum_manifest_cleanup_sampling_bytes: u64,
    ) -> Result<StoreRecoveryBindingFreshnessSample, StoreRecoveryBindingSampleFailure> {
        binding::sample_binding(
            binding::CheckpointCoveredMembers::Skip,
            coordination,
            media,
            basis,
            wal_frames.iter(),
            maximum_operation_bindings,
            maximum_redo_bytes,
            maximum_manifest_cleanup_sampling_bytes,
        )
    }
}
