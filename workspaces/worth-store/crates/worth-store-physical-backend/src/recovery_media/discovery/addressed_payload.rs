use worth_foundational::LimitDimension;
use worth_store_physical_format::{
    ExtentArenaRange, RecordArtifactFile, EXTENT_ARENA_MANIFEST_FRAME_BYTES,
};

use super::super::ceiling::{ArtifactCeiling, CeilingExtent};
use super::super::grant::ReadGrant;
use super::super::refusal::{ArtifactDamage, ArtifactReadOutcome};
use super::charged_read::{outcome, ReadStop, TreeReadFailure};
use super::{record_artifact, BoundedRecoveryFilesystemDiscovery, RecoveryDiscoveryArtifact};

impl BoundedRecoveryFilesystemDiscovery {
    /// Reads the artifact `ceiling` names, within its ceiling and `grant`.
    pub fn read<D: LimitDimension>(
        &mut self,
        ceiling: ArtifactCeiling,
        grant: ReadGrant<D>,
    ) -> ArtifactReadOutcome<D> {
        let file = ceiling.file();
        let context = RecoveryDiscoveryArtifact::Record(file);
        let artifact = match record_artifact(file) {
            Ok(artifact) => artifact,
            Err(failure) => return outcome(Err(ReadStop::stop(failure))),
        };
        outcome(match ceiling.extent() {
            CeilingExtent::Whole { bytes, fixed } => {
                self.read_whole_charged(context, bytes, fixed, &grant, |attempt, limit| {
                    attempt
                        .open()
                        .read_bounded(&artifact, limit)
                        .map_err(TreeReadFailure::Media)
                })
            }
            CeilingExtent::Frame { offset, length } => {
                self.read_range_charged(context, offset, length, &grant, |attempt, capacity| {
                    read_exact(attempt.open(), &artifact, offset, capacity)
                })
            }
        })
    }

    /// Reads exactly `length` bytes at `offset` of a segment, a range a
    /// verified parent declared.
    pub fn read_segment_range<D: LimitDimension>(
        &mut self,
        segment: u64,
        generation: u64,
        offset: u64,
        length: u32,
        grant: ReadGrant<D>,
    ) -> ArtifactReadOutcome<D> {
        self.read_record_range(
            RecordArtifactFile::Segment {
                segment,
                generation,
            },
            offset,
            length,
            grant,
        )
    }

    pub fn read_extent_manifest<D: LimitDimension>(
        &mut self,
        range: ExtentArenaRange,
        grant: ReadGrant<D>,
    ) -> ArtifactReadOutcome<D> {
        self.read_extent_range(range, 0, EXTENT_ARENA_MANIFEST_FRAME_BYTES as u32, grant)
    }

    /// Reads `length` bytes at `offset` within the arena `range`.
    pub fn read_extent_range<D: LimitDimension>(
        &mut self,
        range: ExtentArenaRange,
        offset: u64,
        length: u32,
        grant: ReadGrant<D>,
    ) -> ArtifactReadOutcome<D> {
        let artifact = RecordArtifactFile::ExtentArena {
            arena: range.arena().get(),
        };
        match extent_offset(range, offset, length) {
            Some(absolute) => self.read_record_range(artifact, absolute, length, grant),
            None => outcome(Err(ReadStop::damage(ArtifactDamage::InvalidAddress {
                artifact: RecoveryDiscoveryArtifact::Record(artifact),
            }))),
        }
    }

    fn read_record_range<D: LimitDimension>(
        &mut self,
        file: RecordArtifactFile,
        offset: u64,
        length: u32,
        grant: ReadGrant<D>,
    ) -> ArtifactReadOutcome<D> {
        let context = RecoveryDiscoveryArtifact::Record(file);
        let artifact = match record_artifact(file) {
            Ok(artifact) => artifact,
            Err(failure) => return outcome(Err(ReadStop::stop(failure))),
        };
        outcome(
            self.read_range_charged(context, offset, length, &grant, |attempt, capacity| {
                read_exact(attempt.open(), &artifact, offset, capacity)
            }),
        )
    }
}

/// The absolute offset of `length` bytes at `offset` within `range`; `None`
/// where they do not lie within it.
pub(super) fn extent_offset(range: ExtentArenaRange, offset: u64, length: u32) -> Option<u64> {
    let end = offset
        .checked_add(u64::from(length))
        .filter(|end| *end <= range.length())?;
    range.offset().checked_add(end - u64::from(length))
}

fn read_exact(
    tree: crate::filesystem_media::ArtifactTreeMedia<'_>,
    artifact: &crate::filesystem_media::ArtifactTreeFile,
    offset: u64,
    capacity: usize,
) -> Result<Vec<u8>, TreeReadFailure<ArtifactDamage>> {
    let mut bytes = vec![0; capacity];
    tree.read_exact_at(artifact, offset, &mut bytes)
        .map(|()| bytes)
        .map_err(TreeReadFailure::Media)
}
