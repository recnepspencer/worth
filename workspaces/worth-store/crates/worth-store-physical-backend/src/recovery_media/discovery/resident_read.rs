use worth_foundational::LimitDimension;
use worth_store_physical_format::{
    ExtentArenaRange, RecordArtifactFile, EXTENT_ARENA_MANIFEST_FRAME_BYTES,
};

use crate::filesystem_media::{
    ArtifactTreeAllocatedReadFailure, ArtifactTreeDirectory, ArtifactTreeFailureKind,
    ArtifactTreeFile, ArtifactTreeMedia,
};

use super::super::ceiling::{ArtifactCeiling, CeilingExtent};
use super::super::grant::ReadGrant;
use super::super::refusal::{AllocatedReadFailure, AllocatedReadOutcome, ArtifactDamage};
use super::addressed_payload::extent_offset;
use super::charged_read::{outcome, ReadStop, TreeReadFailure};
use super::{
    record_artifact, FilesystemObservation, ObservedRecoveryArtifact, RecoveryDiscoveryArtifact,
    RecoveryDiscoveryFailure,
};

/// What a read into a caller's buffer met, stated as the observation's result.
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
    /// Reads the artifact `ceiling` names into the buffer `allocate` returns
    /// for its real length.
    pub fn read_with_allocator<D: LimitDimension, E>(
        &mut self,
        ceiling: ArtifactCeiling,
        grant: ReadGrant<D>,
        allocate: impl FnOnce(usize) -> Result<Vec<u8>, E>,
    ) -> AllocatedReadOutcome<D, E> {
        let file = ceiling.file();
        let context = RecoveryDiscoveryArtifact::Record(file);
        let artifact = match record_artifact(file) {
            Ok(artifact) => artifact,
            Err(failure) => return outcome(Err(ReadStop::stop(failure))),
        };
        let caller = context.clone();
        outcome(match ceiling.extent() {
            CeilingExtent::Whole { bytes, fixed } => {
                self.read_whole_charged(context, bytes, fixed, &grant, |attempt, limit| {
                    attempt
                        .open()
                        .read_bounded_with_allocator(&artifact, limit, allocate)
                        .map_err(|failure| allocated(failure, &caller, 0))
                })
            }
            CeilingExtent::Frame { offset, length } => {
                self.read_range_charged(context, offset, length, &grant, |attempt, capacity| {
                    attempt
                        .open()
                        .read_exact_at_with_allocator(&artifact, offset, capacity, allocate)
                        .map_err(|failure| allocated(failure, &caller, offset))
                })
            }
        })
    }

    pub fn read_extent_manifest_with_allocator<D: LimitDimension, E>(
        &mut self,
        range: ExtentArenaRange,
        grant: ReadGrant<D>,
        allocate: impl FnOnce(usize) -> Result<Vec<u8>, E>,
    ) -> AllocatedReadOutcome<D, E> {
        let length = EXTENT_ARENA_MANIFEST_FRAME_BYTES as u32;
        self.read_extent_range_with_allocator(range, 0, length, grant, allocate)
    }

    /// Reads `length` bytes at `offset` within the arena `range`.
    pub fn read_extent_range_with_allocator<D: LimitDimension, E>(
        &mut self,
        range: ExtentArenaRange,
        offset: u64,
        length: u32,
        grant: ReadGrant<D>,
        allocate: impl FnOnce(usize) -> Result<Vec<u8>, E>,
    ) -> AllocatedReadOutcome<D, E> {
        let address = RecordArtifactFile::ExtentArena {
            arena: range.arena().get(),
        };
        match extent_offset(range, offset, length) {
            Some(absolute) => {
                self.read_record_range_with_allocator(address, absolute, length, grant, allocate)
            }
            None => outcome(Err(ReadStop::damage(ArtifactDamage::InvalidAddress {
                artifact: RecoveryDiscoveryArtifact::Record(address),
            }))),
        }
    }

    /// Reads exactly `length` bytes at `offset` of `address`, a range a
    /// verified parent declared.
    pub fn read_record_range_with_allocator<D: LimitDimension, E>(
        &mut self,
        address: RecordArtifactFile,
        offset: u64,
        length: u32,
        grant: ReadGrant<D>,
        allocate: impl FnOnce(usize) -> Result<Vec<u8>, E>,
    ) -> AllocatedReadOutcome<D, E> {
        let context = RecoveryDiscoveryArtifact::Record(address);
        let artifact = match record_artifact(address) {
            Ok(artifact) => artifact,
            Err(failure) => return outcome(Err(ReadStop::stop(failure))),
        };
        let caller = context.clone();
        outcome(
            self.read_range_charged(context, offset, length, &grant, |attempt, capacity| {
                attempt
                    .open()
                    .read_exact_at_with_allocator(&artifact, offset, capacity, allocate)
                    .map_err(|failure| allocated(failure, &caller, offset))
            }),
        )
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

    /// The stream readers' engine (checkpoint and WAL), until they take a
    /// ceiling.
    pub(super) fn read_whole_with<E>(
        &mut self,
        context: RecoveryDiscoveryArtifact,
        byte_limit: u64,
        fixed: bool,
        read: impl FnOnce(
            ArtifactTreeMedia<'_>,
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

/// The tree's failure of a read into a caller's buffer, at `offset` of
/// `artifact`, before any bound classifies it.
pub(super) fn allocated<E>(
    failure: ArtifactTreeAllocatedReadFailure<E>,
    artifact: &RecoveryDiscoveryArtifact,
    offset: u64,
) -> TreeReadFailure<AllocatedReadFailure<E>> {
    match failure {
        ArtifactTreeAllocatedReadFailure::Media(failure) => TreeReadFailure::Media(failure),
        ArtifactTreeAllocatedReadFailure::Allocation { requested, cause } => {
            TreeReadFailure::Caller(AllocatedReadFailure::Allocation {
                artifact: artifact.clone(),
                offset,
                requested,
                cause,
            })
        }
        ArtifactTreeAllocatedReadFailure::BufferLengthMismatch {
            requested,
            observed,
        } => TreeReadFailure::Caller(AllocatedReadFailure::BufferLengthMismatch {
            artifact: artifact.clone(),
            offset,
            requested,
            observed,
        }),
    }
}

pub(super) fn map_allocated_failure<E>(
    failure: ArtifactTreeAllocatedReadFailure<E>,
    artifact: RecoveryDiscoveryArtifact,
    offset: u64,
) -> RecoveryDiscoveryAllocationFailure<E> {
    match failure {
        ArtifactTreeAllocatedReadFailure::Media(failure) => {
            RecoveryDiscoveryAllocationFailure::Discovery(RecoveryDiscoveryFailure::media(
                artifact, failure,
            ))
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
