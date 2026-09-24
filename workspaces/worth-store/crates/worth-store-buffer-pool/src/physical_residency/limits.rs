use std::num::{NonZeroU32, NonZeroU64};

use super::{pressure::PhysicalResidencyLimitsBuilder, PhysicalSpeculativeWorkKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalOperationAllocationScope {
    ForegroundRead,
    ForegroundWrite,
    Recovery,
    Scrub,
    Maintenance,
    Verification,
    Blob,
}

impl PhysicalOperationAllocationScope {
    pub(crate) const fn index(self) -> usize {
        match self {
            Self::ForegroundRead => 0,
            Self::ForegroundWrite => 1,
            Self::Recovery => 2,
            Self::Scrub => 3,
            Self::Maintenance => 4,
            Self::Verification => 5,
            Self::Blob => 6,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalResidencyDimension {
    TotalBytes,
    ResidentBytes,
    MetadataBytes,
    FrameEntries,
    PinnedFrames,
    PinLeases,
    DirtyFrames,
    DirtyReplacementBytes,
    OperationBytes,
    OperationScope(PhysicalOperationAllocationScope),
    SpeculativeFrames(PhysicalSpeculativeWorkKind),
}

impl PhysicalResidencyDimension {
    pub(crate) const COUNT: usize = 19;

    pub(crate) const fn index(self) -> usize {
        match self {
            Self::TotalBytes => 0,
            Self::ResidentBytes => 1,
            Self::MetadataBytes => 2,
            Self::FrameEntries => 3,
            Self::PinnedFrames => 4,
            Self::PinLeases => 5,
            Self::DirtyFrames => 6,
            Self::DirtyReplacementBytes => 7,
            Self::OperationBytes => 8,
            Self::OperationScope(scope) => 9 + scope.index(),
            Self::SpeculativeFrames(kind) => 16 + kind.index(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalResidencyLimitsAdmissionDenial {
    Missing(PhysicalResidencyDimension),
    CategoryExceedsTotal {
        dimension: PhysicalResidencyDimension,
        declared: u64,
        total: u64,
    },
    ScopeExceedsOperation {
        scope: PhysicalOperationAllocationScope,
        declared: u64,
        operation: u64,
    },
    CountExceedsFrameEntries {
        dimension: PhysicalResidencyDimension,
        declared: u32,
        frame_entries: u32,
    },
    PageExceedsResidentBytes {
        page: u64,
        resident: u64,
    },
    PageExceedsOperationBytes {
        page: u64,
        operation: u64,
    },
    PageExceedsDirtyReplacementBytes {
        page: u64,
        dirty_replacement: u64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalResidencyLimits {
    pub(super) total_bytes: NonZeroU64,
    pub(super) resident_bytes: NonZeroU64,
    pub(super) metadata_bytes: NonZeroU64,
    pub(super) frame_entries: NonZeroU32,
    pub(super) pinned_frames: NonZeroU32,
    pub(super) pin_leases: NonZeroU32,
    pub(super) dirty_frames: NonZeroU32,
    pub(super) dirty_replacement_bytes: NonZeroU64,
    pub(super) operation_bytes: NonZeroU64,
    pub(super) scope_bytes: [NonZeroU64; 7],
    pub(super) speculative_frames: [NonZeroU32; 3],
    /// Bytes withheld from ordinary growth so checkpoint and cleanup can still run.
    pub(super) progress_headroom_bytes: u64,
}

impl PhysicalResidencyLimits {
    pub fn builder() -> PhysicalResidencyLimitsBuilder {
        PhysicalResidencyLimitsBuilder::default()
    }

    pub const fn total_bytes(self) -> u64 {
        self.total_bytes.get()
    }

    pub const fn resident_bytes(self) -> u64 {
        self.resident_bytes.get()
    }

    pub const fn metadata_bytes(self) -> u64 {
        self.metadata_bytes.get()
    }

    pub const fn frame_entries(self) -> u32 {
        self.frame_entries.get()
    }

    pub const fn pinned_frames(self) -> u32 {
        self.pinned_frames.get()
    }

    pub const fn pin_leases(self) -> u32 {
        self.pin_leases.get()
    }

    pub const fn dirty_frames(self) -> u32 {
        self.dirty_frames.get()
    }

    pub const fn dirty_replacement_bytes(self) -> u64 {
        self.dirty_replacement_bytes.get()
    }

    pub const fn operation_bytes(self) -> u64 {
        self.operation_bytes.get()
    }

    pub const fn scope_bytes(self, scope: PhysicalOperationAllocationScope) -> u64 {
        self.scope_bytes[scope.index()].get()
    }

    pub const fn speculative_frames(self, kind: PhysicalSpeculativeWorkKind) -> u32 {
        self.speculative_frames[kind.index()].get()
    }

    pub const fn progress_headroom_bytes(self) -> u64 {
        self.progress_headroom_bytes
    }

    /// Ordinary scopes cannot spend the progress headroom. Maintenance can.
    pub fn usable_bytes(self, scope: PhysicalOperationAllocationScope, limit: u64) -> u64 {
        if scope == PhysicalOperationAllocationScope::Maintenance {
            limit
        } else {
            limit.saturating_sub(self.progress_headroom_bytes)
        }
    }
}
