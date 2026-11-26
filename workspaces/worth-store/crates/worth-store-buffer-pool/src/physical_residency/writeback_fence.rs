use super::PhysicalResidencyDenial;
use worth_store_physical_format::ExtentAllocationGeneration;

/// The durable barrier a caller has reached before asking to reuse a range.
/// Only a completed retirement or a checkpoint-covered candidate cancellation
/// may return a range to the allocator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArenaReuseBarrier {
    /// The retirement intent or free-space root exists, but its durable
    /// completion record does not.
    RetirementIncomplete,
    CompletedRetirement,
    /// The candidate cancellation record is durable but no checkpoint covers it.
    CancelUncovered,
    CheckpointCoveredCancel,
}

/// Why `WritebackFenceDrain` did not admit a range. Every variant leaves the
/// range unavailable to the allocator; the fence stays installed until a later
/// attempt succeeds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WritebackFenceDrainDenial {
    PoolClosed,
    BarrierIncomplete(ArenaReuseBarrier),
    /// Overlapping writeback claims remain in flight. Retry after they
    /// complete or release.
    WritebackInFlight {
        overlapping: u32,
    },
    /// An overlapping frame is pinned, loading, dirty or aliased.
    FrameBusy(PhysicalResidencyDenial),
    /// The proposed generation does not exceed one already seen on an
    /// overlapping range.
    GenerationNotAdvanced {
        proposed: ExtentAllocationGeneration,
        active: ExtentAllocationGeneration,
    },
}
