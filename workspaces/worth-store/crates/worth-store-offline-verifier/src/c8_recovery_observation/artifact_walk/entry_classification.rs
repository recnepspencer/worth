use std::path::PathBuf;

use super::super::{
    RecoveryObserverCounters, RecoveryObserverObservationDenial, RecoveryObserverObservationFailure,
};

pub(super) enum ClassifiedEntry {
    Directory(PathBuf),
    Artifact(PathBuf),
    IgnoredLock,
}

pub(super) fn classify(
    entry: std::fs::DirEntry,
    counters: RecoveryObserverCounters,
) -> Result<ClassifiedEntry, RecoveryObserverObservationFailure> {
    let path = entry.path();
    let file_type = entry.file_type().map_err(|error| {
        RecoveryObserverObservationFailure::at_path(
            RecoveryObserverObservationDenial::Media(error.kind()),
            counters,
            &path,
        )
    })?;
    if file_type.is_symlink() {
        return Err(RecoveryObserverObservationFailure::at_path(
            RecoveryObserverObservationDenial::SymbolicLink,
            counters,
            &path,
        ));
    }
    if file_type.is_dir() {
        return Ok(ClassifiedEntry::Directory(path));
    }
    if file_type.is_file() {
        if path
            .file_name()
            .is_some_and(|name| name.to_string_lossy().ends_with(".lock"))
        {
            return Ok(ClassifiedEntry::IgnoredLock);
        }
        return Ok(ClassifiedEntry::Artifact(path));
    }
    Err(RecoveryObserverObservationFailure::at_path(
        RecoveryObserverObservationDenial::UnsupportedFileType,
        counters,
        &path,
    ))
}
