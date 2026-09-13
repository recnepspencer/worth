use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt};
use cap_std::fs::OpenOptions;
use std::marker::PhantomData;

use super::{
    FilesystemMediaOwner, MediaHandleIdentity, MediaOperationFailure, MediaOperationIdentity,
    MediaPathRole, NamespaceRelativePath,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamespaceFileOpenKind {
    Existing,
    CreatedNew,
}

#[derive(Debug)]
pub struct ReadOnlyFileAccess;

#[derive(Debug)]
pub struct MutableFileAccess;

#[derive(Debug)]
pub struct NamespaceFileHandle<'owner, Access = MutableFileAccess> {
    owner: &'owner FilesystemMediaOwner,
    identity: MediaHandleIdentity,
    path: NamespaceRelativePath,
    role: MediaPathRole,
    file: std::fs::File,
    stable_file: same_file::Handle,
    mutation_sequence: Option<super::file_mutation_sequence::FileMutationSequence>,
    _accounting: super::handle_accounting::MediaFileHandleAccounting,
    access: PhantomData<Access>,
}

#[cfg(all(
    feature = "certification-test-authority",
    feature = "recovery-runtime-owner"
))]
#[derive(Debug)]
pub(crate) struct CertificationRetainedMediaFileHandle {
    _file: std::fs::File,
    _accounting: super::handle_accounting::MediaFileHandleAccounting,
}

impl<Access> NamespaceFileHandle<'_, Access> {
    pub const fn identity(&self) -> MediaHandleIdentity {
        self.identity
    }

    pub const fn role(&self) -> MediaPathRole {
        self.role
    }

    pub(super) const fn owner(&self) -> &FilesystemMediaOwner {
        self.owner
    }

    pub(super) const fn file(&self) -> &std::fs::File {
        &self.file
    }

    pub(super) const fn stable_file(&self) -> &same_file::Handle {
        &self.stable_file
    }
}

#[cfg(all(
    feature = "certification-test-authority",
    feature = "recovery-runtime-owner"
))]
impl NamespaceFileHandle<'_, ReadOnlyFileAccess> {
    pub(crate) fn retain_for_certification(self) -> CertificationRetainedMediaFileHandle {
        let Self {
            file, _accounting, ..
        } = self;
        CertificationRetainedMediaFileHandle {
            _file: file,
            _accounting,
        }
    }
}

impl<'owner> NamespaceFileHandle<'owner, MutableFileAccess> {
    pub(super) fn into_deletion_parts(
        self,
    ) -> (
        &'owner FilesystemMediaOwner,
        NamespaceRelativePath,
        MediaHandleIdentity,
        same_file::Handle,
        std::fs::File,
    ) {
        let Self {
            owner,
            identity,
            path,
            file,
            stable_file,
            ..
        } = self;
        (owner, path, identity, stable_file, file)
    }

    pub(super) fn mutation_guard(&self) -> std::sync::MutexGuard<'_, ()> {
        self.mutation_sequence
            .as_ref()
            .expect("mutable handles always carry per-file coordination")
            .lock()
    }
}

#[derive(Debug)]
pub enum NamespaceFileOpenResult<'owner, Access = MutableFileAccess> {
    Opened {
        kind: NamespaceFileOpenKind,
        handle: NamespaceFileHandle<'owner, Access>,
    },
    Failed(MediaOperationFailure),
}

#[derive(Debug)]
pub struct NamespaceFileOpenOutcome<'owner, Access = MutableFileAccess> {
    operation: MediaOperationIdentity,
    result: NamespaceFileOpenResult<'owner, Access>,
}

impl<'owner, Access> NamespaceFileOpenOutcome<'owner, Access> {
    pub const fn operation(&self) -> MediaOperationIdentity {
        self.operation
    }

    pub const fn result(&self) -> &NamespaceFileOpenResult<'owner, Access> {
        &self.result
    }

    pub fn into_result(self) -> NamespaceFileOpenResult<'owner, Access> {
        self.result
    }
}

impl FilesystemMediaOwner {
    pub fn open_existing<'owner>(
        &'owner self,
        path: &NamespaceRelativePath,
    ) -> NamespaceFileOpenOutcome<'owner, ReadOnlyFileAccess> {
        self.open_file(path, NamespaceFileOpenKind::Existing, false)
    }

    pub fn open_existing_for_mutation<'owner>(
        &'owner self,
        path: &NamespaceRelativePath,
    ) -> NamespaceFileOpenOutcome<'owner, MutableFileAccess> {
        let _authority = match self.begin_mutation() {
            Ok(authority) => authority,
            Err(denial) => {
                return failed_open(self, path, NamespaceFileOpenKind::Existing, denial);
            }
        };
        let outcome = self.open_file(path, NamespaceFileOpenKind::Existing, true);
        outcome
    }

    pub fn create_new<'owner>(
        &'owner self,
        path: &NamespaceRelativePath,
    ) -> NamespaceFileOpenOutcome<'owner> {
        let _authority = match self.begin_mutation() {
            Ok(authority) => authority,
            Err(denial) => {
                return failed_open(self, path, NamespaceFileOpenKind::CreatedNew, denial);
            }
        };
        let outcome = self.open_file(path, NamespaceFileOpenKind::CreatedNew, true);
        outcome
    }

    fn open_file<'owner, Access>(
        &'owner self,
        path: &NamespaceRelativePath,
        kind: NamespaceFileOpenKind,
        writable: bool,
    ) -> NamespaceFileOpenOutcome<'owner, Access> {
        let operation = self
            .issue_operation_identity()
            .expect("media operation identity exhausted");
        let attempt = self.boundary().begin_operation(
            operation_role(kind),
            0,
            super::MediaOperationCoordinates::for_path(operation, path.role(), None),
        );
        if let Some(error) = attempt.fail_before_error() {
            attempt.denied();
            return open_failure(
                operation,
                path,
                kind,
                Some(&error),
                super::MediaOperationFailureKind::DeniedBeforeEffect,
                super::MediaCausalBoundary::BeforeOsCall,
            );
        }
        let mut options = OpenOptions::new();
        options
            .read(true)
            .write(writable)
            .follow(FollowSymlinks::No);
        if kind == NamespaceFileOpenKind::CreatedNew {
            options.create_new(true);
        }
        let Some(directory) = self.directory_for_path(path) else {
            attempt.confinement_denied();
            return open_failure(
                operation,
                path,
                kind,
                None,
                super::MediaOperationFailureKind::DeniedBeforeEffect,
                super::MediaCausalBoundary::BeforeOsCall,
            );
        };
        let file = match directory.directory().open_with(path.file_name(), &options) {
            Ok(file) => cap_std::fs::File::into_std(file),
            Err(error) => {
                attempt.denied();
                return open_failure(
                    operation,
                    path,
                    kind,
                    Some(&error),
                    super::MediaOperationFailureKind::DeniedBeforeEffect,
                    super::MediaCausalBoundary::OsCallReturned,
                );
            }
        };
        let stable_file = match file.try_clone().and_then(same_file::Handle::from_file) {
            Ok(identity) => identity,
            Err(error) => {
                let (failure, boundary) = post_open_setup_failure(kind);
                terminalize_open_failure(attempt, failure);
                return open_failure(operation, path, kind, Some(&error), failure, boundary);
            }
        };
        let mutation_sequence = if writable {
            match self.mutation_sequence_for(&file) {
                Ok(sequence) => Some(sequence),
                Err(error) => {
                    let (failure, boundary) = post_open_setup_failure(kind);
                    terminalize_open_failure(attempt, failure);
                    return open_failure(operation, path, kind, Some(&error), failure, boundary);
                }
            }
        } else {
            None
        };
        let identity = match self.issue_file_handle_identity() {
            Ok(identity) => identity,
            Err(_) => {
                drop(file);
                let (failure, boundary) = post_open_setup_failure(kind);
                terminalize_open_failure(attempt, failure);
                return open_failure(operation, path, kind, None, failure, boundary);
            }
        };
        let accounting = self.boundary().shared_counters().file_handle_opened(kind);
        attempt.completed(0);
        NamespaceFileOpenOutcome {
            operation,
            result: NamespaceFileOpenResult::Opened {
                kind,
                handle: NamespaceFileHandle {
                    owner: self,
                    identity,
                    path: path.clone(),
                    role: path.role(),
                    file,
                    stable_file,
                    mutation_sequence,
                    _accounting: accounting,
                    access: PhantomData,
                },
            },
        }
    }
}

fn open_failure<'owner, Access>(
    operation: MediaOperationIdentity,
    path: &NamespaceRelativePath,
    kind: NamespaceFileOpenKind,
    error: Option<&std::io::Error>,
    failure_kind: super::MediaOperationFailureKind,
    boundary: super::MediaCausalBoundary,
) -> NamespaceFileOpenOutcome<'owner, Access> {
    NamespaceFileOpenOutcome {
        operation,
        result: NamespaceFileOpenResult::Failed(super::failure_context::operation_failure(
            operation,
            operation_role(kind),
            path.role(),
            None,
            failure_kind,
            error,
            boundary,
        )),
    }
}

fn failed_open<'owner, Access>(
    owner: &'owner FilesystemMediaOwner,
    path: &NamespaceRelativePath,
    kind: NamespaceFileOpenKind,
    _denial: super::FilesystemMediaOwnerAdmissionDenial,
) -> NamespaceFileOpenOutcome<'owner, Access> {
    let operation = owner
        .issue_operation_identity()
        .expect("media operation identity exhausted");
    owner
        .boundary()
        .begin_operation(
            operation_role(kind),
            0,
            super::MediaOperationCoordinates::for_path(operation, path.role(), None),
        )
        .denied();
    NamespaceFileOpenOutcome {
        operation,
        result: NamespaceFileOpenResult::Failed(super::failure_context::operation_failure(
            operation,
            operation_role(kind),
            path.role(),
            None,
            super::MediaOperationFailureKind::DeniedBeforeEffect,
            None,
            super::MediaCausalBoundary::BeforeOsCall,
        )),
    }
}

fn terminalize_open_failure(
    attempt: super::fault_interposition::MediaBoundaryAttempt<'_>,
    failure: super::MediaOperationFailureKind,
) {
    if matches!(
        failure,
        super::MediaOperationFailureKind::IndeterminateEffect { .. }
    ) {
        attempt.indeterminate(0);
    } else {
        attempt.denied();
    }
}

const fn operation_role(kind: NamespaceFileOpenKind) -> super::MediaOperationRole {
    match kind {
        NamespaceFileOpenKind::Existing => super::MediaOperationRole::OpenExisting,
        NamespaceFileOpenKind::CreatedNew => super::MediaOperationRole::CreateNew,
    }
}

const fn post_open_setup_failure(
    kind: NamespaceFileOpenKind,
) -> (super::MediaOperationFailureKind, super::MediaCausalBoundary) {
    if matches!(kind, NamespaceFileOpenKind::CreatedNew) {
        (
            super::MediaOperationFailureKind::IndeterminateEffect {
                attempted: super::MediaAttemptedEffect::NewFileCreation,
                last_established: super::MediaEstablishedBoundary::NamespaceEntryCreationIssued,
            },
            super::MediaCausalBoundary::OsCallReturned,
        )
    } else {
        (
            super::MediaOperationFailureKind::DeniedBeforeEffect,
            super::MediaCausalBoundary::OsCallReturned,
        )
    }
}
