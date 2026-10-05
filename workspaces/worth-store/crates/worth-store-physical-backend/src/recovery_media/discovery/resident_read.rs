use worth_store_physical_format::{ExtentArenaRange, RecordArtifactFile};

use crate::filesystem_media::{
    ArtifactTreeAllocatedReadFailure, ArtifactTreeDirectory, ArtifactTreeFailureKind,
    ArtifactTreeFile,
};

use super::{
    record_artifact, FilesystemObservation, ObservedRecoveryArtifact, RecoveryDiscoveryArtifact,
    RecoveryDiscoveryFailure,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryDiscoveryAllocationFailure<E> {
    Discovery(RecoveryDiscoveryFailure),
    Allocation {
        artifact: RecoveryDiscoveryArtifact,
        offset: u64,
        requested: usize,
        cause: E,
    },
    BufferLengthMismatch {
        artifact: RecoveryDiscoveryArtifact,
        offset: u64,
        requested: usize,
        observed: usize,
    },
}

impl<E> From<RecoveryDiscoveryFailure> for RecoveryDiscoveryAllocationFailure<E> {
    fn from(failure: RecoveryDiscoveryFailure) -> Self {
        Self::Discovery(failure)
    }
}

impl<M: super::DiscoveryMediaBacking> FilesystemObservation<M> {
    pub fn read_extent_manifest_with_allocator<E>(
        &mut self,
        range: ExtentArenaRange,
        byte_limit: u64,
        allocate: impl FnOnce(usize) -> Result<Vec<u8>, E>,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryAllocationFailure<E>> {
        self.read_extent_range_with_allocator(range, 0, 104, byte_limit, allocate)
    }

    pub fn read_extent_range_with_allocator<E>(
        &mut self,
        range: ExtentArenaRange,
        offset: u64,
        length: u32,
        byte_limit: u64,
        allocate: impl FnOnce(usize) -> Result<Vec<u8>, E>,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryAllocationFailure<E>> {
        let address = RecordArtifactFile::ExtentArena {
            arena: range.arena().get(),
        };
        let context = RecoveryDiscoveryArtifact::Record(address);
        let end = offset
            .checked_add(u64::from(length))
            .filter(|end| *end <= range.length())
            .ok_or_else(|| RecoveryDiscoveryFailure::invalid(context.clone()))?;
        let absolute = range
            .offset()
            .checked_add(end - u64::from(length))
            .ok_or_else(|| RecoveryDiscoveryFailure::invalid(context))?;
        self.read_record_range_with_allocator(address, absolute, length, byte_limit, allocate)
    }

    pub fn read_record_artifact_with_allocator<E>(
        &mut self,
        address: RecordArtifactFile,
        byte_limit: u64,
        allocate: impl FnOnce(usize) -> Result<Vec<u8>, E>,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryAllocationFailure<E>> {
        let context = RecoveryDiscoveryArtifact::Record(address);
        let artifact = record_artifact(address)?;
        let fixed = matches!(
            address,
            RecordArtifactFile::BootstrapCatalog
                | RecordArtifactFile::CurrentRootSelector
                | RecordArtifactFile::PreviousRootSelector
        );
        let result = self.read_whole(artifact, context, byte_limit, fixed, allocate)?;
        if fixed {
            self.counters.fixed_slots_read += 1;
        }
        Ok(result)
    }

    pub fn read_current_checkpoint_with_allocator<E>(
        &mut self,
        byte_limit: u64,
        allocate: impl FnOnce(usize) -> Result<Vec<u8>, E>,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryAllocationFailure<E>> {
        let context = RecoveryDiscoveryArtifact::CurrentCheckpoint;
        let artifact = ArtifactTreeDirectory::families()
            .file("checkpoint.current")
            .map_err(|_| RecoveryDiscoveryFailure::invalid(context.clone()))?;
        self.read_whole(artifact, context, byte_limit, false, allocate)
    }

    pub fn read_record_range_with_allocator<E>(
        &mut self,
        address: RecordArtifactFile,
        offset: u64,
        length: u32,
        byte_limit: u64,
        allocate: impl FnOnce(usize) -> Result<Vec<u8>, E>,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryAllocationFailure<E>> {
        let context = RecoveryDiscoveryArtifact::Record(address);
        let artifact = record_artifact(address)?;
        let length = u64::from(length);
        self.admit_range(&context, length, byte_limit)?;
        self.admit_read()?;
        let capacity = usize::try_from(length)
            .map_err(|_| RecoveryDiscoveryFailure::invalid(context.clone()))?;
        self.remaining_entries -= 1;
        match self
            .parts
            .artifact_tree()
            .read_exact_at_with_allocator(&artifact, offset, capacity, allocate)
        {
            Ok(bytes) => {
                self.remaining_bytes -= length;
                self.counters.bytes_read += length;
                self.counters.addressed_artifacts_read += 1;
                Ok(ObservedRecoveryArtifact::new(
                    self.parts.store_identity(),
                    context,
                    offset,
                    Some(bytes),
                ))
            }
            Err(ArtifactTreeAllocatedReadFailure::Media(failure))
                if failure.kind() == ArtifactTreeFailureKind::Absent =>
            {
                Ok(ObservedRecoveryArtifact::new(
                    self.parts.store_identity(),
                    context,
                    offset,
                    None,
                ))
            }
            Err(failure) => Err(map_allocated_failure(failure, context, offset)),
        }
    }

    pub(super) fn read_whole<E>(
        &mut self,
        artifact: ArtifactTreeFile,
        context: RecoveryDiscoveryArtifact,
        byte_limit: u64,
        fixed: bool,
        allocate: impl FnOnce(usize) -> Result<Vec<u8>, E>,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryAllocationFailure<E>> {
        self.read_whole_with(context, byte_limit, fixed, |tree, limit| {
            tree.read_bounded_with_allocator(&artifact, limit, allocate)
        })
    }

    pub(super) fn read_whole_with<E>(
        &mut self,
        context: RecoveryDiscoveryArtifact,
        byte_limit: u64,
        fixed: bool,
        read: impl FnOnce(
            crate::filesystem_media::ArtifactTreeMedia<'_>,
            u64,
        ) -> Result<Vec<u8>, ArtifactTreeAllocatedReadFailure<E>>,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryAllocationFailure<E>> {
        let effective_byte_limit = byte_limit.min(self.remaining_bytes);
        self.admit_read()?;
        self.remaining_entries -= 1;
        match read(self.parts.artifact_tree(), effective_byte_limit) {
            Ok(bytes) => {
                self.spend_read_bytes(bytes.len() as u64)?;
                if !fixed {
                    self.counters.addressed_artifacts_read += 1;
                }
                Ok(ObservedRecoveryArtifact::new(
                    self.parts.store_identity(),
                    context,
                    0,
                    Some(bytes),
                ))
            }
            Err(ArtifactTreeAllocatedReadFailure::Media(failure))
                if failure.kind() == ArtifactTreeFailureKind::Absent =>
            {
                Ok(ObservedRecoveryArtifact::new(
                    self.parts.store_identity(),
                    context,
                    0,
                    None,
                ))
            }
            Err(failure) => Err(match &failure {
                ArtifactTreeAllocatedReadFailure::Media(media) => {
                    self.whole_read_refused(media, byte_limit, effective_byte_limit)
                }
                _ => None,
            }
            .map_or_else(
                || map_allocated_failure(failure, context, 0),
                RecoveryDiscoveryAllocationFailure::Discovery,
            )),
        }
    }
}

pub(super) fn map_allocated_failure<E>(
    failure: ArtifactTreeAllocatedReadFailure<E>,
    artifact: RecoveryDiscoveryArtifact,
    offset: u64,
) -> RecoveryDiscoveryAllocationFailure<E> {
    match failure {
        ArtifactTreeAllocatedReadFailure::Media(failure) => {
            RecoveryDiscoveryAllocationFailure::Discovery(RecoveryDiscoveryFailure::Media {
                artifact,
                failure,
            })
        }
        ArtifactTreeAllocatedReadFailure::Allocation { requested, cause } => {
            RecoveryDiscoveryAllocationFailure::Allocation {
                artifact,
                offset,
                requested,
                cause,
            }
        }
        ArtifactTreeAllocatedReadFailure::BufferLengthMismatch {
            requested,
            observed,
        } => RecoveryDiscoveryAllocationFailure::BufferLengthMismatch {
            artifact,
            offset,
            requested,
            observed,
        },
    }
}
