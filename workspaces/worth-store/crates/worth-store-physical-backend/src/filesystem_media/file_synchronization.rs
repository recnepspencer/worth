use super::{
    FileDataSynchronization, FileDataSynchronizationOutcome, FileStateSynchronization,
    FileStateSynchronizationOutcome, MediaAttemptedEffect, MediaCausalBoundary,
    MediaEstablishedBoundary, MediaOperationFailure, MediaOperationFailureKind, MediaOperationRole,
    NamespaceFileHandle,
};

impl NamespaceFileHandle<'_, super::MutableFileAccess> {
    pub fn synchronize_data(&self) -> FileDataSynchronizationOutcome {
        let operation = self.operation_identity_for_sync();
        let attempt = self.owner().boundary().begin_operation(
            MediaOperationRole::SynchronizeFileData,
            0,
            super::MediaOperationCoordinates::for_path(
                operation,
                self.role(),
                Some(self.identity()),
            ),
        );
        if let Some(error) = attempt
            .fail_before_error()
            .or_else(|| attempt.barrier_error())
        {
            attempt.denied();
            return FileDataSynchronizationOutcome::Failed(denied(
                self,
                operation,
                MediaOperationRole::SynchronizeFileData,
                Some(&error),
            ));
        }
        let Ok(_authority) = self.owner().begin_mutation() else {
            attempt.denied();
            return FileDataSynchronizationOutcome::Failed(denied(
                self,
                operation,
                MediaOperationRole::SynchronizeFileData,
                None,
            ));
        };
        let _file_sequence = self.mutation_guard();
        match super::os_synchronization::synchronize_data(self.file()) {
            Ok(()) if attempt.effect_observation_is_indeterminate() => {
                attempt.indeterminate(0);
                FileDataSynchronizationOutcome::Failed(unobserved(
                    self,
                    operation,
                    MediaOperationRole::SynchronizeFileData,
                    MediaAttemptedEffect::FileDataSynchronization,
                    MediaEstablishedBoundary::FileDataSynchronizationIssued,
                    None,
                ))
            }
            Ok(()) => {
                self.owner().boundary().counters().file_sync();
                attempt.completed(0);
                FileDataSynchronizationOutcome::Synchronized(FileDataSynchronization {
                    operation,
                    handle: self.identity(),
                })
            }
            Err(error) => {
                attempt.indeterminate(0);
                FileDataSynchronizationOutcome::Failed(unobserved(
                    self,
                    operation,
                    MediaOperationRole::SynchronizeFileData,
                    MediaAttemptedEffect::FileDataSynchronization,
                    MediaEstablishedBoundary::FileDataSynchronizationIssued,
                    Some(&error),
                ))
            }
        }
    }

    pub fn synchronize_state(&self) -> FileStateSynchronizationOutcome {
        let operation = self.operation_identity_for_sync();
        let attempt = self.owner().boundary().begin_operation(
            MediaOperationRole::SynchronizeFileState,
            0,
            super::MediaOperationCoordinates::for_path(
                operation,
                self.role(),
                Some(self.identity()),
            ),
        );
        if let Some(error) = attempt
            .fail_before_error()
            .or_else(|| attempt.barrier_error())
        {
            attempt.denied();
            return FileStateSynchronizationOutcome::Failed(denied(
                self,
                operation,
                MediaOperationRole::SynchronizeFileState,
                Some(&error),
            ));
        }
        let Ok(_authority) = self.owner().begin_mutation() else {
            attempt.denied();
            return FileStateSynchronizationOutcome::Failed(denied(
                self,
                operation,
                MediaOperationRole::SynchronizeFileState,
                None,
            ));
        };
        let _file_sequence = self.mutation_guard();
        match super::os_synchronization::synchronize_state(self.file()) {
            Ok(()) if attempt.effect_observation_is_indeterminate() => {
                attempt.indeterminate(0);
                FileStateSynchronizationOutcome::Failed(unobserved(
                    self,
                    operation,
                    MediaOperationRole::SynchronizeFileState,
                    MediaAttemptedEffect::FileStateSynchronization,
                    MediaEstablishedBoundary::FileStateSynchronizationIssued,
                    None,
                ))
            }
            Ok(()) => {
                self.owner().boundary().counters().file_sync();
                attempt.completed(0);
                FileStateSynchronizationOutcome::Synchronized(FileStateSynchronization {
                    operation,
                    handle: self.identity(),
                })
            }
            Err(error) => {
                attempt.indeterminate(0);
                FileStateSynchronizationOutcome::Failed(unobserved(
                    self,
                    operation,
                    MediaOperationRole::SynchronizeFileState,
                    MediaAttemptedEffect::FileStateSynchronization,
                    MediaEstablishedBoundary::FileStateSynchronizationIssued,
                    Some(&error),
                ))
            }
        }
    }

    fn operation_identity_for_sync(&self) -> super::MediaOperationIdentity {
        self.owner()
            .issue_operation_identity()
            .expect("media operation identity exhausted")
    }
}

fn unobserved(
    handle: &NamespaceFileHandle<'_, super::MutableFileAccess>,
    operation: super::MediaOperationIdentity,
    role: MediaOperationRole,
    attempted: MediaAttemptedEffect,
    established: MediaEstablishedBoundary,
    error: Option<&std::io::Error>,
) -> MediaOperationFailure {
    super::failure_context::operation_failure(
        operation,
        role,
        handle.role(),
        Some(handle.identity()),
        MediaOperationFailureKind::IndeterminateEffect {
            attempted,
            last_established: established,
        },
        error,
        MediaCausalBoundary::CompletionUnconfirmed,
    )
}

fn denied(
    handle: &NamespaceFileHandle<'_, super::MutableFileAccess>,
    operation: super::MediaOperationIdentity,
    role: MediaOperationRole,
    error: Option<&std::io::Error>,
) -> MediaOperationFailure {
    super::failure_context::operation_failure(
        operation,
        role,
        handle.role(),
        Some(handle.identity()),
        MediaOperationFailureKind::DeniedBeforeEffect,
        error,
        MediaCausalBoundary::BeforeOsCall,
    )
}
