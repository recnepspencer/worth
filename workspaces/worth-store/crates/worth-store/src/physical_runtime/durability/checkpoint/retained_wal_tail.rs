use std::cmp::Ordering;
use std::num::NonZeroUsize;

use worth_proof::{CanonicalVec, NonEmpty};
use worth_store_physical_format::{PhysicalCheckpointIdentity, PhysicalCheckpointSource};
use worth_store_wal::{
    LogSequenceNumber, WalLsnRange, WalSegmentArtifactIdentity, WalSegmentGeneration,
};

use crate::physical_runtime::durability::wal::inventory::PhysicalWalInventorySnapshot;
use crate::physical_runtime::durability::RetainedWalTailLimit;

/// One original WAL artifact retained as part of a checkpoint's physical tail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetainedWalSegment {
    artifact: WalSegmentArtifactIdentity,
    observed_lsn_range: WalLsnRange,
    physical_bytes: u64,
}

/// Store authority that exact original WAL artifacts cover a checkpoint tail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContiguousRetainedWalTail {
    checkpoint_source: PhysicalCheckpointSource,
    durable_tail_end_lsn_exclusive: LogSequenceNumber,
    retained_physical_bytes: u64,
    segment_count: NonZeroUsize,
    segments: CanonicalVec<RetainedWalSegment>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime::durability) enum RetainedWalTailAdmissionDenial {
    Empty,
    NonCanonicalOrder,
    EmptyArtifact,
    CheckpointBoundaryNotCovered,
    DurableFrontierBeforeCheckpoint,
    DurableFrontierNotCovered,
    ArtifactGenerationMismatch,
    ArtifactIdentityDiscontinuity,
    LsnGap,
    LsnOverlap,
    RetainedByteCountOverflow,
    RetainedByteLimitExceeded,
}

impl RetainedWalSegment {
    const fn from_inventory(
        segment: crate::physical_runtime::durability::wal::inventory::PhysicalWalSegmentInventoryEntry,
    ) -> Self {
        Self {
            artifact: segment.identity(),
            observed_lsn_range: segment.lsn_range(),
            physical_bytes: segment.byte_count(),
        }
    }

    pub const fn artifact(self) -> WalSegmentArtifactIdentity {
        self.artifact
    }

    pub const fn observed_lsn_range(self) -> WalLsnRange {
        self.observed_lsn_range
    }

    pub const fn physical_bytes(self) -> u64 {
        self.physical_bytes
    }
}

impl Ord for RetainedWalSegment {
    fn cmp(&self, other: &Self) -> Ordering {
        self.observed_lsn_range
            .cmp(&other.observed_lsn_range)
            .then_with(|| self.artifact.cmp(&other.artifact))
    }
}

impl PartialOrd for RetainedWalSegment {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl ContiguousRetainedWalTail {
    pub(super) fn from_inventory(
        checkpoint_source: PhysicalCheckpointSource,
        inventory: &PhysicalWalInventorySnapshot,
        limit: RetainedWalTailLimit,
    ) -> Result<Self, RetainedWalTailAdmissionDenial> {
        let checkpoint_boundary =
            LogSequenceNumber::new(checkpoint_source.wal().covered_end_lsn_exclusive());
        let durable_end = inventory.durable_lsn_end();
        if durable_end < checkpoint_boundary {
            return Err(RetainedWalTailAdmissionDenial::DurableFrontierBeforeCheckpoint);
        }
        let entries = inventory.segments().as_slice();
        let first = first_required_segment(entries, checkpoint_boundary, durable_end)
            .ok_or(RetainedWalTailAdmissionDenial::CheckpointBoundaryNotCovered)?;
        let last = final_required_segment(entries, first, durable_end)
            .ok_or(RetainedWalTailAdmissionDenial::DurableFrontierNotCovered)?;
        let segments = entries[first..=last]
            .iter()
            .copied()
            .map(RetainedWalSegment::from_inventory)
            .collect::<Vec<_>>();
        let nonempty =
            NonEmpty::try_from_vec(segments).map_err(|_| RetainedWalTailAdmissionDenial::Empty)?;
        Self::admit(checkpoint_source, durable_end, nonempty, limit)
    }

    fn admit(
        checkpoint_source: PhysicalCheckpointSource,
        durable_end: LogSequenceNumber,
        segments: NonEmpty<RetainedWalSegment>,
        limit: RetainedWalTailLimit,
    ) -> Result<Self, RetainedWalTailAdmissionDenial> {
        let checkpoint_boundary =
            LogSequenceNumber::new(checkpoint_source.wal().covered_end_lsn_exclusive());
        if durable_end < checkpoint_boundary {
            return Err(RetainedWalTailAdmissionDenial::DurableFrontierBeforeCheckpoint);
        }
        let segment_count =
            NonZeroUsize::new(segments.len()).ok_or(RetainedWalTailAdmissionDenial::Empty)?;
        let canonical = CanonicalVec::try_from_sorted(segments.into_vec())
            .map_err(|_| RetainedWalTailAdmissionDenial::NonCanonicalOrder)?;
        let ordered = canonical.as_slice();
        require_boundary(ordered[0], checkpoint_boundary, durable_end)?;
        require_final_coverage(ordered[ordered.len() - 1], checkpoint_boundary, durable_end)?;
        for segment in ordered {
            if segment.physical_bytes == 0 {
                return Err(RetainedWalTailAdmissionDenial::EmptyArtifact);
            }
        }
        for pair in ordered.windows(2) {
            require_successor(pair[0], pair[1])?;
        }
        let retained_physical_bytes = ordered
            .iter()
            .try_fold(0_u64, |total, segment| {
                total.checked_add(segment.physical_bytes)
            })
            .ok_or(RetainedWalTailAdmissionDenial::RetainedByteCountOverflow)?;
        if retained_physical_bytes > limit.get().get() {
            return Err(RetainedWalTailAdmissionDenial::RetainedByteLimitExceeded);
        }
        Ok(Self {
            checkpoint_source,
            durable_tail_end_lsn_exclusive: durable_end,
            retained_physical_bytes,
            segment_count,
            segments: canonical,
        })
    }

    pub const fn checkpoint_identity(&self) -> PhysicalCheckpointIdentity {
        self.checkpoint_source.identity()
    }

    pub const fn checkpoint_source(&self) -> PhysicalCheckpointSource {
        self.checkpoint_source
    }

    pub const fn checkpoint_boundary_lsn(&self) -> LogSequenceNumber {
        LogSequenceNumber::new(self.checkpoint_source.wal().covered_end_lsn_exclusive())
    }

    pub const fn durable_tail_end_lsn_exclusive(&self) -> LogSequenceNumber {
        self.durable_tail_end_lsn_exclusive
    }

    pub const fn retained_physical_bytes(&self) -> u64 {
        self.retained_physical_bytes
    }

    pub const fn segment_count(&self) -> NonZeroUsize {
        self.segment_count
    }

    pub fn segments(&self) -> &[RetainedWalSegment] {
        self.segments.as_slice()
    }
}

fn first_required_segment(
    entries: &[crate::physical_runtime::durability::wal::inventory::PhysicalWalSegmentInventoryEntry],
    checkpoint_boundary: LogSequenceNumber,
    durable_end: LogSequenceNumber,
) -> Option<usize> {
    if checkpoint_boundary == durable_end {
        return entries.iter().rposition(|entry| {
            let range = entry.lsn_range();
            range.start() < checkpoint_boundary && range.end_exclusive() == checkpoint_boundary
        });
    }
    entries.iter().position(|entry| {
        let range = entry.lsn_range();
        range.start() <= checkpoint_boundary && checkpoint_boundary < range.end_exclusive()
    })
}

fn final_required_segment(
    entries: &[crate::physical_runtime::durability::wal::inventory::PhysicalWalSegmentInventoryEntry],
    first: usize,
    durable_end: LogSequenceNumber,
) -> Option<usize> {
    entries[first..]
        .iter()
        .position(|entry| {
            let range = entry.lsn_range();
            range.start() < durable_end && durable_end <= range.end_exclusive()
        })
        .map(|offset| first + offset)
        .or((entries[first].lsn_range().end_exclusive() == durable_end).then_some(first))
}

fn require_boundary(
    first: RetainedWalSegment,
    checkpoint_boundary: LogSequenceNumber,
    durable_end: LogSequenceNumber,
) -> Result<(), RetainedWalTailAdmissionDenial> {
    let range = first.observed_lsn_range;
    let covered = if checkpoint_boundary == durable_end {
        range.start() < checkpoint_boundary && range.end_exclusive() == checkpoint_boundary
    } else {
        range.start() <= checkpoint_boundary && checkpoint_boundary < range.end_exclusive()
    };
    covered
        .then_some(())
        .ok_or(RetainedWalTailAdmissionDenial::CheckpointBoundaryNotCovered)
}

fn require_final_coverage(
    final_segment: RetainedWalSegment,
    checkpoint_boundary: LogSequenceNumber,
    durable_end: LogSequenceNumber,
) -> Result<(), RetainedWalTailAdmissionDenial> {
    let range = final_segment.observed_lsn_range;
    let covered = if checkpoint_boundary == durable_end {
        range.end_exclusive() == durable_end
    } else {
        range.start() < durable_end && durable_end <= range.end_exclusive()
    };
    covered
        .then_some(())
        .ok_or(RetainedWalTailAdmissionDenial::DurableFrontierNotCovered)
}

fn require_successor(
    current: RetainedWalSegment,
    successor: RetainedWalSegment,
) -> Result<(), RetainedWalTailAdmissionDenial> {
    let generation: WalSegmentGeneration = current.artifact.generation();
    if successor.artifact.generation() != generation {
        return Err(RetainedWalTailAdmissionDenial::ArtifactGenerationMismatch);
    }
    if current.artifact.segment().get().checked_add(1) != Some(successor.artifact.segment().get()) {
        return Err(RetainedWalTailAdmissionDenial::ArtifactIdentityDiscontinuity);
    }
    if current
        .observed_lsn_range
        .overlaps(successor.observed_lsn_range)
    {
        return Err(RetainedWalTailAdmissionDenial::LsnOverlap);
    }
    if !current
        .observed_lsn_range
        .is_contiguous_with(successor.observed_lsn_range)
    {
        return Err(RetainedWalTailAdmissionDenial::LsnGap);
    }
    Ok(())
}

#[cfg(test)]
mod tests;
