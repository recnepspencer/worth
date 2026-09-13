use super::{ArtifactTreeFailure, ArtifactTreeFailureKind, ArtifactTreeFile, ArtifactTreeMedia};
use crate::filesystem_media::{MediaOperationIdentity, MediaOwnerIdentity};
use crate::{
    BackendQueueExecutionAdaptation, BackendQueueExecutionCompletion,
    BackendQueueExecutionPlanBinding,
};
use std::io::{Seek, SeekFrom};
use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, PhysicalArtifactReadRange,
};

/// Actual C.4 acquisition, including a possibly incomplete or changing read.
#[derive(Debug, Clone)]
pub struct ObservedArtifactInspectionRead {
    owner: MediaOwnerIdentity,
    store: StableStoreIdentity,
    artifact: ArtifactTreeFile,
    range: PhysicalArtifactReadRange,
    operation: MediaOperationIdentity,
    completed_bytes: u64,
    stable: bool,
    source_version: Option<InspectionSourceVersion>,
}

/// Observed OS file incarnation and metadata, not persisted integrity proof.
/// The retained handle prevents identity reuse while a multi-window scan compares it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectionSourceVersion {
    identity: std::sync::Arc<same_file::Handle>,
    length: u64,
    modified: cap_std::time::SystemTime,
}
impl ObservedArtifactInspectionRead {
    pub const fn owner(&self) -> MediaOwnerIdentity {
        self.owner
    }
    pub const fn store(&self) -> StableStoreIdentity {
        self.store
    }
    pub const fn artifact(&self) -> &ArtifactTreeFile {
        &self.artifact
    }
    pub const fn range(&self) -> PhysicalArtifactReadRange {
        self.range
    }
    pub const fn operation(&self) -> MediaOperationIdentity {
        self.operation
    }
    pub const fn completed_bytes(&self) -> u64 {
        self.completed_bytes
    }
    pub const fn stable(&self) -> bool {
        self.stable
    }
    pub const fn source_version(&self) -> Option<&InspectionSourceVersion> {
        self.source_version.as_ref()
    }
}

pub enum ScheduledArtifactInspectionReadOutcome {
    Observed {
        physical: ObservedArtifactInspectionRead,
        queue: BackendQueueExecutionCompletion,
    },
    DeniedBeforeEffect(ArtifactTreeFailure),
}

impl ArtifactTreeMedia<'_> {
    /// Executes one scheduler-admitted diagnostic read through C.4's ordinary
    /// capability-relative acquisition boundary. It grants no decoder authority.
    pub fn read_scheduled_inspection(
        &self,
        artifact: &ArtifactTreeFile,
        range: PhysicalArtifactReadRange,
        target: &mut [u8],
        binding: BackendQueueExecutionPlanBinding,
    ) -> ScheduledArtifactInspectionReadOutcome {
        use ScheduledArtifactInspectionReadOutcome as Outcome;
        if target.len() != range.length() as usize {
            return Outcome::DeniedBeforeEffect(ArtifactTreeFailure::structural(
                ArtifactTreeFailureKind::AccessLimitExceeded,
            ));
        }
        let ticket = match crate::BackendQueueExecutionAuthority::store_owned().issue_ticket(
            binding,
            self.execution_capability,
            BackendQueueExecutionAdaptation::None,
        ) {
            Ok(ticket) => ticket,
            Err(_) => {
                return Outcome::DeniedBeforeEffect(ArtifactTreeFailure::structural(
                    ArtifactTreeFailureKind::DeniedBeforeEffect,
                ))
            }
        };
        let mut file = match self.open_inspection_source(artifact, range.offset()) {
            Ok(file) => file,
            Err(failure) => return Outcome::DeniedBeforeEffect(failure),
        };
        let before = file.metadata().ok();
        let identity = file
            .try_clone()
            .and_then(|file| same_file::Handle::from_file(file.into_std()))
            .ok();
        match super::exact_read_effect::execute(self.owner, &mut file, target) {
            super::exact_read_effect::ExactReadEffect::DeniedBeforeEffect(failure) => {
                Outcome::DeniedBeforeEffect(failure)
            }
            super::exact_read_effect::ExactReadEffect::Completed {
                operation,
                completed_bytes,
            } => {
                let after = file.metadata().ok();
                let stable = before
                    .as_ref()
                    .zip(after.as_ref())
                    .is_some_and(|(before, after)| {
                        before.len() == after.len()
                            && before
                                .modified()
                                .ok()
                                .zip(after.modified().ok())
                                .is_some_and(|(before, after)| before == after)
                    });
                let source_version = identity.zip(after).and_then(|(identity, metadata)| {
                    metadata
                        .modified()
                        .ok()
                        .map(|modified| InspectionSourceVersion {
                            identity: std::sync::Arc::new(identity),
                            length: metadata.len(),
                            modified,
                        })
                });
                Outcome::Observed {
                    physical: ObservedArtifactInspectionRead {
                        owner: self.owner.identity(),
                        store: self.store,
                        artifact: artifact.clone(),
                        range,
                        operation,
                        completed_bytes,
                        stable,
                        source_version,
                    },
                    queue: ticket.begin_completion().observe_queue_depth(1).complete(),
                }
            }
        }
    }

    fn open_inspection_source(
        &self,
        artifact: &ArtifactTreeFile,
        offset: u64,
    ) -> Result<cap_std::fs::File, ArtifactTreeFailure> {
        let directory = self.open_directory(&artifact.directory)?;
        let mut file = self.open_readable_file(&directory, &artifact.file_name)?;
        file.seek(SeekFrom::Start(offset)).map_err(|error| {
            ArtifactTreeFailure::io(ArtifactTreeFailureKind::DeniedBeforeEffect, &error)
        })?;
        Ok(file)
    }
}
