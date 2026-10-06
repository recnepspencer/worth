//! Where every store's WAL begins. A store that has never checkpointed is
//! recovered from here, so the WAL it retains must start exactly at this
//! segment, generation and LSN.

use crate::{LogSequenceNumber, WalSegmentArtifactIdentity, WalSegmentGeneration, WalSegmentId};

/// The first segment, its first generation, and the first LSN a frame is
/// assigned. `LogSequenceNumber::GENESIS` is the empty position before it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WalOrigin {
    segment: WalSegmentId,
    generation: WalSegmentGeneration,
    lsn: LogSequenceNumber,
}

/// The canonical WAL origin. Every site that names the start of a WAL uses it.
pub const WAL_ORIGIN: WalOrigin = WalOrigin {
    segment: match WalSegmentId::new(1) {
        Ok(segment) => segment,
        Err(_) => unreachable!(),
    },
    generation: match WalSegmentGeneration::new(1) {
        Ok(generation) => generation,
        Err(_) => unreachable!(),
    },
    lsn: LogSequenceNumber::new(LogSequenceNumber::GENESIS.get() + 1),
};

impl WalOrigin {
    pub const fn segment(self) -> WalSegmentId {
        self.segment
    }

    pub const fn generation(self) -> WalSegmentGeneration {
        self.generation
    }

    /// The first segment artifact of the first generation.
    pub const fn artifact(self) -> WalSegmentArtifactIdentity {
        WalSegmentArtifactIdentity::new(self.segment, self.generation)
    }

    /// The first LSN a frame is assigned.
    pub const fn lsn(self) -> LogSequenceNumber {
        self.lsn
    }

    /// Whether a segment that starts at `start` is the origin segment.
    pub fn begins(self, artifact: WalSegmentArtifactIdentity, start: LogSequenceNumber) -> bool {
        artifact == self.artifact() && start == self.lsn
    }
}
