use crate::filesystem_media::{ArtifactTreeDirectory, ArtifactTreeFailure};

use super::AdmittedRecoveryFilesystemMedia;

impl AdmittedRecoveryFilesystemMedia {
    /// Makes the directory entry of `checkpoint.current` durable under the
    /// admitted recovery media owner. A killed runtime may have replaced the
    /// published checkpoint without synchronizing its namespace; recovery
    /// that selected that checkpoint completes the synchronization before it
    /// settles the dead runtime's publication obligation. The candidate's own
    /// bytes were synchronized before its replacement could start.
    pub fn synchronize_recovery_checkpoint_publication(&self) -> Result<(), ArtifactTreeFailure> {
        self.parts
            .artifact_tree()
            .synchronize_directory(&ArtifactTreeDirectory::families())
    }
}
