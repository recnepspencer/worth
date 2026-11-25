//! Immutable ranges and indices of one admitted checkpoint projection.
use super::refusal::RelationalCheckpointAdmissionRefusal as Refusal;
use crate::history::data::{CommitId, RequiredOutputPublicationPositions};
use std::{ops::Range, sync::Arc};
use worth_execution::{
    ExecutionArray, ExecutionImmutableBytes, ExecutionMemoryReservation, ExecutionOrderedMap,
    ExecutionRequest,
};
use worth_foundational::facade::CheckpointAdmissionCut;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdmissionRowKey {
    pub position: u64,
    pub commit: u64,
}
#[derive(Debug, Clone)]
pub struct AdmissionRowIndex {
    pub key: AdmissionRowKey,
    pub groups: [Range<usize>; 8],
}
pub struct DecodedDirectory {
    pub cut: CheckpointAdmissionCut,
    pub roots: Range<usize>,
    pub rows: ExecutionArray<AdmissionRowIndex>,
    pub commits: ExecutionOrderedMap<u64, usize>,
    pub positions: ExecutionOrderedMap<
        crate::history::data::CommitId,
        crate::history::data::RequiredOutputPublicationPositions,
    >,
}

#[derive(Debug, Clone)]
pub struct CheckpointAdmissionReadImage(Arc<ArtifactBacking>);

#[derive(Debug)]
struct ArtifactBacking {
    bytes: ExecutionImmutableBytes,
    region: Range<usize>,
    cut: CheckpointAdmissionCut,
    roots: Range<usize>,
    rows: ExecutionArray<AdmissionRowIndex>,
    commits: ExecutionOrderedMap<u64, usize>,
    positions: ExecutionOrderedMap<CommitId, RequiredOutputPublicationPositions>,
    _memory: Option<ExecutionMemoryReservation>,
}

impl CheckpointAdmissionReadImage {
    pub fn same_owner(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    pub fn rows(&self) -> &[AdmissionRowIndex] {
        self.0.rows.as_ref()
    }

    pub fn roots(&self) -> &[u8] {
        self.group(&self.0.roots)
    }

    pub fn from_directory(
        bytes: ExecutionImmutableBytes,
        region: Range<usize>,
        decoded: DecodedDirectory,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Refusal> {
        let size = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<ArtifactBacking>())
            .map_err(|_| Refusal::CapacityOverflow)?
            .0
            .pad_to_align()
            .size();
        let memory = request
            .reserve_retained_memory(size as u64)
            .map_err(|cause| Refusal::Request(cause.into()))?;
        Ok(Self(Arc::new(ArtifactBacking {
            bytes,
            region,
            cut: decoded.cut,
            roots: decoded.roots,
            rows: decoded.rows,
            commits: decoded.commits,
            positions: decoded.positions,
            _memory: memory,
        })))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0.bytes[self.0.region.clone()]
    }
    pub fn cut(&self) -> &CheckpointAdmissionCut {
        &self.0.cut
    }

    pub fn group(&self, range: &Range<usize>) -> &[u8] {
        &self.0.bytes[range.clone()]
    }

    pub fn row(&self, commit: CommitId) -> Option<&AdmissionRowIndex> {
        self.0
            .commits
            .get(&commit.0)
            .and_then(|ordinal| self.0.rows.get(*ordinal))
    }

    pub(super) fn positions(
        &self,
        commit: CommitId,
    ) -> Option<&RequiredOutputPublicationPositions> {
        self.0.positions.get(&commit)
    }
}

/// Exact row and scope lent by the admitted checkpoint custody.
pub struct CheckpointOriginalRow<'a> {
    image: &'a CheckpointAdmissionReadImage,
    index: &'a crate::history::commit::RelationalCommitIndexEntry,
}

impl<'a> CheckpointOriginalRow<'a> {
    pub(super) fn new(
        image: &'a CheckpointAdmissionReadImage,
        index: &'a crate::history::commit::RelationalCommitIndexEntry,
    ) -> Self {
        Self { image, index }
    }

    pub fn index(&self) -> &'a crate::history::commit::RelationalCommitIndexEntry {
        self.index
    }

    pub fn effect_groups(&self) -> Option<(&'a [u8], &'a [u8])> {
        let row = self.image.row(self.index.commit.commit_id)?;
        Some((
            self.image.group(&row.groups[0]),
            self.image.group(&row.groups[6]),
        ))
    }

    pub fn scope(&self) -> &'a worth_foundational::facade::CheckpointAdmissionScope {
        self.image.cut().scope()
    }
}
