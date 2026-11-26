use super::ExtentArenaRange;

/// Crash- and cancel-monotone allocation epoch of one extent-arena byte range.
///
/// Ordering is lexicographic: the durable root publication generation first,
/// then the WAL sequence of the barrier that returned the range to the
/// allocator. Both components are read from state the barrier itself made
/// durable, so no extra persisted field exists and reopen cannot rewind them:
/// a candidate cancellation that publishes no root still advances the fence
/// sequence through its own durable WAL record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtentAllocationGeneration {
    root_publication_generation: u64,
    extent_fence_seq: u64,
}

impl ExtentAllocationGeneration {
    /// The epoch every range holds when a residency incarnation opens. No
    /// writeback or resident frame survives into a new incarnation, and every
    /// durable barrier generation orders strictly after it.
    pub const OPENED: Self = Self {
        root_publication_generation: 0,
        extent_fence_seq: 0,
    };

    /// The epoch established by a durable reuse barrier. `barrier_wal_seq` is
    /// the WAL sequence of the barrier's own durable record; zero is refused
    /// because it cannot name a durable record.
    pub const fn at_durable_barrier(
        root_publication_generation: u64,
        barrier_wal_seq: u64,
    ) -> Option<Self> {
        if barrier_wal_seq == 0 {
            return None;
        }
        Some(Self {
            root_publication_generation,
            extent_fence_seq: barrier_wal_seq,
        })
    }

    pub const fn root_publication_generation(self) -> u64 {
        self.root_publication_generation
    }

    pub const fn extent_fence_seq(self) -> u64 {
        self.extent_fence_seq
    }
}

/// Allocation identity carried by resident arena frames and writeback claims.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArenaExtentFrameIdentity {
    range: ExtentArenaRange,
    generation: ExtentAllocationGeneration,
}

impl ArenaExtentFrameIdentity {
    pub const fn new(range: ExtentArenaRange, generation: ExtentAllocationGeneration) -> Self {
        Self { range, generation }
    }

    pub const fn range(self) -> ExtentArenaRange {
        self.range
    }

    pub const fn generation(self) -> ExtentAllocationGeneration {
        self.generation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ExtentArenaId;

    fn range(arena: u64, offset: u64, length: u64) -> ExtentArenaRange {
        ExtentArenaRange::new(ExtentArenaId::new(arena).unwrap(), offset, length).unwrap()
    }

    #[test]
    fn cancel_barrier_without_root_publication_orders_after_the_cancelled_epoch() {
        let cancelled = ExtentAllocationGeneration::at_durable_barrier(7, 40).unwrap();
        let reissued = ExtentAllocationGeneration::at_durable_barrier(7, 41).unwrap();
        let published = ExtentAllocationGeneration::at_durable_barrier(8, 1).unwrap();
        assert!(ExtentAllocationGeneration::OPENED < cancelled);
        assert!(cancelled < reissued);
        assert!(reissued < published);
        assert!(ExtentAllocationGeneration::at_durable_barrier(7, 0).is_none());
    }

    #[test]
    fn overlap_covers_exact_split_and_coalesced_ranges_without_overflow() {
        let base = range(3, 4096, 8192);
        assert!(base.overlaps(base));
        assert!(base.overlaps(range(3, 4096, 4096)));
        assert!(base.overlaps(range(3, 8192, 4096)));
        assert!(base.overlaps(range(3, 0, 16384)));
        assert!(!base.overlaps(range(3, 0, 4096)));
        assert!(!base.overlaps(range(3, 12288, 4096)));
        assert!(!base.overlaps(range(4, 4096, 8192)));
        let tail = range(3, u64::MAX - 4096, 4096);
        assert!(tail.overlaps(range(3, u64::MAX - 1, 1)));
        assert!(!tail.overlaps(base));
    }
}
