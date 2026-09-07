use crate::{
    PhysicalCheckpointIdentity, PhysicalWorkObligationIdentity, RecordArtifactFile,
    WalSegmentIdentity,
};

/// Canonical location of persisted physical bytes; no runtime or read authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalArtifactReadTarget {
    Record(RecordArtifactFile),
    Wal(WalSegmentIdentity),
    Checkpoint(PhysicalCheckpointIdentity),
    PhysicalWork(PhysicalWorkObligationIdentity),
}

impl PhysicalArtifactReadTarget {
    /// Checkpoint identity is expected content; published identities occupy the
    /// same mutable checkpoint.current location within a Store namespace.
    pub fn same_location(self, other: Self) -> bool {
        matches!((self, other), (Self::Checkpoint(_), Self::Checkpoint(_))) || self == other
    }
}

/// Exact bounded range in one canonical artifact. Unlike a record frame,
/// this range may describe a WAL or checkpoint record or a pending obligation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalArtifactReadRange {
    target: PhysicalArtifactReadTarget,
    offset: u64,
    length: u32,
}

impl PhysicalArtifactReadRange {
    pub const fn new(target: PhysicalArtifactReadTarget, offset: u64, length: u32) -> Option<Self> {
        if length == 0 || offset.checked_add(length as u64).is_none() {
            return None;
        }
        Some(Self {
            target,
            offset,
            length,
        })
    }
    pub const fn target(self) -> PhysicalArtifactReadTarget {
        self.target
    }
    pub const fn offset(self) -> u64 {
        self.offset
    }
    pub const fn length(self) -> u32 {
        self.length
    }
    pub fn overlaps(self, other: Self) -> bool {
        self.target.same_location(other.target)
            && self.offset < other.offset + u64::from(other.length)
            && other.offset < self.offset + u64::from(self.length)
    }
}
