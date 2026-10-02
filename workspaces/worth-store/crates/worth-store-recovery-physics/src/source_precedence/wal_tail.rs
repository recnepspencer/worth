use worth_store_wal::WalLsnRange;

mod admission;
mod candidate;
mod continuation;

pub use admission::admit_physical_wal_tail;
pub use candidate::PhysicalWalSegmentCandidate;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalWalFrameFacts {
    lsn_range: WalLsnRange,
    encoded_bytes: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalWalInterruptionFacts {
    valid_prefix_bytes: u64,
    observed_bytes: u64,
}
#[derive(Debug, PartialEq, Eq)]
pub struct SelectedPhysicalWalTail {
    segments: Vec<PhysicalWalSegmentCandidate>,
    checkpoint_covered: Vec<super::CheckpointCoveredWalArtifact>,
    protected_covered_start: usize,
    admitted_frontier: u64,
    admitted_cutoff: Option<u64>,
    frame_count: u64,
    byte_count: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedPhysicalWalTailDenial {
    PreparedStorage {
        required: usize,
        covered_capacity: usize,
        retained_capacity: usize,
        covered_length: usize,
        retained_length: usize,
    },
    DuplicateArtifact,
    GenerationMismatch,
    SegmentGap,
    LsnGap,
    CheckpointFrontierMismatch,
    CheckpointContinuationMissing,
    CheckpointContinuationAmbiguous,
    CheckpointContinuationInterrupted,
    CheckpointContinuationDiscontinuous,
    InterruptedMiddleSegment,
    CounterOverflow,
}
impl SelectedPhysicalWalTail {
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        let segments = u64::try_from(self.segments.capacity())
            .ok()?
            .checked_mul(u64::try_from(std::mem::size_of::<PhysicalWalSegmentCandidate>()).ok()?)?;
        let covered = u64::try_from(self.checkpoint_covered.capacity())
            .ok()?
            .checked_mul(
                u64::try_from(std::mem::size_of::<super::CheckpointCoveredWalArtifact>()).ok()?,
            )?;
        self.segments
            .iter()
            .try_fold(segments.checked_add(covered)?, |sum, segment| {
                sum.checked_add(segment.owned_heap_bytes()?)
            })
    }

    pub(super) const fn admitted_checkpoint_basis(&self) -> (u64, Option<u64>) {
        (self.admitted_frontier, self.admitted_cutoff)
    }

    pub fn segments(&self) -> &[PhysicalWalSegmentCandidate] {
        &self.segments
    }

    pub fn segment_capacity(&self) -> usize {
        self.segments.capacity()
    }

    pub fn checkpoint_covered(&self) -> &[super::CheckpointCoveredWalArtifact] {
        &self.checkpoint_covered
    }

    pub fn checkpoint_covered_capacity(&self) -> usize {
        self.checkpoint_covered.capacity()
    }

    /// Authenticated physical WAL suffix that cleanup must preserve so an
    /// ordinary reopen can continue from the selected checkpoint cutoff.
    pub fn protected_checkpoint_covered(&self) -> &[super::CheckpointCoveredWalArtifact] {
        &self.checkpoint_covered[self.protected_covered_start..]
    }

    pub const fn frame_count(&self) -> u64 {
        self.frame_count
    }

    pub const fn byte_count(&self) -> u64 {
        self.byte_count
    }

    pub fn frame_facts(&self) -> impl Iterator<Item = &PhysicalWalFrameFacts> {
        self.segments
            .iter()
            .flat_map(|segment| segment.frame_facts())
    }
}

impl PhysicalWalFrameFacts {
    pub fn new(lsn_range: WalLsnRange, encoded_bytes: u64) -> Option<Self> {
        (encoded_bytes != 0).then_some(Self {
            lsn_range,
            encoded_bytes,
        })
    }

    pub const fn lsn_range(self) -> WalLsnRange {
        self.lsn_range
    }

    pub const fn encoded_bytes(self) -> u64 {
        self.encoded_bytes
    }
}

impl PhysicalWalInterruptionFacts {
    pub fn new(valid_prefix_bytes: u64, observed_bytes: u64) -> Option<Self> {
        (valid_prefix_bytes != 0 && observed_bytes > valid_prefix_bytes).then_some(Self {
            valid_prefix_bytes,
            observed_bytes,
        })
    }

    pub const fn valid_prefix_bytes(self) -> u64 {
        self.valid_prefix_bytes
    }

    pub const fn observed_bytes(self) -> u64 {
        self.observed_bytes
    }
}

#[cfg(test)]
mod tests;
