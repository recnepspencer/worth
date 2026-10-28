//! Transfer a qualified media owner into recovery without releasing its lease.
use super::QualifiedFilesystemMedia;
use crate::recovery_media::{
    QualifiedRecoveryFilesystemMedia, RecoveryFilesystemQualificationError,
};
impl QualifiedFilesystemMedia {
    pub fn into_recovery(
        self,
        root: &std::path::Path,
    ) -> Result<QualifiedRecoveryFilesystemMedia, RecoveryFilesystemQualificationError> {
        super::super::recovery_qualification::finish_existing_qualification(
            root,
            self.owner,
            super::super::FilesystemAccessContract::CoordinatedServiceAccount,
        )
        .map(QualifiedRecoveryFilesystemMedia::from_parts)
    }
}
