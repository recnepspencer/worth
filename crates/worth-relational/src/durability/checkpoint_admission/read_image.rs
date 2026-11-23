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
pub(crate) struct AdmissionRowKey {
    pub(crate) position: u64,
    pub(crate) commit: u64,
}
#[derive(Debug, Clone)]
pub(crate) struct AdmissionRowIndex {
    pub(crate) key: AdmissionRowKey,
    pub(crate) groups: [Range<usize>; 8],
}
pub(crate) struct DecodedDirectory {
    pub(crate) cut: CheckpointAdmissionCut,
    pub(crate) roots: Range<usize>,
    pub(crate) rows: ExecutionArray<AdmissionRowIndex>,
    pub(crate) commits: ExecutionOrderedMap<u64, usize>,
    pub(crate) positions: ExecutionOrderedMap<
        crate::history::data::CommitId,
        crate::history::data::RequiredOutputPublicationPositions,
    >,
}

#[derive(Debug, Clone)]
pub(crate) struct CheckpointAdmissionReadImage(Arc<ArtifactBacking>);

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
    pub(super) fn same_owner(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    pub(super) fn rows(&self) -> &[AdmissionRowIndex] {
        self.0.rows.as_ref()
    }

    pub(super) fn roots(&self) -> &[u8] {
        self.group(&self.0.roots)
    }

    pub(super) fn from_directory(
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

    pub(super) fn as_bytes(&self) -> &[u8] {
        &self.0.bytes[self.0.region.clone()]
    }
    pub(super) fn cut(&self) -> &CheckpointAdmissionCut {
        &self.0.cut
    }

    pub(super) fn group(&self, range: &Range<usize>) -> &[u8] {
        &self.0.bytes[range.clone()]
    }

    pub(super) fn row(&self, commit: CommitId) -> Option<&AdmissionRowIndex> {
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
pub(crate) struct CheckpointOriginalRow<'a> {
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

    pub(crate) fn index(&self) -> &'a crate::history::commit::RelationalCommitIndexEntry {
        self.index
    }

    pub(crate) fn scope(&self) -> &'a worth_foundational::facade::CheckpointAdmissionScope {
        self.image.cut().scope()
    }
}
