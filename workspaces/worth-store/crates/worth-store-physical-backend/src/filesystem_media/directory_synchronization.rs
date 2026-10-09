use super::{
    DirectoryPublicationSynchronization, DirectoryPublicationSynchronizationOutcome,
    FilesystemMediaOwner, MediaAttemptedEffect, MediaCausalBoundary, MediaEstablishedBoundary,
    MediaOperationFailure, MediaOperationFailureKind, MediaOperationRole, NamespaceDirectoryHandle,
    RootParentPublicationSynchronization, RootParentPublicationSynchronizationOutcome,
    StoreRootPublicationSynchronization, StoreRootPublicationSynchronizationOutcome,
};

impl FilesystemMediaOwner {
    pub fn synchronize_directory_publication(
        &self,
        directory: &NamespaceDirectoryHandle,
    ) -> DirectoryPublicationSynchronizationOutcome {
        let Ok(_authority) = self.begin_mutation() else {
            return self.directory_sync_denied(
                directory,
                MediaOperationRole::SynchronizeDirectoryPublication,
            );
        };
        self.synchronize_owned_directory_publication(
            directory,
            MediaOperationRole::SynchronizeDirectoryPublication,
        )
    }

    pub(super) fn synchronize_coordinated_directory_publication(
        &self,
        directory: &NamespaceDirectoryHandle,
        authority: &super::mutation_ownership::CoordinatedNamespaceMutation<'_>,
    ) -> DirectoryPublicationSynchronizationOutcome {
        if !authority.belongs_to(self.identity()) {
            return self.directory_sync_denied(
                directory,
                MediaOperationRole::SynchronizeDirectoryPublication,
            );
        }
        self.synchronize_owned_directory_publication(
            directory,
            MediaOperationRole::SynchronizeDirectoryPublication,
        )
    }

    fn synchronize_owned_directory_publication(
        &self,
        directory: &NamespaceDirectoryHandle,
        operation_role: MediaOperationRole,
    ) -> DirectoryPublicationSynchronizationOutcome {
        if self.require_owned_directory(directory).is_err() {
            return self.directory_sync_denied(directory, operation_role);
        }
        let operation = self
            .issue_operation_identity()
            .expect("media operation identity exhausted");
        let attempt = self.boundary().begin_operation(
            operation_role,
            0,
            super::MediaOperationCoordinates::for_path(
                operation,
                directory.role(),
                Some(directory.identity()),
            ),
        );
        if let Some(error) = attempt
            .fail_before_error()
            .or_else(|| attempt.barrier_error())
        {
            attempt.denied();
            return DirectoryPublicationSynchronizationOutcome::Failed(directory_failure(
                directory,
                operation,
                Some(&error),
                false,
                operation_role,
            ));
        }
        let result = open_directory_sync_handle(directory.directory())
            .and_then(|file| super::os_synchronization::synchronize_state(&file));
        match result {
            Ok(()) if attempt.effect_observation_is_indeterminate() => {
                attempt.indeterminate(0);
                DirectoryPublicationSynchronizationOutcome::Failed(directory_failure(
                    directory,
                    operation,
                    None,
                    true,
                    operation_role,
                ))
            }
            Ok(()) => {
                self.boundary().counters().directory_sync();
                attempt.completed(0);
                DirectoryPublicationSynchronizationOutcome::Synchronized(
                    DirectoryPublicationSynchronization {
                        operation,
                        handle: directory.identity(),
                    },
                )
            }
            Err(error) => {
                attempt.indeterminate(0);
                DirectoryPublicationSynchronizationOutcome::Failed(directory_failure(
                    directory,
                    operation,
                    Some(&error),
                    true,
                    operation_role,
                ))
            }
        }
    }

    fn directory_sync_denied(
        &self,
        directory: &NamespaceDirectoryHandle,
        operation_role: MediaOperationRole,
    ) -> DirectoryPublicationSynchronizationOutcome {
        let operation = self
            .issue_operation_identity()
            .expect("media operation identity exhausted");
        self.boundary()
            .begin_operation(
                operation_role,
                0,
                super::MediaOperationCoordinates::for_path(
                    operation,
                    directory.role(),
                    Some(directory.identity()),
                ),
            )
            .denied();
        DirectoryPublicationSynchronizationOutcome::Failed(
            super::failure_context::operation_failure(
                operation,
                operation_role,
                directory.role(),
                Some(directory.identity()),
                MediaOperationFailureKind::DeniedBeforeEffect,
                None,
                MediaCausalBoundary::BeforeOsCall,
            ),
        )
    }

    pub(super) fn synchronize_coordinated_store_root_publication(
        &self,
        authority: &super::mutation_ownership::CoordinatedNamespaceMutation<'_>,
    ) -> StoreRootPublicationSynchronizationOutcome {
        self.synchronize_store_root_with(|owner, directory| {
            if !authority.belongs_to(owner.identity()) {
                owner.directory_sync_denied(
                    directory,
                    MediaOperationRole::SynchronizeStoreRootPublication,
                )
            } else {
                owner.synchronize_owned_directory_publication(
                    directory,
                    MediaOperationRole::SynchronizeStoreRootPublication,
                )
            }
        })
    }

    fn synchronize_store_root_with(
        &self,
        synchronize: impl FnOnce(
            &Self,
            &NamespaceDirectoryHandle,
        ) -> DirectoryPublicationSynchronizationOutcome,
    ) -> StoreRootPublicationSynchronizationOutcome {
        if !self.store_root_publication_required() {
            return StoreRootPublicationSynchronizationOutcome::NotRequired;
        }
        match synchronize(self, self.root_directory_handle()) {
            DirectoryPublicationSynchronizationOutcome::Synchronized(synchronization) => {
                self.mark_store_root_published();
                StoreRootPublicationSynchronizationOutcome::Synchronized(
                    StoreRootPublicationSynchronization {
                        operation: synchronization.operation(),
                        handle: synchronization.handle(),
                    },
                )
            }
            DirectoryPublicationSynchronizationOutcome::Failed(failure) => {
                StoreRootPublicationSynchronizationOutcome::Failed(failure)
            }
        }
    }

    pub(super) fn synchronize_coordinated_created_root_parent(
        &self,
        authority: &super::mutation_ownership::CoordinatedNamespaceMutation<'_>,
    ) -> RootParentPublicationSynchronizationOutcome {
        if !authority.belongs_to(self.identity()) {
            return self.root_parent_sync_denied();
        }
        self.synchronize_owned_created_root_parent()
    }

    fn synchronize_owned_created_root_parent(&self) -> RootParentPublicationSynchronizationOutcome {
        if !self.root_parent_publication_required() {
            return RootParentPublicationSynchronizationOutcome::NotRequired;
        }
        let Some(parent) = self.root_parent_directory() else {
            return RootParentPublicationSynchronizationOutcome::NotRequired;
        };
        let operation = self
            .issue_operation_identity()
            .expect("media operation identity exhausted");
        let attempt = self.boundary().begin_operation(
            MediaOperationRole::SynchronizeRootParentPublication,
            0,
            super::MediaOperationCoordinates::for_path(
                operation,
                super::MediaPathRole::ArtifactOwned,
                None,
            ),
        );
        if let Some(error) = attempt
            .fail_before_error()
            .or_else(|| attempt.barrier_error())
        {
            attempt.denied();
            return RootParentPublicationSynchronizationOutcome::Failed(root_parent_failure(
                operation,
                Some(&error),
                false,
            ));
        }
        match open_directory_sync_handle(parent)
            .and_then(|file| super::os_synchronization::synchronize_state(&file))
        {
            Ok(()) if attempt.effect_observation_is_indeterminate() => {
                attempt.indeterminate(0);
                RootParentPublicationSynchronizationOutcome::Failed(root_parent_failure(
                    operation, None, true,
                ))
            }
            Ok(()) => {
                self.boundary().counters().directory_sync();
                attempt.completed(0);
                self.mark_root_parent_published();
                RootParentPublicationSynchronizationOutcome::Synchronized(
                    RootParentPublicationSynchronization { operation },
                )
            }
            Err(error) => {
                attempt.indeterminate(0);
                RootParentPublicationSynchronizationOutcome::Failed(root_parent_failure(
                    operation,
                    Some(&error),
                    true,
                ))
            }
        }
    }

    fn root_parent_sync_denied(&self) -> RootParentPublicationSynchronizationOutcome {
        let operation = self
            .issue_operation_identity()
            .expect("media operation identity exhausted");
        self.boundary()
            .begin_operation(
                MediaOperationRole::SynchronizeRootParentPublication,
                0,
                super::MediaOperationCoordinates::for_path(
                    operation,
                    super::MediaPathRole::ArtifactOwned,
                    None,
                ),
            )
            .denied();
        RootParentPublicationSynchronizationOutcome::Failed(
            super::failure_context::operation_failure(
                operation,
                MediaOperationRole::SynchronizeRootParentPublication,
                super::MediaPathRole::ArtifactOwned,
                None,
                MediaOperationFailureKind::DeniedBeforeEffect,
                None,
                MediaCausalBoundary::BeforeOsCall,
            ),
        )
    }

    pub fn synchronize_store_root_publication(&self) -> StoreRootPublicationSynchronizationOutcome {
        self.synchronize_store_root_with(|owner, directory| {
            let Ok(_authority) = owner.begin_mutation() else {
                return owner.directory_sync_denied(
                    directory,
                    MediaOperationRole::SynchronizeStoreRootPublication,
                );
            };
            owner.synchronize_owned_directory_publication(
                directory,
                MediaOperationRole::SynchronizeStoreRootPublication,
            )
        })
    }

    pub fn synchronize_created_root_parent(&self) -> RootParentPublicationSynchronizationOutcome {
        let Ok(_authority) = self.begin_mutation() else {
            return self.root_parent_sync_denied();
        };
        self.synchronize_owned_created_root_parent()
    }
}

pub(super) fn synchronize_directory_handle(directory: &cap_std::fs::Dir) -> std::io::Result<()> {
    super::os_synchronization::synchronize_state(&open_directory_sync_handle(directory)?)
}

fn open_directory_sync_handle(directory: &cap_std::fs::Dir) -> std::io::Result<std::fs::File> {
    use cap_std::fs::OpenOptionsExt;
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    let mut options = cap_std::fs::OpenOptions::new();
    options
        .read(true)
        .write(true)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS);
    directory
        .open_with(".", &options)
        .map(cap_std::fs::File::into_std)
}

#[cfg(not(windows))]
fn open_directory_sync_handle(directory: &cap_std::fs::Dir) -> std::io::Result<std::fs::File> {
    directory.try_clone().map(cap_std::fs::Dir::into_std_file)
}

fn directory_failure(
    directory: &NamespaceDirectoryHandle,
    operation: super::MediaOperationIdentity,
    error: Option<&std::io::Error>,
    issued: bool,
    operation_role: MediaOperationRole,
) -> MediaOperationFailure {
    failure(
        operation,
        directory.role(),
        Some(directory.identity()),
        error,
        issued,
        operation_role,
    )
}

fn root_parent_failure(
    operation: super::MediaOperationIdentity,
    error: Option<&std::io::Error>,
    issued: bool,
) -> MediaOperationFailure {
    failure(
        operation,
        super::MediaPathRole::ArtifactOwned,
        None,
        error,
        issued,
        MediaOperationRole::SynchronizeRootParentPublication,
    )
}

fn failure(
    operation: super::MediaOperationIdentity,
    role: super::MediaPathRole,
    handle: Option<super::MediaHandleIdentity>,
    error: Option<&std::io::Error>,
    issued: bool,
    operation_role: MediaOperationRole,
) -> MediaOperationFailure {
    super::failure_context::operation_failure(
        operation,
        operation_role,
        role,
        handle,
        if issued {
            MediaOperationFailureKind::IndeterminateEffect {
                attempted: MediaAttemptedEffect::DirectoryPublicationSynchronization,
                last_established:
                    MediaEstablishedBoundary::DirectoryPublicationSynchronizationIssued,
            }
        } else {
            MediaOperationFailureKind::DeniedBeforeEffect
        },
        error,
        if issued {
            MediaCausalBoundary::CompletionUnconfirmed
        } else {
            MediaCausalBoundary::BeforeOsCall
        },
    )
}
