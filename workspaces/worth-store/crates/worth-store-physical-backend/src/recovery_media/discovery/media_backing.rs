//! Sealed mechanical read access; this cannot extract or promote a media owner.

use super::super::PhysicalRecoveryMediaGeneration;
use crate::filesystem_media::recovery_qualification::AdmittedRecoveryParts;
use crate::filesystem_media::{ArtifactTreeMedia, QualifiedFilesystemMedia};
use worth_store_physical_format::store_namespace::StableStoreIdentity;

pub trait DiscoveryMediaBacking {
    fn artifact_tree(&self) -> ArtifactTreeMedia<'_>;
    fn store_identity(&self) -> StableStoreIdentity;
    fn media_generation(&self) -> PhysicalRecoveryMediaGeneration;
}

impl DiscoveryMediaBacking for AdmittedRecoveryParts {
    fn artifact_tree(&self) -> ArtifactTreeMedia<'_> {
        self.artifact_tree()
    }
    fn store_identity(&self) -> StableStoreIdentity {
        self.store_identity
    }
    fn media_generation(&self) -> PhysicalRecoveryMediaGeneration {
        self.media_generation
    }
}

pub(super) struct BorrowedMediaBacking<'media> {
    pub(super) media: &'media QualifiedFilesystemMedia,
}

impl DiscoveryMediaBacking for BorrowedMediaBacking<'_> {
    fn artifact_tree(&self) -> ArtifactTreeMedia<'_> {
        self.media.artifact_tree()
    }
    fn store_identity(&self) -> StableStoreIdentity {
        self.media.store_identity()
    }
    fn media_generation(&self) -> PhysicalRecoveryMediaGeneration {
        PhysicalRecoveryMediaGeneration::from_owner_attempt(
            self.media.mutation_owner().attempt().bytes(),
        )
    }
}
