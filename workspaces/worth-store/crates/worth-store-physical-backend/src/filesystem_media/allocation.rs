use super::{
    MediaCausalBoundary, MediaOperationFailure, MediaOperationFailureKind, MediaOperationRole,
    NamespaceFileHandle,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaAllocationMode {
    LogicalLengthOnly,
    SparsePhysicalRange,
    EagerPhysicalRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocationRequest {
    offset: u64,
    length: u64,
    mode: MediaAllocationMode,
}

impl AllocationRequest {
    pub const fn new(offset: u64, length: u64, mode: MediaAllocationMode) -> Self {
        Self {
            offset,
            length,
            mode,
        }
    }

    pub const fn offset(self) -> u64 {
        self.offset
    }

    pub const fn length(self) -> u64 {
        self.length
    }

    pub const fn mode(self) -> MediaAllocationMode {
        self.mode
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaPhysicalAllocationPosture {
    NotRequested,
    SparseRangeEstablished,
    EagerRangeEstablished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MediaAllocationObservation {
    logical_length: u64,
    physical: MediaPhysicalAllocationPosture,
}

impl MediaAllocationObservation {
    pub const fn logical_length(self) -> u64 {
        self.logical_length
    }

    pub const fn physical(self) -> MediaPhysicalAllocationPosture {
        self.physical
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaAllocationResult {
    Completed(MediaAllocationObservation),
    Failed(MediaOperationFailure),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MediaAllocationOutcome {
    operation: super::MediaOperationIdentity,
    result: MediaAllocationResult,
}

impl MediaAllocationOutcome {
    pub const fn operation(self) -> super::MediaOperationIdentity {
        self.operation
    }

    pub const fn result(self) -> MediaAllocationResult {
        self.result
    }
}

impl NamespaceFileHandle<'_, super::MutableFileAccess> {
    pub fn allocate(&self, request: AllocationRequest) -> MediaAllocationOutcome {
        let operation = self
            .owner()
            .issue_operation_identity()
            .expect("media operation identity exhausted");
        let attempt = self.owner().boundary().begin_operation(
            MediaOperationRole::Allocate,
            request.length(),
            super::MediaOperationCoordinates::for_path(
                operation,
                self.role(),
                Some(self.identity()),
            )
            .at_offset(request.offset()),
        );
        if let Some(error) = attempt.fail_before_error() {
            attempt.denied();
            return self.allocation_failure(
                operation,
                MediaOperationFailureKind::DeniedBeforeEffect,
                Some(&error),
            );
        }
        if request.length() == 0 {
            attempt.denied();
            return self.allocation_failure(
                operation,
                MediaOperationFailureKind::DeniedBeforeEffect,
                None,
            );
        }
        let Ok(_authority) = self.owner().begin_mutation() else {
            attempt.denied();
            return self.allocation_failure(
                operation,
                MediaOperationFailureKind::DeniedBeforeEffect,
                None,
            );
        };
        let Some(end) = request.offset().checked_add(request.length()) else {
            attempt.denied();
            return self.allocation_failure(
                operation,
                MediaOperationFailureKind::DeniedBeforeEffect,
                None,
            );
        };
        let _guard = self.mutation_guard();
        match request.mode() {
            MediaAllocationMode::LogicalLengthOnly => {
                self.extend_logical_length(attempt, operation, end)
            }
            MediaAllocationMode::SparsePhysicalRange => {
                attempt.unsupported_capability();
                self.unsupported_allocation(operation, super::MediaCapability::SparseAllocation)
            }
            MediaAllocationMode::EagerPhysicalRange => {
                attempt.unsupported_capability();
                self.unsupported_allocation(operation, super::MediaCapability::EagerAllocation)
            }
        }
    }

    fn extend_logical_length(
        &self,
        attempt: super::fault_interposition::MediaBoundaryAttempt<'_>,
        operation: super::MediaOperationIdentity,
        requested_end: u64,
    ) -> MediaAllocationOutcome {
        let current = match self.file().metadata() {
            Ok(metadata) => metadata.len(),
            Err(error) => {
                attempt.denied();
                return self.allocation_failure(
                    operation,
                    MediaOperationFailureKind::DeniedBeforeEffect,
                    Some(&error),
                );
            }
        };
        let logical_length = current.max(requested_end);
        if logical_length > current
            && let Err(error) = self.file().set_len(logical_length)
        {
            attempt.indeterminate(0);
            return self.allocation_failure(
                operation,
                MediaOperationFailureKind::IndeterminateEffect {
                    attempted: super::MediaAttemptedEffect::Allocation,
                    last_established: super::MediaEstablishedBoundary::AllocationIssued,
                },
                Some(&error),
            );
        }
        if attempt.effect_observation_is_indeterminate() {
            attempt.indeterminate(0);
            return self.allocation_failure(
                operation,
                MediaOperationFailureKind::IndeterminateEffect {
                    attempted: super::MediaAttemptedEffect::Allocation,
                    last_established: super::MediaEstablishedBoundary::AllocationIssued,
                },
                None,
            );
        }
        attempt.completed(0);
        MediaAllocationOutcome {
            operation,
            result: MediaAllocationResult::Completed(MediaAllocationObservation {
                logical_length,
                physical: MediaPhysicalAllocationPosture::NotRequested,
            }),
        }
    }

    fn unsupported_allocation(
        &self,
        operation: super::MediaOperationIdentity,
        capability: super::MediaCapability,
    ) -> MediaAllocationOutcome {
        self.allocation_failure(
            operation,
            MediaOperationFailureKind::UnsupportedCapability(capability),
            None,
        )
    }

    fn allocation_failure(
        &self,
        operation: super::MediaOperationIdentity,
        kind: MediaOperationFailureKind,
        error: Option<&std::io::Error>,
    ) -> MediaAllocationOutcome {
        MediaAllocationOutcome {
            operation,
            result: MediaAllocationResult::Failed(super::failure_context::operation_failure(
                operation,
                MediaOperationRole::Allocate,
                self.role(),
                Some(self.identity()),
                kind,
                error,
                if error.is_some() {
                    MediaCausalBoundary::CompletionUnconfirmed
                } else {
                    MediaCausalBoundary::BeforeOsCall
                },
            )),
        }
    }
}
