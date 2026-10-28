use std::path::Path;

use super::{
    AdmittedRecoveryFilesystemMedia, PhysicalRecoveryMediaGeneration,
    QualifiedPhysicalBackendProfile, RecoveryFilesystemQualificationError,
};

pub struct QualifiedRecoveryFilesystemMedia {
    parts: crate::filesystem_media::recovery_qualification::QualifiedRecoveryParts,
}

impl QualifiedRecoveryFilesystemMedia {
    /// Verify the caller's location against the still-held root directory.
    /// This reads identity only and never reacquires or mutates the lease.
    pub fn verify_root(&self, root: &Path) -> Result<(), RecoveryFilesystemQualificationError> {
        self.parts.verify_root(root)
    }

    pub(crate) fn from_parts(
        parts: crate::filesystem_media::recovery_qualification::QualifiedRecoveryParts,
    ) -> Self {
        Self { parts }
    }

    pub fn qualify_existing(
        root: impl AsRef<Path>,
    ) -> Result<Self, RecoveryFilesystemQualificationError> {
        crate::filesystem_media::recovery_qualification::qualify_existing_recovery(root.as_ref())
            .map(|parts| Self { parts })
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn qualify_existing_for_certification(
        root: impl AsRef<Path>,
        schedule: crate::filesystem_media::MediaFaultSchedule,
    ) -> Result<Self, RecoveryFilesystemQualificationError> {
        crate::filesystem_media::recovery_qualification::qualify_existing_recovery_for_certification(
            root.as_ref(),
            schedule,
        )
        .map(|parts| Self { parts })
    }

    pub fn backend_profile(&self) -> &QualifiedPhysicalBackendProfile {
        self.parts.backend_profile()
    }

    pub fn root_ownership_identity(&self) -> crate::MediaOwnerIdentity {
        self.parts.root_ownership_identity()
    }

    pub const fn media_generation(&self) -> PhysicalRecoveryMediaGeneration {
        self.parts.media_generation()
    }

    pub fn recovery_effect_count(&self) -> u64 {
        self.parts.recovery_effect_count()
    }

    pub fn admit_persisted_store(
        self,
    ) -> Result<AdmittedRecoveryFilesystemMedia, RecoveryFilesystemQualificationError> {
        self.parts
            .admit_persisted_store()
            .map(AdmittedRecoveryFilesystemMedia::from_parts)
    }
}
