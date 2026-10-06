//! One WAL member read within its ceiling and what is left of the
//! inventory's grant. A selected member's declared length is its ceiling, and
//! gates payload allocation at the ordinary metadata boundary.

use super::super::super::ceiling::CeilingBytes;
use super::super::resident_read::allocated;
use super::super::selected_wal::{SelectedReadDenial, SelectedReadStorage};
use super::*;
use crate::filesystem_media::{ArtifactTreeFile, ArtifactTreeReadAllocator};

impl<M: super::super::DiscoveryMediaBacking> FilesystemObservation<M> {
    /// Reads one WAL file. A file longer than `expected` is damage with its
    /// real length; one shorter is a mismatch with the selection.
    pub(super) fn read_wal_member<D: LimitDimension, S: RecoveryWalReadStorage>(
        &mut self,
        file: ArtifactTreeBackedPath<ArtifactTreeFile, S::PathBacking>,
        context: RecoveryDiscoveryArtifact,
        share: &GrantShare<'_, D>,
        native: bool,
        expected: Option<u64>,
        storage: &mut S,
    ) -> Result<Outcome<Option<Vec<u8>>>, InventoryStop<D, S::Denial>> {
        let mut allocator = SelectedReadStorage { storage, expected };
        let ceiling = expected.map_or(CeilingBytes::Undeclared, CeilingBytes::Declared);
        let caller = context.clone();
        let read = self.read_whole_charged(context, ceiling, false, share, |attempt, limit| {
            let tree = attempt.open();
            let result = if native {
                tree.read_backed_bounded_with_allocator(file, limit, &mut allocator)
            } else {
                let result = tree.read_bounded_with_allocator(file.get(), limit, |length| {
                    allocator.allocate_read_buffer(length)
                });
                drop(file);
                result
            };
            result.map_err(|failure| allocated(failure, &caller, 0))
        });
        match read {
            Ok(artifact) => Ok(Outcome::Observed(artifact.into_bytes())),
            Err(ReadStop::Failed(AllocatedReadFailure::Allocation {
                cause: SelectedReadDenial::Length(mismatch),
                ..
            })) => Ok(Outcome::Mismatch(mismatch)),
            Err(ReadStop::Failed(AllocatedReadFailure::Allocation {
                artifact,
                offset,
                requested,
                cause: SelectedReadDenial::Storage(cause),
            })) => Err(ReadStop::Failed(AllocatedReadFailure::Allocation {
                artifact,
                offset,
                requested,
                cause,
            })),
            Err(ReadStop::Failed(AllocatedReadFailure::BufferLengthMismatch {
                artifact,
                offset,
                requested,
                observed,
            })) => Err(ReadStop::Failed(
                AllocatedReadFailure::BufferLengthMismatch {
                    artifact,
                    offset,
                    requested,
                    observed,
                },
            )),
            Err(ReadStop::Failed(AllocatedReadFailure::Damage(damage))) => {
                Err(ReadStop::damage(damage))
            }
            Err(ReadStop::Refused(refusal)) => Err(ReadStop::Refused(refusal)),
        }
    }
}
