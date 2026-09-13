use std::env;
use std::fs::{self, File};
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, Instant};

use super::file_identity::open_directory_identity;
use super::OfflineIntegrityObservationLimits;

#[derive(Debug, Clone, PartialEq, Eq)]
enum OfflineIntegrityReportDestinationKind {
    StandardOutput,
    File(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfflineIntegrityReportDestination {
    kind: OfflineIntegrityReportDestinationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflineIntegrityReportDestinationDenial {
    EmptyFilePath,
}

pub(crate) enum ProvenReportDestination {
    StandardOutput,
    File { path: PathBuf, _parent_guard: File },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflineIntegrityReportBoundaryDenial {
    StoreRootUnavailable,
    StoreRootNotDirectory,
    CurrentDirectoryUnavailable,
    DestinationHasNoFileName,
    DestinationParentUnavailable,
    OpenFileBoundInsufficient,
    ElapsedBoundExceeded,
    FilesystemIdentityUnavailable,
    DestinationAlreadyExists,
    DestinationInsideStoreRoot,
}

impl OfflineIntegrityReportDestination {
    pub const fn standard_output() -> Self {
        Self {
            kind: OfflineIntegrityReportDestinationKind::StandardOutput,
        }
    }

    pub fn file(path: PathBuf) -> Result<Self, OfflineIntegrityReportDestinationDenial> {
        if path.as_os_str().is_empty() {
            return Err(OfflineIntegrityReportDestinationDenial::EmptyFilePath);
        }
        Ok(Self {
            kind: OfflineIntegrityReportDestinationKind::File(path),
        })
    }

    pub fn file_path(&self) -> Option<&Path> {
        match &self.kind {
            OfflineIntegrityReportDestinationKind::StandardOutput => None,
            OfflineIntegrityReportDestinationKind::File(path) => Some(path),
        }
    }

    pub const fn is_standard_output(&self) -> bool {
        matches!(
            self.kind,
            OfflineIntegrityReportDestinationKind::StandardOutput
        )
    }
}

pub(crate) fn prove_report_destination(
    store_root: &Path,
    destination: &OfflineIntegrityReportDestination,
    limits: OfflineIntegrityObservationLimits,
    started: Instant,
) -> Result<(PathBuf, ProvenReportDestination), OfflineIntegrityReportBoundaryDenial> {
    let canonical_store = fs::canonicalize(store_root)
        .map_err(|_| OfflineIntegrityReportBoundaryDenial::StoreRootUnavailable)?;
    if !canonical_store.is_dir() {
        return Err(OfflineIntegrityReportBoundaryDenial::StoreRootNotDirectory);
    }
    let Some(file_path) = destination.file_path() else {
        return Ok((canonical_store, ProvenReportDestination::StandardOutput));
    };
    if limits.maximum_open_files() < 2 {
        return Err(OfflineIntegrityReportBoundaryDenial::OpenFileBoundInsufficient);
    }
    let absolute = absolute_lexical_path(file_path)?;
    let file_name = absolute
        .file_name()
        .ok_or(OfflineIntegrityReportBoundaryDenial::DestinationHasNoFileName)?;
    let parent = absolute
        .parent()
        .ok_or(OfflineIntegrityReportBoundaryDenial::DestinationHasNoFileName)?;
    let canonical_parent = fs::canonicalize(parent)
        .map_err(|_| OfflineIntegrityReportBoundaryDenial::DestinationParentUnavailable)?;
    if !canonical_parent.is_dir() {
        return Err(OfflineIntegrityReportBoundaryDenial::DestinationParentUnavailable);
    }
    let canonical_target = canonical_parent.join(file_name);
    if canonical_target.exists() {
        return Err(OfflineIntegrityReportBoundaryDenial::DestinationAlreadyExists);
    }
    if canonical_target == canonical_store || canonical_target.starts_with(&canonical_store) {
        return Err(OfflineIntegrityReportBoundaryDenial::DestinationInsideStoreRoot);
    }
    let identity_output_bound = limits.maximum_bytes().min(4_096);
    let remaining = remaining_elapsed(started, limits)?;
    let (_, store_identity) =
        open_directory_identity(&canonical_store, identity_output_bound, remaining)
            .map_err(|_| OfflineIntegrityReportBoundaryDenial::FilesystemIdentityUnavailable)?;
    let remaining = remaining_elapsed(started, limits)?;
    let (parent_guard, parent_identity) =
        open_directory_identity(&canonical_parent, identity_output_bound, remaining)
            .map_err(|_| OfflineIntegrityReportBoundaryDenial::FilesystemIdentityUnavailable)?;
    if parent_identity == store_identity {
        return Err(OfflineIntegrityReportBoundaryDenial::DestinationInsideStoreRoot);
    }
    Ok((
        canonical_store,
        ProvenReportDestination::File {
            path: canonical_target,
            _parent_guard: parent_guard,
        },
    ))
}

fn remaining_elapsed(
    started: Instant,
    limits: OfflineIntegrityObservationLimits,
) -> Result<Duration, OfflineIntegrityReportBoundaryDenial> {
    let remaining = Duration::from_millis(limits.maximum_elapsed_milliseconds())
        .saturating_sub(started.elapsed());
    (!remaining.is_zero())
        .then_some(remaining)
        .ok_or(OfflineIntegrityReportBoundaryDenial::ElapsedBoundExceeded)
}

fn absolute_lexical_path(path: &Path) -> Result<PathBuf, OfflineIntegrityReportBoundaryDenial> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()
            .map_err(|_| OfflineIntegrityReportBoundaryDenial::CurrentDirectoryUnavailable)?
            .join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(OfflineIntegrityReportBoundaryDenial::DestinationParentUnavailable);
                }
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    Ok(normalized)
}
