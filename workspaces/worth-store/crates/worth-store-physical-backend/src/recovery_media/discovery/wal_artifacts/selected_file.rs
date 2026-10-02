//! Exact selected lengths gate payload allocation at the ordinary metadata boundary.

use super::super::selected_wal::{SelectedReadDenial, SelectedReadStorage};
use super::*;
use crate::filesystem_media::{
    ArtifactTreeAllocatedReadFailure, ArtifactTreeFailureKind, ArtifactTreeFile,
};

impl<M: super::super::DiscoveryMediaBacking> FilesystemObservation<M> {
    pub(super) fn read_wal_artifact_with_storage<S: RecoveryWalReadStorage>(
        &mut self,
        file: ArtifactTreeBackedPath<ArtifactTreeFile, S::PathBacking>,
        context: RecoveryDiscoveryArtifact,
        remaining_wal_bytes: u64,
        byte_limit: u64,
        native: bool,
        expected: Option<u64>,
        storage: &mut S,
    ) -> Result<Outcome<Option<Vec<u8>>>, RecoveryDiscoveryAllocationFailure<S::Denial>> {
        let mut allocator = SelectedReadStorage { storage, expected };
        let read_limit = expected.map_or(remaining_wal_bytes, |length| {
            length.min(remaining_wal_bytes)
        });
        let result = self.read_whole_with(context, read_limit, false, |tree, limit| {
            let result = if native {
                tree.read_backed_bounded_with_allocator(file, limit, &mut allocator)
            } else {
                let result = tree.read_bounded_with_allocator(file.get(), limit, |length| {
                    crate::filesystem_media::ArtifactTreeReadAllocator::allocate_read_buffer(
                        &mut allocator,
                        length,
                    )
                });
                drop(file);
                result
            };
            // A larger selected file fails before the allocator is invoked. Do
            // not reinterpret a tighter independent observation budget as drift.
            match result {
                Err(ArtifactTreeAllocatedReadFailure::Media(failure))
                    if expected == Some(limit)
                        && failure.kind() == ArtifactTreeFailureKind::AccessLimitExceeded =>
                {
                    let observed = failure
                        .access_limit()
                        .map_or(limit.saturating_add(1), |bound| bound.observed);
                    Err(ArtifactTreeAllocatedReadFailure::Allocation {
                        requested: 0,
                        cause: SelectedReadDenial::Length(
                            RecoveryWalSelectionMismatch::FileLength {
                                expected: limit,
                                observed,
                            },
                        ),
                    })
                }
                result => result,
            }
        });
        match result {
            Ok(artifact) => Ok(Outcome::Observed(artifact.into_bytes())),
            Err(RecoveryDiscoveryAllocationFailure::Allocation {
                cause: SelectedReadDenial::Length(mismatch),
                ..
            }) => Ok(Outcome::Mismatch(mismatch)),
            Err(RecoveryDiscoveryAllocationFailure::Allocation {
                artifact,
                offset,
                requested,
                cause: SelectedReadDenial::Storage(cause),
            }) => Err(RecoveryDiscoveryAllocationFailure::Allocation {
                artifact,
                offset,
                requested,
                cause,
            }),
            Err(RecoveryDiscoveryAllocationFailure::BufferLengthMismatch {
                artifact,
                offset,
                requested,
                observed,
            }) => Err(RecoveryDiscoveryAllocationFailure::BufferLengthMismatch {
                artifact,
                offset,
                requested,
                observed,
            }),
            Err(RecoveryDiscoveryAllocationFailure::Discovery(
                RecoveryDiscoveryFailure::ByteLimitExceeded {
                    observed,
                    scope: RecoveryDiscoveryByteLimitScope::Requested,
                    ..
                },
            )) => Err(RecoveryDiscoveryFailure::ByteLimitExceeded {
                observed: byte_limit
                    .saturating_sub(remaining_wal_bytes)
                    .saturating_add(observed),
                admitted: byte_limit,
                scope: RecoveryDiscoveryByteLimitScope::Requested,
            }
            .into()),
            Err(RecoveryDiscoveryAllocationFailure::Discovery(failure)) => Err(failure.into()),
        }
    }
}
