use sha2::{Digest, Sha256};
use worth_store_physical_format::store_namespace::StableStoreIdentity;

use super::{
    range_io::ArtifactTreeCreateFileOutcome, ArtifactTreeFailure, ArtifactTreeFailureKind,
    ArtifactTreeFile, ArtifactTreeMedia,
};
use crate::{
    filesystem_media::{MediaOperationIdentity, MediaOwnerIdentity},
    BackendQueueExecutionAdaptation, BackendQueueExecutionCompletion,
    BackendQueueExecutionPlanBinding,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArtifactNewWriteRange {
    offset: u64,
    byte_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletedArtifactNewWrite {
    owner: MediaOwnerIdentity,
    store: StableStoreIdentity,
    artifact: ArtifactTreeFile,
    range: ArtifactNewWriteRange,
    payload_digest: [u8; 32],
    create_operation: MediaOperationIdentity,
    write_operation: MediaOperationIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndeterminateArtifactNewWrite {
    failure: ArtifactTreeFailure,
    owner: MediaOwnerIdentity,
    store: StableStoreIdentity,
    artifact: ArtifactTreeFile,
    range: ArtifactNewWriteRange,
    payload_digest: [u8; 32],
    completed_bytes: u64,
    create_operation: MediaOperationIdentity,
    write_operation: Option<MediaOperationIdentity>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletedScheduledArtifactNewWrite {
    physical: CompletedArtifactNewWrite,
    queue: BackendQueueExecutionCompletion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactNewWriteOutcome {
    Completed(CompletedArtifactNewWrite),
    DeniedBeforeEffect(ArtifactTreeFailure),
    Indeterminate(IndeterminateArtifactNewWrite),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduledArtifactNewWriteOutcome {
    Completed(Box<CompletedScheduledArtifactNewWrite>),
    DeniedBeforeEffect(ArtifactTreeFailure),
    Indeterminate(IndeterminateArtifactNewWrite),
}

pub(super) enum ArtifactNewFileWriteOutcome {
    Completed(MediaOperationIdentity),
    DeniedBeforeEffect(ArtifactTreeFailure),
    Indeterminate {
        failure: ArtifactTreeFailure,
        completed_bytes: u64,
        operation: MediaOperationIdentity,
    },
}

impl ArtifactNewWriteRange {
    pub const fn new(byte_count: u64) -> Option<Self> {
        if byte_count == 0 {
            None
        } else {
            Some(Self {
                offset: 0,
                byte_count,
            })
        }
    }

    pub const fn byte_count(self) -> u64 {
        self.byte_count
    }

    pub const fn at(offset: u64, byte_count: u64) -> Option<Self> {
        if byte_count == 0 || offset.checked_add(byte_count).is_none() {
            None
        } else {
            Some(Self { offset, byte_count })
        }
    }

    pub const fn offset(self) -> u64 {
        self.offset
    }
}

impl ArtifactTreeMedia<'_> {
    pub fn write_new_exact(
        &self,
        artifact: &ArtifactTreeFile,
        range: ArtifactNewWriteRange,
        bytes: &[u8],
    ) -> ArtifactNewWriteOutcome {
        if range.byte_count() != bytes.len() as u64 {
            return ArtifactNewWriteOutcome::DeniedBeforeEffect(ArtifactTreeFailure::structural(
                ArtifactTreeFailureKind::AccessLimitExceeded,
            ));
        }
        let mut file = match self.create_new_file_observed(artifact) {
            ArtifactTreeCreateFileOutcome::Created(file) => file,
            ArtifactTreeCreateFileOutcome::DeniedBeforeEffect(failure) => {
                return ArtifactNewWriteOutcome::DeniedBeforeEffect(failure);
            }
            ArtifactTreeCreateFileOutcome::Indeterminate { failure, operation } => {
                return ArtifactNewWriteOutcome::Indeterminate(
                    IndeterminateArtifactNewWrite::after_create(
                        failure,
                        self.owner.identity(),
                        self.store,
                        artifact.clone(),
                        range,
                        bytes,
                        operation,
                    ),
                );
            }
        };
        let create_operation = file.create_operation();
        match file.write_exact_artifact_chunk(range.offset(), bytes) {
            ArtifactNewFileWriteOutcome::Completed(write_operation) => {
                ArtifactNewWriteOutcome::Completed(CompletedArtifactNewWrite::new(
                    self.owner.identity(),
                    self.store,
                    artifact.clone(),
                    range,
                    bytes,
                    create_operation,
                    write_operation,
                ))
            }
            ArtifactNewFileWriteOutcome::DeniedBeforeEffect(failure) => {
                ArtifactNewWriteOutcome::Indeterminate(IndeterminateArtifactNewWrite::new(
                    failure,
                    self.owner.identity(),
                    self.store,
                    artifact.clone(),
                    range,
                    bytes,
                    0,
                    create_operation,
                    None,
                ))
            }
            ArtifactNewFileWriteOutcome::Indeterminate {
                failure,
                completed_bytes,
                operation,
            } => ArtifactNewWriteOutcome::Indeterminate(IndeterminateArtifactNewWrite::new(
                failure,
                self.owner.identity(),
                self.store,
                artifact.clone(),
                range,
                bytes,
                completed_bytes,
                create_operation,
                Some(operation),
            )),
        }
    }

    pub fn write_scheduled_new_exact(
        &self,
        artifact: &ArtifactTreeFile,
        range: ArtifactNewWriteRange,
        bytes: &[u8],
        binding: BackendQueueExecutionPlanBinding,
        adaptation: BackendQueueExecutionAdaptation,
    ) -> ScheduledArtifactNewWriteOutcome {
        let ticket = match crate::BackendQueueExecutionAuthority::store_owned().issue_ticket(
            binding,
            self.execution_capability,
            adaptation,
        ) {
            Ok(ticket) => ticket,
            Err(_) => {
                return ScheduledArtifactNewWriteOutcome::DeniedBeforeEffect(
                    ArtifactTreeFailure::structural(ArtifactTreeFailureKind::DeniedBeforeEffect),
                );
            }
        };
        match self.write_new_exact(artifact, range, bytes) {
            ArtifactNewWriteOutcome::Completed(physical) => {
                ScheduledArtifactNewWriteOutcome::Completed(Box::new(
                    CompletedScheduledArtifactNewWrite {
                        physical,
                        queue: ticket.begin_completion().observe_queue_depth(1).complete(),
                    },
                ))
            }
            ArtifactNewWriteOutcome::DeniedBeforeEffect(failure) => {
                ScheduledArtifactNewWriteOutcome::DeniedBeforeEffect(failure)
            }
            ArtifactNewWriteOutcome::Indeterminate(failure) => {
                ScheduledArtifactNewWriteOutcome::Indeterminate(failure)
            }
        }
    }
}

impl CompletedArtifactNewWrite {
    #[allow(clippy::too_many_arguments)]
    fn new(
        owner: MediaOwnerIdentity,
        store: StableStoreIdentity,
        artifact: ArtifactTreeFile,
        range: ArtifactNewWriteRange,
        bytes: &[u8],
        create_operation: MediaOperationIdentity,
        write_operation: MediaOperationIdentity,
    ) -> Self {
        Self {
            owner,
            store,
            artifact,
            range,
            payload_digest: Sha256::digest(bytes).into(),
            create_operation,
            write_operation,
        }
    }

    pub const fn owner(&self) -> MediaOwnerIdentity {
        self.owner
    }
    pub const fn store(&self) -> StableStoreIdentity {
        self.store
    }
    pub const fn artifact(&self) -> &ArtifactTreeFile {
        &self.artifact
    }
    pub const fn range(&self) -> ArtifactNewWriteRange {
        self.range
    }
    pub const fn payload_digest(&self) -> [u8; 32] {
        self.payload_digest
    }
    pub const fn completed_bytes(&self) -> u64 {
        self.range.byte_count()
    }
    pub const fn create_operation(&self) -> MediaOperationIdentity {
        self.create_operation
    }
    pub const fn write_operation(&self) -> MediaOperationIdentity {
        self.write_operation
    }
}

impl CompletedScheduledArtifactNewWrite {
    pub const fn physical(&self) -> &CompletedArtifactNewWrite {
        &self.physical
    }
    pub const fn queue(&self) -> BackendQueueExecutionCompletion {
        self.queue
    }
}

impl IndeterminateArtifactNewWrite {
    #[allow(clippy::too_many_arguments)]
    fn after_create(
        failure: ArtifactTreeFailure,
        owner: MediaOwnerIdentity,
        store: StableStoreIdentity,
        artifact: ArtifactTreeFile,
        range: ArtifactNewWriteRange,
        bytes: &[u8],
        create_operation: MediaOperationIdentity,
    ) -> Self {
        Self::new(
            failure,
            owner,
            store,
            artifact,
            range,
            bytes,
            0,
            create_operation,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new(
        failure: ArtifactTreeFailure,
        owner: MediaOwnerIdentity,
        store: StableStoreIdentity,
        artifact: ArtifactTreeFile,
        range: ArtifactNewWriteRange,
        bytes: &[u8],
        completed_bytes: u64,
        create_operation: MediaOperationIdentity,
        write_operation: Option<MediaOperationIdentity>,
    ) -> Self {
        Self {
            failure,
            owner,
            store,
            artifact,
            range,
            payload_digest: Sha256::digest(bytes).into(),
            completed_bytes,
            create_operation,
            write_operation,
        }
    }

    pub const fn failure(&self) -> ArtifactTreeFailure {
        self.failure
    }
    pub const fn owner(&self) -> MediaOwnerIdentity {
        self.owner
    }
    pub const fn store(&self) -> StableStoreIdentity {
        self.store
    }
    pub const fn artifact(&self) -> &ArtifactTreeFile {
        &self.artifact
    }
    pub const fn range(&self) -> ArtifactNewWriteRange {
        self.range
    }
    pub const fn payload_digest(&self) -> [u8; 32] {
        self.payload_digest
    }
    pub const fn completed_bytes(&self) -> u64 {
        self.completed_bytes
    }
    pub const fn create_operation(&self) -> MediaOperationIdentity {
        self.create_operation
    }
    pub const fn write_operation(&self) -> Option<MediaOperationIdentity> {
        self.write_operation
    }
}
